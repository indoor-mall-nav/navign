# Orchestrator Protocol Implementation Guide

This guide provides step-by-step instructions for implementing the orchestrator-central server communication protocol in the Navign system.

## Quick Start

### Prerequisites

1. **Central Server:** Running Axum server with MongoDB
2. **Orchestrator:** Rust binary with Tonic gRPC support
3. **Network:** HTTPS connectivity from orchestrator to central server
4. **Credentials:** P-256 ECDSA key pair for orchestrator authentication

### Installation

```bash
# Add dependencies to admin/orchestrator/Cargo.toml
[dependencies]
tonic = "0.12"
prost = "0.13"
tokio = { version = "1.47", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "stream"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
p256 = { version = "0.13", features = ["ecdsa"] }
sha2 = "0.10"
eventsource-stream = "0.2"  # For SSE
futures = "0.3"

[build-dependencies]
tonic-build = "0.12"
```

```bash
# Update build.rs to compile proto files
cd admin/orchestrator
cargo build
```

## Implementation Steps

### Step 1: Orchestrator Registration

**File:** `admin/orchestrator/src/client/registration.rs`

```rust
use p256::ecdsa::{SigningKey, signature::Signer};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Serialize)]
pub struct RegisterRequest {
    pub entity_id: String,
    pub orchestrator_id: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub public_key: String,
    pub heartbeat_interval: u32,
    pub local_address: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterResponse {
    pub token: String,
    pub expires_at: i64,
    pub assigned_id: String,
    pub sync_endpoints: SyncEndpoints,
    pub initial_sync_required: bool,
}

#[derive(Debug, Deserialize)]
pub struct SyncEndpoints {
    pub events: String,
    pub data_sync: String,
    pub firmware: String,
}

pub struct OrchestratorClient {
    http_client: reqwest::Client,
    server_url: String,
    signing_key: SigningKey,
    token: Option<String>,
}

impl OrchestratorClient {
    pub fn new(server_url: String, signing_key: SigningKey) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            server_url,
            signing_key,
            token: None,
        }
    }

    pub async fn register(
        &mut self,
        entity_id: String,
        orchestrator_id: String,
    ) -> Result<RegisterResponse, Box<dyn std::error::Error>> {
        let public_key = self.signing_key.verifying_key();
        let public_key_pem = encode_public_key_pem(public_key)?;

        let request = RegisterRequest {
            entity_id: entity_id.clone(),
            orchestrator_id: orchestrator_id.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: vec![
                "task_assignment".to_string(),
                "firmware_distribution".to_string(),
                "beacon_management".to_string(),
            ],
            public_key: public_key_pem,
            heartbeat_interval: 60,
            local_address: "https://orchestrator.local:50051".to_string(),
        };

        let request_body = serde_json::to_string(&request)?;
        let signature = self.sign_request(&request_body)?;

        let response = self
            .http_client
            .post(format!("{}/api/orchestrators/register", self.server_url))
            .header("Content-Type", "application/json")
            .header("X-Signature", base64::encode(&signature))
            .body(request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let register_response: RegisterResponse = response.json().await?;
            self.token = Some(register_response.token.clone());
            Ok(register_response)
        } else {
            Err(format!("Registration failed: {}", response.status()).into())
        }
    }

    fn sign_request(&self, data: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let hash = Sha256::digest(data.as_bytes());
        let signature = self.signing_key.sign(&hash);
        Ok(signature.to_vec())
    }
}

fn encode_public_key_pem(key: &p256::ecdsa::VerifyingKey) -> Result<String, Box<dyn std::error::Error>> {
    // Convert to PEM format
    // Implementation depends on your crypto library
    todo!("Implement PEM encoding")
}
```

### Step 2: Event Stream Listener (SSE)

**File:** `admin/orchestrator/src/client/events.rs`

