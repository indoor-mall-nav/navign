use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::str::FromStr;
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

#[allow(dead_code)]
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
    
    pub async fn get_specific_merchant_beacons(pool: &SqlitePool, merchant_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        let beacons = sqlx::query_as::<_, BeaconInfo>("SELECT * FROM beacons WHERE merchant = ?")
            .bind(merchant_id)
            .fetch_all(pool)
            .await?;
        Ok(beacons)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locate::area::ActiveArea;
    use crate::locate::merchant::Merchant;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_beacon_info_crud() {
        let migrator = sqlx::migrate!("./migrations");
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        migrator.run(&pool).await.unwrap();

        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&pool)
            .await
            .unwrap();
        // Execute `navign.sql` to create necessary tables

        let merchant = Merchant {
            id: "merchant1".to_string(),
            name: "Test Merchant".to_string(),
            entry: "entity1".to_string(),
            ..Default::default()
        };
        let area = ActiveArea {
            id: "area1".to_string(),
            name: "Test Area".to_string(),
            entity: "entity1".to_string(),
            polygon: "POLYGON((0 0,0 1,1 1,1 0,0 0))".to_string(),
            updated_at: chrono::Utc::now().timestamp() as u64,
            stored_at: chrono::Utc::now().timestamp() as u64,
        };
        merchant.insert(&pool).await.unwrap();
        area.insert(&pool).await.unwrap();

        let beacon = BeaconInfo::new(
            "beacon1".to_string(),
            "AA:BB:CC:DD:EE:FF".to_string(),
            (37.7749, -122.4194),
            "merchant1".to_string(),
            "area1".to_string(),
            "entity1".to_string(),
        );

        // Insert
        beacon.insert(&pool).await.unwrap();

        // Get by ID
        let fetched = BeaconInfo::get_from_id(&pool, "beacon1")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fetched.mac, "AA:BB:CC:DD:EE:FF");

        // Update
        let updated_beacon = BeaconInfo::new(
            "beacon1".to_string(),
            "11:22:33:44:55:66".to_string(),
            (37.7749, -122.4194),
            "merchant1".to_string(),
            "area1".to_string(),
            "entity1".to_string(),
        );
        updated_beacon.update(&pool).await.unwrap();

        let fetched_updated = BeaconInfo::get_from_id(&pool, "beacon1")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fetched_updated.mac, "11:22:33:44:55:66");
        assert_eq!(fetched_updated.merchant, "merchant1");

        // Remove
        BeaconInfo::remove(&pool, "beacon1").await.unwrap();
        let fetched_none = BeaconInfo::get_from_id(&pool, "beacon1").await.unwrap();
        assert!(fetched_none.is_none());
    }
}
