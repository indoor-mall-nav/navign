#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "postgres")]
use super::postgis::{PgPoint, PgPolygon};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Merchant schema - represents a shop, store, or service location
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "sql", derive(sqlx::FromRow))]
pub struct Merchant {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub r#chain: Option<String>,
    #[cfg(feature = "postgres")]
    pub entity_id: sqlx::types::Uuid,
    #[cfg(not(feature = "postgres"))]
    #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
    pub entity_id: String,
    pub beacon_code: String,
    pub area_id: i32,
    pub r#type: MerchantType,
    pub color: Option<String>,
    pub tags: Vec<String>,
    #[cfg(feature = "postgres")]
    pub location: PgPoint,
    #[cfg(not(feature = "postgres"))]
    pub location: (f64, f64),
    pub style: MerchantStyle,
    #[cfg(feature = "postgres")]
    pub polygon: PgPolygon,
    #[cfg(not(feature = "postgres"))]
    pub polygon: Vec<(f64, f64)>,
    pub available_period: Option<Vec<(i64, i64)>>,
    /// Opening hours for each day of the week (Sunday=0 to Saturday=6)
    /// Each entry is (start_time_ms, end_time_ms) from midnight
    /// Empty vec means closed that day
    #[cfg_attr(feature = "ts-rs", ts(type = "Array<Array<[number, number]>> | null"))]
    pub opening_hours: Option<Vec<Vec<(i32, i32)>>>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub social_media: Option<Vec<SocialMedia>>,
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

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
pub enum MerchantType {
    Food {
        cuisine: Option<FoodCuisine>,
        r#type: FoodType,
    },
    Electronics {
        mobile: bool,
        computer: bool,
        accessories: bool,
    },
    Clothing {
        menswear: bool,
        womenswear: bool,
        childrenswear: bool,
    },
    Supermarket,
    Health,
    Entertainment,
    Facility {
        r#type: FacilityType,
    },
    Room,
    Other,
}

impl core::fmt::Display for MerchantType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MerchantType::Food { cuisine, r#type } => {
                if let Some(cuisine) = cuisine {
                    write!(f, "{:?} {:?}", r#type, cuisine)
                } else {
                    write!(f, "{:?}", r#type)
                }
            }
            MerchantType::Electronics {
                mobile,
                computer,
                accessories,
            } => {
                let mut types = Vec::new();
                if *mobile {
                    types.push("Mobile");
                }
                if *computer {
                    types.push("Computer");
                }
                if *accessories {
                    types.push("Accessories");
                }
                write!(f, "Electronics ({})", types.join(", "))
            }
            MerchantType::Clothing {
                menswear,
                womenswear,
                childrenswear,
            } => {
                let mut types = Vec::new();
                if *menswear {
                    types.push("Menswear");
                }
                if *womenswear {
                    types.push("Womenswear");
                }
                if *childrenswear {
                    types.push("Childrenswear");
                }
                write!(f, "Clothing ({})", types.join(", "))
            }
            MerchantType::Supermarket => write!(f, "Supermarket"),
            MerchantType::Health => write!(f, "Health"),
            MerchantType::Entertainment => write!(f, "Entertainment"),
            MerchantType::Facility { r#type } => write!(f, "Facility ({:?})", r#type),
            MerchantType::Room => write!(f, "Room"),
            MerchantType::Other => write!(f, "Other"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
pub enum FacilityType {
    Restroom,
    Atm,
    InformationDesk,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
pub enum FoodType {
    Restaurant(FoodCuisine),
    Cafe,
    FastFood,
    Bakery,
    Bar,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
pub enum FoodCuisine {
    Italian,
    Chinese {
        cuisine: ChineseFoodCuisine,
        specific: Option<String>,
    },
    Indian,
    American,
    Japanese,
    Korean,
    French,
    Thai,
    Mexican,
    Mediterranean,
    Spanish,
    Vietnamese,
    MiddleEastern,
    African,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
pub enum ChineseFoodCuisine {
    Cantonese,
    Sichuan,
    Hunan,
    Jiangxi,
    Shanghai,
    Hangzhou,
    Ningbo,
    Northern,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
pub struct SocialMedia {
    pub platform: SocialMediaPlatform,
    pub handle: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
pub enum SocialMediaPlatform {
    Facebook,
    Twitter,
    Instagram,
    LinkedIn,
    TikTok,
    WeChat,
    Weibo,
    Bilibili,
    RedNote,
    Reddit,
    Discord,
    Bluesky,
    WhatsApp,
    Telegram,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, export_to = "generated/"))]
#[cfg_attr(feature = "sql", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sql",
    sqlx(type_name = "VARCHAR", rename_all = "kebab-case")
)]
pub enum MerchantStyle {
    Store,
    Kiosk,
    PopUp,
    FoodTruck,
    Room,
}

impl core::fmt::Display for MerchantStyle {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MerchantStyle::Store => write!(f, "store"),
            MerchantStyle::Kiosk => write!(f, "kiosk"),
            MerchantStyle::PopUp => write!(f, "pop-up"),
            MerchantStyle::FoodTruck => write!(f, "food-truck"),
            MerchantStyle::Room => write!(f, "room"),
        }
    }
}

#[cfg(all(feature = "sql", feature = "postgres"))]
fn merchant_from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Merchant> {
    use sqlx::Row;

    let type_json: sqlx::types::Json<serde_json::Value> = row.try_get("type")?;
    let r#type: MerchantType =
        serde_json::from_value(type_json.0).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

    let social_media_json: Option<sqlx::types::Json<serde_json::Value>> =
        row.try_get("social_media")?;
    let social_media = social_media_json
        .map(|j| serde_json::from_value(j.0))
        .transpose()
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

    let available_period_json: Option<sqlx::types::Json<serde_json::Value>> =
        row.try_get("available_period")?;
    let available_period = available_period_json
        .map(|j| serde_json::from_value(j.0))
        .transpose()
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

    let opening_hours_json: Option<sqlx::types::Json<serde_json::Value>> =
        row.try_get("opening_hours")?;
    let opening_hours = opening_hours_json
        .map(|j| serde_json::from_value(j.0))
        .transpose()
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

    Ok(Merchant {
        id: row.try_get("id")?,
        entity_id: row.try_get("entity_id")?,
        area_id: row.try_get("area_id")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        chain: row.try_get("chain")?,
        beacon_code: row.try_get("beacon_code")?,
        r#type,
        color: row.try_get("color")?,
        tags: row.try_get("tags")?,
        location: row.try_get("location")?,
        style: row.try_get("style")?,
        polygon: row.try_get("polygon")?,
        available_period,
        opening_hours,
        email: row.try_get("email")?,
        phone: row.try_get("phone")?,
        website: row.try_get("website")?,
        social_media,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[cfg(all(feature = "sql", feature = "postgres"))]
#[async_trait::async_trait]
impl crate::schema::repository::IntRepository for Merchant {
    async fn create(pool: &sqlx::PgPool, item: &Self, entity: uuid::Uuid) -> sqlx::Result<()> {
        // Serialize MerchantType to JSON (complex enum with nested data)
        let type_json =
            serde_json::to_value(&item.r#type).map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

        // Serialize social_media to JSON if present
        let social_media_json = item
            .social_media
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

        // Serialize available_period to JSON if present
        let available_period_json = item
            .available_period
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

        // Serialize opening_hours to JSON if present
        let opening_hours_json = item
            .opening_hours
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

        sqlx::query(
            r#"INSERT INTO merchants (entity_id, area_id, name, description, chain, beacon_code,
                                      type, color, tags, location, style, polygon, available_period,
                                      opening_hours, email, phone, website, social_media)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)"#
        )
        .bind(entity)
        .bind(item.area_id)
        .bind(&item.name)
        .bind(&item.description)
        .bind(&item.chain)
        .bind(&item.beacon_code)
        .bind(type_json)
        .bind(&item.color)
        .bind(&item.tags)
        .bind(item.location)
        .bind(item.style)
        .bind(&item.polygon)
        .bind(available_period_json)
        .bind(opening_hours_json)
        .bind(&item.email)
        .bind(&item.phone)
        .bind(&item.website)
        .bind(social_media_json)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn get_by_id(
        pool: &sqlx::PgPool,
        id: i32,
        entity: uuid::Uuid,
    ) -> sqlx::Result<Option<Self>> {
        let row = sqlx::query(
            r#"SELECT id, entity_id, area_id, name, description, chain, beacon_code, type,
                      color, tags, location, style, polygon, available_period, opening_hours,
                      email, phone, website, social_media, created_at, updated_at
               FROM merchants WHERE id = $1 AND entity_id = $2"#,
        )
        .bind(id)
        .bind(entity)
        .fetch_optional(pool)
        .await?;

        row.map(|r| merchant_from_row(&r)).transpose()
    }

    async fn update(pool: &sqlx::PgPool, item: &Self, entity: uuid::Uuid) -> sqlx::Result<()> {
        // Serialize MerchantType to JSON
        let type_json =
            serde_json::to_value(&item.r#type).map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

        // Serialize social_media to JSON if present
        let social_media_json = item
            .social_media
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

        // Serialize available_period to JSON if present
        let available_period_json = item
            .available_period
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

        // Serialize opening_hours to JSON if present
        let opening_hours_json = item
            .opening_hours
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

        sqlx::query(
            r#"UPDATE merchants
               SET area_id = $3, name = $4, description = $5, chain = $6, beacon_code = $7,
                   type = $8, color = $9, tags = $10, location = $11, style = $12, polygon = $13,
                   available_period = $14, opening_hours = $15, email = $16, phone = $17,
                   website = $18, social_media = $19
               WHERE id = $1 AND entity_id = $2"#,
        )
        .bind(item.id)
        .bind(entity)
        .bind(item.area_id)
        .bind(&item.name)
        .bind(&item.description)
        .bind(&item.chain)
        .bind(&item.beacon_code)
        .bind(type_json)
        .bind(&item.color)
        .bind(&item.tags)
        .bind(item.location)
        .bind(item.style)
        .bind(&item.polygon)
        .bind(available_period_json)
        .bind(opening_hours_json)
        .bind(&item.email)
        .bind(&item.phone)
        .bind(&item.website)
        .bind(social_media_json)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn delete(pool: &sqlx::PgPool, id: i32, entity: uuid::Uuid) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM merchants WHERE id = $1 AND entity_id = $2")
            .bind(id)
            .bind(entity)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn list(
        pool: &sqlx::PgPool,
        offset: i64,
        limit: i64,
        entity: uuid::Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        let rows = sqlx::query(
            r#"SELECT id, entity_id, area_id, name, description, chain, beacon_code, type,
                      color, tags, location, style, polygon, available_period, opening_hours,
                      email, phone, website, social_media, created_at, updated_at
               FROM merchants WHERE entity_id = $1
               ORDER BY created_at DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(entity)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        rows.iter().map(merchant_from_row).collect()
    }

    async fn search(
        pool: &sqlx::PgPool,
        query: &str,
        case_insensitive: bool,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
        entity: uuid::Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        let like_pattern = format!("%{}%", query);
        let order_by = sort.unwrap_or("created_at");
        let direction = if asc { "ASC" } else { "DESC" };

        let sql = if case_insensitive {
            format!(
                r#"SELECT id, entity_id, area_id, name, description, chain, beacon_code, type,
                          color, tags, location, style, polygon, available_period, opening_hours,
                          email, phone, website, social_media, created_at, updated_at
                   FROM merchants
                   WHERE entity_id = $1 AND (name ILIKE $2 OR description ILIKE $2 OR beacon_code ILIKE $2)
                   ORDER BY {} {}
                   LIMIT $3 OFFSET $4"#,
                order_by, direction
            )
        } else {
            format!(
                r#"SELECT id, entity_id, area_id, name, description, chain, beacon_code, type,
                          color, tags, location, style, polygon, available_period, opening_hours,
                          email, phone, website, social_media, created_at, updated_at
                   FROM merchants
                   WHERE entity_id = $1 AND (name LIKE $2 OR description LIKE $2 OR beacon_code LIKE $2)
                   ORDER BY {} {}
                   LIMIT $3 OFFSET $4"#,
                order_by, direction
            )
        };

        let rows = sqlx::query(&sql)
            .bind(entity)
            .bind(&like_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        rows.iter().map(merchant_from_row).collect()
    }
}

#[cfg(all(feature = "sql", feature = "postgres"))]
#[async_trait::async_trait]
impl crate::schema::repository::IntRepositoryInArea for Merchant {
    async fn search_in_area(
        pool: &sqlx::PgPool,
        query: &str,
        case_insensitive: bool,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
        area: i32,
        entity: uuid::Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        let like_pattern = format!("%{}%", query);
        let order_by = sort.unwrap_or("created_at");
        let direction = if asc { "ASC" } else { "DESC" };

        let sql = if case_insensitive {
            format!(
                r#"SELECT id, entity_id, area_id, name, description, chain, beacon_code, type,
                          color, tags, location, style, polygon, available_period, opening_hours,
                          email, phone, website, social_media, created_at, updated_at
                   FROM merchants
                   WHERE entity_id = $1 AND area_id = $2 AND (name ILIKE $3 OR description ILIKE $3 OR beacon_code ILIKE $3)
                   ORDER BY {} {}
                   LIMIT $4 OFFSET $5"#,
                order_by, direction
            )
        } else {
            format!(
                r#"SELECT id, entity_id, area_id, name, description, chain, beacon_code, type,
                          color, tags, location, style, polygon, available_period, opening_hours,
                          email, phone, website, social_media, created_at, updated_at
                   FROM merchants
                   WHERE entity_id = $1 AND area_id = $2 AND (name LIKE $3 OR description LIKE $3 OR beacon_code LIKE $3)
                   ORDER BY {} {}
                   LIMIT $4 OFFSET $5"#,
                order_by, direction
            )
        };

        let rows = sqlx::query(&sql)
            .bind(entity)
            .bind(area)
            .bind(&like_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        rows.iter().map(merchant_from_row).collect()
    }
}

// SQLite repository implementation for Merchant
#[cfg(all(not(feature = "postgres"), feature = "sql", feature = "geo"))]
#[cfg(all(not(feature = "postgres"), feature = "sql", feature = "geo"))]
use crate::schema::postgis::{point_to_wkb, polygon_to_wkb, wkb_to_point, wkb_to_polygon};
#[cfg(all(not(feature = "postgres"), feature = "sql", feature = "geo"))]
use crate::schema::repository::{IntRepository, IntRepositoryInArea};

#[cfg(all(not(feature = "postgres"), feature = "sql", feature = "geo"))]
#[async_trait::async_trait]
impl IntRepository for Merchant {
    async fn create(pool: &sqlx::SqlitePool, item: &Self, entity: uuid::Uuid) -> sqlx::Result<()> {
        let location_wkb = point_to_wkb(item.location)
            .map_err(|e| sqlx::Error::Encode(format!("WKB encode: {}", e).into()))?;
        let polygon_wkb = polygon_to_wkb(&item.polygon)
            .map_err(|e| sqlx::Error::Encode(format!("WKB encode: {}", e).into()))?;
        let type_json = serde_json::to_string(&item.r#type)
            .map_err(|e| sqlx::Error::Encode(format!("JSON encode: {}", e).into()))?;
        let tags_json = serde_json::to_string(&item.tags)
            .map_err(|e| sqlx::Error::Encode(format!("JSON encode: {}", e).into()))?;
        let available_period_json = serde_json::to_string(&item.available_period)
            .map_err(|e| sqlx::Error::Encode(format!("JSON encode: {}", e).into()))?;
        let opening_hours_json = serde_json::to_string(&item.opening_hours)
            .map_err(|e| sqlx::Error::Encode(format!("JSON encode: {}", e).into()))?;
        let social_media_json = serde_json::to_string(&item.social_media)
            .map_err(|e| sqlx::Error::Encode(format!("JSON encode: {}", e).into()))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        sqlx::query(
            r#"INSERT INTO merchants (entity_id, area_id, name, description, chain, beacon_code,
                                     type, color, tags, location_wkb, style, polygon_wkb, available_period,
                                     opening_hours, email, phone, website, social_media, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)"#,
        )
        .bind(entity.to_string())
        .bind(item.area_id)
        .bind(&item.name)
        .bind(&item.description)
        .bind(&item.chain)
        .bind(&item.beacon_code)
        .bind(type_json)
        .bind(&item.color)
        .bind(tags_json)
        .bind(location_wkb)
        .bind(item.style.to_string())
        .bind(polygon_wkb)
        .bind(available_period_json)
        .bind(opening_hours_json)
        .bind(&item.email)
        .bind(&item.phone)
        .bind(&item.website)
        .bind(social_media_json)
        .bind(item.created_at.unwrap_or(now))
        .bind(item.updated_at.unwrap_or(now))
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn get_by_id(
        pool: &sqlx::SqlitePool,
        id: i32,
        entity: uuid::Uuid,
    ) -> sqlx::Result<Option<Self>> {
        use sqlx::Row;

        let row = sqlx::query(
            r#"SELECT id, entity_id, area_id, name, description, chain, beacon_code, type,
                      color, tags, location_wkb, style, polygon_wkb, available_period, opening_hours,
                      email, phone, website, social_media, created_at, updated_at
               FROM merchants WHERE id = ?1 AND entity_id = ?2"#,
        )
        .bind(id)
        .bind(entity.to_string())
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => {
                let location = wkb_to_point(row.get::<Vec<u8>, _>("location_wkb").as_slice())
                    .map_err(|e| sqlx::Error::Decode(format!("WKB decode: {}", e).into()))?;
                let polygon = wkb_to_polygon(row.get::<Vec<u8>, _>("polygon_wkb").as_slice())
                    .map_err(|e| sqlx::Error::Decode(format!("WKB decode: {}", e).into()))?;
                let r#type: MerchantType = serde_json::from_str(&row.get::<String, _>("type"))
                    .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
                let tags: Vec<String> = serde_json::from_str(&row.get::<String, _>("tags"))
                    .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
                let available_period: Option<Vec<(i64, i64)>> =
                    serde_json::from_str(&row.get::<String, _>("available_period"))
                        .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
                let opening_hours: Option<Vec<Vec<(i32, i32)>>> =
                    serde_json::from_str(&row.get::<String, _>("opening_hours"))
                        .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
                let social_media: Option<Vec<SocialMedia>> =
                    serde_json::from_str(&row.get::<String, _>("social_media"))
                        .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
                let style = match row.get::<String, _>("style").as_str() {
                    "store" => MerchantStyle::Store,
                    "kiosk" => MerchantStyle::Kiosk,
                    "pop-up" => MerchantStyle::PopUp,
                    "food-truck" => MerchantStyle::FoodTruck,
                    "room" => MerchantStyle::Room,
                    _ => MerchantStyle::Store,
                };

                Ok(Some(Merchant {
                    id: row.get("id"),
                    name: row.get("name"),
                    description: row.get("description"),
                    chain: row.get("chain"),
                    entity_id: row.get("entity_id"),
                    beacon_code: row.get("beacon_code"),
                    area_id: row.get("area_id"),
                    r#type,
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
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }))
            }
            None => Ok(None),
        }
    }

    async fn update(pool: &sqlx::SqlitePool, item: &Self, entity: uuid::Uuid) -> sqlx::Result<()> {
        let location_wkb = point_to_wkb(item.location)
            .map_err(|e| sqlx::Error::Encode(format!("WKB encode: {}", e).into()))?;
        let polygon_wkb = polygon_to_wkb(&item.polygon)
            .map_err(|e| sqlx::Error::Encode(format!("WKB encode: {}", e).into()))?;
        let type_json = serde_json::to_string(&item.r#type)
            .map_err(|e| sqlx::Error::Encode(format!("JSON encode: {}", e).into()))?;
        let tags_json = serde_json::to_string(&item.tags)
            .map_err(|e| sqlx::Error::Encode(format!("JSON encode: {}", e).into()))?;
        let available_period_json = serde_json::to_string(&item.available_period)
            .map_err(|e| sqlx::Error::Encode(format!("JSON encode: {}", e).into()))?;
        let opening_hours_json = serde_json::to_string(&item.opening_hours)
            .map_err(|e| sqlx::Error::Encode(format!("JSON encode: {}", e).into()))?;
        let social_media_json = serde_json::to_string(&item.social_media)
            .map_err(|e| sqlx::Error::Encode(format!("JSON encode: {}", e).into()))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        sqlx::query(
            r#"UPDATE merchants
               SET area_id = ?3, name = ?4, description = ?5, chain = ?6, beacon_code = ?7,
                   type = ?8, color = ?9, tags = ?10, location_wkb = ?11, style = ?12, polygon_wkb = ?13,
                   available_period = ?14, opening_hours = ?15, email = ?16, phone = ?17,
                   website = ?18, social_media = ?19, updated_at = ?20
               WHERE id = ?1 AND entity_id = ?2"#,
        )
        .bind(item.id)
        .bind(entity.to_string())
        .bind(item.area_id)
        .bind(&item.name)
        .bind(&item.description)
        .bind(&item.chain)
        .bind(&item.beacon_code)
        .bind(type_json)
        .bind(&item.color)
        .bind(tags_json)
        .bind(location_wkb)
        .bind(item.style.to_string())
        .bind(polygon_wkb)
        .bind(available_period_json)
        .bind(opening_hours_json)
        .bind(&item.email)
        .bind(&item.phone)
        .bind(&item.website)
        .bind(social_media_json)
        .bind(now)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn delete(pool: &sqlx::SqlitePool, id: i32, entity: uuid::Uuid) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM merchants WHERE id = ?1 AND entity_id = ?2")
            .bind(id)
            .bind(entity.to_string())
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn list(
        pool: &sqlx::SqlitePool,
        offset: i64,
        limit: i64,
        entity: uuid::Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        use sqlx::Row;

        let rows = sqlx::query(
            r#"SELECT id, entity_id, area_id, name, description, chain, beacon_code, type,
                      color, tags, location_wkb, style, polygon_wkb, available_period, opening_hours,
                      email, phone, website, social_media, created_at, updated_at
               FROM merchants WHERE entity_id = ?1
               ORDER BY created_at DESC
               LIMIT ?2 OFFSET ?3"#,
        )
        .bind(entity.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let mut merchants = Vec::new();
        for row in rows {
            let location = wkb_to_point(row.get::<Vec<u8>, _>("location_wkb").as_slice())
                .map_err(|e| sqlx::Error::Decode(format!("WKB decode: {}", e).into()))?;
            let polygon = wkb_to_polygon(row.get::<Vec<u8>, _>("polygon_wkb").as_slice())
                .map_err(|e| sqlx::Error::Decode(format!("WKB decode: {}", e).into()))?;
            let r#type: MerchantType = serde_json::from_str(&row.get::<String, _>("type"))
                .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
            let tags: Vec<String> = serde_json::from_str(&row.get::<String, _>("tags"))
                .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
            let available_period: Option<Vec<(i64, i64)>> =
                serde_json::from_str(&row.get::<String, _>("available_period"))
                    .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
            let opening_hours: Option<Vec<Vec<(i32, i32)>>> =
                serde_json::from_str(&row.get::<String, _>("opening_hours"))
                    .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
            let social_media: Option<Vec<SocialMedia>> =
                serde_json::from_str(&row.get::<String, _>("social_media"))
                    .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
            let style = match row.get::<String, _>("style").as_str() {
                "store" => MerchantStyle::Store,
                "kiosk" => MerchantStyle::Kiosk,
                "pop-up" => MerchantStyle::PopUp,
                "food-truck" => MerchantStyle::FoodTruck,
                "room" => MerchantStyle::Room,
                _ => MerchantStyle::Store,
            };

            merchants.push(Merchant {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                chain: row.get("chain"),
                entity_id: row.get("entity_id"),
                beacon_code: row.get("beacon_code"),
                area_id: row.get("area_id"),
                r#type,
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
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(merchants)
    }

    async fn search(
        pool: &sqlx::SqlitePool,
        query: &str,
        case_insensitive: bool,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
        entity: uuid::Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        use sqlx::Row;

        let like_pattern = format!("%{}%", query);
        let order_by = sort.unwrap_or("created_at");
        let direction = if asc { "ASC" } else { "DESC" };

        let sql = if case_insensitive {
            format!(
                r#"SELECT id, entity_id, area_id, name, description, chain, beacon_code, type,
                          color, tags, location_wkb, style, polygon_wkb, available_period, opening_hours,
                          email, phone, website, social_media, created_at, updated_at
                   FROM merchants
                   WHERE entity_id = ?1 AND (name LIKE ?2 COLLATE NOCASE OR description LIKE ?2 COLLATE NOCASE OR beacon_code LIKE ?2 COLLATE NOCASE)
                   ORDER BY {} {}
                   LIMIT ?3 OFFSET ?4"#,
                order_by, direction
            )
        } else {
            format!(
                r#"SELECT id, entity_id, area_id, name, description, chain, beacon_code, type,
                          color, tags, location_wkb, style, polygon_wkb, available_period, opening_hours,
                          email, phone, website, social_media, created_at, updated_at
                   FROM merchants
                   WHERE entity_id = ?1 AND (name LIKE ?2 OR description LIKE ?2 OR beacon_code LIKE ?2)
                   ORDER BY {} {}
                   LIMIT ?3 OFFSET ?4"#,
                order_by, direction
            )
        };

        let rows = sqlx::query(&sql)
            .bind(entity.to_string())
            .bind(&like_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        let mut merchants = Vec::new();
        for row in rows {
            let location = wkb_to_point(row.get::<Vec<u8>, _>("location_wkb").as_slice())
                .map_err(|e| sqlx::Error::Decode(format!("WKB decode: {}", e).into()))?;
            let polygon = wkb_to_polygon(row.get::<Vec<u8>, _>("polygon_wkb").as_slice())
                .map_err(|e| sqlx::Error::Decode(format!("WKB decode: {}", e).into()))?;
            let r#type: MerchantType = serde_json::from_str(&row.get::<String, _>("type"))
                .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
            let tags: Vec<String> = serde_json::from_str(&row.get::<String, _>("tags"))
                .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
            let available_period: Option<Vec<(i64, i64)>> =
                serde_json::from_str(&row.get::<String, _>("available_period"))
                    .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
            let opening_hours: Option<Vec<Vec<(i32, i32)>>> =
                serde_json::from_str(&row.get::<String, _>("opening_hours"))
                    .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
            let social_media: Option<Vec<SocialMedia>> =
                serde_json::from_str(&row.get::<String, _>("social_media"))
                    .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
            let style = match row.get::<String, _>("style").as_str() {
                "store" => MerchantStyle::Store,
                "kiosk" => MerchantStyle::Kiosk,
                "pop-up" => MerchantStyle::PopUp,
                "food-truck" => MerchantStyle::FoodTruck,
                "room" => MerchantStyle::Room,
                _ => MerchantStyle::Store,
            };

            merchants.push(Merchant {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                chain: row.get("chain"),
                entity_id: row.get("entity_id"),
                beacon_code: row.get("beacon_code"),
                area_id: row.get("area_id"),
                r#type,
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
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(merchants)
    }
}

#[cfg(all(not(feature = "postgres"), feature = "sql", feature = "geo"))]
#[async_trait::async_trait]
impl IntRepositoryInArea for Merchant {
    async fn search_in_area(
        pool: &sqlx::SqlitePool,
        query: &str,
        case_insensitive: bool,
        offset: i64,
        limit: i64,
        sort: Option<&str>,
        asc: bool,
        area: i32,
        entity: uuid::Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        use sqlx::Row;

        let like_pattern = format!("%{}%", query);
        let order_by = sort.unwrap_or("created_at");
        let direction = if asc { "ASC" } else { "DESC" };

        let sql = if case_insensitive {
            format!(
                r#"SELECT id, entity_id, area_id, name, description, chain, beacon_code, type,
                          color, tags, location_wkb, style, polygon_wkb, available_period, opening_hours,
                          email, phone, website, social_media, created_at, updated_at
                   FROM merchants
                   WHERE entity_id = ?1 AND area_id = ?2 AND (name LIKE ?3 COLLATE NOCASE OR description LIKE ?3 COLLATE NOCASE OR beacon_code LIKE ?3 COLLATE NOCASE)
                   ORDER BY {} {}
                   LIMIT ?4 OFFSET ?5"#,
                order_by, direction
            )
        } else {
            format!(
                r#"SELECT id, entity_id, area_id, name, description, chain, beacon_code, type,
                          color, tags, location_wkb, style, polygon_wkb, available_period, opening_hours,
                          email, phone, website, social_media, created_at, updated_at
                   FROM merchants
                   WHERE entity_id = ?1 AND area_id = ?2 AND (name LIKE ?3 OR description LIKE ?3 OR beacon_code LIKE ?3)
                   ORDER BY {} {}
                   LIMIT ?4 OFFSET ?5"#,
                order_by, direction
            )
        };

        let rows = sqlx::query(&sql)
            .bind(entity.to_string())
            .bind(area)
            .bind(&like_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        let mut merchants = Vec::new();
        for row in rows {
            let location = wkb_to_point(row.get::<Vec<u8>, _>("location_wkb").as_slice())
                .map_err(|e| sqlx::Error::Decode(format!("WKB decode: {}", e).into()))?;
            let polygon = wkb_to_polygon(row.get::<Vec<u8>, _>("polygon_wkb").as_slice())
                .map_err(|e| sqlx::Error::Decode(format!("WKB decode: {}", e).into()))?;
            let r#type: MerchantType = serde_json::from_str(&row.get::<String, _>("type"))
                .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
            let tags: Vec<String> = serde_json::from_str(&row.get::<String, _>("tags"))
                .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
            let available_period: Option<Vec<(i64, i64)>> =
                serde_json::from_str(&row.get::<String, _>("available_period"))
                    .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
            let opening_hours: Option<Vec<Vec<(i32, i32)>>> =
                serde_json::from_str(&row.get::<String, _>("opening_hours"))
                    .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
            let social_media: Option<Vec<SocialMedia>> =
                serde_json::from_str(&row.get::<String, _>("social_media"))
                    .map_err(|e| sqlx::Error::Decode(format!("JSON decode: {}", e).into()))?;
            let style = match row.get::<String, _>("style").as_str() {
                "store" => MerchantStyle::Store,
                "kiosk" => MerchantStyle::Kiosk,
                "pop-up" => MerchantStyle::PopUp,
                "food-truck" => MerchantStyle::FoodTruck,
                "room" => MerchantStyle::Room,
                _ => MerchantStyle::Store,
            };

            merchants.push(Merchant {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                chain: row.get("chain"),
                entity_id: row.get("entity_id"),
                beacon_code: row.get("beacon_code"),
                area_id: row.get("area_id"),
                r#type,
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
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(merchants)
    }
}