```rust
use eventsource_stream::Eventsource;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Event {
    pub id: String,
    pub r#type: String,
    pub data: serde_json::Value,
}

pub struct EventListener {
    client: reqwest::Client,
    server_url: String,
    token: String,
    entity_id: String,
    orchestrator_id: String,
    last_event_id: Option<String>,
}

impl EventListener {
    pub fn new(
        server_url: String,
        token: String,
        entity_id: String,
        orchestrator_id: String,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            server_url,
            token,
            entity_id,
            orchestrator_id,
            last_event_id: None,
        }
    }

    pub async fn listen<F>(&mut self, mut handler: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(Event) -> futures::future::BoxFuture<'static, Result<(), Box<dyn std::error::Error>>>,
    {
        loop {
            match self.connect_and_stream(&mut handler).await {
                Ok(_) => {
                    tracing::info!("Event stream ended normally");
                    break;
                }
                Err(e) => {
                    tracing::error!("Event stream error: {}", e);
                    // Exponential backoff
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
        Ok(())
    }

    async fn connect_and_stream<F>(
        &mut self,
        handler: &mut F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(Event) -> futures::future::BoxFuture<'static, Result<(), Box<dyn std::error::Error>>>,
    {
        let mut request = self
            .client
            .get(format!("{}/api/orchestrators/events", self.server_url))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "text/event-stream")
            .header("X-Entity-ID", &self.entity_id)
            .header("X-Orchestrator-ID", &self.orchestrator_id);

        if let Some(last_id) = &self.last_event_id {
            request = request.header("Last-Event-ID", last_id);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(format!("Failed to connect: {}", response.status()).into());
        }

        let mut stream = response.bytes_stream().eventsource();

        while let Some(event_result) = stream.next().await {
            match event_result {
                Ok(event) => {
                    if let Some(id) = event.id {
                        self.last_event_id = Some(id.clone());
                    }

                    let parsed_event = Event {
                        id: event.id.unwrap_or_default(),
                        r#type: event.event,
                        data: serde_json::from_str(&event.data)?,
                    };

                    handler(parsed_event).await?;
                }
                Err(e) => {
                    return Err(format!("Event stream error: {}", e).into());
                }
            }
        }

        Ok(())
    }
}
```

### Step 3: Event Handler Implementation

**File:** `admin/orchestrator/src/handlers/event_handler.rs`

```rust
use super::events::Event;
use crate::sync::DataSync;
use crate::firmware::FirmwareManager;
use futures::future::BoxFuture;

pub struct EventHandler {
    data_sync: DataSync,
    firmware_manager: FirmwareManager,
    entity_id: String,
}

impl EventHandler {
    pub fn new(
        data_sync: DataSync,
        firmware_manager: FirmwareManager,
        entity_id: String,
    ) -> Self {
        Self {
            data_sync,
            firmware_manager,
            entity_id,
        }
    }

    pub fn handle(&mut self, event: Event) -> BoxFuture<'static, Result<(), Box<dyn std::error::Error>>> {
        Box::pin(async move {
            match event.r#type.as_str() {
                "data_update" => self.handle_data_update(event).await,
                "firmware_update" => self.handle_firmware_update(event).await,
                "task_create" => self.handle_task_create(event).await,
                "config_update" => self.handle_config_update(event).await,
                "ping" => Ok(()),  // No action needed
                _ => {
                    tracing::warn!("Unknown event type: {}", event.r#type);
                    Ok(())
                }
            }
        })
    }

    async fn handle_data_update(&mut self, event: Event) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Handling data update event: {}", event.id);

        let changes = event.data["changes"].clone();
        let areas = changes["areas"].as_array().unwrap_or(&vec![]);
        let beacons = changes["beacons"].as_array().unwrap_or(&vec![]);
        let merchants = changes["merchants"].as_array().unwrap_or(&vec![]);
        let connections = changes["connections"].as_array().unwrap_or(&vec![]);

        // Fetch delta sync for changed entities
        self.data_sync.sync_delta(&self.entity_id).await?;

        tracing::info!(
            "Data sync completed: {} areas, {} beacons, {} merchants, {} connections",
            areas.len(),
            beacons.len(),
            merchants.len(),
            connections.len()
        );

        Ok(())
    }

    async fn handle_firmware_update(&mut self, event: Event) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Handling firmware update event: {}", event.id);

        let firmware_id = event.data["firmware_id"].as_str().unwrap();
        let version = event.data["version"].as_str().unwrap();
        let target = event.data["target"].as_str().unwrap();

        // Download firmware to local cache
        self.firmware_manager
            .download_firmware(firmware_id, version, target)
            .await?;

        // Announce to beacons via mDNS/BLE broadcast
        // (Implementation depends on beacon discovery mechanism)

        tracing::info!("Firmware {} downloaded and cached", firmware_id);

        Ok(())
    }

    async fn handle_task_create(&mut self, event: Event) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Handling task create event: {}", event.id);

        // Add task to local queue (existing orchestrator logic)
        let task_id = event.data["task_id"].as_str().unwrap();
        let priority = event.data["priority"].as_str().unwrap();

        tracing::info!("Task {} created with priority {}", task_id, priority);

        Ok(())
    }

    async fn handle_config_update(&mut self, event: Event) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Handling config update event: {}", event.id);

        let config_type = event.data["type"].as_str().unwrap();
        let config = event.data["config"].clone();

        // Apply configuration (e.g., robot selection parameters)
        tracing::info!("Applied config update: {}", config_type);

        Ok(())
    }
}
```

