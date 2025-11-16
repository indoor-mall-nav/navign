use crate::schema::service::{OneInArea, Service};

// Re-export from navign-shared
pub use navign_shared::{
    ChineseFoodCuisine, FacilityType, FoodCuisine, FoodType, Merchant, MerchantStyle, MerchantType,
    SocialMedia, SocialMediaPlatform,
};

impl Service for Merchant {
    fn get_id(&self) -> String {
        self.id.clone()
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

impl OneInArea for Merchant {}
