#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "mongodb")]
use bson::oid::ObjectId;

/// Merchant schema - represents a shop, store, or service location
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "mongodb", derive(Default))]
pub struct Merchant {
    #[cfg_attr(feature = "serde", serde(rename = "_id"))]
    #[cfg(feature = "mongodb")]
    pub id: ObjectId,
    #[cfg(not(feature = "mongodb"))]
    pub id: String,
    
    pub name: String,
    pub description: Option<String>,
    pub chain: Option<String>,
    
    #[cfg(feature = "mongodb")]
    pub entity: ObjectId,
    #[cfg(not(feature = "mongodb"))]
    pub entity: String,
    
    pub beacon_code: String,
    
    #[cfg(feature = "mongodb")]
    pub area: ObjectId,
    #[cfg(not(feature = "mongodb"))]
    pub area: String,
    
    pub r#type: MerchantType,
    pub color: Option<String>,
    pub tags: Vec<String>,
    pub location: (f64, f64),
    pub style: MerchantStyle,
    pub polygon: Option<Vec<(f64, f64)>>,
    pub available_period: Option<Vec<(i64, i64)>>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub social_media: Option<Vec<SocialMedia>>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "mongodb", derive(Default))]
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
    #[cfg_attr(feature = "mongodb", default)]
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
#[cfg_attr(feature = "mongodb", derive(Default))]
pub enum FacilityType {
    Restroom,
    Atm,
    InformationDesk,
    #[cfg_attr(feature = "mongodb", default)]
    Other,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "mongodb", derive(Default))]
pub enum FoodType {
    Restaurant(FoodCuisine),
    Cafe,
    FastFood,
    Bakery,
    Bar,
    #[cfg_attr(feature = "mongodb", default)]
    Other,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
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

#[cfg(feature = "mongodb")]
impl Default for FoodCuisine {
    fn default() -> Self {
        FoodCuisine::Other(String::from("Unknown"))
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "mongodb", derive(Default))]
pub enum ChineseFoodCuisine {
    Cantonese,
    Sichuan,
    Hunan,
    Jiangxi,
    Shanghai,
    Hangzhou,
    Ningbo,
    Northern,
    #[cfg_attr(feature = "mongodb", default)]
    Other,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "mongodb", derive(Default))]
pub struct SocialMedia {
    pub platform: SocialMediaPlatform,
    pub handle: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
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

#[cfg(feature = "mongodb")]
impl Default for SocialMediaPlatform {
    fn default() -> Self {
        SocialMediaPlatform::Other(String::from("Unknown"))
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "mongodb", derive(Default))]
pub enum MerchantStyle {
    #[cfg_attr(feature = "mongodb", default)]
    Store,
    Kiosk,
    PopUp,
    FoodTruck,
    Room,
}

// Mobile-specific version for SQLite storage
#[cfg(feature = "sql")]
pub mod mobile {
    #[cfg(feature = "alloc")]
    use alloc::string::String;
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;
    use sqlx::FromRow;
    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, FromRow)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct MerchantMobile {
        pub id: String,
        pub name: String,
        pub description: Option<String>,
        pub chain: Option<String>,
        pub entity: String,
        pub beacon_code: String,
        pub area: String,
        pub r#type: String, // JSON serialized MerchantType
        pub color: Option<String>,
        pub tags: String, // JSON array
        /// Stored as WKT POINT string
        pub location: String,
        pub style: String,
        /// Stored as WKT POLYGON string
        pub polygon: Option<String>,
        pub available_period: Option<String>, // JSON array
        pub email: Option<String>,
        pub phone: Option<String>,
        pub website: Option<String>,
        pub social_media: Option<String>, // JSON array
    }

    impl MerchantMobile {
        #[cfg(feature = "sql")]
        pub async fn create_table(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
            sqlx::query(
                r#"
                CREATE TABLE IF NOT EXISTS merchants (
                    id VARCHAR(24) PRIMARY KEY,
                    name TEXT NOT NULL,
                    description TEXT,
                    chain TEXT,
                    entity VARCHAR(24) NOT NULL,
                    beacon_code TEXT NOT NULL,
                    area VARCHAR(24) NOT NULL,
                    type TEXT NOT NULL,
                    color TEXT,
                    tags TEXT NOT NULL,
                    location TEXT NOT NULL,
                    style TEXT NOT NULL,
                    polygon TEXT,
                    available_period TEXT,
                    email TEXT,
                    phone TEXT,
                    website TEXT,
                    social_media TEXT
                )
                "#,
            )
            .execute(pool)
            .await?;
            Ok(())
        }

        #[cfg(feature = "sql")]
        pub async fn insert(&self, pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO merchants 
                (id, name, description, chain, entity, beacon_code, area, type, color, tags, 
                 location, style, polygon, available_period, email, phone, website, social_media)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&self.id)
            .bind(&self.name)
            .bind(&self.description)
            .bind(&self.chain)
            .bind(&self.entity)
            .bind(&self.beacon_code)
            .bind(&self.area)
            .bind(&self.r#type)
            .bind(&self.color)
            .bind(&self.tags)
            .bind(&self.location)
            .bind(&self.style)
            .bind(&self.polygon)
            .bind(&self.available_period)
            .bind(&self.email)
            .bind(&self.phone)
            .bind(&self.website)
            .bind(&self.social_media)
            .execute(pool)
            .await?;
            Ok(())
        }

        #[cfg(feature = "sql")]
        pub async fn get_by_id(pool: &sqlx::SqlitePool, id: &str) -> Result<Option<Self>, sqlx::Error> {
            sqlx::query_as::<_, Self>("SELECT * FROM merchants WHERE id = ?")
                .bind(id)
                .fetch_optional(pool)
                .await
        }

        #[cfg(feature = "sql")]
        pub async fn get_all(pool: &sqlx::SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
            sqlx::query_as::<_, Self>("SELECT * FROM merchants")
                .fetch_all(pool)
                .await
        }

        #[cfg(feature = "sql")]
        pub async fn get_by_area(pool: &sqlx::SqlitePool, area: &str) -> Result<Vec<Self>, sqlx::Error> {
            sqlx::query_as::<_, Self>("SELECT * FROM merchants WHERE area = ?")
                .bind(area)
                .fetch_all(pool)
                .await
        }

        #[cfg(feature = "sql")]
        pub async fn delete(pool: &sqlx::SqlitePool, id: &str) -> Result<(), sqlx::Error> {
            sqlx::query("DELETE FROM merchants WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await?;
            Ok(())
        }
    }
}
