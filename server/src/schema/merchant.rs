use crate::schema::service::{OneInArea, Service};
use uuid::Uuid;

// Re-export from navign-shared
pub use navign_shared::{
    ChineseFoodCuisine, FacilityType, FoodCuisine, FoodType, Merchant, MerchantStyle, MerchantType,
    SocialMedia, SocialMediaPlatform,
};

impl Service for Merchant {
    type Id = Uuid;

    fn get_id(&self) -> Uuid {
        self.id.expect("Merchant must have an ID")
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

    fn get_table_name() -> &'static str {
        "merchants"
    }

    fn require_unique_name() -> bool {
        false
    }
}

impl OneInArea for Merchant {}
