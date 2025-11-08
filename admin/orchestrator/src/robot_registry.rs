use crate::task_queue::TaskQueue;
use crate::types::{RobotInfo, RobotState, Task, TaskAssignment, TaskChannelMap};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tonic::Status;

#[derive(Debug, Clone)]
pub struct RobotRegistry {
    robots: Arc<RwLock<HashMap<String, RobotInfo>>>,
    task_queue: Arc<RwLock<TaskQueue>>,
    task_channels: Arc<RwLock<TaskChannelMap>>,
}

impl RobotRegistry {
    pub fn new() -> Self {
        Self {
            robots: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(RwLock::new(TaskQueue::new())),
            task_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_robot(&self, robot: RobotInfo) {
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

    pub async fn update_robot_status(&self, robot: RobotInfo) {
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

    pub async fn find_best_robot(&self, task: &Task) -> Option<RobotInfo> {
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

    pub async fn assign_task(&self, task: Task) -> Result<String, String> {
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

    pub async fn register_task_channel(
        &self,
        entity_id: String,
        tx: mpsc::Sender<Result<TaskAssignment, Status>>,
    ) {
        let mut channels = self.task_channels.write().await;
        channels.insert(entity_id, tx);
    }

    pub async fn unregister_task_channel(&self, entity_id: &str) {
        let mut channels = self.task_channels.write().await;
        channels.remove(entity_id);
    }

    pub async fn get_robot(&self, robot_id: &str) -> Option<RobotInfo> {
        let robots = self.robots.read().await;
        robots.get(robot_id).cloned()
    }

    pub async fn robot_count(&self) -> usize {
        let robots = self.robots.read().await;
        robots.len()
    }
}

impl Default for RobotRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::task::Location;
    use crate::types::{Priority, TaskType};
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_test_task(id: &str, priority: Priority) -> Task {
        Task {
            id: id.to_string(),
            r#type: TaskType::Delivery as i32,
            sources: vec![],
            terminals: vec![],
            priority: priority as i32,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            entity_id: "entity-1".to_string(),
            metadata: HashMap::new(),
        }
    }

    fn create_test_robot(id: &str, entity_id: &str, state: RobotState, battery: f64) -> RobotInfo {
        RobotInfo {
            id: id.to_string(),
            name: format!("Test Robot {}", id),
            state: state as i32,
            current_location: None,
            battery_level: battery,
            current_task_id: String::new(),
            last_seen: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            entity_id: entity_id.to_string(),
        }
    }

    #[tokio::test]
    async fn test_robot_registry_new() {
        let registry = RobotRegistry::new();
        assert_eq!(registry.robot_count().await, 0);
    }

    #[tokio::test]
    async fn test_robot_registry_register_robot() {
        let registry = RobotRegistry::new();
        let robot = create_test_robot("robot-1", "entity-1", RobotState::Idle, 80.0);

        registry.register_robot(robot).await;

        assert_eq!(registry.robot_count().await, 1);
        let retrieved = registry.get_robot("robot-1").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "robot-1");
    }

    #[tokio::test]
    async fn test_robot_registry_update_existing_robot() {
        let registry = RobotRegistry::new();
        let robot = create_test_robot("robot-1", "entity-1", RobotState::Idle, 80.0);
        registry.register_robot(robot).await;

        // Update robot status
        let updated_robot = create_test_robot("robot-1", "entity-1", RobotState::Busy, 75.0);
        registry.update_robot_status(updated_robot).await;

        let retrieved = registry.get_robot("robot-1").await.unwrap();
        assert_eq!(retrieved.state, RobotState::Busy as i32);
        assert_eq!(retrieved.battery_level, 75.0);
        assert_eq!(registry.robot_count().await, 1); // Should still be 1
    }

    #[tokio::test]
    async fn test_robot_registry_update_nonexistent_robot_creates_it() {
        let registry = RobotRegistry::new();
        let robot = create_test_robot("robot-1", "entity-1", RobotState::Idle, 80.0);

        registry.update_robot_status(robot).await;

        assert_eq!(registry.robot_count().await, 1);
    }

    #[tokio::test]
    async fn test_find_best_robot_no_robots() {
        let registry = RobotRegistry::new();
        let task = create_test_task("task-1", Priority::Normal);

        let result = registry.find_best_robot(&task).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_best_robot_wrong_entity() {
        let registry = RobotRegistry::new();
        let robot = create_test_robot("robot-1", "entity-1", RobotState::Idle, 80.0);
        registry.register_robot(robot).await;

        let mut task = create_test_task("task-1", Priority::Normal);
        task.entity_id = "entity-2".to_string(); // Different entity

        let result = registry.find_best_robot(&task).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_best_robot_busy_robot_excluded() {
        let registry = RobotRegistry::new();
        let robot = create_test_robot("robot-1", "entity-1", RobotState::Busy, 80.0);
        registry.register_robot(robot).await;

        let task = create_test_task("task-1", Priority::Normal);

        let result = registry.find_best_robot(&task).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_best_robot_selects_by_battery() {
        let registry = RobotRegistry::new();

        let robot1 = create_test_robot("robot-1", "entity-1", RobotState::Idle, 50.0);
        let robot2 = create_test_robot("robot-2", "entity-1", RobotState::Idle, 90.0);
        let robot3 = create_test_robot("robot-3", "entity-1", RobotState::Idle, 70.0);

        registry.register_robot(robot1).await;
        registry.register_robot(robot2).await;
        registry.register_robot(robot3).await;

        let task = create_test_task("task-1", Priority::Normal);

        let result = registry.find_best_robot(&task).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, "robot-2"); // Highest battery
    }

    #[tokio::test]
    async fn test_find_best_robot_proximity_bonus() {
        let registry = RobotRegistry::new();

        let mut robot1 = create_test_robot("robot-1", "entity-1", RobotState::Idle, 80.0);
        robot1.current_location = Some(Location {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            floor: "1F".to_string(),
        });

        let mut robot2 = create_test_robot("robot-2", "entity-1", RobotState::Idle, 80.0);
        robot2.current_location = Some(Location {
            x: 500.0,
            y: 500.0,
            z: 0.0,
            floor: "1F".to_string(),
        });

        registry.register_robot(robot1).await;
        registry.register_robot(robot2).await;

        let mut task = create_test_task("task-1", Priority::Normal);
        task.sources = vec![Location {
            x: 10.0,
            y: 10.0,
            z: 0.0,
            floor: "1F".to_string(),
        }];

        let result = registry.find_best_robot(&task).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, "robot-1"); // Closer to task source
    }

    #[tokio::test]
    async fn test_assign_task_no_suitable_robot() {
        let registry = RobotRegistry::new();
        let task = create_test_task("task-1", Priority::Normal);

        let result = registry.assign_task(task).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No suitable robot available");
    }

    #[tokio::test]
    async fn test_assign_task_no_tower_connected() {
        let registry = RobotRegistry::new();
        let robot = create_test_robot("robot-1", "entity-1", RobotState::Idle, 80.0);
        registry.register_robot(robot).await;

        let task = create_test_task("task-1", Priority::Normal);

        let result = registry.assign_task(task).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No tower connected"));
    }

    #[tokio::test]
    async fn test_assign_task_success() {
        let registry = RobotRegistry::new();
        let robot = create_test_robot("robot-1", "entity-1", RobotState::Idle, 80.0);
        registry.register_robot(robot).await;

        // Register task channel
        let (tx, mut rx) = mpsc::channel(10);
        registry
            .register_task_channel("entity-1".to_string(), tx)
            .await;

        let task = create_test_task("task-1", Priority::Normal);

        let result = registry.assign_task(task.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "robot-1");

        // Verify robot state updated to busy
        let updated_robot = registry.get_robot("robot-1").await.unwrap();
        assert_eq!(updated_robot.state, RobotState::Busy as i32);
        assert_eq!(updated_robot.current_task_id, "task-1");

        // Verify task was sent to channel
        let assignment = rx.recv().await.unwrap().unwrap();
        assert_eq!(assignment.robot_id, "robot-1");
        assert!(assignment.task.is_some());
        assert_eq!(assignment.task.unwrap().id, "task-1");
    }
}
