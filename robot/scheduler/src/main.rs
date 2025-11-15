use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};
use zenoh::prelude::*;

// Include generated protobuf code
pub mod proto {
    pub mod common {
        include!(concat!(env!("OUT_DIR"), "/navign.robot.common.rs"));
    }
    pub mod scheduler {
        include!(concat!(env!("OUT_DIR"), "/navign.robot.scheduler.rs"));
    }
    pub mod vision {
        include!(concat!(env!("OUT_DIR"), "/navign.robot.vision.rs"));
    }
    pub mod audio {
        include!(concat!(env!("OUT_DIR"), "/navign.robot.audio.rs"));
    }
    pub mod serial {
        include!(concat!(env!("OUT_DIR"), "/navign.robot.serial.rs"));
    }
    pub mod network {
        include!(concat!(env!("OUT_DIR"), "/navign.robot.network.rs"));
    }
}

use proto::common::*;
use proto::scheduler::*;

mod database;
mod task_manager;
mod zenoh_client;

/// Main scheduler state
pub struct Scheduler {
    zenoh_session: Arc<zenoh::Session>,
    task_manager: Arc<RwLock<task_manager::TaskManager>>,
    robot_state: Arc<RwLock<RobotState>>,
}

impl Scheduler {
    /// Create a new scheduler instance
    pub async fn new() -> Result<Self> {
        // Initialize Zenoh session
        info!("Initializing Zenoh session...");
        let config = zenoh::Config::default();
        let session = zenoh::open(config).await?;

        // Initialize task manager
        let task_manager = task_manager::TaskManager::new().await?;

        // Initialize robot state
        let robot_state = RobotState {
            mode: RobotMode::Idle as i32,
            current_location: None,
            battery_percent: 100.0,
            speed_mps: 0.0,
            emergency_stop: false,
            active_warnings: vec![],
        };

        Ok(Self {
            zenoh_session: Arc::new(session),
            task_manager: Arc::new(RwLock::new(task_manager)),
            robot_state: Arc::new(RwLock::new(robot_state)),
        })
    }

    /// Start the scheduler
    pub async fn start(&self) -> Result<()> {
        info!("Starting scheduler...");

        // Subscribe to task submissions from tower
        self.subscribe_task_submissions().await?;

        // Subscribe to vision updates
        self.subscribe_vision_updates().await?;

        // Subscribe to audio events
        self.subscribe_audio_events().await?;

        // Subscribe to serial sensor data
        self.subscribe_serial_updates().await?;

        // Publish scheduler status
        self.publish_status().await?;

        info!("Scheduler started successfully");
        Ok(())
    }

    /// Subscribe to task submissions from tower
    async fn subscribe_task_submissions(&self) -> Result<()> {
        let subscriber = self
            .zenoh_session
            .declare_subscriber("robot/scheduler/task/submit")
            .await?;

        let task_manager = self.task_manager.clone();

        tokio::spawn(async move {
            while let Ok(sample) = subscriber.recv_async().await {
                match prost::Message::decode(sample.payload().as_ref()) {
                    Ok(task_submission) => {
                        info!("Received task submission: {:?}", task_submission);
                        let mut tm = task_manager.write().await;
                        if let Err(e) = tm.submit_task(task_submission).await {
                            error!("Failed to submit task: {}", e);
                        }
                    }
                    Err(e) => error!("Failed to decode task submission: {}", e),
                }
            }
        });

        Ok(())
    }

    /// Subscribe to vision updates
    async fn subscribe_vision_updates(&self) -> Result<()> {
        let subscriber = self
            .zenoh_session
            .declare_subscriber("robot/vision/updates")
            .await?;

        tokio::spawn(async move {
            while let Ok(sample) = subscriber.recv_async().await {
                info!("Received vision update: {} bytes", sample.payload().len());
                // TODO: Process vision updates for localization/obstacle detection
            }
        });

        Ok(())
    }

    /// Subscribe to audio events
    async fn subscribe_audio_events(&self) -> Result<()> {
        let subscriber = self
            .zenoh_session
            .declare_subscriber("robot/audio/events")
            .await?;

        tokio::spawn(async move {
            while let Ok(sample) = subscriber.recv_async().await {
                info!("Received audio event: {} bytes", sample.payload().len());
                // TODO: Process audio events (wake word detections, etc.)
            }
        });

        Ok(())
    }

    /// Subscribe to serial sensor updates
    async fn subscribe_serial_updates(&self) -> Result<()> {
        let subscriber = self
            .zenoh_session
            .declare_subscriber("robot/serial/sensors")
            .await?;

        let robot_state = self.robot_state.clone();

        tokio::spawn(async move {
            while let Ok(sample) = subscriber.recv_async().await {
                // TODO: Decode sensor data and update robot state
                info!(
                    "Received serial sensor update: {} bytes",
                    sample.payload().len()
                );
            }
        });

        Ok(())
    }

    /// Publish scheduler status
    async fn publish_status(&self) -> Result<()> {
        let zenoh_session = self.zenoh_session.clone();
        let task_manager = self.task_manager.clone();
        let robot_state = self.robot_state.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

            loop {
                interval.tick().await;

                let tm = task_manager.read().await;
                let rs = robot_state.read().await;

                let status = StatusResponse {
                    component: Some(ComponentInfo {
                        component_id: "scheduler".to_string(),
                        r#type: ComponentType::Scheduler as i32,
                        status: ComponentStatus::Ready as i32,
                        timestamp: Some(prost_types::Timestamp {
                            seconds: chrono::Utc::now().timestamp(),
                            nanos: 0,
                        }),
                        metadata: std::collections::HashMap::new(),
                    }),
                    metrics: Some(tm.get_metrics()),
                    components: vec![],
                    robot_state: Some(rs.clone()),
                };

                let mut buf = Vec::new();
                if let Err(e) = prost::Message::encode(&status, &mut buf) {
                    error!("Failed to encode status: {}", e);
                    continue;
                }

                if let Err(e) = zenoh_session.put("robot/scheduler/status", buf).await {
                    error!("Failed to publish status: {}", e);
                }
            }
        });

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Robot Scheduler...");

    // Create and start scheduler
    let scheduler = Scheduler::new().await?;
    scheduler.start().await?;

    // Keep running
    tokio::signal::ctrl_c().await?;

    info!("Scheduler shutting down...");
    Ok(())
}
