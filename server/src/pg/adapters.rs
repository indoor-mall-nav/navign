//! Schema adapters for converting between PostgreSQL and MongoDB models
//!
//! This module provides conversion functions to bridge the gap between
//! the PostgreSQL models (PgEntity, PgArea, etc.) and the current navign_shared
//! schema (Entity, Area, etc.).

use crate::pg::models::*;
use bson::oid::ObjectId;
use navign_shared::*;
use sqlx::types::Uuid;

// ============================================================================
// Entity Conversions
// ============================================================================

/// Convert PostgreSQL Entity to shared Entity
pub fn pg_entity_to_entity(pg: PgEntity) -> Entity {
    Entity {
        id: ObjectId::new(), // Placeholder - client should track UUIDs separately
        r#type: match pg.r#type.as_str() {
            "Mall" => EntityType::Mall,
            "Transportation" => EntityType::Transportation,
            "School" => EntityType::School,
            "Hospital" => EntityType::Hospital,
            _ => EntityType::Mall,
        },
        name: pg.name,
        description: pg.description,
        longitude_range: (pg.longitude_min, pg.longitude_max),
        latitude_range: (pg.latitude_min, pg.latitude_max),
        altitude_range: None, // Not in PostgreSQL schema
        nation: pg.nation,
        region: pg.region,
        city: pg.city,
        tags: Vec::new(), // Not in PostgreSQL schema
        created_at: pg.created_at.map(|dt| dt.timestamp_millis()).unwrap_or(0),
        updated_at: pg.updated_at.map(|dt| dt.timestamp_millis()).unwrap_or(0),
    }
}

/// Convert shared Entity to PostgreSQL Entity
pub fn entity_to_pg_entity(entity: Entity) -> PgEntity {
    PgEntity {
        id: Uuid::new_v4(), // Will be set by database
        r#type: entity.r#type.to_string(),
        name: entity.name,
        description: entity.description,
        nation: entity.nation,
        region: entity.region,
        city: entity.city,
        address: None, // Not in current shared schema
        longitude_min: entity.longitude_range.0,
        longitude_max: entity.longitude_range.1,
        latitude_min: entity.latitude_range.0,
        latitude_max: entity.latitude_range.1,
        floors: sqlx::types::Json(Vec::new()), // Not in current shared schema
        created_at: Some(chrono::Utc::now()),
        updated_at: Some(chrono::Utc::now()),
    }
}

// ============================================================================
// Area Conversions
// ============================================================================

/// Convert PostgreSQL Area to shared Area
pub fn pg_area_to_area(pg: PgArea) -> Area {
    // Parse floor string to Floor struct
    let floor = parse_floor_string(&pg.floor);

    Area {
        id: ObjectId::new(),     // Placeholder
        entity: ObjectId::new(), // Placeholder - client should track UUIDs
        name: pg.name,
        description: pg.description,
        beacon_code: pg.beacon_code,
        floor,
        polygon: serde_json::from_value(pg.polygon.0).unwrap_or_default(),
        created_at: pg.created_at.map(|dt| dt.timestamp_millis()).unwrap_or(0),
        updated_at: pg.updated_at.map(|dt| dt.timestamp_millis()).unwrap_or(0),
    }
}

/// Convert shared Area to PostgreSQL Area
pub fn area_to_pg_area(area: Area, entity_id: Uuid) -> PgArea {
    // Convert Floor to string
    let floor_str = match area.floor {
        Some(f) => format!("{}", i32::from(f)),
        None => "0".to_string(),
    };

    PgArea {
        id: 0, // Will be set by database
        entity_id,
        name: area.name,
        description: area.description,
        floor: floor_str,
        beacon_code: area.beacon_code,
        polygon: sqlx::types::Json(serde_json::to_value(&area.polygon).unwrap()),
        centroid: None, // Will be calculated from polygon
        created_at: Some(chrono::Utc::now()),
        updated_at: Some(chrono::Utc::now()),
    }
}

