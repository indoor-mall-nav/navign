use crate::unlocker::Unlocker;
use api::map::{
    generate_svg_map_handler, get_all_areas_handler, get_all_beacons_handler,
    get_all_merchants_handler, get_area_details_handler, get_map_data_handler,
    get_merchant_details_handler, get_route_handler, get_route_offline_handler,
    search_merchants_handler,
};
use blufi::{
    blufi_connect, blufi_disconnect, blufi_provision, blufi_scan_beacons, blufi_scan_wifi,
};
use locate::locate_handler;
use login::handlers::{
    guest_login_handler, login_handler, logout_handler, register_handler, validate_token_handler,
};
use login::handshake::bind_with_server;
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_log::log::info;
use tauri_plugin_sql::{Migration, MigrationKind};
use tokio::sync::Mutex;
use unlocker::unlock_handler;

pub(crate) mod api;
pub(crate) mod blufi;
pub(crate) mod locate;
pub(crate) mod login;
pub(crate) mod shared;
pub(crate) mod unlocker;
pub(crate) mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let migrations = vec![Migration {
                version: 1,
                description: "aligned with PostgreSQL schema using WKB and INTEGER ids",
                sql: include_str!("../migrations/01_navign.sql"),
                kind: MigrationKind::Up,
            }];
            app.handle().plugin(tauri_plugin_opener::init())?;
            app.handle().plugin(tauri_plugin_fs::init())?;
            app.handle().plugin(tauri_plugin_http::init())?;
            app.handle().plugin(tauri_plugin_notification::init())?;
            app.handle().plugin(tauri_plugin_blec::init())?;
            app.handle().plugin(
                tauri_plugin_sql::Builder::default()
                    .add_migrations("sqlite:navign.db", migrations)
                    .build(),
            )?;
            app.handle()
                .plugin(tauri_plugin_log::Builder::new().build())?;
            app.handle().plugin(tauri_plugin_os::init())?;
            app.handle().plugin(tauri_plugin_persisted_scope::init())?;
            app.handle()
                .plugin(tauri_plugin_store::Builder::new().build())?;
            if app
                .path()
                .app_local_data_dir()
                .map(|x| x.join("salt.txt").exists())
                .unwrap_or(false)
            {
                info!("Salt file exists.");
            } else {
                let salt = nanoid::nanoid!();
                info!("Hello, {:?}", app.path().app_local_data_dir());
                if !app
                    .path()
                    .app_local_data_dir()
                    .map(|x| x.exists())
                    .unwrap_or(true)
                {
                    std::fs::create_dir_all(app.path().app_local_data_dir().unwrap()).ok();
                }
                std::fs::write(
                    app.path().app_local_data_dir().unwrap().join("salt.txt"),
                    salt.as_bytes(),
                )?;
            }
            let path = app.path().app_local_data_dir()?.join("salt.txt");
            app.handle()
                .plugin(tauri_plugin_stronghold::Builder::with_argon2(&path).build())?;

            let state = Arc::new(Mutex::new(Unlocker::new(
                "7086cmd".to_string(),
                "example_token".to_string(),
            )));
            app.manage(state);
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_biometric::init())?;
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_barcode_scanner::init())?;
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_geolocation::init())?;
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_nfc::init())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            locate_handler,
            bind_with_server,
            unlock_handler,
            login_handler,
            register_handler,
            logout_handler,
            guest_login_handler,
            validate_token_handler,
            get_map_data_handler,
            generate_svg_map_handler,
            search_merchants_handler,
            get_route_handler,
            get_route_offline_handler,
            get_all_merchants_handler,
            get_all_areas_handler,
            get_all_beacons_handler,
            get_area_details_handler,
            get_merchant_details_handler,
            blufi_scan_beacons,
            blufi_connect,
            blufi_scan_wifi,
            blufi_provision,
            blufi_disconnect
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
