use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, transport::Server};

pub mod task {
    tonic::include_proto!("task");
}

use task::orchestrator_service_server::{OrchestratorService, OrchestratorServiceServer};
use task::{
    Priority, RobotDistributionRequest, RobotInfo, RobotReportRequest, RobotReportResponse,
    RobotState, Task, TaskAssignment, TaskType,
};

#[derive(Debug)]
struct TaskQueue {
    pending: Vec<Task>,
}

impl TaskQueue {
    fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    fn add_task(&mut self, task: Task) {
        self.pending.push(task);
        // Sort by priority (higher priority first)
        self.pending.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    fn get_next_task(&mut self) -> Option<Task> {
        if self.pending.is_empty() {
            None
        } else {
            Some(self.pending.remove(0))
        }
    }
}

#[derive(Debug, Clone)]
struct RobotRegistry {
    robots: Arc<RwLock<HashMap<String, RobotInfo>>>,
    task_queue: Arc<RwLock<TaskQueue>>,
    task_channels: Arc<RwLock<HashMap<String, mpsc::Sender<Result<TaskAssignment, Status>>>>>,
}

impl RobotRegistry {
    fn new() -> Self {
        Self {
            robots: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(RwLock::new(TaskQueue::new())),
            task_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn register_robot(&self, robot: RobotInfo) {
        let mut robots = self.robots.write().await;
        log::info!(
            "Robot registered: {} (Entity: {}, Battery: {:.1}%, State: {:?})",
            robot.id,
            robot.entity_id,
            robot.battery_level,
            RobotState::try_from(robot.state).unwrap_or(RobotState::Unspecified)
        );
        robots.insert(robot.id.clone(), robot);
    }

    async fn update_robot_status(&self, robot: RobotInfo) {
        let mut robots = self.robots.write().await;
        if let Some(existing) = robots.get_mut(&robot.id) {
            *existing = robot.clone();
            log::debug!("Robot status updated: {} - {:?}", robot.id, robot.state);
        } else {
            // If robot doesn't exist, register it
            drop(robots);
            self.register_robot(robot).await;
        }
    }

    async fn find_best_robot(&self, task: &Task) -> Option<RobotInfo> {
        let robots = self.robots.read().await;

        let mut best_robot: Option<&RobotInfo> = None;
        let mut best_score = f64::MIN;

        for robot in robots.values() {
            // Only consider idle robots in the same entity
            if robot.entity_id != task.entity_id
                || RobotState::try_from(robot.state) != Ok(RobotState::Idle)
            {
                continue;
            }

            // Calculate score: higher battery = higher score
            let mut score = robot.battery_level;

            // If we have location info, add proximity bonus
            if let (Some(robot_loc), Some(task_source)) =
                (robot.current_location.as_ref(), task.sources.first())
            {
                let dx = robot_loc.x - task_source.x;
                let dy = robot_loc.y - task_source.y;
                let distance_sq = dx * dx + dy * dy;

                // Closer robots get higher scores
                if distance_sq > 0.0 {
                    score += 100.0 / (1.0 + distance_sq / 10000.0);
                }
            }

            if best_robot.is_none() || score > best_score {
                best_robot = Some(robot);
                best_score = score;
            }
        }

        best_robot.cloned()
    }

    async fn assign_task(&self, task: Task) -> Result<String, String> {
        // Find the best robot for this task
        let robot = self
            .find_best_robot(&task)
            .await
            .ok_or_else(|| "No suitable robot available".to_string())?;

        let robot_id = robot.id.clone();
        let entity_id = robot.entity_id.clone();

        log::info!(
            "Assigning task {} to robot {} (Battery: {:.1}%)",
            task.id,
            robot_id,
            robot.battery_level
        );

        // Send task to entity's tower via task channel
        let channels = self.task_channels.read().await;
        if let Some(tx) = channels.get(&entity_id) {
            let assignment = TaskAssignment {
                robot_id: robot_id.clone(),
                task: Some(task.clone()),
            };

            if tx.send(Ok(assignment)).await.is_err() {
                return Err(format!("Failed to send task for entity {}", entity_id));
            }

            // Update robot state to busy
            drop(channels);
            let mut robots = self.robots.write().await;
            if let Some(robot) = robots.get_mut(&robot_id) {
                robot.state = RobotState::Busy as i32;
                robot.current_task_id = task.id.clone();
            }

            Ok(robot_id)
        } else {
            Err(format!("No tower connected for entity {}", entity_id))
        }
    }

    async fn register_task_channel(
        &self,
        entity_id: String,
        tx: mpsc::Sender<Result<TaskAssignment, Status>>,
    ) {
        let mut channels = self.task_channels.write().await;
        channels.insert(entity_id, tx);
    }

    async fn unregister_task_channel(&self, entity_id: &str) {
        let mut channels = self.task_channels.write().await;
        channels.remove(entity_id);
    }
}

pub struct OrchestratorServiceImpl {
    registry: RobotRegistry,
}

impl OrchestratorServiceImpl {
    fn new() -> Self {
        Self {
            registry: RobotRegistry::new(),
        }
    }
}

#[tonic::async_trait]
impl OrchestratorService for OrchestratorServiceImpl {
    async fn report_robot_status(
        &self,
        request: Request<RobotReportRequest>,
    ) -> Result<Response<RobotReportResponse>, Status> {
        let robot = request
            .into_inner()
            .robot
            .ok_or_else(|| Status::invalid_argument("Robot info is required"))?;

        self.registry.update_robot_status(robot).await;

        Ok(Response::new(RobotReportResponse {
            success: true,
            message: "Robot status updated".to_string(),
        }))
    }

    type GetTaskAssignmentStream = ReceiverStream<Result<TaskAssignment, Status>>;

    async fn get_task_assignment(
        &self,
        request: Request<RobotDistributionRequest>,
    ) -> Result<Response<Self::GetTaskAssignmentStream>, Status> {
        let entity_id = request.into_inner().entity_id;
        log::info!("Task assignment stream requested for entity: {}", entity_id);

        let (tx, rx) = mpsc::channel(100);

        // Register this channel for task assignments
        self.registry
            .register_task_channel(entity_id.clone(), tx)
            .await;

        // Convert mpsc::Receiver to ReceiverStream for tonic
        let stream = ReceiverStream::new(rx);

        log::info!("Task assignment stream active for entity: {}", entity_id);

        Ok(Response::new(stream))
    }
}

// Example function to demonstrate task creation (in real app, this would be called based on business logic)
async fn create_example_task(registry: &RobotRegistry, entity_id: &str) {
    use std::time::{SystemTime, UNIX_EPOCH};

    let task = Task {
        id: uuid::Uuid::new_v4().to_string(),
        r#type: TaskType::Delivery as i32,
        sources: vec![task::Location {
            x: 100.0,
            y: 200.0,
            z: 0.0,
            floor: "1F".to_string(),
        }],
        terminals: vec![task::Location {
            x: 500.0,
            y: 600.0,
            z: 0.0,
            floor: "2F".to_string(),
        }],
        priority: Priority::Normal as i32,
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        entity_id: entity_id.to_string(),
        metadata: HashMap::new(),
    };

    match registry.assign_task(task).await {
        Ok(robot_id) => log::info!("Task assigned to robot: {}", robot_id),
        Err(e) => log::warn!("Failed to assign task: {}", e),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let addr = "[::1]:50051".parse()?;
    let orchestrator = OrchestratorServiceImpl::new();

    log::info!("Orchestrator gRPC server listening on {}", addr);

    Server::builder()
        .add_service(OrchestratorServiceServer::new(orchestrator))
        .serve(addr)
        .await?;

    Ok(())
}
