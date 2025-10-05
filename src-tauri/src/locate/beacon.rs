//! # Beacon Data
//!
//! After
use tauri_plugin_blec::get_handler;
use tauri_plugin_blec::models::{BleDevice, ScanFilter};
use uuid::{Bytes, Uuid};

fn service_id_to_uuid(service_id: i32) -> Uuid {
    let mut bytes = [0u8; 16];
    bytes[0..4].copy_from_slice(&service_id.to_be_bytes());
    Uuid::from_bytes(bytes)
}

async fn test() {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<BleDevice>>(10);
    let handler = get_handler()
        .unwrap();
    handler
        .discover(
            Some(tx),
            3000,
            ScanFilter::AllServices(vec![
                service_id_to_uuid(0x1819),
                service_id_to_uuid(0x1821),
            ]),
            true
        )
        .await.unwrap();
    while let Some(devices) = rx.recv().await {
        for device in devices {
            println!("{:?}", device);
            device.address;
        }
        handler.stop_scan().await.ok();
    }
}
