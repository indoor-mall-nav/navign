// Custom FromRow implementations for SQLite
// These handle WKB decoding and allow us to reuse the PostgreSQL repository implementations

#[cfg(all(not(feature = "postgres"), feature = "sql", feature = "geo"))]
use crate::schema::postgis::{wkb_to_point, wkb_to_polygon};
#[cfg(all(not(feature = "postgres"), feature = "sql", feature = "geo"))]
use sqlx::sqlite::SqliteRow;
#[cfg(all(not(feature = "postgres"), feature = "sql", feature = "geo"))]
use sqlx::{FromRow, Row};

// Entity FromRow for SQLite
#[cfg(all(not(feature = "postgres"), feature = "sql", feature = "geo"))]
impl FromRow<'_, SqliteRow> for super::Entity {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let point_min = wkb_to_point(row.get::<Vec<u8>, _>("point_min_wkb").as_slice())
            .map_err(|e| sqlx::Error::Decode(format!("WKB decode: {}", e).into()))?;
        let point_max = wkb_to_point(row.get::<Vec<u8>, _>("point_max_wkb").as_slice())
            .map_err(|e| sqlx::Error::Decode(format!("WKB decode: {}", e).into()))?;
        let tags: Vec<String> = serde_json::from_str(&row.get::<String, _>("tags"))
            .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
        let entity_type = match row.get::<String, _>("type").as_str() {
            "Mall" => super::EntityType::Mall,
            "Transportation" => super::EntityType::Transportation,
            "School" => super::EntityType::School,
            "Hospital" => super::EntityType::Hospital,
            _ => super::EntityType::Mall,
        };

        Ok(Self {
            id: row.get("id"),
            r#type: entity_type,
            name: row.get("name"),
            description: row.get("description"),
            point_min,
            point_max,
            altitude_min: row.get("altitude_min"),
            altitude_max: row.get("altitude_max"),
            nation: row.get("nation"),
            region: row.get("region"),
            city: row.get("city"),
            tags,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}

// Area FromRow for SQLite
#[cfg(all(not(feature = "postgres"), feature = "sql", feature = "geo"))]
impl FromRow<'_, SqliteRow> for super::Area {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let polygon = wkb_to_polygon(row.get::<Vec<u8>, _>("polygon_wkb").as_slice())
            .map_err(|e| sqlx::Error::Decode(format!("WKB decode: {}", e).into()))?;

        Ok(Self {
            id: row.get("id"),
            entity_id: row.get("entity_id"),
            name: row.get("name"),
            description: row.get("description"),
            beacon_code: row.get("beacon_code"),
            floor_type: row.get("floor_type"),
            floor_name: row.get("floor_name"),
            polygon,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}

// Beacon FromRow for SQLite
#[cfg(all(not(feature = "postgres"), feature = "sql", feature = "geo"))]
impl FromRow<'_, SqliteRow> for super::Beacon {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let location = wkb_to_point(row.get::<Vec<u8>, _>("location_wkb").as_slice())
            .map_err(|e| sqlx::Error::Decode(format!("WKB decode: {}", e).into()))?;

        let beacon_type = match row.get::<String, _>("type").as_str() {
            "navigation" => super::BeaconType::Navigation,
            "marketing" => super::BeaconType::Marketing,
            "tracking" => super::BeaconType::Tracking,
            "environmental" => super::BeaconType::Environmental,
            "security" => super::BeaconType::Security,
            _ => super::BeaconType::Other,
        };

        let device = match row.get::<String, _>("device").as_str() {
            "esp32" => super::BeaconDevice::Esp32,
            "esp32c3" => super::BeaconDevice::Esp32C3,
            "esp32c5" => super::BeaconDevice::Esp32C5,
            "esp32c6" => super::BeaconDevice::Esp32C6,
            "esp32s3" => super::BeaconDevice::Esp32S3,
            _ => super::BeaconDevice::Esp32C3,
        };

        Ok(Self {
            id: row.get("id"),
            entity_id: row.get("entity_id"),
            area_id: row.get("area_id"),
            merchant_id: row.get("merchant_id"),
            connection_id: row.get("connection_id"),
            name: row.get("name"),
            description: row.get("description"),
            r#type: beacon_type,
            location,
            device,
            mac: row.get("mac"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}

// Merchant FromRow for SQLite
#[cfg(all(not(feature = "postgres"), feature = "sql", feature = "geo"))]
impl FromRow<'_, SqliteRow> for super::Merchant {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let location = wkb_to_point(row.get::<Vec<u8>, _>("location_wkb").as_slice())
            .map_err(|e| sqlx::Error::Decode(format!("WKB decode: {}", e).into()))?;
        let polygon = wkb_to_polygon(row.get::<Vec<u8>, _>("polygon_wkb").as_slice())
            .map_err(|e| sqlx::Error::Decode(format!("WKB decode: {}", e).into()))?;

        let merchant_type: super::MerchantType =
            serde_json::from_str(&row.get::<String, _>("type"))
                .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
        let tags: Vec<String> = serde_json::from_str(&row.get::<String, _>("tags"))
            .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
        let available_period: Option<Vec<(i64, i64)>> = row
            .get::<Option<String>, _>("available_period")
            .and_then(|s| serde_json::from_str(&s).ok());
        let opening_hours: Option<Vec<Vec<(i32, i32)>>> = row
            .get::<Option<String>, _>("opening_hours")
            .and_then(|s| serde_json::from_str(&s).ok());
        let social_media: Option<Vec<super::SocialMedia>> = row
            .get::<Option<String>, _>("social_media")
            .and_then(|s| serde_json::from_str(&s).ok());

        let style = match row.get::<String, _>("style").as_str() {
            "store" => super::MerchantStyle::Store,
            "kiosk" => super::MerchantStyle::Kiosk,
            "pop-up" => super::MerchantStyle::PopUp,
            "food-truck" => super::MerchantStyle::FoodTruck,
            "room" => super::MerchantStyle::Room,
            _ => super::MerchantStyle::Store,
        };

        Ok(Self {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            r#chain: row.get("chain"),
            entity_id: row.get("entity_id"),
            beacon_code: row.get("beacon_code"),
            area_id: row.get("area_id"),
            r#type: merchant_type,
            color: row.get("color"),
            tags,
            location,
            style,
            polygon,
            available_period,
            opening_hours,
            email: row.get("email"),
            phone: row.get("phone"),
            website: row.get("website"),
            social_media,
            image_url: row.get("image_url"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}

// Connection FromRow for SQLite
#[cfg(all(not(feature = "postgres"), feature = "sql", feature = "geo"))]
impl FromRow<'_, SqliteRow> for super::Connection {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let gnd = row
            .get::<Option<Vec<u8>>, _>("gnd_wkb")
            .map(|bytes| wkb_to_point(&bytes))
            .transpose()
            .map_err(|e| sqlx::Error::Decode(format!("WKB decode: {}", e).into()))?;

        let connected_areas: Vec<super::ConnectedArea> =
            serde_json::from_str(&row.get::<String, _>("connected_areas"))
                .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
        let available_period: Vec<(i32, i32)> =
            serde_json::from_str(&row.get::<String, _>("available_period"))
                .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
        let tags: Vec<String> = serde_json::from_str(&row.get::<String, _>("tags"))
            .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;

        let connection_type = match row.get::<String, _>("type").as_str() {
            "gate" => super::ConnectionType::Gate,
            "escalator" => super::ConnectionType::Escalator,
            "elevator" => super::ConnectionType::Elevator,
            "stairs" => super::ConnectionType::Stairs,
            "rail" => super::ConnectionType::Rail,
            "shuttle" => super::ConnectionType::Shuttle,
            _ => super::ConnectionType::Gate,
        };

        Ok(Self {
            id: row.get("id"),
            entity_id: row.get("entity_id"),
            name: row.get("name"),
            description: row.get("description"),
            r#type: connection_type,
            connected_areas,
            available_period,
            tags,
            gnd,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}
