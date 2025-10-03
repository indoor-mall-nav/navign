mod certification;
mod database;
mod kernel;
mod schema;
mod shared;

use crate::kernel::beacon::initiate_unlocker;
use crate::schema::{Area, Beacon, Connection, Entity, EntityServiceAddons, Merchant, Service};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::{
    Router,
    http::{Method, StatusCode},
    routing::{delete, get, post, put},
};
use bson::doc;
use bumpalo::Bump;
use log::{LevelFilter, info, warn};
use mongodb::{Client, Database};
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use rsa::pkcs1::{EncodeRsaPublicKey, LineEnding};
use simple_logger::SimpleLogger;
use tower_http::cors::CorsLayer;
use crate::kernel::route::types::CloneIn;
use crate::kernel::route::utils::connectivity::{AgentInstance, ConnectWithInstance, ConnectivityGraph};
// use crate::certification::ensure_key;

async fn root() -> impl IntoResponse {
    (StatusCode::OK, "Hello, World!")
}

async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    // Here you can add logic to check the health of your application, e.g., database connection
    match state.db.run_command(doc! { "ping": 1 }).await {
        Ok(_) => (StatusCode::OK, "Healthy"),
        Err(e) => {
            info!("Health check failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Unhealthy")
        }
    }
}

#[derive(Clone)]
pub(crate) struct AppState {
    db: Database,
    private_key: SigningKey,
}

async fn cert(State(state): State<AppState>) -> impl IntoResponse {
    match state
        .private_key
        .verifying_key()
        .to_pkcs1_pem(LineEnding::LF)
    {
        Ok(res) => (StatusCode::OK, res),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log::set_boxed_logger(Box::new(SimpleLogger::new()))
        .map(|()| log::set_max_level(LevelFilter::Info))?;
    let alloc = Bump::default();
    let db = Client::with_uri_str("mongodb://localhost:27017")
        .await?
        .database("indoor-mall-nav");
    let entity = crate::kernel::route::types::entity::Entity::convert_area_in(&alloc, "68a8301fbdfa76608b934ae1", &db).await.unwrap();

    for area in entity.areas.iter() {
        let graph = area.connectivity_graph(&alloc, crate::kernel::route::utils::connectivity::ConnectivityLimits {
            elevator: true,
            ..Default::default()
        });
        let agent_inst = area.agent_instance(&alloc, crate::kernel::route::utils::connectivity::ConnectivityLimits {
            elevator: true,
            ..Default::default()
        });
        if let Some((agent_instance, conn)) = agent_inst {
            info!("Agent Instance:\n{agent_instance} via {conn}\n\n");
        }
        info!("{area}");
        for (arr, conn, conn_type, x, y) in graph {
            info!("- Connects with {} as {conn_type} at ({x}, {y}) via {conn}", arr.name);
        }
    }

    let ent = entity.clone_in(&alloc);

    let [departure, _, arrival] = ent.areas.as_slice() else {
        panic!("Wrong!")
    };

    warn!("Start.");

    let result = ent.find_path(departure.database_id, (30f64, 38f64), arrival.database_id, crate::kernel::route::utils::connectivity::ConnectivityLimits {
        elevator: false,
        ..Default::default()
    }, &alloc);

    info!("Route: {:?}", result);
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(tower_http::cors::Any);
    info!("Cors layer configured.");
    // ensure_key();
    let db = database::connect_with_db().await?;
    let private_key = SigningKey::random(&mut OsRng);
    let public_key = private_key.verifying_key();
    info!(
        "Public key: {:?}",
        public_key.to_encoded_point(false).as_bytes()
    );
    let state = AppState { db, private_key };
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/cert", get(cert))
        .route("/api/entities/{eid}/beacons/", get(Beacon::get_handler))
        .route("/api/entities/{eid}/beacons", get(Beacon::get_handler))
        .route(
            "/api/entities/{eid}/beacons/{id}",
            get(Beacon::get_one_handler),
        )
        .route("/api/entities/{eid}/beacons", post(Beacon::create_handler))
        .route(
            "/api/entities/{eid}/beacons/unlocker",
            post(initiate_unlocker),
        )
        .route("/api/entities/{eid}/beacons", put(Beacon::update_handler))
        .route("/api/entities/{eid}/beacons/", post(Beacon::create_handler))
        .route("/api/entities/{eid}/beacons/", put(Beacon::update_handler))
        .route(
            "/api/entities/{eid}/beacons/{id}",
            delete(Beacon::delete_handler),
        )
        .route("/api/entities/{eid}/areas", get(Area::get_handler))
        .route("/api/entities/{eid}/areas/", get(Area::get_handler))
        .route("/api/entities/{eid}/areas/{id}", get(Area::get_one_handler))
        .route("/api/entities/{eid}/areas", post(Area::create_handler))
        .route("/api/entities/{eid}/areas", put(Area::update_handler))
        .route("/api/entities/{eid}/areas/", post(Area::create_handler))
        .route("/api/entities/{eid}/areas/", put(Area::update_handler))
        .route(
            "/api/entities/{eid}/areas/{id}",
            delete(Area::delete_handler),
        )
        .route("/api/entities", get(Entity::search_entity_handler))
        .route("/api/entities/", get(Entity::search_entity_handler))
        .route("/api/entities/{id}", get(Entity::get_one_handler))
        .route("/api/entities", post(Entity::create_handler))
        .route("/api/entities", put(Entity::update_handler))
        .route("/api/entities/", post(Entity::create_handler))
        .route("/api/entities/", put(Entity::update_handler))
        .route("/api/entities/{id}", delete(Entity::delete_handler))
        .route("/api/entities/{eid}/merchants", get(Merchant::get_handler))
        .route("/api/entities/{eid}/merchants/", get(Merchant::get_handler))
        .route(
            "/api/entities/{eid}/merchants/{id}",
            get(Merchant::get_one_handler),
        )
        .route(
            "/api/entities/{eid}/merchants",
            post(Merchant::create_handler),
        )
        .route(
            "/api/entities/{eid}/merchants",
            put(Merchant::update_handler),
        )
        .route(
            "/api/entities/{eid}/merchants/",
            post(Merchant::create_handler),
        )
        .route(
            "/api/entities/{eid}/merchants/",
            put(Merchant::update_handler),
        )
        .route(
            "/api/entities/{eid}/merchants/{id}",
            delete(Merchant::delete_handler),
        )
        .route(
            "/api/entities/{eid}/connections",
            get(Connection::get_handler),
        )
        .route(
            "/api/entities/{eid}/connections/",
            get(Connection::get_handler),
        )
        .route(
            "/api/entities/{eid}/connections/{id}",
            get(Connection::get_one_handler),
        )
        .route(
            "/api/entities/{eid}/connections",
            post(Connection::create_handler),
        )
        .route(
            "/api/entities/{eid}/connections",
            put(Connection::update_handler),
        )
        .route(
            "/api/entities/{eid}/connections/",
            post(Connection::create_handler),
        )
        .route(
            "/api/entities/{eid}/connections/",
            put(Connection::update_handler),
        )
        .route(
            "/api/entities/{eid}/connections/{id}",
            delete(Connection::delete_handler),
        )
        .layer(cors)
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
