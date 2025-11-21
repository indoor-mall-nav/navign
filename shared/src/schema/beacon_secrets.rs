#[cfg(feature = "postgres")]
use crate::schema::UuidRepository;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "postgres")]
use sqlx::PgPool;
#[cfg(feature = "postgres")]
use uuid::Uuid;

/// Beacon secrets schema - stores private keys and counter for beacons
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "sql", derive(sqlx::FromRow))]
pub struct BeaconSecrets {
    pub id: i32,
    pub beacon_id: i32,
    /// The private key in bytes (typically 32 bytes for P-256)
    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    pub private_key: [u8; 32],
    #[cfg(feature = "postgres")]
    #[cfg_attr(
        all(feature = "serde", not(feature = "postgres")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[cfg(not(feature = "postgres"))]
    #[cfg_attr(
        all(feature = "serde", not(feature = "postgres")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub created_at: Option<i64>, // Timestamp in milliseconds
    #[cfg(feature = "postgres")]
    #[cfg_attr(
        all(feature = "serde", not(feature = "postgres")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    #[cfg(not(feature = "postgres"))]
    #[cfg_attr(
        all(feature = "serde", not(feature = "postgres")),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub updated_at: Option<i64>, // Timestamp in milliseconds
}

impl BeaconSecrets {
    /// Get beacon secrets by beacon_id
    #[cfg(all(feature = "postgres", feature = "sql"))]
    pub async fn get_by_beacon_id(pool: &PgPool, beacon_id: i32) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, BeaconSecrets>(
            "SELECT id, beacon_id, private_key, created_at, updated_at
             FROM beacon_secrets
             WHERE beacon_id = $1",
        )
        .bind(beacon_id)
        .fetch_optional(pool)
        .await
    }
}

#[cfg(feature = "postgres")]
#[async_trait::async_trait]
impl UuidRepository<sqlx::Postgres> for BeaconSecrets {
    // Implement required methods here
    async fn create(pool: &PgPool, item: &Self) -> sqlx::Result<()> {
        sqlx::query(
            "
            INSERT INTO beacon_secrets (beacon_id, private_key, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())",
        )
        .bind(item.beacon_id)
        .bind(item.private_key)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn delete(pool: &PgPool, uuid: Uuid) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM beacon_secrets WHERE id = $1")
            .bind(uuid)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn get_by_uuid(pool: &PgPool, uuid: Uuid) -> sqlx::Result<Option<Self>> {
        let record = sqlx::query_as::<_, BeaconSecrets>(
            "
            SELECT id, beacon_id, private_key, created_at, updated_at
            FROM beacon_secrets
            WHERE id = $1",
        )
        .bind(uuid)
        .fetch_optional(pool)
        .await?;
        Ok(record)
    }

    async fn list(pool: &PgPool, offset: i64, limit: i64) -> sqlx::Result<Vec<Self>> {
        let records = sqlx::query_as::<_, BeaconSecrets>(
            "
            SELECT id, beacon_id, private_key, created_at, updated_at
            FROM beacon_secrets
            ORDER BY id
            OFFSET $1 LIMIT $2",
        )
        .bind(offset)
        .bind(limit)
        .fetch_all(pool)
        .await?;
        Ok(records)
    }

    async fn search(
        pool: &PgPool,
        query: &str,
        case_insensitive: bool,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
    ) -> sqlx::Result<Vec<Self>> {
        let mut sql = String::from(
            "
            SELECT id, beacon_id, private_key, created_at, updated_at
            FROM beacon_secrets
            WHERE beacon_id::text LIKE ",
        );
        if case_insensitive {
            sql.push_str("LOWER($1)");
        } else {
            sql.push_str("$1");
        }
        if let Some(sort_field) = sort {
            sql.push_str(&format!(
                " ORDER BY {} {}",
                sort_field,
                if asc { "ASC" } else { "DESC" }
            ));
        } else {
            sql.push_str(" ORDER BY id");
        }
        sql.push_str(" OFFSET $2 LIMIT $3");

        let pattern = if case_insensitive {
            format!("%{}%", query.to_lowercase())
        } else {
            format!("%{}%", query)
        };

        let records = sqlx::query_as::<_, BeaconSecrets>(sql.as_str())
            .bind(pattern)
            .bind(offset)
            .bind(limit)
            .fetch_all(pool)
            .await?;
        Ok(records)
    }

    async fn update(pool: &PgPool, item: &Self) -> sqlx::Result<()> {
        sqlx::query(
            "
            UPDATE beacon_secrets
            SET beacon_id = $1, private_key = $2, updated_at = NOW()
            WHERE id = $3",
        )
        .bind(item.beacon_id)
        .bind(item.private_key)
        .bind(item.id)
        .execute(pool)
        .await?;
        Ok(())
    }
}
