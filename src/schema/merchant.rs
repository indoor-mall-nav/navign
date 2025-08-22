use crate::schema::service::Service;
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Merchant {
    _id: ObjectId,
    name: String,
    description: Option<String>,
    chain: Option<String>, // Name of the chain if part of a chain store series
    entity: ObjectId,      // Reference to the Entity
    beacon_code: String,   // Unique identifier for the merchant for displaying in the beacon name
    area: ObjectId,
    r#type: MerchantType,
    /// List of tags for categorization, e.g., "food", "electronics", "clothing"
    /// Tags can be used for search and filtering
    tags: Vec<String>,
    location: (f64, f64),
    style: MerchantStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MerchantType {
    Food {
        /// Type of cuisine, e.g., `Italian`, `Chinese`.
        cuisine: Option<FoodCuisine>,
        /// Type of food establishment, e.g., `Restaurant`, `Cafe`.
        r#type: FoodType,
    },
    Electronics {
        is_mobile: bool,      // Whether it specializes in mobile devices
        is_computer: bool,    // Whether it specializes in computers
        is_accessories: bool, // Whether it sells accessories
    },
    Clothing {
        is_menswear: bool,      // Whether it's menswear
        is_womenswear: bool,    // Whether it's womenswear
        is_childrenswear: bool, // Whether it's childrenswear
    },
    Supermarket,
    Health,
    Entertainment,
    Service,
    /// The room is, for example, a hotel room, office room, or meeting room.
    /// It may or may not use authentication or access control, but it only has several doors
    /// that can be used to enter or exit the room.
    Room,
    Other, // For any other type not listed
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FoodType {
    Restaurant(FoodCuisine),
    Cafe,
    FastFood,
    Bakery,
    Bar,
    Other, // For any other type not listed
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub enum MerchantStyle {
    Store,
    Kiosk,
    PopUp,
    FoodTruck,
    Room,
}

impl Service for Merchant {
    fn get_id(&self) -> String {
        self._id.to_hex()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_id(&mut self, id: String) {
        self._id = ObjectId::parse_str(&id).expect("Invalid ObjectId format");
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
