use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, SqlitePool};
use std::str::FromStr;
use tauri::AppHandle;
use tauri_plugin_sql::Builder;
use wkt::types::Point;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct BeaconInfo {
    pub id: String,
    pub mac: String,
    pub location: String,
    pub merchant: String,
    pub area: String,
    pub entity: String,
}

impl BeaconInfo {
    pub fn new(
        id: String,
        mac: String,
        location: (f64, f64),
        merchant: String,
        area: String,
        entity: String,
    ) -> Self {
        Self {
            id,
            mac,
            location: format!("POINT({} {})", location.0, location.1),
            merchant,
            area,
            entity,
        }
    }

    pub fn location(&self) -> Option<Point<f64>> {
        let geom = wkt::Wkt::from_str(&self.location).ok()?;
        if let wkt::Wkt::Point(point) = geom {
            Some(point)
        } else {
            None
        }
    }

    pub async fn get_from_id(pool: &SqlitePool, id: &str) -> Result<Option<Self>, sqlx::Error> {
        let info = sqlx::query_as::<_, BeaconInfo>("SELECT * FROM beacons WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(info)
    }

    pub async fn get_from_mac(pool: &SqlitePool, mac: &str) -> Result<Option<Self>, sqlx::Error> {
        let info = sqlx::query_as::<_, BeaconInfo>("SELECT * FROM beacons WHERE mac = ?")
            .bind(mac)
            .fetch_optional(pool)
            .await?;
        Ok(info)
    }

    pub async fn insert(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO beacons (id, mac, location, merchant, area, entity) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(&self.id)
            .bind(&self.mac)
            .bind(&self.location)
            .bind(&self.merchant)
            .bind(&self.area)
            .bind(&self.entity)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE beacons SET mac = ?, location = ?, merchant = ?, area = ?, entity = ? WHERE id = ?")
            .bind(&self.mac)
            .bind(&self.location)
            .bind(&self.merchant)
            .bind(&self.area)
            .bind(&self.entity)
            .bind(&self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn remove(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM beacons WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn create_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        // Merchant and Area links to the `ActiveArea` and `Merchants` tables and stores the id as primary key
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS beacons (
                id TEXT PRIMARY KEY,
                mac TEXT NOT NULL,
                location TEXT NOT NULL,
                merchant TEXT NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
                area TEXT NOT NULL REFERENCES active_areas(id) ON DELETE CASCADE,
                entity TEXT NOT NULL
            )",
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}
