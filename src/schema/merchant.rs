use crate::schema::polygon::line::Path;
use crate::schema::service::Service;
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Merchant {
    #[serde(rename = "_id")]
    id: ObjectId,
    name: String,
    description: Option<String>,
    chain: Option<String>, // Name of the chain if part of a chain store series
    entity: ObjectId,      // Reference to the Entity
    beacon_code: String,   // Unique identifier for the merchant for displaying in the beacon name
    area: ObjectId,
    r#type: MerchantType,
    /// Hex color code for UI representation,
    /// e.g., `#00704a` for Starbucks green,
    /// whereas `#ffc72c` for McDonald's yellow.
    color: Option<String>,
    /// List of tags for categorization, e.g., "food", "electronics", "clothing"
    /// Tags can be used for search and filtering
    tags: Vec<String>,
    location: (f64, f64),
    style: MerchantStyle,
    polygon: Vec<(f64, f64)>,          // List of (x, y) pairs of coordinates
    available_period: Vec<(i64, i64)>, // List of (start_time, end_time) in milliseconds on a 24-hour clock
    email: Option<String>,
    phone: Option<String>,
    website: Option<String>,
    social_media: Vec<SocialMedia>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    Other, // For any other type not listed
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum FacilityType {
    Restroom,
    Atm,
    InformationDesk,
    Other, // For any other type not listed
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum FoodType {
    Restaurant(FoodCuisine),
    Cafe,
    FastFood,
    Bakery,
    Bar,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
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
    Other, // For any other type not listed
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum MerchantStyle {
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
