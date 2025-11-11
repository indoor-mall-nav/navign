use crate::api::map::Area;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::str::FromStr;
use uuid::Uuid;
use wkt::Wkt;
use wkt::types::{Coord, Dimension, LineString, Polygon};

#[derive(Clone, Debug, FromRow, Serialize, Deserialize)]
pub struct ActiveArea {
    pub id: i64,
    pub name: String,
    // Well-known text representation of the polygon
    pub polygon: String,
    pub entity: String, // Stored as Uuid string in SQLite
    pub updated_at: u64,
    pub stored_at: u64,
}

impl Default for ActiveArea {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            polygon: String::new(),
            entity: Uuid::nil().to_string(),
            updated_at: 0,
            stored_at: 0,
        }
    }
}

fn coords_to_polygon(coords: &[(f64, f64)]) -> String {
    let polygon = Polygon::new(
        vec![LineString::new(
            coords
                .iter()
                .map(|&(x, y)| Coord {
                    x,
                    y,
                    z: None,
                    m: None,
                })
                .collect(),
            Dimension::XY,
        )],
        Dimension::XY,
    );
    polygon.to_string()
}

impl From<Area> for ActiveArea {
    fn from(area: Area) -> Self {
        Self {
            id: area.id.unwrap_or(0), // SQLite auto-increment will assign ID if 0
            name: area.name,
            entity: area.entity.to_string(), // Store Uuid as string in SQLite
            polygon: coords_to_polygon(area.polygon.as_slice()),
            updated_at: area.updated_at as u64,
            stored_at: chrono::Utc::now().timestamp() as u64,
        }
    }
}

#[allow(unused)]
impl ActiveArea {
    pub fn new(id: i64, name: String, polygon: String, entity: String, updated_at: u64) -> Self {
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

    pub async fn get_by_id(pool: &sqlx::SqlitePool, id: i64) -> Result<Option<Self>, sqlx::Error> {
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

    pub async fn remove(pool: &sqlx::SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM active_areas WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn create_table(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS active_areas (
                id INTEGER PRIMARY KEY,
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
