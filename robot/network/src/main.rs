use anyhow::Result;
use std::sync::Arc;
use std::time::SystemTime;
use tracing::{error, info};

// Include generated protobuf code
pub mod proto {
    pub mod common {
        include!(concat!(env!("OUT_DIR"), "/navign.robot.common.rs"));
    }
    pub mod network {
        include!(concat!(env!("OUT_DIR"), "/navign.robot.network.rs"));
    }
}

use proto::common::*;
use proto::network::*;

/// Network component for external communication
pub struct NetworkComponent {
    zenoh_session: Arc<zenoh::Session>,
    server_url: String,
    client: reqwest::Client,
}

impl NetworkComponent {
    pub async fn new(server_url: String) -> Result<Self> {
        info!("Initializing Network component");

        let config = zenoh::Config::from_env().expect("Failed to load Zenoh config from environment");
        let session = zenoh::open(config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to open Zenoh session: {}", e))?;
        let client = reqwest::Client::new();

        Ok(Self {
            zenoh_session: Arc::new(session),
            server_url,
            client,
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting network component...");

        // Subscribe to pathfinding requests
        self.subscribe_pathfinding_requests().await?;

        // Subscribe to entity data requests
        self.subscribe_entity_data_requests().await?;

        // Publish status
        self.publish_status().await?;

        info!("Network component started");
        Ok(())
    }

    async fn subscribe_pathfinding_requests(&self) -> Result<()> {
        let subscriber = self
            .zenoh_session
            .declare_subscriber("robot/network/pathfinding/request")
            .await
            .map_err(|e| anyhow::anyhow!("Failed to declare subscriber: {}", e))?;

        let _server_url = self.server_url.clone();
        let _client = self.client.clone();

        tokio::spawn(async move {
            while let Ok(sample) = subscriber.recv_async().await {
                let _payload_bytes = sample.payload().to_bytes();
                info!("Received pathfinding request");
                // TODO: Forward to server and return response
            }
        });

        Ok(())
    }

    async fn subscribe_entity_data_requests(&self) -> Result<()> {
        let subscriber = self
            .zenoh_session
            .declare_subscriber("robot/network/entity/request")
            .await
            .map_err(|e| anyhow::anyhow!("Failed to declare subscriber: {}", e))?;

        tokio::spawn(async move {
            while let Ok(sample) = subscriber.recv_async().await {
                let _payload_bytes = sample.payload().to_bytes();
                info!("Received entity data request");
                // TODO: Fetch from server and return response
            }
        });

        Ok(())
    }

    async fn publish_status(&self) -> Result<()> {
        let zenoh_session = self.zenoh_session.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

            loop {
                interval.tick().await;

                let status = StatusResponse {
                    component: Some(ComponentInfo {
                        component_id: "network".to_string(),
                        r#type: ComponentType::Network as i32,
                        status: ComponentStatus::Ready as i32,
                        timestamp: Some(prost_types::Timestamp {
                            seconds: SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs() as i64,
                            nanos: 0,
                        }),
                        metadata: std::collections::HashMap::new(),
                    }),
                    metrics: None,
                    tower: None,
                    server: None,
                    ble: None,
                };

                let mut buf = Vec::new();
                if let Err(e) = prost::Message::encode(&status, &mut buf) {
                    error!("Failed to encode status: {}", e);
                    continue;
                }

                if let Err(e) = zenoh_session.put("robot/network/status", buf).await {
                    error!("Failed to publish status: {}", e);
                }
            }
        });

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Robot Network Component...");

    let server_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    let network = NetworkComponent::new(server_url).await?;
    network.start().await?;

    tokio::signal::ctrl_c().await?;

    info!("Network component shutting down...");
    Ok(())
}
