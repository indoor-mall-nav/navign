//! Shared test utilities for orchestrator tests

#[cfg(test)]
use crate::types::{Priority, RobotInfo, RobotState, Task, TaskType};
#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

/// Create a test task with minimal configuration
#[cfg(test)]
pub fn create_test_task(id: &str, priority: Priority) -> Task {
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

/// Create a test robot with specified parameters
#[cfg(test)]
pub fn create_test_robot(id: &str, entity_id: &str, state: RobotState, battery: f64) -> RobotInfo {
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
