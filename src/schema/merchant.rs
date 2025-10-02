use std::path::Display;
use crate::schema::service::Service;
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Merchant {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub description: Option<String>,
    pub chain: Option<String>, // Name of the chain if part of a chain store series
    pub entity: ObjectId,      // Reference to the Entity
    pub beacon_code: String, // Unique identifier for the merchant for displaying in the beacon name
    pub area: ObjectId,
    pub r#type: MerchantType,
    /// Hex color code for UI representation,
    /// e.g., `#00704a` for Starbucks green,
    /// whereas `#ffc72c` for McDonald's yellow.
    pub color: Option<String>,
    /// List of tags for categorization, e.g., "food", "electronics", "clothing"
    /// Tags can be used for search and filtering
    pub tags: Vec<String>,
    pub location: (f64, f64),
    pub style: MerchantStyle,
    pub polygon: Option<Vec<(f64, f64)>>, // List of (x, y) pairs of coordinates
    pub available_period: Option<Vec<(i64, i64)>>, // List of (start_time, end_time) in milliseconds on a 24-hour clock
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub social_media: Option<Vec<SocialMedia>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum MerchantType {
    Food {
        /// Type of cuisine, e.g., `Italian`, `Chinese`.
        cuisine: Option<FoodCuisine>,
        /// Type of food establishment, e.g., `Restaurant`, `Cafe`.
        r#type: FoodType,
    },
    Electronics {
        mobile: bool,      // Whether it sells in mobile devices
        computer: bool,    // Whether it sells in computers
        accessories: bool, // Whether it sells accessories
    },
    Clothing {
        menswear: bool,      // Whether it's menswear
        womenswear: bool,    // Whether it's womenswear
        childrenswear: bool, // Whether it's childrenswear
    },
    Supermarket,
    Health,
    Entertainment,
    Facility {
        /// Type of facility, e.g., `Restroom`, `ATM`.
        r#type: FacilityType,
    },
    /// The room is, for example, a hotel room, office room, or meeting room.
    /// It may or may not use authentication or access control, but it only has several doors
    /// that can be used to enter or exit the room.
    Room,
    #[default]
    Other, // For any other type not listed
}

impl std::fmt::Display for MerchantType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
                let mut types = vec![];
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
                let mut types = vec![];
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum FacilityType {
    Restroom,
    Atm,
    InformationDesk,
    #[default]
    Other, // For any other type not listed
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum FoodType {
    Restaurant(FoodCuisine),
    Cafe,
    FastFood,
    Bakery,
    Bar,
    #[default]
    Other, // For any other type not listed
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum FoodCuisine {
    Italian,
    Chinese {
        cuisine: ChineseFoodCuisine, // Specific type of Chinese cuisine
        specific: Option<String>,    // Specific dish or style, e.g., "Dim Sum", "Ningbo Cuisine"
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
    Other(String), // For any other type not listed
}

impl Default for FoodCuisine {
    fn default() -> Self {
        FoodCuisine::Other("Unknown".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ChineseFoodCuisine {
    Cantonese,
    Sichuan,
    Hunan,
    Jiangxi,
    Shanghai,
    Hangzhou,
    Ningbo,
    Northern,
    #[default]
    Other, // For any other type not listed
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SocialMedia {
    platform: SocialMediaPlatform,
    handle: String,      // e.g., @starbucks
    url: Option<String>, // e.g., https://www.instagram.com/starbucks
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
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

impl Default for SocialMediaPlatform {
    fn default() -> Self {
        SocialMediaPlatform::Other("Unknown".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy, Default)]
#[serde(rename_all = "kebab-case")]
pub enum MerchantStyle {
    #[default]
    Store,
    Kiosk,
    PopUp,
    FoodTruck,
    Room,
}

impl Service for Merchant {
    fn get_id(&self) -> String {
        self.id.to_hex()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn get_description(&self) -> Option<String> {
        self.description.clone()
    }

    fn set_description(&mut self, description: Option<String>) {
        self.description = description;
    }

    fn get_collection_name() -> &'static str {
        "merchants"
    }

    fn require_unique_name() -> bool {
        false
    }
}