/// Parse floor string into Floor struct
fn parse_floor_string(floor_str: &str) -> Option<Floor> {
    if let Ok(floor_num) = floor_str.parse::<i32>() {
        if floor_num < 0 {
            Some(Floor {
                r#type: FloorType::Basement,
                name: floor_num.unsigned_abs(),
            })
        } else if floor_num == 0 {
            Some(Floor {
                r#type: FloorType::Level,
                name: 0,
            })
        } else {
            Some(Floor {
                r#type: FloorType::Floor,
                name: floor_num as u32,
            })
        }
    } else {
        None
    }
}

// ============================================================================
// Beacon Conversions
// ============================================================================

/// Convert PostgreSQL Beacon to shared Beacon
pub fn pg_beacon_to_beacon(pg: PgBeacon) -> Beacon {
    // Parse beacon type from PostgreSQL type field (kebab-case)
    let beacon_type = match pg.r#type.as_str() {
        "navigation" => BeaconType::Navigation,
        "marketing" => BeaconType::Marketing,
        "tracking" => BeaconType::Tracking,
        "environmental" => BeaconType::Environmental,
        "security" => BeaconType::Security,
        _ => BeaconType::Other,
    };

    // Parse device type from device_id string (lowercase)
    // The device_id might be a MAC address or contain device type info
    // For now, we'll default to Esp32C3
    let device = BeaconDevice::Esp32C3;

    // Extract location from PostGIS point
    // For now, use placeholder coordinates
    let location = (0.0, 0.0);

    Beacon {
        id: ObjectId::new(),     // Placeholder
        entity: ObjectId::new(), // Placeholder
        area: ObjectId::new(),   // Placeholder
        merchant: None,
        connection: None,
        name: pg.name,
        description: pg.description,
        r#type: beacon_type,
        location,
        device,
        mac: pg.device_id.clone(), // Use device_id as MAC address
        created_at: pg.created_at.map(|dt| dt.timestamp_millis()).unwrap_or(0),
        updated_at: pg.updated_at.map(|dt| dt.timestamp_millis()).unwrap_or(0),
    }
}

/// Convert shared Beacon to PostgreSQL Beacon
/// Note: This requires additional context (IDs, floor) that aren't in the Beacon struct
pub fn beacon_to_pg_beacon(
    beacon: Beacon,
    entity_id: Uuid,
    area_id: i32,
    merchant_id: Option<i32>,
    connection_id: Option<i32>,
    floor: String,
) -> PgBeacon {
    use serde::Serialize;

    // Convert BeaconType to kebab-case string
    let beacon_type = match beacon.r#type {
        BeaconType::Navigation => "navigation",
        BeaconType::Marketing => "marketing",
        BeaconType::Tracking => "tracking",
        BeaconType::Environmental => "environmental",
        BeaconType::Security => "security",
        BeaconType::Other => "other",
    }
    .to_string();

    // Create PostGIS point from location tuple (longitude, latitude)
    let location = PgPoint::new(beacon.location.0, beacon.location.1);

    PgBeacon {
        id: 0, // Will be set by database
        entity_id,
        area_id,
        merchant_id,
        connection_id,
        name: beacon.name,
        description: beacon.description,
        r#type: beacon_type,
        device_id: beacon.mac.clone(), // Use MAC as device_id
        floor,
        location,
        public_key: None,                        // Not in current Beacon schema
        capabilities: sqlx::types::Json(vec![]), // Not in current Beacon schema
        unlock_method: None,                     // Not in current Beacon schema
        created_at: Some(chrono::Utc::now()),
        updated_at: Some(chrono::Utc::now()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floor_parsing() {
        assert_eq!(
            parse_floor_string("0"),
            Some(Floor {
                r#type: FloorType::Level,
                name: 0
            })
        );
        assert_eq!(
            parse_floor_string("5"),
            Some(Floor {
                r#type: FloorType::Floor,
                name: 5
            })
        );
        assert_eq!(
            parse_floor_string("-2"),
            Some(Floor {
                r#type: FloorType::Basement,
                name: 2
            })
        );
        assert_eq!(parse_floor_string("invalid"), None);
    }
}