### Step 4: Data Synchronization

**File:** `admin/orchestrator/src/sync/mod.rs`

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SyncFullResponse {
    pub entity_id: String,
    pub sync_id: String,
    pub timestamp: String,
    pub checksum: String,
    pub data: SyncData,
    pub metadata: SyncMetadata,
}

#[derive(Debug, Deserialize)]
pub struct SyncData {
    pub areas: Vec<serde_json::Value>,
    pub beacons: Vec<serde_json::Value>,
    pub merchants: Vec<serde_json::Value>,
    pub connections: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct SyncMetadata {
    pub total_areas: u32,
    pub total_beacons: u32,
    pub total_merchants: u32,
    pub total_connections: u32,
}

#[derive(Debug, Deserialize)]
pub struct SyncDeltaResponse {
    pub entity_id: String,
    pub sync_id: String,
    pub timestamp: String,
    pub base_checksum: String,
    pub new_checksum: String,
    pub changes: Vec<Change>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Change {
    pub r#type: String,
    pub operation: String,
    pub id: String,
    pub data: Option<serde_json::Value>,
    pub updated_at: String,
}

pub struct DataSync {
    client: Client,
    server_url: String,
    token: String,
    local_cache: LocalCache,
}

impl DataSync {
    pub fn new(server_url: String, token: String, local_cache: LocalCache) -> Self {
        Self {
            client: Client::new(),
            server_url,
            token,
            local_cache,
        }
    }

    pub async fn sync_full(&self, entity_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(format!("{}/api/orchestrators/sync/full", self.server_url))
            .header("Authorization", format!("Bearer {}", self.token))
            .query(&[
                ("entity_id", entity_id),
                ("include", "areas,beacons,merchants,connections"),
                ("format", "compact"),
            ])
            .send()
            .await?;

        let sync_response: SyncFullResponse = response.json().await?;

        // Update local cache
        self.local_cache.update_full(sync_response.data).await?;
        self.local_cache.update_checksum(&sync_response.checksum).await?;

        tracing::info!(
            "Full sync completed: {} areas, {} beacons",
            sync_response.metadata.total_areas,
            sync_response.metadata.total_beacons
        );

        Ok(())
    }

