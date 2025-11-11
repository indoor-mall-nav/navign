use crate::schema::metadata::{PaginationResponse, PaginationResponseMetadata};
use crate::schema::service::{OneInArea, SearchQueryParams, Service};
use crate::AppState;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

// Re-export from navign-shared
pub use navign_shared::{Beacon, BeaconDevice, BeaconType};

#[async_trait]
impl Service for Beacon {
    type Id = i64;

    fn get_id(&self) -> i64 {
        self.id.expect("Beacon must have an ID")
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn get_description(&self) -> Option<String> {
        self.description.clone()
    }

    fn set_description(&mut self, description: Option<String>) {
        self.description = description;
    }

    fn get_table_name() -> &'static str {
        "beacons"
    }

    fn require_unique_name() -> bool {
        true
    }

    async fn get_one_by_id(pool: &PgPool, id: i64) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Beacon,
            r#"
            SELECT
                id,
                entity,
                area,
                merchant,
                connection,
                name,
                description,
                type as "type: String",
                ST_X(location::geometry) as "longitude!",
                ST_Y(location::geometry) as "latitude!",
                device as "device: String",
                mac,
                created_at,
                updated_at
            FROM beacons WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
        .map(|opt| opt.map(|b| Beacon {
            id: Some(b.id.unwrap()),
            entity: b.entity,
            area: b.area,
            merchant: b.merchant,
            connection: b.connection,
            name: b.name,
            description: b.description,
            r#type: match b.r#type.as_str() {
                "navigation" => BeaconType::Navigation,
                "marketing" => BeaconType::Marketing,
                "tracking" => BeaconType::Tracking,
                "environmental" => BeaconType::Environmental,
                "security" => BeaconType::Security,
                _ => BeaconType::Other,
            },
            location: (b.longitude, b.latitude),
            device: match b.device.as_str() {
                "esp32" => BeaconDevice::Esp32,
                "esp32c3" => BeaconDevice::Esp32C3,
                "esp32c5" => BeaconDevice::Esp32C5,
                "esp32c6" => BeaconDevice::Esp32C6,
                "esp32s3" => BeaconDevice::Esp32S3,
                _ => BeaconDevice::Esp32C3,
            },
            mac: b.mac,
            created_at: b.created_at,
            updated_at: b.updated_at,
        }))
    }

    async fn get_one_by_name(pool: &PgPool, name: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Beacon,
            r#"
            SELECT
                id,
                entity,
                area,
                merchant,
                connection,
                name,
                description,
                type as "type: String",
                ST_X(location::geometry) as "longitude!",
                ST_Y(location::geometry) as "latitude!",
                device as "device: String",
                mac,
                created_at,
                updated_at
            FROM beacons WHERE name = $1 LIMIT 1
            "#,
            name
        )
        .fetch_optional(pool)
        .await
        .map(|opt| opt.map(|b| Beacon {
            id: Some(b.id.unwrap()),
            entity: b.entity,
            area: b.area,
            merchant: b.merchant,
            connection: b.connection,
            name: b.name,
            description: b.description,
            r#type: match b.r#type.as_str() {
                "navigation" => BeaconType::Navigation,
                "marketing" => BeaconType::Marketing,
                "tracking" => BeaconType::Tracking,
                "environmental" => BeaconType::Environmental,
                "security" => BeaconType::Security,
                _ => BeaconType::Other,
            },
            location: (b.longitude, b.latitude),
            device: match b.device.as_str() {
                "esp32" => BeaconDevice::Esp32,
                "esp32c3" => BeaconDevice::Esp32C3,
                "esp32c5" => BeaconDevice::Esp32C5,
                "esp32c6" => BeaconDevice::Esp32C6,
                "esp32s3" => BeaconDevice::Esp32S3,
                _ => BeaconDevice::Esp32C3,
            },
            mac: b.mac,
            created_at: b.created_at,
            updated_at: b.updated_at,
        }))
    }

    async fn get_all(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id,
                entity,
                area,
                merchant,
                connection,
                name,
                description,
                type,
                ST_X(location::geometry) as longitude,
                ST_Y(location::geometry) as latitude,
                device,
                mac,
                created_at,
                updated_at
            FROM beacons
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|b| Beacon {
            id: Some(b.id),
            entity: b.entity,
            area: b.area,
            merchant: b.merchant,
            connection: b.connection,
            name: b.name,
            description: b.description,
            r#type: match b.r#type.as_str() {
                "navigation" => BeaconType::Navigation,
                "marketing" => BeaconType::Marketing,
                "tracking" => BeaconType::Tracking,
                "environmental" => BeaconType::Environmental,
                "security" => BeaconType::Security,
                _ => BeaconType::Other,
            },
            location: (b.longitude.unwrap_or_default(), b.latitude.unwrap_or_default()),
            device: match b.device.as_str() {
                "esp32" => BeaconDevice::Esp32,
                "esp32c3" => BeaconDevice::Esp32C3,
                "esp32c5" => BeaconDevice::Esp32C5,
                "esp32c6" => BeaconDevice::Esp32C6,
                "esp32s3" => BeaconDevice::Esp32S3,
                _ => BeaconDevice::Esp32C3,
            },
            mac: b.mac,
            created_at: b.created_at,
            updated_at: b.updated_at,
        }).collect())
    }

    async fn get_with_pagination(
        pool: &PgPool,
        page: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let offset = page * limit;
        let sort_column = sort.unwrap_or("name");
        let order = if asc { "ASC" } else { "DESC" };

        let query = format!(
            r#"
            SELECT
                id, entity, area, merchant, connection,
                name, description, type,
                ST_X(location::geometry) as longitude,
                ST_Y(location::geometry) as latitude,
                device, mac, created_at, updated_at
            FROM beacons
            ORDER BY {} {}
            LIMIT $1 OFFSET $2
            "#,
            sort_column, order
        );

        let rows = sqlx::query(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        Ok(rows.into_iter().map(|row| {
            let id: i64 = row.get("id");
            let entity: Uuid = row.get("entity");
            let area: i64 = row.get("area");
            let merchant: Option<Uuid> = row.get("merchant");
            let connection: Option<Uuid> = row.get("connection");
            let name: String = row.get("name");
            let description: Option<String> = row.get("description");
            let type_str: String = row.get("type");
            let longitude: f64 = row.get("longitude");
            let latitude: f64 = row.get("latitude");
            let device_str: String = row.get("device");
            let mac: String = row.get("mac");
            let created_at: i64 = row.get("created_at");
            let updated_at: i64 = row.get("updated_at");

            Beacon {
                id: Some(id),
                entity,
                area,
                merchant,
                connection,
                name,
                description,
                r#type: match type_str.as_str() {
                    "navigation" => BeaconType::Navigation,
                    "marketing" => BeaconType::Marketing,
                    "tracking" => BeaconType::Tracking,
                    "environmental" => BeaconType::Environmental,
                    "security" => BeaconType::Security,
                    _ => BeaconType::Other,
                },
                location: (longitude, latitude),
                device: match device_str.as_str() {
                    "esp32" => BeaconDevice::Esp32,
                    "esp32c3" => BeaconDevice::Esp32C3,
                    "esp32c5" => BeaconDevice::Esp32C5,
                    "esp32c6" => BeaconDevice::Esp32C6,
                    "esp32s3" => BeaconDevice::Esp32S3,
                    _ => BeaconDevice::Esp32C3,
                },
                mac,
                created_at,
                updated_at,
            }
        }).collect())
    }

    async fn create(&self, pool: &PgPool) -> Result<i64, sqlx::Error> {
        let beacon_type = match self.r#type {
            BeaconType::Navigation => "navigation",
            BeaconType::Marketing => "marketing",
            BeaconType::Tracking => "tracking",
            BeaconType::Environmental => "environmental",
            BeaconType::Security => "security",
            BeaconType::Other => "other",
        };

        let device = match self.device {
            BeaconDevice::Esp32 => "esp32",
            BeaconDevice::Esp32C3 => "esp32c3",
            BeaconDevice::Esp32C5 => "esp32c5",
            BeaconDevice::Esp32C6 => "esp32c6",
            BeaconDevice::Esp32S3 => "esp32s3",
        };

        let now = chrono::Utc::now().timestamp_millis();

        let result = sqlx::query!(
            r#"
            INSERT INTO beacons (
                entity, area, merchant, connection,
                name, description, type,
                location, device, mac,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, ST_SetSRID(ST_MakePoint($8, $9), 4326), $10, $11, $12, $13)
            RETURNING id
            "#,
            self.entity,
            self.area,
            self.merchant,
            self.connection,
            self.name,
            self.description,
            beacon_type,
            self.location.0,
            self.location.1,
            device,
            self.mac,
            now,
            now
        )
        .fetch_one(pool)
        .await?;

        Ok(result.id)
    }

    async fn update(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        let beacon_type = match self.r#type {
            BeaconType::Navigation => "navigation",
            BeaconType::Marketing => "marketing",
            BeaconType::Tracking => "tracking",
            BeaconType::Environmental => "environmental",
            BeaconType::Security => "security",
            BeaconType::Other => "other",
        };

        let device = match self.device {
            BeaconDevice::Esp32 => "esp32",
            BeaconDevice::Esp32C3 => "esp32c3",
            BeaconDevice::Esp32C5 => "esp32c5",
            BeaconDevice::Esp32C6 => "esp32c6",
            BeaconDevice::Esp32S3 => "esp32s3",
        };

        let now = chrono::Utc::now().timestamp_millis();
        let id = self.id.expect("Beacon must have an ID for update");

        sqlx::query!(
            r#"
            UPDATE beacons SET
                entity = $1, area = $2, merchant = $3, connection = $4,
                name = $5, description = $6, type = $7,
                location = ST_SetSRID(ST_MakePoint($8, $9), 4326),
                device = $10, mac = $11, updated_at = $12
            WHERE id = $13
            "#,
            self.entity,
            self.area,
            self.merchant,
            self.connection,
            self.name,
            self.description,
            beacon_type,
            self.location.0,
            self.location.1,
            device,
            self.mac,
            now,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn delete_by_id(pool: &PgPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM beacons WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn delete_by_name(pool: &PgPool, name: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM beacons WHERE name = $1", name)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn search_and_page_by_name_pattern(
        pool: &PgPool,
        params: SearchQueryParams<'_>,
    ) -> Result<PaginationResponse<Self>, sqlx::Error> {
        let sort_column = params.sort.unwrap_or("name");
        let order = if params.asc { "ASC" } else { "DESC" };
        let pattern = if params.case_insensitive {
            format!("%{}%", params.pattern.to_lowercase())
        } else {
            format!("%{}%", params.pattern)
        };

        let entity_uuid = Uuid::parse_str(params.entity)
            .map_err(|e| sqlx::Error::Protocol(format!("Invalid entity UUID: {}", e)))?;

        let count_query = if params.case_insensitive {
            "SELECT COUNT(*) as count FROM beacons WHERE entity = $1 AND LOWER(name) LIKE $2"
        } else {
            "SELECT COUNT(*) as count FROM beacons WHERE entity = $1 AND name LIKE $2"
        };

        let total: i64 = sqlx::query_scalar(count_query)
            .bind(entity_uuid)
            .bind(&pattern)
            .fetch_one(pool)
            .await?;

        let query = format!(
            r#"
            SELECT
                id, entity, area, merchant, connection,
                name, description, type,
                ST_X(location::geometry) as longitude,
                ST_Y(location::geometry) as latitude,
                device, mac, created_at, updated_at
            FROM beacons
            WHERE entity = $1 AND {} LIKE $2
            ORDER BY {} {}
            LIMIT $3 OFFSET $4
            "#,
            if params.case_insensitive { "LOWER(name)" } else { "name" },
            sort_column,
            order
        );

        let rows = sqlx::query(&query)
            .bind(entity_uuid)
            .bind(&pattern)
            .bind(params.limit)
            .bind(params.offset)
            .fetch_all(pool)
            .await?;

        let items = rows.into_iter().map(|row| {
            let id: i64 = row.get("id");
            let entity: Uuid = row.get("entity");
            let area: i64 = row.get("area");
            let merchant: Option<Uuid> = row.get("merchant");
            let connection: Option<Uuid> = row.get("connection");
            let name: String = row.get("name");
            let description: Option<String> = row.get("description");
            let type_str: String = row.get("type");
            let longitude: f64 = row.get("longitude");
            let latitude: f64 = row.get("latitude");
            let device_str: String = row.get("device");
            let mac: String = row.get("mac");
            let created_at: i64 = row.get("created_at");
            let updated_at: i64 = row.get("updated_at");

            Beacon {
                id: Some(id),
                entity,
                area,
                merchant,
                connection,
                name,
                description,
                r#type: match type_str.as_str() {
                    "navigation" => BeaconType::Navigation,
                    "marketing" => BeaconType::Marketing,
                    "tracking" => BeaconType::Tracking,
                    "environmental" => BeaconType::Environmental,
                    "security" => BeaconType::Security,
                    _ => BeaconType::Other,
                },
                location: (longitude, latitude),
                device: match device_str.as_str() {
                    "esp32" => BeaconDevice::Esp32,
                    "esp32c3" => BeaconDevice::Esp32C3,
                    "esp32c5" => BeaconDevice::Esp32C5,
                    "esp32c6" => BeaconDevice::Esp32C6,
                    "esp32s3" => BeaconDevice::Esp32S3,
                    _ => BeaconDevice::Esp32C3,
                },
                mac,
                created_at,
                updated_at,
            }
        }).collect();

        Ok(PaginationResponse {
            items,
            metadata: PaginationResponseMetadata {
                total,
                offset: params.offset,
                limit: params.limit,
            },
        })
    }

    async fn search_by_description_pattern(
        pool: &PgPool,
        pattern: &str,
        case_insensitive: bool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let pattern = if case_insensitive {
            format!("%{}%", pattern.to_lowercase())
        } else {
            format!("%{}%", pattern)
        };

        let query = if case_insensitive {
            r#"
            SELECT
                id, entity, area, merchant, connection,
                name, description, type,
                ST_X(location::geometry) as longitude,
                ST_Y(location::geometry) as latitude,
                device, mac, created_at, updated_at
            FROM beacons
            WHERE LOWER(description) LIKE $1
            "#
        } else {
            r#"
            SELECT
                id, entity, area, merchant, connection,
                name, description, type,
                ST_X(location::geometry) as longitude,
                ST_Y(location::geometry) as latitude,
                device, mac, created_at, updated_at
            FROM beacons
            WHERE description LIKE $1
            "#
        };

        let rows = sqlx::query(query)
            .bind(&pattern)
            .fetch_all(pool)
            .await?;

        Ok(rows.into_iter().map(|row| {
            let id: i64 = row.get("id");
            let entity: Uuid = row.get("entity");
            let area: i64 = row.get("area");
            let merchant: Option<Uuid> = row.get("merchant");
            let connection: Option<Uuid> = row.get("connection");
            let name: String = row.get("name");
            let description: Option<String> = row.get("description");
            let type_str: String = row.get("type");
            let longitude: f64 = row.get("longitude");
            let latitude: f64 = row.get("latitude");
            let device_str: String = row.get("device");
            let mac: String = row.get("mac");
            let created_at: i64 = row.get("created_at");
            let updated_at: i64 = row.get("updated_at");

            Beacon {
                id: Some(id),
                entity,
                area,
                merchant,
                connection,
                name,
                description,
                r#type: match type_str.as_str() {
                    "navigation" => BeaconType::Navigation,
                    "marketing" => BeaconType::Marketing,
                    "tracking" => BeaconType::Tracking,
                    "environmental" => BeaconType::Environmental,
                    "security" => BeaconType::Security,
                    _ => BeaconType::Other,
                },
                location: (longitude, latitude),
                device: match device_str.as_str() {
                    "esp32" => BeaconDevice::Esp32,
                    "esp32c3" => BeaconDevice::Esp32C3,
                    "esp32c5" => BeaconDevice::Esp32C5,
                    "esp32c6" => BeaconDevice::Esp32C6,
                    "esp32s3" => BeaconDevice::Esp32S3,
                    _ => BeaconDevice::Esp32C3,
                },
                mac,
                created_at,
                updated_at,
            }
        }).collect())
    }

    async fn bulk_create(
        pool: &PgPool,
        beacons: Vec<Self>,
    ) -> Result<Vec<i64>, sqlx::Error> {
        let mut ids = Vec::with_capacity(beacons.len());

        for beacon in beacons {
            let id = beacon.create(pool).await?;
            ids.push(id);
        }

        Ok(ids)
    }
}

