//! MongoDB to PostgreSQL Migration Tool
//!
//! This tool migrates all data from MongoDB to PostgreSQL while preserving
//! relationships and converting IDs appropriately.
//!
//! Usage:
//!   cargo run --bin migrate -- [OPTIONS]
//!
//! Options:
//!   --dry-run         Print migration plan without executing
//!   --batch-size N    Number of records to migrate at once (default: 100)
//!   --skip-existing   Skip records that already exist in PostgreSQL
//!
//! Environment variables:
//!   MONGODB_HOST      MongoDB connection string (default: localhost:27017)
//!   MONGODB_DB_NAME   MongoDB database name (default: navign)
//!   POSTGRES_URL      PostgreSQL connection string (required)

use bson::{Document, doc};
use futures::TryStreamExt;
use mongodb::Database as MongoDatabase;
use navign_shared::*;
use sqlx::types::Uuid;
use std::collections::HashMap;
use std::env;
use tracing::{error, info, warn};

// Import from the library (lib.rs exports these modules)
use navign_server::pg::adapters::*;
use navign_server::pg::repository::*;
use navign_server::pg::{PgPool, create_pool};

/// Convert MerchantStyle enum to kebab-case string
fn merchant_style_to_string(style: &MerchantStyle) -> String {
    match style {
        MerchantStyle::Store => "store".to_string(),
        MerchantStyle::Kiosk => "kiosk".to_string(),
        MerchantStyle::PopUp => "pop-up".to_string(),
        MerchantStyle::FoodTruck => "food-truck".to_string(),
        MerchantStyle::Room => "room".to_string(),
    }
}

/// Migration statistics
#[derive(Debug, Default)]
struct MigrationStats {
    entities: usize,
    users: usize,
    areas: usize,
    beacons: usize,
    merchants: usize,
    connections: usize,
    beacon_secrets: usize,
    user_public_keys: usize,
    firmwares: usize,
    errors: usize,
}

impl MigrationStats {
    fn total(&self) -> usize {
        self.entities
            + self.users
            + self.areas
            + self.beacons
            + self.merchants
            + self.connections
            + self.beacon_secrets
            + self.user_public_keys
            + self.firmwares
    }

    fn print_summary(&self) {
        info!("=== Migration Summary ===");
        info!("Entities:         {}", self.entities);
        info!("Users:            {}", self.users);
        info!("Areas:            {}", self.areas);
        info!("Beacons:          {}", self.beacons);
        info!("Merchants:        {}", self.merchants);
        info!("Connections:      {}", self.connections);
        info!("Beacon Secrets:   {}", self.beacon_secrets);
        info!("User Public Keys: {}", self.user_public_keys);
        info!("Firmwares:        {}", self.firmwares);
        info!("========================");
        info!("Total migrated:   {}", self.total());
        info!("Errors:           {}", self.errors);
    }
}

/// Migration context
struct MigrationContext {
    mongo_db: MongoDatabase,
    pg_pool: PgPool,
    dry_run: bool,
    batch_size: usize,
    skip_existing: bool,
    // Maps MongoDB ObjectId to PostgreSQL UUID/Integer
    entity_id_map: HashMap<String, Uuid>,
    user_id_map: HashMap<String, Uuid>,
    area_id_map: HashMap<String, i32>,
    beacon_id_map: HashMap<String, i32>,
    merchant_id_map: HashMap<String, i32>,
    connection_id_map: HashMap<String, i32>,
}

impl MigrationContext {
    async fn new(
        mongo_db: MongoDatabase,
        pg_pool: PgPool,
        dry_run: bool,
        batch_size: usize,
        skip_existing: bool,
    ) -> Self {
        Self {
            mongo_db,
            pg_pool,
            dry_run,
            batch_size,
            skip_existing,
            entity_id_map: HashMap::new(),
            user_id_map: HashMap::new(),
            area_id_map: HashMap::new(),
            beacon_id_map: HashMap::new(),
            merchant_id_map: HashMap::new(),
            connection_id_map: HashMap::new(),
        }
    }

