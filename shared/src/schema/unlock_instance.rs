#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "postgres")]
use uuid::Uuid;

/// Authentication type for unlock instances
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "sql", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sql",
    sqlx(type_name = "VARCHAR", rename_all = "kebab-case")
)]
pub enum AuthenticationType {
    /// Bluetooth Low Energy, the pipeline implemented in this project
    Ble,
    /// Near-field communication
    Nfc,
    /// Traditional username/password
    Password,
    /// One-time password, usually from an authenticator app
    Otp,
    /// Direct biometrics on the unlocker device
    Biometrics,
}

impl core::fmt::Display for AuthenticationType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AuthenticationType::Ble => write!(f, "ble"),
            AuthenticationType::Nfc => write!(f, "nfc"),
            AuthenticationType::Password => write!(f, "password"),
            AuthenticationType::Otp => write!(f, "otp"),
            AuthenticationType::Biometrics => write!(f, "biometrics"),
        }
    }
}

/// Unlock stage for tracking the progress of an unlock instance
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "sql", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sql",
    sqlx(type_name = "VARCHAR", rename_all = "kebab-case")
)]
pub enum UnlockStage {
    Initiated,
    Verified,
    Completed,
    Failed,
}

impl core::fmt::Display for UnlockStage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            UnlockStage::Initiated => write!(f, "initiated"),
            UnlockStage::Verified => write!(f, "verified"),
            UnlockStage::Completed => write!(f, "completed"),
            UnlockStage::Failed => write!(f, "failed"),
        }
    }
}

/// Unlock instance schema - represents a single unlock attempt
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "sql", derive(sqlx::FromRow))]
pub struct UnlockInstance {
    pub id: i32,
    pub beacon_id: i32,
    #[cfg(feature = "postgres")]
    pub user_id: Uuid,
    #[cfg(not(feature = "postgres"))]
    pub user_id: String,
    pub device_id: String,
    pub timestamp: i64,
    pub beacon_nonce: String,
    pub challenge_nonce: String,
    pub stage: UnlockStage,
    pub outcome: String,
    pub r#type: AuthenticationType,
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

#[cfg(all(feature = "postgres", feature = "sql"))]
#[async_trait::async_trait]
impl crate::schema::repository::IntRepository<sqlx::Postgres> for UnlockInstance {
    async fn create(pool: &sqlx::PgPool, item: &Self, _entity: uuid::Uuid) -> sqlx::Result<()> {
        sqlx::query(
            "INSERT INTO unlock_instances (beacon_id, user_id, device_id, timestamp, beacon_nonce, challenge_nonce, stage, outcome, type, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())"
        )
        .bind(item.beacon_id)
        .bind(item.user_id)
        .bind(&item.device_id)
        .bind(item.timestamp)
        .bind(&item.beacon_nonce)
        .bind(&item.challenge_nonce)
        .bind(item.stage.to_string())
        .bind(&item.outcome)
        .bind(item.r#type.to_string())
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn get_by_id(
        pool: &sqlx::PgPool,
        id: i32,
        _entity: uuid::Uuid,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, UnlockInstance>(
            "SELECT id, beacon_id, user_id, device_id, timestamp, beacon_nonce, challenge_nonce, stage, outcome, type, created_at, updated_at
             FROM unlock_instances
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    async fn update(pool: &sqlx::PgPool, item: &Self, _entity: uuid::Uuid) -> sqlx::Result<()> {
        sqlx::query(
            "UPDATE unlock_instances
             SET stage = $1, outcome = $2, updated_at = NOW()
             WHERE id = $3",
        )
        .bind(item.stage.to_string())
        .bind(&item.outcome)
        .bind(item.id)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn delete(pool: &sqlx::PgPool, id: i32, _entity: uuid::Uuid) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM unlock_instances WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn list(
        pool: &sqlx::PgPool,
        offset: i64,
        limit: i64,
        _entity: uuid::Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, UnlockInstance>(
            "SELECT id, beacon_id, user_id, device_id, timestamp, beacon_nonce, challenge_nonce, stage, outcome, type, created_at, updated_at
             FROM unlock_instances
             ORDER BY created_at DESC
             OFFSET $1 LIMIT $2"
        )
        .bind(offset)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    async fn search(
        pool: &sqlx::PgPool,
        query: &str,
        case_insensitive: bool,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
        _entity: uuid::Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        let mut sql = String::from(
            "SELECT id, beacon_id, user_id, device_id, timestamp, beacon_nonce, challenge_nonce, stage, outcome, type, created_at, updated_at
             FROM unlock_instances
             WHERE "
        );

        if case_insensitive {
            sql.push_str("LOWER(device_id) LIKE LOWER($1)");
        } else {
            sql.push_str("device_id LIKE $1");
        }

        if let Some(sort_field) = sort {
            sql.push_str(&format!(
                " ORDER BY {} {}",
                sort_field,
                if asc { "ASC" } else { "DESC" }
            ));
        } else {
            sql.push_str(" ORDER BY created_at DESC");
        }

        sql.push_str(" OFFSET $2 LIMIT $3");

        let pattern = format!("%{}%", query);

        sqlx::query_as::<_, UnlockInstance>(&sql)
            .bind(pattern)
            .bind(offset)
            .bind(limit)
            .fetch_all(pool)
            .await
    }
}

impl UnlockInstance {
    /// Helper method to update the stage of an unlock instance
    #[cfg(all(feature = "postgres", feature = "sql"))]
    pub async fn update_stage(
        &mut self,
        pool: &sqlx::PgPool,
        stage: UnlockStage,
        outcome: Option<String>,
    ) -> sqlx::Result<()> {
        self.stage = stage;
        if let Some(o) = outcome {
            self.outcome = o;
        }

        sqlx::query(
            "UPDATE unlock_instances
             SET stage = $1, outcome = $2, updated_at = NOW()
             WHERE id = $3",
        )
        .bind(self.stage.to_string())
        .bind(&self.outcome)
        .bind(self.id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
