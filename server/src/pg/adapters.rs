//! Schema adapters for converting between PostgreSQL and MongoDB models
//!
//! This module provides conversion functions to bridge the gap between
//! the PostgreSQL models (PgEntity, PgArea, etc.) and the current navign_shared
//! schema (Entity, Area, etc.).

#![allow(dead_code)] // Functions used by migration binary and will be used by handlers

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
        id: pg.id.to_string(),
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
        id: pg.id.to_string(),
        entity: pg.entity_id.to_string(),
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
pub fn area_to_pg_area(area: Area) -> PgArea {
    let entity_id = Uuid::parse_str(&area.entity).unwrap_or_else(|_| Uuid::new_v4());
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
        id: pg.id.to_string(),
        entity: pg.entity_id.to_string(),
        area: pg.area_id.to_string(),
        merchant: pg.merchant_id.map(|id| id.to_string()),
        connection: pg.connection_id.map(|id| id.to_string()),
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
pub fn beacon_to_pg_beacon(beacon: Beacon) -> PgBeacon {
    let entity_id = Uuid::parse_str(&beacon.entity).unwrap_or_else(|_| Uuid::new_v4());
    let area_id = beacon.area.parse::<i32>().unwrap_or(0);
    let merchant_id = beacon.merchant.and_then(|m| m.parse::<i32>().ok());
    let connection_id = beacon.connection.and_then(|c| c.parse::<i32>().ok());

    // Extract floor from area if needed (placeholder for now)
    let floor = "0".to_string();
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

// ============================================================================
// Merchant Conversions
// ============================================================================

/// Convert PostgreSQL Merchant to shared Merchant
pub fn pg_merchant_to_merchant(pg: PgMerchant) -> Merchant {
    // Parse floor string to Floor struct
    let floor = parse_floor_string(&pg.floor);

    Merchant {
        id: pg.id.to_string(),
        entity: pg.entity_id.to_string(),
        area: pg.area_id.to_string(),
        name: pg.name,
        description: pg.description,
        r#type: MerchantType::Other, // Would need mapping from string
        floor,
        location: (pg.location.x, pg.location.y),
        business_hours: Vec::new(), // Not in PostgreSQL schema
        contact_info: Vec::new(),   // Not in PostgreSQL schema
        created_at: pg.created_at.map(|dt| dt.timestamp_millis()).unwrap_or(0),
        updated_at: pg.updated_at.map(|dt| dt.timestamp_millis()).unwrap_or(0),
    }
}

/// Convert shared Merchant to PostgreSQL Merchant
pub fn merchant_to_pg_merchant(merchant: Merchant) -> PgMerchant {
    let entity_id = Uuid::parse_str(&merchant.entity).unwrap_or_else(|_| Uuid::new_v4());
    let area_id = merchant.area.parse::<i32>().unwrap_or(0);
    // Convert Floor to string
    let floor_str = match merchant.floor {
        Some(f) => format!("{}", i32::from(f)),
        None => "0".to_string(),
    };

    let location = PgPoint::new(merchant.location.0, merchant.location.1);

    PgMerchant {
        id: 0, // Will be set by database
        entity_id,
        area_id,
        name: merchant.name,
        description: merchant.description,
        r#type: "retail".to_string(), // Default type
        floor: floor_str,
        location,
        business_hours: None,
        contact_info: None,
        created_at: Some(chrono::Utc::now()),
        updated_at: Some(chrono::Utc::now()),
    }
}

// ============================================================================
// Connection Conversions
// ============================================================================

/// Convert PostgreSQL Connection to shared Connection
pub fn pg_connection_to_connection(pg: PgConnection) -> Connection {
    Connection {
        id: pg.id.to_string(),
        entity: pg.entity_id.to_string(),
        name: pg.name,
        description: pg.description,
        r#type: match pg.r#type.as_str() {
            "elevator" => ConnectionType::Elevator,
            "stairs" => ConnectionType::Stairs,
            "escalator" => ConnectionType::Escalator,
            _ => ConnectionType::Elevator,
        },
        from_floor: parse_floor_string(&pg.from_floor),
        to_floor: parse_floor_string(&pg.to_floor),
        location: (pg.location.x, pg.location.y),
        created_at: pg.created_at.map(|dt| dt.timestamp_millis()).unwrap_or(0),
        updated_at: pg.updated_at.map(|dt| dt.timestamp_millis()).unwrap_or(0),
    }
}

/// Convert shared Connection to PostgreSQL Connection
pub fn connection_to_pg_connection(connection: Connection) -> PgConnection {
    let entity_id = Uuid::parse_str(&connection.entity).unwrap_or_else(|_| Uuid::new_v4());
    // Convert Floors to strings
    let from_floor_str = match connection.from_floor {
        Some(f) => format!("{}", i32::from(f)),
        None => "0".to_string(),
    };
    let to_floor_str = match connection.to_floor {
        Some(f) => format!("{}", i32::from(f)),
        None => "0".to_string(),
    };

    let location = PgPoint::new(connection.location.0, connection.location.1);

    let connection_type = match connection.r#type {
        ConnectionType::Elevator => "elevator",
        ConnectionType::Stairs => "stairs",
        ConnectionType::Escalator => "escalator",
    }
    .to_string();

    PgConnection {
        id: 0, // Will be set by database
        entity_id,
        name: connection.name,
        description: connection.description,
        r#type: connection_type,
        from_floor: from_floor_str,
        to_floor: to_floor_str,
        location,
        created_at: Some(chrono::Utc::now()),
        updated_at: Some(chrono::Utc::now()),
    }
}

// ============================================================================
// User Conversions
// ============================================================================

/// Convert PostgreSQL User to shared User
pub fn pg_user_to_user(pg: PgUser) -> crate::schema::User {
    crate::schema::User {
        id: pg.id.to_string(),
        username: pg.username,
        email: pg.email,
        phone: pg.phone,
        google: pg.google,
        wechat: pg.wechat,
        hashed_password: pg.hashed_password,
        activated: pg.activated,
        privileged: pg.privileged,
        created_at: pg.created_at.map(|dt| dt.timestamp_millis()).unwrap_or(0),
        updated_at: pg.updated_at.map(|dt| dt.timestamp_millis()).unwrap_or(0),
    }
}

/// Convert shared User to PostgreSQL User
pub fn user_to_pg_user(user: crate::schema::User) -> PgUser {
    PgUser {
        id: Uuid::new_v4(), // Will be set by database
        username: user.username,
        email: user.email,
        phone: user.phone,
        google: user.google,
        wechat: user.wechat,
        hashed_password: user.hashed_password,
        activated: user.activated,
        privileged: user.privileged,
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