    pub async fn sync_delta(&self, entity_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let last_sync = self.local_cache.get_last_sync_time().await?;

        let response = self
            .client
            .get(format!("{}/api/orchestrators/sync/delta", self.server_url))
            .header("Authorization", format!("Bearer {}", self.token))
            .query(&[
                ("entity_id", entity_id),
                ("since", &last_sync.to_rfc3339()),
            ])
            .send()
            .await?;

        let sync_response: SyncDeltaResponse = response.json().await?;

        // Apply changes to local cache
        for change in sync_response.changes {
            match change.operation.as_str() {
                "create" | "update" => {
                    self.local_cache
                        .upsert(&change.r#type, &change.id, change.data.unwrap())
                        .await?;
                }
                "delete" => {
                    self.local_cache.delete(&change.r#type, &change.id).await?;
                }
                _ => {
                    tracing::warn!("Unknown operation: {}", change.operation);
                }
            }
        }

        self.local_cache
            .update_checksum(&sync_response.new_checksum)
            .await?;

        tracing::info!(
            "Delta sync completed: {} changes applied",
            sync_response.changes.len()
        );

        Ok(())
    }

    pub async fn verify_checksum(&self, entity_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(format!("{}/api/orchestrators/sync/checksum", self.server_url))
            .header("Authorization", format!("Bearer {}", self.token))
            .query(&[("entity_id", entity_id)])
            .send()
            .await?;

        let checksum_response: ChecksumResponse = response.json().await?;
        let local_checksum = self.local_cache.calculate_checksum().await?;

        Ok(local_checksum == checksum_response.checksums.global)
    }
}

#[derive(Debug, Deserialize)]
struct ChecksumResponse {
    pub entity_id: String,
    pub timestamp: String,
    pub checksums: Checksums,
}

#[derive(Debug, Deserialize)]
struct Checksums {
    pub areas: String,
    pub beacons: String,
    pub merchants: String,
    pub connections: String,
    pub global: String,
}

// Local cache implementation (SQLite or in-memory)
pub struct LocalCache {
    // Implementation depends on storage mechanism
}

impl LocalCache {
    pub async fn update_full(&self, data: SyncData) -> Result<(), Box<dyn std::error::Error>> {
        todo!("Implement local cache update")
    }

    pub async fn upsert(
        &self,
        entity_type: &str,
        id: &str,
        data: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!("Implement upsert")
    }

    pub async fn delete(&self, entity_type: &str, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        todo!("Implement delete")
    }

    pub async fn update_checksum(&self, checksum: &str) -> Result<(), Box<dyn std::error::Error>> {
        todo!("Implement checksum update")
    }

    pub async fn calculate_checksum(&self) -> Result<String, Box<dyn std::error::Error>> {
        todo!("Implement checksum calculation")
    }

    pub async fn get_last_sync_time(&self) -> Result<chrono::DateTime<chrono::Utc>, Box<dyn std::error::Error>> {
        todo!("Implement last sync time retrieval")
    }
}
```

### Step 5: Firmware Manager

**File:** `admin/orchestrator/src/firmware/mod.rs`

```rust
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct FirmwareManager {
    client: Client,
    server_url: String,
    token: String,
    cache_dir: PathBuf,
}

impl FirmwareManager {
    pub fn new(server_url: String, token: String, cache_dir: PathBuf) -> Self {
        Self {
            client: Client::new(),
            server_url,
            token,
            cache_dir,
        }
    }

    pub async fn download_firmware(
        &self,
        firmware_id: &str,
        version: &str,
        target: &str,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let cache_path = self.cache_dir.join(format!("{}-{}-{}.bin", target, version, firmware_id));

        // Check if already cached
        if cache_path.exists() {
            tracing::info!("Firmware {} already cached", firmware_id);
            return Ok(cache_path);
        }

        // Query firmware metadata
        let metadata = self.query_firmware_metadata(target, "beacon", "stable").await?;

        // Download firmware
        let response = self
            .client
            .get(format!(
                "{}/api/firmware/{}/download",
                self.server_url, firmware_id
            ))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Download failed: {}", response.status()).into());
        }

        // Write to cache
        let mut file = File::create(&cache_path).await?;
        let bytes = response.bytes().await?;
        file.write_all(&bytes).await?;

        // Verify checksum
        let checksum = format!("sha256:{}", hex::encode(Sha256::digest(&bytes)));
        if checksum != metadata.checksum {
            tokio::fs::remove_file(&cache_path).await?;
            return Err("Checksum mismatch".into());
        }

        tracing::info!("Firmware {} downloaded and verified", firmware_id);

        Ok(cache_path)
    }

    async fn query_firmware_metadata(
        &self,
        target: &str,
        device_type: &str,
        channel: &str,
    ) -> Result<FirmwareMetadata, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(format!("{}/api/firmware/latest", self.server_url))
            .header("Authorization", format!("Bearer {}", self.token))
            .query(&[
                ("target", target),
                ("device_type", device_type),
                ("channel", channel),
            ])
            .send()
            .await?;

        let metadata: FirmwareMetadata = response.json().await?;
        Ok(metadata)
    }

    pub async fn serve_firmware(&self, firmware_id: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Find firmware in cache
        let cache_files = tokio::fs::read_dir(&self.cache_dir).await?;
        // Search for file matching firmware_id
        // Return file contents

        todo!("Implement firmware serving")
    }
}

#[derive(Debug, serde::Deserialize)]
struct FirmwareMetadata {
    pub firmware_id: String,
    pub version: String,
    pub checksum: String,
    pub size_bytes: u64,
}
```

### Step 6: Main Orchestrator Loop

**File:** `admin/orchestrator/src/main.rs`

```rust
use p256::ecdsa::SigningKey;
use tokio::signal;

mod client;
mod handlers;
mod sync;
mod firmware;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Load configuration
    let server_url = std::env::var("CENTRAL_SERVER_URL")?;
    let entity_id = std::env::var("ENTITY_ID")?;
    let orchestrator_id = std::env::var("ORCHESTRATOR_ID")?;

    // Load signing key (from file or environment)
    let signing_key = load_signing_key()?;