    /// Migrate entities (MongoDB ObjectId -> PostgreSQL UUID)
    async fn migrate_entities(&mut self, stats: &mut MigrationStats) -> anyhow::Result<()> {
        info!("Migrating entities...");
        let collection = self.mongo_db.collection::<Entity>("entities");
        let mut cursor = collection.find(doc! {}).await?;

        while let Some(entity) = cursor.try_next().await? {
            let mongo_id = entity.id.to_hex();

            if self.dry_run {
                info!("  [DRY RUN] Would migrate entity: {}", entity.name);
                stats.entities += 1;
                continue;
            }

            // Check if already exists
            if self.skip_existing {
                let existing: Option<(Uuid,)> = sqlx::query_as(
                    "SELECT id FROM entities WHERE name = $1 AND nation = $2 AND city = $3",
                )
                .bind(&entity.name)
                .bind(&entity.nation)
                .bind(&entity.city)
                .fetch_optional(self.pg_pool.inner())
                .await?;

                if let Some((uuid,)) = existing {
                    info!("  Entity '{}' already exists, skipping", entity.name);
                    self.entity_id_map.insert(mongo_id, uuid);
                    continue;
                }
            }

            // Convert MongoDB Entity to PostgreSQL Entity using adapter
            let pg_entity = entity_to_pg_entity(entity.clone());

            // Insert into PostgreSQL using repository
            let repo = EntityRepository::new(self.pg_pool.clone());
            let pg_id_str = repo.create(&pg_entity).await.map_err(|e| {
                error!("Failed to insert entity '{}': {}", entity.name, e);
                e
            })?;

            // Parse UUID from string
            let pg_id = Uuid::parse_str(&pg_id_str)
                .map_err(|e| anyhow::anyhow!("Failed to parse UUID: {}", e))?;

            self.entity_id_map.insert(mongo_id, pg_id);
            stats.entities += 1;
            info!("  ✓ Migrated entity: {} ({})", entity.name, pg_id);
        }

        Ok(())
    }

    /// Migrate users (MongoDB ObjectId -> PostgreSQL UUID)
    async fn migrate_users(&mut self, stats: &mut MigrationStats) -> anyhow::Result<()> {
        info!("Migrating users...");
        let collection = self.mongo_db.collection::<Document>("users");
        let mut cursor = collection.find(doc! {}).await?;

        while let Some(doc) = cursor.try_next().await? {
            let mongo_id = doc.get_object_id("_id")?.to_hex();
            let username = doc.get_str("username")?;

            if self.dry_run {
                info!("  [DRY RUN] Would migrate user: {}", username);
                stats.users += 1;
                continue;
            }

            // Check if already exists
            if self.skip_existing {
                let existing: Option<(Uuid,)> =
                    sqlx::query_as("SELECT id FROM users WHERE username = $1")
                        .bind(username)
                        .fetch_optional(self.pg_pool.inner())
                        .await?;

                if let Some((uuid,)) = existing {
                    info!("  User '{}' already exists, skipping", username);
                    self.user_id_map.insert(mongo_id, uuid);
                    continue;
                }
            }

            let pg_id: Uuid = sqlx::query_scalar(
                r#"
                INSERT INTO users (username, email, phone, google, wechat, hashed_password, activated, privileged)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING id
                "#,
            )
            .bind(username)
            .bind(doc.get_str("email")?)
            .bind(doc.get_str("phone").ok())
            .bind(doc.get_str("google").ok())
            .bind(doc.get_str("wechat").ok())
            .bind(doc.get_str("hashed_password")?)
            .bind(doc.get_bool("activated").unwrap_or(false))
            .bind(doc.get_bool("privileged").unwrap_or(false))
            .fetch_one(self.pg_pool.inner())
            .await
            .map_err(|e| {
                error!("Failed to insert user '{}': {}", username, e);
                e
            })?;

            self.user_id_map.insert(mongo_id, pg_id);
            stats.users += 1;
            info!("  ✓ Migrated user: {} ({})", username, pg_id);
        }

        Ok(())
    }

