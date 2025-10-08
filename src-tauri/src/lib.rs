use crate::unlocker::Unlocker;
use locate::locate_handler;
use login::handshake::bind_with_server;
use p256::ecdsa::VerifyingKey;
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_sql::{Migration, MigrationKind};
use tokio::sync::Mutex;
use unlocker::unlock_handler;

pub(crate) mod api;
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
                description: "initialize",
                sql: include_str!("navign.sql"),
                kind: MigrationKind::Up,
            }];
            app.handle().plugin(tauri_plugin_opener::init())?;
            app.handle().plugin(tauri_plugin_http::init())?;
            app.handle().plugin(tauri_plugin_notification::init())?;
            app.handle().plugin(
                tauri_plugin_sql::Builder::default()
                    .add_migrations("sqlite:navign.db", migrations)
                    .build(),
            )?;
            app.handle().plugin(tauri_plugin_log::Builder::new().build())?;
            app.handle().plugin(tauri_plugin_os::init())?;
            app.handle().plugin(tauri_plugin_persisted_scope::init())?;
            app.handle().plugin(tauri_plugin_store::Builder::new().build())?;
            if app
                .path()
                .app_local_data_dir()
                .map(|x| x.join("salt.txt").exists())
                .unwrap_or(false)
            {
                println!("Salt file exists.");
            } else {
                let salt = nanoid::nanoid!();
                println!("Hello, {:?}", app.path().app_local_data_dir());
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
            let server_pub_key = [
                4, 29, 160, 114, 228, 62, 157, 118, 19, 35, 126, 85, 206, 135, 190, 151, 236, 195,
                95, 99, 206, 111, 205, 177, 216, 26, 195, 79, 55, 241, 128, 164, 145, 102, 56, 204,
                234, 113, 61, 127, 195, 42, 145, 240, 3, 252, 125, 166, 19, 72, 90, 139, 188, 180,
                164, 185, 54, 236, 168, 224, 71, 40, 179, 51, 105,
            ];
            let state = Arc::new(Mutex::new(Unlocker::new(
                VerifyingKey::from_sec1_bytes(&server_pub_key).unwrap(),
                "7086cmd".to_string(),
                "example_token".to_string(),
            )));
            app.manage(state);
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_biometric::init())?;
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_barcode_scanner::init())?;
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_blec::init())?;
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_geolocation::init())?;
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_nfc::init())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            locate_handler,
            bind_with_server,
            unlock_handler
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