    // Register with central server
    let mut client = client::OrchestratorClient::new(server_url.clone(), signing_key);
    let register_response = client.register(entity_id.clone(), orchestrator_id.clone()).await?;

    tracing::info!(
        "Registered successfully: {}",
        register_response.assigned_id
    );

    // Initialize components
    let local_cache = sync::LocalCache::new();
    let data_sync = sync::DataSync::new(
        server_url.clone(),
        register_response.token.clone(),
        local_cache,
    );

    let firmware_cache_dir = std::path::PathBuf::from("/var/cache/navign/firmware");
    tokio::fs::create_dir_all(&firmware_cache_dir).await?;
    let firmware_manager = firmware::FirmwareManager::new(
        server_url.clone(),
        register_response.token.clone(),
        firmware_cache_dir,
    );

    // Perform initial sync if required
    if register_response.initial_sync_required {
        tracing::info!("Performing initial full sync");
        data_sync.sync_full(&entity_id).await?;
    }

    // Start event listener
    let mut event_listener = client::EventListener::new(
        register_response.sync_endpoints.events,
        register_response.token.clone(),
        entity_id.clone(),
        orchestrator_id.clone(),
    );

    let mut event_handler = handlers::EventHandler::new(
        data_sync,
        firmware_manager,
        entity_id.clone(),
    );

    // Start heartbeat task
    let heartbeat_task = tokio::spawn(async move {
        // Send heartbeat every 60 seconds
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            // Send heartbeat request
        }
    });

    // Start gRPC server (existing orchestrator logic)
    let grpc_server_task = tokio::spawn(async move {
        // Start gRPC server for robot communication
    });

    // Listen for events
    let event_listener_task = tokio::spawn(async move {
        event_listener
            .listen(|event| event_handler.handle(event))
            .await
    });

    // Wait for shutdown signal
    tokio::select! {
        _ = signal::ctrl_c() => {
            tracing::info!("Shutdown signal received");
        }
        _ = heartbeat_task => {}
        _ = grpc_server_task => {}
        _ = event_listener_task => {}
    }

    Ok(())
}

fn load_signing_key() -> Result<SigningKey, Box<dyn std::error::Error>> {
    // Load from file or environment variable
    todo!("Implement key loading")
}
```

## Server-Side Implementation

### Central Server Endpoints

**File:** `server/src/kernel/orchestrator/mod.rs`

```rust
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::sse::{Event, Sse},
    Json,
};
use futures::stream::Stream;
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::StreamExt as _;

pub async fn register_orchestrator(
    State(db): State<Database>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    // Verify ECDSA signature
    // Store orchestrator in database
    // Generate JWT token
    // Return response

    todo!("Implement registration")
}

pub async fn event_stream(
    State(db): State<Database>,
    headers: HeaderMap,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap();

    let entity_id = headers.get("X-Entity-ID").unwrap().to_str().unwrap();
    let orchestrator_id = headers.get("X-Orchestrator-ID").unwrap().to_str().unwrap();
    let last_event_id = headers.get("Last-Event-ID").and_then(|v| v.to_str().ok());

    // Create event stream
    let stream = tokio_stream::wrappers::BroadcastStream::new(event_tx.subscribe())
        .filter(move |event| {
            // Filter events for this entity_id
            event.entity_id == entity_id
        })
        .map(|event| {
            Ok(Event::default()
                .event(event.type_)
                .id(event.id)
                .data(serde_json::to_string(&event.data).unwrap()))
        })
        .throttle(Duration::from_millis(100));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(30))
            .event(Event::default().event("ping")),
    )
}

pub async fn sync_full(
    State(db): State<Database>,
    Query(params): Query<SyncFullParams>,
    headers: HeaderMap,
) -> Result<Json<SyncFullResponse>, StatusCode> {
    // Verify JWT token
    // Fetch all data for entity
    // Calculate checksums
    // Return response

    todo!("Implement full sync")
}

pub async fn sync_delta(
    State(db): State<Database>,
    Query(params): Query<SyncDeltaParams>,
    headers: HeaderMap,
) -> Result<Json<SyncDeltaResponse>, StatusCode> {
    // Verify JWT token
    // Fetch changes since timestamp
    // Return delta response

    todo!("Implement delta sync")
}