impl OneInArea for Beacon {
    async fn get_all_in_area(
        pool: &PgPool,
        area_id: i64,
        entity_id: Uuid,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
    ) -> anyhow::Result<PaginationResponse<Self>> {
        let sort_column = sort.unwrap_or("name");
        let order = if asc { "ASC" } else { "DESC" };

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM beacons WHERE entity = $1 AND area = $2"
        )
        .bind(entity_id)
        .bind(area_id)
        .fetch_one(pool)
        .await?;

        let query = format!(
            r#"
            SELECT
                id, entity, area, merchant, connection,
                name, description, type,
                ST_X(location::geometry) as longitude,
                ST_Y(location::geometry) as latitude,
                device, mac, created_at, updated_at
            FROM beacons
            WHERE entity = $1 AND area = $2
            ORDER BY {} {}
            LIMIT $3 OFFSET $4
            "#,
            sort_column, order
        );

        let rows = sqlx::query(&query)
            .bind(entity_id)
            .bind(area_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        let items = rows.into_iter().map(|row| {
            let id: i64 = row.get("id");
            let entity: Uuid = row.get("entity");
            let area: i64 = row.get("area");
            let merchant: Option<Uuid> = row.get("merchant");
            let connection: Option<Uuid> = row.get("connection");
            let name: String = row.get("name");
            let description: Option<String> = row.get("description");
            let type_str: String = row.get("type");
            let longitude: f64 = row.get("longitude");
            let latitude: f64 = row.get("latitude");
            let device_str: String = row.get("device");
            let mac: String = row.get("mac");
            let created_at: i64 = row.get("created_at");
            let updated_at: i64 = row.get("updated_at");

            Beacon {
                id: Some(id),
                entity,
                area,
                merchant,
                connection,
                name,
                description,
                r#type: match type_str.as_str() {
                    "navigation" => BeaconType::Navigation,
                    "marketing" => BeaconType::Marketing,
                    "tracking" => BeaconType::Tracking,
                    "environmental" => BeaconType::Environmental,
                    "security" => BeaconType::Security,
                    _ => BeaconType::Other,
                },
                location: (longitude, latitude),
                device: match device_str.as_str() {
                    "esp32" => BeaconDevice::Esp32,
                    "esp32c3" => BeaconDevice::Esp32C3,
                    "esp32c5" => BeaconDevice::Esp32C5,
                    "esp32c6" => BeaconDevice::Esp32C6,
                    "esp32s3" => BeaconDevice::Esp32S3,
                    _ => BeaconDevice::Esp32C3,
                },
                mac,
                created_at,
                updated_at,
            }
        }).collect();

        Ok(PaginationResponse {
            items,
            metadata: PaginationResponseMetadata {
                total,
                offset,
                limit,
            },
        })
    }
}
