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
pub struct Merchant {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub r#chain: Option<String>,
    #[cfg(feature = "postgres")]
    pub entity: sqlx::types::Uuid,
    #[cfg(not(feature = "postgres"))]
    #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
    pub entity: String,
    pub beacon_code: String,
    pub area: i32,
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
pub enum MerchantStyle {
    Store,
    Kiosk,
    PopUp,
    FoodTruck,
    Room,
}
