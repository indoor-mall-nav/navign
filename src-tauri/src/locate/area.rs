use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::str::FromStr;
use wkt::types::Polygon;
use wkt::Wkt;

#[derive(Clone, Debug, FromRow, Serialize, Deserialize, Default)]
pub struct ActiveArea {
    pub id: String,
    pub name: String,
    // Well-known text representation of the polygon
    pub polygon: String,
    pub entity: String,
    pub updated_at: u64,
    pub stored_at: u64,
}

#[allow(unused)]
impl ActiveArea {
    pub fn new(id: String, name: String, polygon: String, entity: String, updated_at: u64) -> Self {
        Self {
            id,
            name,
            polygon,
            entity,
            updated_at,
            stored_at: chrono::Utc::now().timestamp() as u64,
        }
    }

    pub fn polygon(&self) -> Option<Polygon<f64>> {
        let geom = wkt::Wkt::from_str(&self.polygon).ok()?;
        if let Wkt::Polygon(polygon) = geom {
            Some(polygon)
        } else {
            None
        }
    }

    pub async fn insert(&self, pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO active_areas (id, name, polygon, entity, updated_at, stored_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(&self.id)
            .bind(&self.name)
            .bind(&self.polygon)
            .bind(&self.entity)
            .bind(self.updated_at as i64)
            .bind(chrono::Utc::now().timestamp())
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update(&self, pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE active_areas SET name = ?, polygon = ?, entity = ?, updated_at = ?, stored_at = ? WHERE id = ?")
            .bind(&self.name)
            .bind(&self.polygon)
            .bind(&self.entity)
            .bind(self.updated_at as i64)
            .bind(chrono::Utc::now().timestamp())
            .bind(&self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn get_by_id(pool: &sqlx::SqlitePool, id: &str) -> Result<Option<Self>, sqlx::Error> {
        let area = sqlx::query_as::<_, ActiveArea>("SELECT * FROM active_areas WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(area)
    }

    pub async fn search_by_name(
        pool: &sqlx::SqlitePool,
        name: &str,
        entity: String,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let areas = sqlx::query_as::<_, ActiveArea>(
            "SELECT * FROM active_areas WHERE name LIKE ? AND entity = ?",
        )
        .bind(format!("%{}%", name))
        .bind(entity)
        .fetch_all(pool)
        .await?;
        Ok(areas)
    }

    pub async fn remove(pool: &sqlx::SqlitePool, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM active_areas WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn create_table(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS active_areas (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                polygon TEXT NOT NULL,
                entity TEXT NOT NULL,
                updated_at INTEGER NOT NULL,
                stored_at INTEGER NOT NULL
            )",
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}
