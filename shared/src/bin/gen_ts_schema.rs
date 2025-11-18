#![deny(clippy::all)]

//! TypeScript schema generator for Navign
//!
//! This binary generates TypeScript type definitions from Rust types in the shared crate.
//! It uses ts-rs to automatically generate .d.ts files.
//!
//! ## Usage
//!
//! Run the generation by executing:
//! ```bash
//! cargo run --bin gen-ts-schema --features ts-rs
//! ```
//!
//! This will generate TypeScript definitions in `./bindings/generated/`

use navign_shared::schema::*;
use ts_rs::TS;

fn main() {
    // Generate all types - export_to paths are set in the type definitions
    Entity::export_all().expect("Failed to export Entity");
    EntityType::export_all().expect("Failed to export EntityType");

    Area::export_all().expect("Failed to export Area");
    Floor::export_all().expect("Failed to export Floor");
    FloorType::export_all().expect("Failed to export FloorType");

    Beacon::export_all().expect("Failed to export Beacon");
    BeaconDevice::export_all().expect("Failed to export BeaconDevice");
    BeaconType::export_all().expect("Failed to export BeaconType");

    Merchant::export_all().expect("Failed to export Merchant");
    MerchantType::export_all().expect("Failed to export MerchantType");
    MerchantStyle::export_all().expect("Failed to export MerchantStyle");
    FoodType::export_all().expect("Failed to export FoodType");
    FoodCuisine::export_all().expect("Failed to export FoodCuisine");
    ChineseFoodCuisine::export_all().expect("Failed to export ChineseFoodCuisine");
    FacilityType::export_all().expect("Failed to export FacilityType");
    SocialMedia::export_all().expect("Failed to export SocialMedia");
    SocialMediaPlatform::export_all().expect("Failed to export SocialMediaPlatform");

    Connection::export_all().expect("Failed to export Connection");
    ConnectionType::export_all().expect("Failed to export ConnectionType");

    println!("✓ Generated all TypeScript definitions");
}