    /// Migrate areas (MongoDB ObjectId -> PostgreSQL Integer)
    async fn migrate_areas(&mut self, stats: &mut MigrationStats) -> anyhow::Result<()> {
        info!("Migrating areas...");
        let collection = self.mongo_db.collection::<Area>("areas");
        let mut cursor = collection.find(doc! {}).await?;

        while let Some(area) = cursor.try_next().await? {
            let mongo_id = area.id.to_hex();
            let entity_mongo_id = area.entity.to_hex();

            // Get mapped entity UUID
            let entity_uuid = self
                .entity_id_map
                .get(&entity_mongo_id)
                .ok_or_else(|| anyhow::anyhow!("Entity {} not found in map", entity_mongo_id))?;

            if self.dry_run {
                info!("  [DRY RUN] Would migrate area: {}", area.name);
                stats.areas += 1;
                continue;
            }

            // Convert MongoDB Area to PostgreSQL Area using adapter
            let pg_area = area_to_pg_area(area.clone(), *entity_uuid);

            // Insert into PostgreSQL using repository
            let repo = AreaRepository::new(self.pg_pool.clone());
            let pg_id_str = repo.create(&pg_area).await.map_err(|e| {
                error!("Failed to insert area '{}': {}", area.name, e);
                e
            })?;

            // Parse i32 from string
            let pg_id = pg_id_str
                .parse::<i32>()
                .map_err(|e| anyhow::anyhow!("Failed to parse area ID: {}", e))?;

            self.area_id_map.insert(mongo_id, pg_id);
            stats.areas += 1;
            info!("  ✓ Migrated area: {} ({})", area.name, pg_id);
        }

        Ok(())
    }

    /// Migrate beacons (MongoDB ObjectId -> PostgreSQL Integer)
    async fn migrate_beacons(&mut self, stats: &mut MigrationStats) -> anyhow::Result<()> {
        info!("Migrating beacons...");
        let collection = self.mongo_db.collection::<Beacon>("beacons");
        let mut cursor = collection.find(doc! {}).await?;

        while let Some(beacon) = cursor.try_next().await? {
            let mongo_id = beacon.id.to_hex();
            let entity_uuid = *self
                .entity_id_map
                .get(&beacon.entity.to_hex())
                .ok_or_else(|| anyhow::anyhow!("Entity not found"))?;
            let area_id = *self
                .area_id_map
                .get(&beacon.area.to_hex())
                .ok_or_else(|| anyhow::anyhow!("Area not found"))?;

            if self.dry_run {
                info!("  [DRY RUN] Would migrate beacon: {}", beacon.name);
                stats.beacons += 1;
                continue;
            }

            // Get merchant_id and connection_id if they exist
            let merchant_id = beacon
                .merchant
                .as_ref()
                .and_then(|oid| self.merchant_id_map.get(&oid.to_hex()).copied());
            let connection_id = beacon
                .connection
                .as_ref()
                .and_then(|oid| self.connection_id_map.get(&oid.to_hex()).copied());

            // TODO: Get actual floor from area - for now use placeholder
            let floor = "0".to_string();

            // Convert MongoDB Beacon to PostgreSQL Beacon using adapter
            let pg_beacon = beacon_to_pg_beacon(
                beacon.clone(),
                entity_uuid,
                area_id,
                merchant_id,
                connection_id,
                floor,
            );

            // Insert into PostgreSQL using repository
            let repo = BeaconRepository::new(self.pg_pool.clone());
            let pg_id_str = repo.create(&pg_beacon).await.map_err(|e| {
                error!("Failed to insert beacon '{}': {}", beacon.name, e);
                e
            })?;

            // Parse i32 from string
            let pg_id = pg_id_str
                .parse::<i32>()
                .map_err(|e| anyhow::anyhow!("Failed to parse beacon ID: {}", e))?;

            self.beacon_id_map.insert(mongo_id, pg_id);
            stats.beacons += 1;
            info!("  ✓ Migrated beacon: {} ({})", beacon.name, pg_id);
        }

        Ok(())
    }

