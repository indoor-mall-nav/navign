use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_serial::SerialPortBuilderExt;
use tracing::{error, info, warn};

// Include generated protobuf code
pub mod proto {
    pub mod common {
        include!(concat!(env!("OUT_DIR"), "/navign.robot.common.rs"));
    }
    pub mod serial {
        include!(concat!(env!("OUT_DIR"), "/navign.robot.serial.rs"));
    }
}

use proto::common::*;
use proto::serial::*;

/// Serial component for UART communication with lower controller
pub struct SerialComponent {
    zenoh_session: Arc<zenoh::Session>,
    port_path: String,
    baud_rate: u32,
    sensor_data: Arc<RwLock<Option<SensorDataResponse>>>,
}

impl SerialComponent {
    /// Create a new serial component
    pub async fn new(port_path: String, baud_rate: u32) -> Result<Self> {
        // Initialize Zenoh session
        let config = zenoh::Config::default();

        info!("Opening Zenoh session...");

        let session = zenoh::open(config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to open Zenoh session: {}", e))?;

        Ok(Self {
            zenoh_session: Arc::new(session),
            port_path,
            baud_rate,
            sensor_data: Arc::new(RwLock::new(None)),
        })
    }

    /// Start the serial component
    pub async fn start(&self) -> Result<()> {
        info!("Starting serial component...");

        tokio_serial::available_ports()
            .map_err(|e| anyhow::anyhow!("Failed to list serial ports: {}", e))?
            .iter()
            .for_each(|p| info!("Found serial port: {}", p.port_name));

        // Open serial port
        let serial_port = tokio_serial::new(&self.port_path, self.baud_rate).open_native_async()?;

        info!("Serial port opened successfully");

        // Start sensor data reading task
        self.start_sensor_reader(serial_port).await?;

        // Subscribe to motor commands
        self.subscribe_motor_commands().await?;

        // Publish sensor data
        self.publish_sensor_data().await?;

        // Publish status
        self.publish_status().await?;

        info!("Serial component started successfully");
        Ok(())
    }

    /// Start sensor data reading task
    async fn start_sensor_reader(&self, _serial_port: tokio_serial::SerialStream) -> Result<()> {
        let sensor_data = self.sensor_data.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));

            loop {
                interval.tick().await;

                // TODO: Read from serial port and parse sensor data
                // For now, generate mock sensor data
                let mock_data = SensorDataResponse {
                    timestamp: Some(prost_types::Timestamp {
                        seconds: chrono::Utc::now().timestamp(),
                        nanos: 0,
                    }),
                    encoders: Some(EncoderData {
                        left_ticks: 0,
                        right_ticks: 0,
                        left_velocity: 0.0,
                        right_velocity: 0.0,
                        left_distance: 0.0,
                        right_distance: 0.0,
                    }),
                    imu: None,
                    ultrasonic: None,
                    infrared: None,
                    bumper: None,
                    battery: Some(BatteryData {
                        voltage: 24.0,
                        current: 0.5,
                        charge_percent: 85.0,
                        temperature: 25.0,
                        status: BatteryStatus::Normal as i32,
                        remaining_minutes: 240,
                    }),
                    temperature: None,
                };

                let mut data = sensor_data.write().await;
                *data = Some(mock_data);
            }
        });

        Ok(())
    }

    /// Subscribe to motor commands from scheduler
    async fn subscribe_motor_commands(&self) -> Result<()> {
        let subscriber = self
            .zenoh_session
            .declare_subscriber("robot/serial/motor/command")
            .await
            .map_err(|e| anyhow::anyhow!("Failed to declare subscriber: {}", e))?;

        tokio::spawn(async move {
            while let Ok(sample) = subscriber.recv_async().await {
                let payload_bytes = sample.payload().to_bytes();
                match prost::Message::decode(payload_bytes.as_ref()) {
                    Ok::<MotorCommand, _>(cmd) => {
                        info!("Received motor command: {:?}", cmd.mode);
                        // TODO: Send command to lower controller via serial port
                    }
                    Err(e) => error!("Failed to decode motor command: {}", e),
                }
            }
        });

        Ok(())
    }

    /// Publish sensor data to Zenoh
    async fn publish_sensor_data(&self) -> Result<()> {
        let zenoh_session = self.zenoh_session.clone();
        let sensor_data = self.sensor_data.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));

            loop {
                interval.tick().await;

                let data = sensor_data.read().await;
                if let Some(sensor_data) = data.as_ref() {
                    let mut buf = Vec::new();
                    if let Err(e) = prost::Message::encode(sensor_data, &mut buf) {
                        error!("Failed to encode sensor data: {}", e);
                        continue;
                    }

                    if let Err(e) = zenoh_session.put("robot/serial/sensors", buf).await {
                        error!("Failed to publish sensor data: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Publish component status
    async fn publish_status(&self) -> Result<()> {
        let zenoh_session = self.zenoh_session.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

            loop {
                interval.tick().await;

                let status = StatusResponse {
                    component: Some(ComponentInfo {
                        component_id: "serial".to_string(),
                        r#type: ComponentType::Serial as i32,
                        status: ComponentStatus::Ready as i32,
                        timestamp: Some(prost_types::Timestamp {
                            seconds: chrono::Utc::now().timestamp(),
                            nanos: 0,
                        }),
                        metadata: std::collections::HashMap::new(),
                    }),
                    metrics: Some(SerialMetrics {
                        messages_sent: 0,
                        messages_received: 0,
                        errors: 0,
                        timeouts: 0,
                        average_latency_ms: 0.0,
                        bytes_sent: 0,
                        bytes_received: 0,
                        uptime_seconds: 0,
                    }),
                    lower_status: None,
                    port_status: None,
                };

                let mut buf = Vec::new();
                if let Err(e) = prost::Message::encode(&status, &mut buf) {
                    error!("Failed to encode status: {}", e);
                    continue;
                }

                if let Err(e) = zenoh_session.put("robot/serial/status", buf).await {
                    error!("Failed to publish status: {}", e);
                }
            }
        });

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    info!("Starting...");

    match dotenv::dotenv() {
        Ok(_) => info!("Loaded .env file"),
        Err(_) => warn!(".env file not found, proceeding without it"),
    }

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Robot Serial Component...");

    // Get serial port from environment or use default
    let port_path = std::env::var("SERIAL_PORT").unwrap_or_else(|_| "/dev/ttyUSB0".to_string());
    let baud_rate = std::env::var("SERIAL_BAUD_RATE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(115200);

    info!("Using serial port: {} @ {}", port_path, baud_rate);

    // Create and start serial component
    let serial = SerialComponent::new(port_path, baud_rate).await?;
    serial.start().await?;

    // Keep running
    tokio::signal::ctrl_c().await?;

    info!("Serial component shutting down...");
    Ok(())
}
