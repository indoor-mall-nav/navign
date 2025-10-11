use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
pub struct Merchant {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub polygon: String,
    pub entry: String,
    /// TODO: Change to enum
    pub r#type: String,
}

impl Merchant {
    pub fn new(
        id: String,
        name: String,
        description: Option<String>,
        polygon: String,
        entry: String,
        r#type: String,
    ) -> Self {
        Self {
            id,
            name,
            description,
            polygon,
            entry,
            r#type,
        }
    }

    pub async fn get_from_id(pool: &sqlx::SqlitePool, id: &str) -> Result<Option<Self>, sqlx::Error> {
        let merchant = sqlx::query_as::<_, Merchant>("SELECT * FROM merchants WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(merchant)
    }

    pub async fn insert(&self, pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO merchants (id, name, entry) VALUES (?, ?, ?)")
            .bind(&self.id)
            .bind(&self.name)
            .bind(&self.entry)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update(&self, pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE merchants SET name = ?, entry = ? WHERE id = ?")
            .bind(&self.name)
            .bind(&self.entry)
            .bind(&self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM merchants WHERE id = ?")
            .bind(&self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn create_table(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS merchants (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                entry TEXT NOT NULL
            )",
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}