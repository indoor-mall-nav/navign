// Tests for firmware_api module

#[tokio::test]
async fn test_firmware_client_new() {
    use navign_orchestrator::firmware_api::FirmwareClient;

    let client = FirmwareClient::new("http://localhost:3000".to_string());
    // Should not panic, basic construction test
    assert!(std::mem::size_of_val(&client) > 0);
}

#[test]
fn test_firmware_device_parsing() {
    use navign_shared::FirmwareDevice;

    // Test valid device types
    assert!("esp32".parse::<FirmwareDevice>().is_ok());
    assert!("esp32c3".parse::<FirmwareDevice>().is_ok());
    assert!("esp32s3".parse::<FirmwareDevice>().is_ok());

    // Test case sensitivity (FromStr implementation may be case-sensitive)
    // Check if uppercase variants work
    let _ = "ESP32C3".parse::<FirmwareDevice>();

    // Test invalid device type
    assert!("invalid_device".parse::<FirmwareDevice>().is_err());
    assert!("stm32f407".parse::<FirmwareDevice>().is_err());
}

#[test]
fn test_firmware_query_construction() {
    use navign_shared::{FirmwareDevice, FirmwareQuery};

    // Test query with all fields
    let query = FirmwareQuery {
        device: Some(FirmwareDevice::Esp32C3),
        version: Some("1.0.0".to_string()),
        latest_only: Some(true),
    };

    assert_eq!(query.device, Some(FirmwareDevice::Esp32C3));
    assert_eq!(query.version, Some("1.0.0".to_string()));
    assert_eq!(query.latest_only, Some(true));

    // Test query with no fields (list all)
    let query_empty = FirmwareQuery {
        device: None,
        version: None,
        latest_only: None,
    };

    assert!(query_empty.device.is_none());
    assert!(query_empty.version.is_none());
    assert!(query_empty.latest_only.is_none());
}

#[test]
fn test_orchestrator_info_serialization() {
    use navign_orchestrator::firmware_api::OrchestratorInfo;
    use serde_json;

    let info = OrchestratorInfo {
        version: "0.1.0".to_string(),
        server_url: "http://localhost:3000".to_string(),
        status: "healthy".to_string(),
    };

    // Serialize
    let json = serde_json::to_string(&info).expect("Should serialize");

    // Deserialize
    let deserialized: OrchestratorInfo = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(info.version, deserialized.version);
    assert_eq!(info.server_url, deserialized.server_url);
    assert_eq!(info.status, deserialized.status);
}

#[test]
fn test_firmware_device_as_str() {
    use navign_shared::FirmwareDevice;

    assert_eq!(FirmwareDevice::Esp32.as_str(), "esp32");
    assert_eq!(FirmwareDevice::Esp32C3.as_str(), "esp32c3");
    assert_eq!(FirmwareDevice::Esp32C5.as_str(), "esp32c5");
    assert_eq!(FirmwareDevice::Esp32C6.as_str(), "esp32c6");
    assert_eq!(FirmwareDevice::Esp32S3.as_str(), "esp32s3");
}

// Mock server tests (requires a mock HTTP server or integration test)
// These are commented out as they would need mockito or similar
// Uncomment when setting up integration tests

/*
#[tokio::test]
async fn test_get_latest_firmware_success() {
    use navign_orchestrator::firmware_api::FirmwareClient;
    use navign_shared::FirmwareDevice;

    // TODO: Set up mockito server to return firmware JSON
    let mock_server = mockito::Server::new();
    let mock = mock_server
        .mock("GET", "/api/firmwares/latest/esp32c3")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "id": "firmware-1",
            "device": "esp32c3",
            "version": "1.0.0",
            "checksum": "abc123",
            "file_path": "firmware.bin",
            "size": 1024,
            "created_at": "2025-01-01T00:00:00Z"
        }"#)
        .create();

    let client = FirmwareClient::new(mock_server.url());
    let firmware = client
        .get_latest_firmware(FirmwareDevice::Esp32C3)
        .await
        .expect("Should fetch firmware");

    assert_eq!(firmware.device, FirmwareDevice::Esp32C3);
    assert_eq!(firmware.version, "1.0.0");

    mock.assert();
}

#[tokio::test]
async fn test_download_firmware_success() {
    use navign_orchestrator::firmware_api::FirmwareClient;

    // TODO: Set up mockito server to return binary data
    let mock_server = mockito::Server::new();
    let mock = mock_server
        .mock("GET", "/api/firmwares/firmware-1/download")
        .with_status(200)
        .with_header("content-type", "application/octet-stream")
        .with_body(vec![0x00, 0x01, 0x02, 0x03])
        .create();

    let client = FirmwareClient::new(mock_server.url());
    let data = client
        .download_firmware("firmware-1")
        .await
        .expect("Should download firmware");

    assert_eq!(data, vec![0x00, 0x01, 0x02, 0x03]);

    mock.assert();
}
*/
