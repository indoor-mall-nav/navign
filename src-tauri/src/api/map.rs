use crate::api::page_results::PaginationResponse;
use crate::api::unlocker::CustomizedObjectId;
use crate::locate::merchant::Merchant;
use crate::shared::BASE_URL;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri_plugin_http::reqwest;

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
    pub entity: CustomizedObjectId,
    pub area: CustomizedObjectId,
    pub location: (f64, f64),
    pub polygon: Vec<(f64, f64)>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRequest {
    pub from: String,  // merchant/area id
    pub to: String,    // merchant/area id
    pub disallow: Option<String>,  // "e" (elevator), "s" (stairs), "c" (escalator)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResponse {
    pub instructions: Vec<RouteInstruction>,
    pub total_distance: f64,
    pub areas: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInstruction {
    pub r#type: String,  // "move", "elevator", "stairs", "escalator", "gate"
    pub area: String,
    pub from: (f64, f64),
    pub to: (f64, f64),
    pub description: Option<String>,
    pub distance: Option<f64>,
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
    let area_response: AreaResponse = client
        .get(&area_url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch area: {}", e))?
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse area: {}", e))?;

    // Fetch beacons in the area
    let beacons_url = format!("{}/beacons", area_url);
    let beacons_response: PaginationResponse<BeaconResponse> = client
        .get(&beacons_url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch beacons: {}", e))?
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse beacons: {}", e))?;

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

    // Fetch merchants in the area
    let merchants_url = format!("{}api/entities/{}/merchants?area={}", BASE_URL, entity, area);
    let merchants_response: PaginationResponse<MerchantResponse> = client
        .get(&merchants_url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch merchants: {}", e))?
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse merchants: {}", e))?;

    let map_merchants: Vec<MapMerchant> = merchants_response
        .data
        .into_iter()
        .map(|m| MapMerchant {
            id: m.id.to_string(),
            name: m.name,
            location: m.location,
            polygon: m.polygon,
            tags: m.tags,
        })
        .collect();

    Ok(MapArea {
        id: area_response.id.to_string(),
        name: area_response.name,
        polygon: area_response.polygon,
        beacons: map_beacons,
        merchants: map_merchants,
    })
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

    let transform = |x: f64, y: f64| -> (f64, f64) {
        (
            (x - min_x) * scale + 10.0,
            (y - min_y) * scale + 10.0,
        )
    };

    // Draw area polygon
    svg.push_str(r#"<g id="area-boundary">"#);
    svg.push_str(r#"<polygon points=""#);
    for (x, y) in &map_data.polygon {
        let (tx, ty) = transform(*x, *y);
        svg.push_str(&format!("{},{} ", tx, ty));
    }
    svg.push_str(r##"" fill="#f0f0f0" stroke="#333" stroke-width="2"/>"##);
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
        svg.push_str(r##"" fill="#e3f2fd" stroke="#1976d2" stroke-width="1.5"/>"##);

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
            tx, ty + 15.0, beacon.name
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
        "SELECT * FROM merchants WHERE entry = ? AND name LIKE ? LIMIT 20"
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

    let mut url = format!("{}api/entities/{}/route?from={}&to={}", BASE_URL, entity, from, to);

    if let Some(limits) = limits {
        let mut disallow = String::new();
        if !limits.elevator { disallow.push('e'); }
        if !limits.stairs { disallow.push('s'); }
        if !limits.escalator { disallow.push('c'); }

        if !disallow.is_empty() {
            url.push_str(&format!("&disallow={}", disallow));
        }
    }

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
            merchants: vec![
                MapMerchant {
                    id: "merchant1".to_string(),
                    name: "Store A".to_string(),
                    location: (25.0, 25.0),
                    polygon: vec![(20.0, 20.0), (30.0, 20.0), (30.0, 30.0), (20.0, 30.0)],
                    tags: vec!["food".to_string(), "restaurant".to_string()],
                },
            ],
        };

        let svg = generate_svg_map(&map_data, 800, 600);

        assert!(svg.contains("merchants"));
        assert!(svg.contains("Store A"));
        assert!(svg.contains("merchant-merchant1"));
        assert!(svg.contains("#e3f2fd"));
        assert!(svg.contains("#1976d2"));
    }

    #[test]
    fn test_route_instruction_serialization() {
        let instruction = RouteInstruction {
            r#type: "move".to_string(),
            area: "area1".to_string(),
            from: (0.0, 0.0),
            to: (10.0, 10.0),
            description: Some("Walk straight".to_string()),
            distance: Some(14.14),
        };

        let json = serde_json::to_string(&instruction).unwrap();
        assert!(json.contains("move"));
        assert!(json.contains("area1"));
        assert!(json.contains("Walk straight"));
    }

    #[test]
    fn test_route_response_total_distance() {
        let route = RouteResponse {
            instructions: vec![
                RouteInstruction {
                    r#type: "move".to_string(),
                    area: "area1".to_string(),
                    from: (0.0, 0.0),
                    to: (10.0, 0.0),
                    description: None,
                    distance: Some(10.0),
                },
                RouteInstruction {
                    r#type: "move".to_string(),
                    area: "area1".to_string(),
                    from: (10.0, 0.0),
                    to: (10.0, 10.0),
                    description: None,
                    distance: Some(10.0),
                },
            ],
            total_distance: 20.0,
            areas: vec!["area1".to_string()],
        };

        assert_eq!(route.total_distance, 20.0);
        assert_eq!(route.instructions.len(), 2);
        assert_eq!(route.areas.len(), 1);
    }
}