    /// Migrate merchants (MongoDB ObjectId -> PostgreSQL Integer)
    async fn migrate_merchants(&mut self, stats: &mut MigrationStats) -> anyhow::Result<()> {
        info!("Migrating merchants...");
        let collection = self.mongo_db.collection::<Merchant>("merchants");
        let mut cursor = collection.find(doc! {}).await?;

        while let Some(merchant) = cursor.try_next().await? {
            let mongo_id = merchant.id.to_hex();
            let entity_uuid = self
                .entity_id_map
                .get(&merchant.entity.to_hex())
                .ok_or_else(|| anyhow::anyhow!("Entity not found"))?;
            let area_id = self
                .area_id_map
                .get(&merchant.area.to_hex())
                .ok_or_else(|| anyhow::anyhow!("Area not found"))?;

            if self.dry_run {
                info!("  [DRY RUN] Would migrate merchant: {}", merchant.name);
                stats.merchants += 1;
                continue;
            }

            let location = format!("POINT({} {})", merchant.location.0, merchant.location.1);

            // Extract floor from beacon_code (simplified - might need adjustment)
            let floor = merchant.beacon_code.split('-').next().unwrap_or("0");

            // Convert MerchantType to simple string and extract details
            let (type_str, merchant_style, food_type, food_cuisine, facility_type) =
                match &merchant.r#type {
                    MerchantType::Food { cuisine, r#type } => (
                        "Food".to_string(),
                        Some(merchant_style_to_string(&merchant.style)),
                        Some(format!("{:?}", r#type)),
                        cuisine.as_ref().map(|c| format!("{:?}", c)),
                        None,
                    ),
                    MerchantType::Facility { r#type } => (
                        "Facility".to_string(),
                        Some(merchant_style_to_string(&merchant.style)),
                        None,
                        None,
                        Some(format!("{:?}", r#type)),
                    ),
                    _ => (
                        format!("{:?}", merchant.r#type),
                        Some(merchant_style_to_string(&merchant.style)),
                        None,
                        None,
                        None,
                    ),
                };

            let pg_id: i32 = sqlx::query_scalar(
                r#"
                INSERT INTO merchants (entity_id, area_id, name, description, chain, type, logo, images,
                                      social_media, floor, location, merchant_style, food_type, food_cuisine,
                                      chinese_food_cuisine, facility_type, rating, reviews, opening_hours)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, ST_GeomFromText($11, 4326), $12, $13, $14, $15, $16, $17, $18, $19)
                RETURNING id
                "#,
            )
            .bind(entity_uuid)
            .bind(area_id)
            .bind(&merchant.name)
            .bind(&merchant.description)
            .bind(&merchant.chain)
            .bind(type_str)
            .bind(merchant.color.as_deref())
            .bind(sqlx::types::Json(&merchant.tags))
            .bind(sqlx::types::Json(&merchant.social_media))
            .bind(floor)
            .bind(&location)
            .bind(merchant_style)
            .bind(food_type)
            .bind(food_cuisine)
            .bind(None::<String>) // chinese_food_cuisine not in schema
            .bind(facility_type)
            .bind(None::<f64>) // rating not in schema
            .bind(None::<i32>) // reviews not in schema
            .bind(sqlx::types::Json(&merchant.available_period))
            .fetch_one(self.pg_pool.inner())
            .await
            .map_err(|e| {
                error!("Failed to insert merchant '{}': {}", merchant.name, e);
                e
            })?;

            self.merchant_id_map.insert(mongo_id, pg_id);
            stats.merchants += 1;
            info!("  ✓ Migrated merchant: {} ({})", merchant.name, pg_id);
        }

        Ok(())
    }

    /// Migrate connections (MongoDB ObjectId -> PostgreSQL Integer)
    async fn migrate_connections(&mut self, stats: &mut MigrationStats) -> anyhow::Result<()> {
        info!("Migrating connections...");
        let collection = self.mongo_db.collection::<Connection>("connections");
        let mut cursor = collection.find(doc! {}).await?;

        while let Some(connection) = cursor.try_next().await? {
            let mongo_id = connection.id.to_hex();
            let entity_uuid = self
                .entity_id_map
                .get(&connection.entity.to_hex())
                .ok_or_else(|| anyhow::anyhow!("Entity not found"))?;

            if self.dry_run {
                info!("  [DRY RUN] Would migrate connection: {}", connection.name);
                stats.connections += 1;
                continue;
            }

            // Map connected_areas - each is (ObjectId, f64, f64, bool)
            // We only need the ObjectId part, convert to PostgreSQL integer
            let connected_areas_pg: Vec<i32> = connection
                .connected_areas
                .iter()
                .filter_map(|(oid, _, _, _)| self.area_id_map.get(&oid.to_hex()).copied())
                .collect();

            let pg_id: i32 = sqlx::query_scalar(
                r#"
                INSERT INTO connections (entity_id, name, description, type, connected_areas)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id
                "#,
            )
            .bind(entity_uuid)
            .bind(&connection.name)
            .bind(&connection.description)
            .bind(connection.r#type.to_string())
            .bind(sqlx::types::Json(&connected_areas_pg))
            .fetch_one(self.pg_pool.inner())
            .await
            .map_err(|e| {
                error!("Failed to insert connection '{}': {}", connection.name, e);
                e
            })?;

            self.connection_id_map.insert(mongo_id, pg_id);
            stats.connections += 1;
            info!("  ✓ Migrated connection: {} ({})", connection.name, pg_id);
        }

        Ok(())
    }

    /// Migrate beacon secrets
    async fn migrate_beacon_secrets(&mut self, stats: &mut MigrationStats) -> anyhow::Result<()> {
        info!("Migrating beacon secrets...");
        let collection = self.mongo_db.collection::<Document>("beacon_secrets");
        let mut cursor = collection.find(doc! {}).await?;

        while let Some(doc) = cursor.try_next().await? {
            let beacon_mongo_id = doc.get_object_id("beacon_id")?.to_hex();
            let beacon_pg_id = self
                .beacon_id_map
                .get(&beacon_mongo_id)
                .ok_or_else(|| anyhow::anyhow!("Beacon not found"))?;

            if self.dry_run {
                info!(
                    "  [DRY RUN] Would migrate beacon secret for beacon: {}",
                    beacon_pg_id
                );
                stats.beacon_secrets += 1;
                continue;
            }

            let private_key = doc.get_binary_generic("private_key")?;

            sqlx::query(
                r#"
                INSERT INTO beacon_secrets (beacon_id, private_key)
                VALUES ($1, $2)
                ON CONFLICT (beacon_id) DO NOTHING
                "#,
            )
            .bind(beacon_pg_id)
            .bind(private_key)
            .execute(self.pg_pool.inner())
            .await
            .map_err(|e| {
                error!("Failed to insert beacon secret for {}: {}", beacon_pg_id, e);
                e
            })?;

            stats.beacon_secrets += 1;
            info!("  ✓ Migrated beacon secret for beacon {}", beacon_pg_id);
        }

        Ok(())
    }

    /// Migrate user public keys
    async fn migrate_user_public_keys(&mut self, stats: &mut MigrationStats) -> anyhow::Result<()> {
        info!("Migrating user public keys...");
        let collection = self.mongo_db.collection::<Document>("user_public_keys");
        let mut cursor = collection.find(doc! {}).await?;

        while let Some(doc) = cursor.try_next().await? {
            let user_mongo_id = doc.get_object_id("user_id")?.to_hex();
            let user_uuid = self
                .user_id_map
                .get(&user_mongo_id)
                .ok_or_else(|| anyhow::anyhow!("User not found"))?;

            if self.dry_run {
                info!(
                    "  [DRY RUN] Would migrate user public key for user: {}",
                    user_uuid
                );
                stats.user_public_keys += 1;
                continue;
            }

            let public_key = doc.get_str("public_key")?;
            let device_id = doc.get_str("device_id")?;
            let device_name = doc.get_str("device_name").ok();

            sqlx::query(
                r#"
                INSERT INTO user_public_keys (user_id, public_key, device_id, device_name)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (user_id, device_id) DO NOTHING
                "#,
            )
            .bind(user_uuid)
            .bind(public_key)
            .bind(device_id)
            .bind(device_name)
            .execute(self.pg_pool.inner())
            .await
            .map_err(|e| {
                error!("Failed to insert user public key for {}: {}", user_uuid, e);
                e
            })?;

            stats.user_public_keys += 1;
            info!("  ✓ Migrated user public key for user {}", user_uuid);
        }

        Ok(())
    }

    /// Migrate firmwares
    async fn migrate_firmwares(&mut self, stats: &mut MigrationStats) -> anyhow::Result<()> {
        info!("Migrating firmwares...");
        let collection = self.mongo_db.collection::<Document>("firmwares");
        let mut cursor = collection.find(doc! {}).await?;

        while let Some(doc) = cursor.try_next().await? {
            let version = doc.get_str("version")?;

            if self.dry_run {
                info!("  [DRY RUN] Would migrate firmware: {}", version);
                stats.firmwares += 1;
                continue;
            }

            // Check if already exists
            if self.skip_existing {
                let existing: Option<(i32,)> =
                    sqlx::query_as("SELECT id FROM firmwares WHERE version = $1")
                        .bind(version)
                        .fetch_optional(self.pg_pool.inner())
                        .await?;

                if existing.is_some() {
                    info!("  Firmware '{}' already exists, skipping", version);
                    continue;
                }
            }

            sqlx::query(
                r#"
                INSERT INTO firmwares (version, chip, file_name, file_size, checksum, release_notes, is_stable)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (version) DO NOTHING
                "#,
            )
            .bind(version)
            .bind(doc.get_str("chip")?)
            .bind(doc.get_str("file_name")?)
            .bind(doc.get_i64("file_size")?)
            .bind(doc.get_str("checksum")?)
            .bind(doc.get_str("release_notes").ok())
            .bind(doc.get_bool("is_stable").unwrap_or(false))
            .execute(self.pg_pool.inner())
            .await
            .map_err(|e| {
                error!("Failed to insert firmware '{}': {}", version, e);
                e
            })?;

            stats.firmwares += 1;
            info!("  ✓ Migrated firmware: {}", version);
        }

        Ok(())
    }

    /// Run complete migration
    async fn run_migration(&mut self) -> anyhow::Result<MigrationStats> {
        let mut stats = MigrationStats::default();

        info!("Starting migration from MongoDB to PostgreSQL...");
        if self.dry_run {
            warn!("DRY RUN MODE - No data will be written to PostgreSQL");
        }

        // Phase 1: Migrate top-level entities (no foreign keys)
        self.migrate_entities(&mut stats).await?;
        self.migrate_users(&mut stats).await?;

        // Phase 2: Migrate entities that depend on phase 1
        self.migrate_areas(&mut stats).await?;
        self.migrate_merchants(&mut stats).await?;
        self.migrate_connections(&mut stats).await?;

        // Phase 3: Migrate beacons (depends on areas, merchants, connections)
        self.migrate_beacons(&mut stats).await?;

        // Phase 4: Migrate related tables
        self.migrate_beacon_secrets(&mut stats).await?;
        self.migrate_user_public_keys(&mut stats).await?;
        self.migrate_firmwares(&mut stats).await?;

        info!("Migration completed successfully!");
        stats.print_summary();

        Ok(stats)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("MongoDB to PostgreSQL Migration Tool");

    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    let dry_run = args.contains(&"--dry-run".to_string());
    let skip_existing = args.contains(&"--skip-existing".to_string());
    let batch_size = args
        .iter()
        .position(|arg| arg == "--batch-size")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    // Connect to MongoDB
    let mongo_host = env::var("MONGODB_HOST").unwrap_or_else(|_| "localhost:27017".to_string());
    let mongo_db_name = env::var("MONGODB_DB_NAME").unwrap_or_else(|_| "navign".to_string());

    info!("Connecting to MongoDB at {}...", mongo_host);
    let mongo_client = mongodb::Client::with_uri_str(&format!("mongodb://{}", mongo_host)).await?;
    let mongo_db = mongo_client.database(&mongo_db_name);

    // Test MongoDB connection
    mongo_db
        .run_command(doc! { "ping": 1 })
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to MongoDB: {}", e))?;
    info!("MongoDB connection successful");

    // Connect to PostgreSQL
    let postgres_url =
        env::var("POSTGRES_URL").map_err(|_| anyhow::anyhow!("POSTGRES_URL not set"))?;

    info!("Connecting to PostgreSQL...");
    let pg_pool = create_pool(&postgres_url).await?;
    info!("PostgreSQL connection successful");

    // Check if PostgreSQL schema exists
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'entities')",
    )
    .fetch_one(pg_pool.inner())
    .await?;

    if !table_exists {
        return Err(anyhow::anyhow!(
            "PostgreSQL schema not initialized. Please run migrations first with: \
             POSTGRES_RUN_MIGRATIONS=true cargo run --bin navign-server"
        ));
    }

    // Create migration context and run
    let mut ctx =
        MigrationContext::new(mongo_db, pg_pool, dry_run, batch_size, skip_existing).await;
    ctx.run_migration().await?;

    info!("All done! 🎉");
    Ok(())
}
