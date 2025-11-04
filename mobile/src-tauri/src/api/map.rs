use crate::api::page_results::PaginationResponse;
use crate::api::unlocker::CustomizedObjectId;
use crate::locate::merchant::Merchant;
use crate::shared::BASE_URL;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::fmt::Display;
use tauri::AppHandle;
use tauri_plugin_http::reqwest;
use tauri_plugin_log::log::trace;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapArea {
    pub id: String,
    pub name: String,
    pub polygon: Vec<(f64, f64)>,
    pub beacons: Vec<MapBeacon>,
    pub merchants: Vec<MapMerchant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapBeacon {
    pub id: String,
    pub name: String,
    pub location: (f64, f64),
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapMerchant {
    pub id: String,
    pub name: String,
    pub location: (f64, f64),
    pub polygon: Vec<(f64, f64)>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaResponse {
    #[serde(rename = "_id")]
    pub id: CustomizedObjectId,
    pub entity: CustomizedObjectId,
    pub name: String,
    pub description: Option<String>,
    pub beacon_code: String,
    pub floor: Option<Floor>,
    pub polygon: Vec<(f64, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Floor {
    pub r#type: String,
    pub name: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconResponse {
    #[serde(rename = "_id")]
    pub id: CustomizedObjectId,
    pub entity: CustomizedObjectId,
    pub area: CustomizedObjectId,
    pub merchant: Option<CustomizedObjectId>,
    pub connection: Option<CustomizedObjectId>,
    pub name: String,
    pub description: Option<String>,
    pub r#type: String,
    pub location: (f64, f64),
    pub device: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantResponse {
    #[serde(rename = "_id")]
    pub id: CustomizedObjectId,
    pub name: String,
    pub description: Option<String>,
    pub chain: Option<String>,
    pub entity: CustomizedObjectId,
    pub beacon_code: String,
    pub area: CustomizedObjectId,
    pub location: (f64, f64),
    pub polygon: Option<Vec<(f64, f64)>>,
    pub tags: Vec<String>,
    pub r#type: Value,
    pub style: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub social_media: Option<Vec<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRequest {
    pub from: String,             // merchant/area id
    pub to: String,               // merchant/area id
    pub disallow: Option<String>, // "e" (elevator), "s" (stairs), "c" (escalator)
}

impl Display for RouteRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "?from={}&to={}", self.from, self.to)?;
        if let Some(disallow) = &self.disallow {
            write!(f, "&disallow={}", disallow)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResponse {
    pub instructions: Vec<InstructionType>,
    pub total_distance: Option<f64>,
    pub areas: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, Eq, Copy)]
#[serde(rename_all = "kebab-case")]
/// Represents the type of connection between areas or entities.
pub enum ConnectionType {
    /// A connection that allows people to pass through, such as a door or gate.
    /// Usually involve authentication or access control.
    Gate,
    /// A connection that allows people to move between different areas, such as a hallway or corridor.
    #[default]
    Escalator,
    /// A connection that allows people to move between different levels, such as stairs or elevators.
    Elevator,
    /// A connection that allows people to move between different areas, such as a pathway or tunnel.
    Stairs,
    /// Like in Hong Kong International Airport, Singapore Changi Airport, or Shanghai Pudong International Airport.
    /// There is a dedicated transportation system that connects different terminals or areas.
    Rail,
    /// Shuttle bus.
    Shuttle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstructionType {
    Move(f64, f64),
    Transport(String, String, ConnectionType),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityLimits {
    pub elevator: bool,
    pub stairs: bool,
    pub escalator: bool,
}

impl Default for ConnectivityLimits {
    fn default() -> Self {
        Self {
            elevator: true,
            stairs: true,
            escalator: true,
        }
    }
}

/// Fetch map data for a specific area including beacons and merchants
pub async fn fetch_map_data(entity: &str, area: &str) -> anyhow::Result<MapArea> {
    let client = reqwest::Client::new();

    // Fetch area data
    let area_url = format!("{}api/entities/{}/areas/{}", BASE_URL, entity, area);
    trace!("Fetching area from URL: {}", area_url);
    let area_response: AreaResponse = client
        .get(&area_url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch area: {}", e))?
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse area: {}", e))?;

    trace!(
        "Fetched area: {} with ID {}",
        area_response.name, area_response.id
    );

    // Fetch beacons in the area
    let beacons_url = format!("{}/beacons", area_url);
    trace!("Fetching beacons from URL: {}", beacons_url);
    let beacons_response: PaginationResponse<BeaconResponse> = client
        .get(&beacons_url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch beacons: {}", e))?
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse beacons: {}", e))?;

    trace!("Fetched {} beacons", beacons_response.data.len());

    let map_beacons: Vec<MapBeacon> = beacons_response
        .data
        .into_iter()
        .map(|b| MapBeacon {
            id: b.id.to_string(),
            name: b.name,
            location: b.location,
            r#type: b.r#type,
        })
        .collect();

    trace!("Mapped {} beacons", map_beacons.len());

    // Fetch merchants in the area
    let merchants_url = format!("{}/merchants", area_url);
    trace!("Fetching merchants from URL: {}", merchants_url);
    let merchants_response: PaginationResponse<MerchantResponse> = client
        .get(&merchants_url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch merchants: {}", e))?
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse merchants: {}", e))?;

    trace!("Fetched {} merchants", merchants_response.data.len());

    let map_merchants: Vec<MapMerchant> = merchants_response
        .data
        .into_iter()
        .map(|m| MapMerchant {
            id: m.id.to_string(),
            name: m.name,
            location: m.location,
            polygon: m.polygon.unwrap_or_default(),
            tags: m.tags,
        })
        .collect();

    trace!("Mapped {} merchants", map_merchants.len());

    Ok(MapArea {
        id: area_response.id.to_string(),
        name: area_response.name,
        polygon: area_response.polygon,
        beacons: map_beacons,
        merchants: map_merchants,
    })
}

pub async fn get_all_merchants(entity: &str) -> anyhow::Result<Vec<MerchantResponse>> {
    let client = reqwest::Client::new();
    let url = format!("{}api/entities/{}/merchants?limit=1000", BASE_URL, entity);
    trace!("Fetching all merchants from URL: {}", url);
    let response: PaginationResponse<MerchantResponse> = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch merchants: {}", e))?
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse merchants: {}", e))?;
    Ok(response.data)
}

/// Fetch detailed information for a specific area
pub async fn fetch_area_details(entity: &str, area: &str) -> anyhow::Result<AreaResponse> {
    let client = reqwest::Client::new();
    let url = format!("{}api/entities/{}/areas/{}", BASE_URL, entity, area);
    trace!("Fetching area details from URL: {}", url);
    
    let response: AreaResponse = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch area details: {}", e))?
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse area details: {}", e))?;
    
    Ok(response)
}

/// Fetch detailed information for a specific merchant
pub async fn fetch_merchant_details(entity: &str, merchant: &str) -> anyhow::Result<MerchantResponse> {
    let client = reqwest::Client::new();
    let url = format!("{}api/entities/{}/merchants/{}", BASE_URL, entity, merchant);
    trace!("Fetching merchant details from URL: {}", url);
    
    let response: MerchantResponse = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch merchant details: {}", e))?
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse merchant details: {}", e))?;
    
    Ok(response)
}

/// Generate SVG map representation of the area
pub fn generate_svg_map(map_data: &MapArea, width: u32, height: u32) -> String {
    let mut svg = format!(
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
        width, height
    );

    // Calculate bounds for scaling
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;

    for (x, y) in &map_data.polygon {
        min_x = min_x.min(*x);
        max_x = max_x.max(*x);
        min_y = min_y.min(*y);
        max_y = max_y.max(*y);
    }

    let scale_x = (width as f64 - 20.0) / (max_x - min_x);
    let scale_y = (height as f64 - 20.0) / (max_y - min_y);
    let scale = scale_x.min(scale_y);

    let transform =
        |x: f64, y: f64| -> (f64, f64) { ((x - min_x) * scale + 10.0, (y - min_y) * scale + 10.0) };

    // Draw area polygon
    svg.push_str(r#"<g id="area-boundary">"#);
    svg.push_str(r#"<polygon points=""#);
    for (x, y) in &map_data.polygon {
        let (tx, ty) = transform(*x, *y);
        svg.push_str(&format!("{},{} ", tx, ty));
    }
    svg.push_str(r##"" fill="#f0f0f0" stroke="#333" stroke-width="2" style="cursor: pointer;"/>"##);
    svg.push_str("</g>");

    // Draw merchants
    svg.push_str(r#"<g id="merchants">"#);
    for merchant in &map_data.merchants {
        svg.push_str(&format!(r#"<g id="merchant-{}">"#, merchant.id));
        svg.push_str(r#"<polygon points=""#);
        for (x, y) in &merchant.polygon {
            let (tx, ty) = transform(*x, *y);
            svg.push_str(&format!("{},{} ", tx, ty));
        }
        svg.push_str(r##"" fill="#e3f2fd" stroke="#1976d2" stroke-width="1.5" style="cursor: pointer;"/>"##);

        // Add merchant label
        let (tx, ty) = transform(merchant.location.0, merchant.location.1);
        svg.push_str(&format!(
            r##"<text x="{}" y="{}" font-size="12" text-anchor="middle" fill="#000">{}</text>"##,
            tx, ty, merchant.name
        ));
        svg.push_str("</g>");
    }
    svg.push_str("</g>");

    // Draw beacons
    svg.push_str(r#"<g id="beacons">"#);
    for beacon in &map_data.beacons {
        let (tx, ty) = transform(beacon.location.0, beacon.location.1);
        svg.push_str(&format!(
            r##"<circle cx="{}" cy="{}" r="5" fill="#ff5722" stroke="#d32f2f" stroke-width="1.5"/>"##,
            tx, ty
        ));
        svg.push_str(&format!(
            r##"<text x="{}" y="{}" font-size="10" text-anchor="middle" fill="#666">{}</text>"##,
            tx,
            ty + 15.0,
            beacon.name
        ));
    }
    svg.push_str("</g>");

    svg.push_str("</svg>");
    svg
}

#[tauri::command]
pub async fn get_map_data_handler(
    _app: AppHandle,
    entity: String,
    area: String,
) -> Result<String, String> {
    match fetch_map_data(&entity, &area).await {
        Ok(map_data) => {
            let result = json!({
                "status": "success",
                "data": map_data
            });
            Ok(result.to_string())
        }
        Err(e) => {
            let result = json!({
                "status": "error",
                "message": e.to_string()
            });
            Ok(result.to_string())
        }
    }
}

#[tauri::command]
pub async fn generate_svg_map_handler(
    _app: AppHandle,
    entity: String,
    area: String,
    width: u32,
    height: u32,
) -> Result<String, String> {
    match fetch_map_data(&entity, &area).await {
        Ok(map_data) => {
            let svg = generate_svg_map(&map_data, width, height);
            let result = json!({
                "status": "success",
                "svg": svg
            });
            Ok(result.to_string())
        }
        Err(e) => {
            let result = json!({
                "status": "error",
                "message": e.to_string()
            });
            Ok(result.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_all_merchants_handler(_app: AppHandle, entity: String) -> Result<String, String> {
    match get_all_merchants(&entity).await {
        Ok(merchants) => {
            let result = json!({
                "status": "success",
                "data": merchants
            });
            Ok(result.to_string())
        }
        Err(e) => {
            let result = json!({
                "status": "error",
                "message": e.to_string()
            });
            Ok(result.to_string())
        }
    }
}

#[tauri::command]
pub async fn search_merchants_handler(
    _app: AppHandle,
    entity: String,
    query: String,
) -> Result<String, String> {
    let conn = match SqlitePool::connect("sqlite:navign.db").await {
        Ok(c) => c,
        Err(e) => {
            let result = json!({
                "status": "error",
                "message": format!("Database connection failed: {}", e)
            });
            return Ok(result.to_string());
        }
    };

    let merchants = sqlx::query_as::<_, Merchant>(
        "SELECT * FROM merchants WHERE entry = ? AND name LIKE ? LIMIT 20",
    )
    .bind(&entity)
    .bind(format!("%{}%", query))
    .fetch_all(&conn)
    .await;

    match merchants {
        Ok(merchants) => {
            let result = json!({
                "status": "success",
                "data": merchants
            });
            Ok(result.to_string())
        }
        Err(e) => {
            let result = json!({
                "status": "error",
                "message": e.to_string()
            });
            Ok(result.to_string())
        }
    }
}

/// Fetch route from server between two merchants/areas
pub async fn fetch_route(
    entity: &str,
    from: &str,
    to: &str,
    limits: Option<ConnectivityLimits>,
) -> anyhow::Result<RouteResponse> {
    let client = reqwest::Client::new();

    let mut req = RouteRequest {
        from: from.to_string(),
        to: to.to_string(),
        disallow: None,
    };

    if let Some(limits) = limits {
        trace!("Applying connectivity limits: {:?}", limits);
        let mut disallow = String::new();
        if !limits.elevator {
            disallow.push('e');
        }
        if !limits.stairs {
            disallow.push('s');
        }
        if !limits.escalator {
            disallow.push('c');
        }

        if !disallow.is_empty() {
            req.disallow = Some(disallow.clone());
        }
    }

    let url = format!("{}api/entities/{}/route{}", BASE_URL, entity, req);

    trace!("Fetching route from URL: {}", url);

    let response: RouteResponse = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch route: {}", e))?
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse route: {}", e))?;

    Ok(response)
}

#[tauri::command]
pub async fn get_route_handler(
    _app: AppHandle,
    entity: String,
    from: String,
    to: String,
    allow_elevator: bool,
    allow_stairs: bool,
    allow_escalator: bool,
) -> Result<String, String> {
    let limits = ConnectivityLimits {
        elevator: allow_elevator,
        stairs: allow_stairs,
        escalator: allow_escalator,
    };

    match fetch_route(&entity, &from, &to, Some(limits)).await {
        Ok(route) => {
            let result = json!({
                "status": "success",
                "data": route
            });
            Ok(result.to_string())
        }
        Err(e) => {
            let result = json!({
                "status": "error",
                "message": e.to_string()
            });
            Ok(result.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_area_details_handler(
    _app: AppHandle,
    entity: String,
    area: String,
) -> Result<String, String> {
    match fetch_area_details(&entity, &area).await {
        Ok(area_details) => {
            let result = json!({
                "status": "success",
                "data": area_details
            });
            Ok(result.to_string())
        }
        Err(e) => {
            let result = json!({
                "status": "error",
                "message": e.to_string()
            });
            Ok(result.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_merchant_details_handler(
    _app: AppHandle,
    entity: String,
    merchant: String,
) -> Result<String, String> {
    match fetch_merchant_details(&entity, &merchant).await {
        Ok(merchant_details) => {
            let result = json!({
                "status": "success",
                "data": merchant_details
            });
            Ok(result.to_string())
        }
        Err(e) => {
            let result = json!({
                "status": "error",
                "message": e.to_string()
            });
            Ok(result.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connectivity_limits_default() {
        let limits = ConnectivityLimits::default();
        assert!(limits.elevator);
        assert!(limits.stairs);
        assert!(limits.escalator);
    }

    #[test]
    fn test_connectivity_limits_custom() {
        let limits = ConnectivityLimits {
            elevator: false,
            stairs: true,
            escalator: false,
        };
        assert!(!limits.elevator);
        assert!(limits.stairs);
        assert!(!limits.escalator);
    }

    #[test]
    fn test_generate_svg_map_with_empty_data() {
        let map_data = MapArea {
            id: "test_area".to_string(),
            name: "Test Area".to_string(),
            polygon: vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)],
            beacons: vec![],
            merchants: vec![],
        };

        let svg = generate_svg_map(&map_data, 800, 600);

        assert!(svg.contains("<svg"));
        assert!(svg.contains("width=\"800\""));
        assert!(svg.contains("height=\"600\""));
        assert!(svg.contains("area-boundary"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_generate_svg_map_with_beacons() {
        let map_data = MapArea {
            id: "test_area".to_string(),
            name: "Test Area".to_string(),
            polygon: vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)],
            beacons: vec![
                MapBeacon {
                    id: "beacon1".to_string(),
                    name: "Beacon 1".to_string(),
                    location: (50.0, 50.0),
                    r#type: "navigation".to_string(),
                },
                MapBeacon {
                    id: "beacon2".to_string(),
                    name: "Beacon 2".to_string(),
                    location: (75.0, 75.0),
                    r#type: "marketing".to_string(),
                },
            ],
            merchants: vec![],
        };

        let svg = generate_svg_map(&map_data, 800, 600);

        assert!(svg.contains("beacons"));
        assert!(svg.contains("Beacon 1"));
        assert!(svg.contains("Beacon 2"));
        assert!(svg.contains("circle"));
        assert!(svg.contains("#ff5722"));
    }

    #[test]
    fn test_generate_svg_map_with_merchants() {
        let map_data = MapArea {
            id: "test_area".to_string(),
            name: "Test Area".to_string(),
            polygon: vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)],
            beacons: vec![],
            merchants: vec![MapMerchant {
                id: "merchant1".to_string(),
                name: "Store A".to_string(),
                location: (25.0, 25.0),
                polygon: vec![(20.0, 20.0), (30.0, 20.0), (30.0, 30.0), (20.0, 30.0)],
                tags: vec!["food".to_string(), "restaurant".to_string()],
            }],
        };

        let svg = generate_svg_map(&map_data, 800, 600);

        assert!(svg.contains("merchants"));
        assert!(svg.contains("Store A"));
        assert!(svg.contains("merchant-merchant1"));
        assert!(svg.contains("#e3f2fd"));
        assert!(svg.contains("#1976d2"));
    }
}