// Add routes in main.rs
pub fn orchestrator_routes() -> Router {
    Router::new()
        .route("/api/orchestrators/register", post(register_orchestrator))
        .route("/api/orchestrators/events", get(event_stream))
        .route("/api/orchestrators/sync/full", get(sync_full))
        .route("/api/orchestrators/sync/delta", get(sync_delta))
        .route("/api/orchestrators/sync/checksum", get(verify_checksum))
        .route("/api/orchestrators/heartbeat", post(heartbeat))
        // ... more routes
}
```

## Testing

### Integration Tests

**File:** `admin/orchestrator/tests/integration_test.rs`

```rust
use navign_orchestrator::client::OrchestratorClient;
use p256::ecdsa::SigningKey;

#[tokio::test]
async fn test_registration() {
    let server_url = "http://localhost:3000".to_string();
    let signing_key = SigningKey::random(&mut rand::thread_rng());

    let mut client = OrchestratorClient::new(server_url, signing_key);

    let response = client
        .register("test-entity".to_string(), "test-orch".to_string())
        .await
        .unwrap();

    assert!(!response.token.is_empty());
    assert!(!response.assigned_id.is_empty());
}

#[tokio::test]
async fn test_event_stream() {
    // Test SSE connection and event reception
    todo!("Implement event stream test")
}

#[tokio::test]
async fn test_data_sync() {
    // Test full and delta sync
    todo!("Implement data sync test")
}
```

## Deployment

### Docker Compose

**File:** `docker-compose.yml`

```yaml
version: '3.8'

services:
  central-server:
    build: ./server
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=mongodb://mongo:27017
      - DATABASE_NAME=navign
      - RUST_LOG=info
    depends_on:
      - mongo

  orchestrator:
    build: ./admin/orchestrator
    environment:
      - CENTRAL_SERVER_URL=http://central-server:3000
      - ENTITY_ID=mall-a-uuid
      - ORCHESTRATOR_ID=orch-mall-a-001
      - RUST_LOG=info
    volumes:
      - firmware-cache:/var/cache/navign/firmware
    depends_on:
      - central-server

  tower:
    build: ./admin/tower
    ports:
      - "8080:8080"
    environment:
      - ORCHESTRATOR_ADDR=orchestrator:50051
    depends_on:
      - orchestrator

  mongo:
    image: mongo:8.0
    ports:
      - "27017:27017"
    volumes:
      - mongo-data:/data/db

volumes:
  firmware-cache:
  mongo-data:
```

## Monitoring

### Prometheus Metrics

Add metrics to orchestrator:

```rust
use prometheus::{Counter, Histogram, Registry};

pub struct Metrics {
    pub events_received: Counter,
    pub sync_duration: Histogram,
    pub firmware_downloads: Counter,
}

impl Metrics {
    pub fn new(registry: &Registry) -> Self {
        let events_received = Counter::new("orchestrator_events_received_total", "Total events received").unwrap();
        let sync_duration = Histogram::new("orchestrator_sync_duration_seconds", "Sync operation duration").unwrap();
        let firmware_downloads = Counter::new("orchestrator_firmware_downloads_total", "Firmware downloads").unwrap();

        registry.register(Box::new(events_received.clone())).unwrap();
        registry.register(Box::new(sync_duration.clone())).unwrap();
        registry.register(Box::new(firmware_downloads.clone())).unwrap();

        Self {
            events_received,
            sync_duration,
            firmware_downloads,
        }
    }
}
```

## Security Considerations

1. **Key Management:** Store private keys in secure locations (HSM, cloud KMS)
2. **Token Rotation:** Implement automatic JWT token rotation before expiration
3. **Rate Limiting:** Add rate limiting on orchestrator client side
4. **Encryption:** Use TLS 1.3 for all connections
5. **Audit Logging:** Log all authentication and data access events

## Troubleshooting

### Common Issues

1. **SSE Connection Drops:**
   - Check firewall rules
   - Verify timeout settings
   - Ensure Last-Event-ID header is sent on reconnection

2. **Checksum Mismatch:**
   - Trigger full sync
   - Verify local cache integrity
   - Check for concurrent modifications

3. **Firmware Download Fails:**
   - Check network connectivity
   - Verify firmware ID exists
   - Try fallback URL

## Next Steps

1. Implement remaining endpoints (beacon management, task reporting)
2. Add comprehensive error handling
3. Implement local cache with SQLite
4. Add monitoring dashboards
5. Write end-to-end tests
6. Deploy to production environment

---

**Document Version:** 1.0.0
**Last Updated:** 2025-01-08
