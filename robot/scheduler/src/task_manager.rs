use anyhow::Result;
use std::collections::HashMap;
use tracing::{info, warn};

use crate::proto::scheduler::*;

/// TaskManager handles task lifecycle and execution
pub struct TaskManager {
    tasks: HashMap<String, TaskState>,
}

struct TaskState {
    submission: TaskSubmission,
    status: TaskStatus,
    progress: u32,
    execution: Option<TaskExecution>,
}

impl TaskManager {
    /// Create a new task manager
    pub async fn new() -> Result<Self> {
        Ok(Self {
            tasks: HashMap::new(),
        })
    }

    /// Submit a new task
    pub async fn submit_task(&mut self, task: TaskSubmission) -> Result<()> {
        info!("Submitting task: {}", task.task_id);

        let task_state = TaskState {
            submission: task.clone(),
            status: TaskStatus::Queued,
            progress: 0,
            execution: None,
        };

        self.tasks.insert(task.task_id.clone(), task_state);

        // TODO: Start task execution
        info!("Task {} queued successfully", task.task_id);

        Ok(())
    }

    /// Get task status
    pub fn get_task_status(&self, task_id: &str) -> Option<TaskStatus> {
        self.tasks.get(task_id).map(|t| t.status)
    }

    /// Update task progress
    pub fn update_progress(&mut self, task_id: &str, progress: u32) {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.progress = progress;
        } else {
            warn!("Attempted to update progress for unknown task: {}", task_id);
        }
    }

    /// Get scheduler metrics
    pub fn get_metrics(&self) -> SchedulerMetrics {
        let total_tasks = self.tasks.len() as u32;
        let queued_tasks = self.count_tasks_by_status(TaskStatus::Queued);
        let active_tasks = self.count_tasks_by_status(TaskStatus::Executing);
        let completed_tasks = self.count_tasks_by_status(TaskStatus::Completed);
        let failed_tasks = self.count_tasks_by_status(TaskStatus::Failed);

        SchedulerMetrics {
            total_tasks,
            queued_tasks,
            active_tasks,
            completed_tasks,
            failed_tasks,
            last_task_completed: None,
            average_task_duration_seconds: 0.0,
            uptime_seconds: 0,
        }
    }

    fn count_tasks_by_status(&self, status: TaskStatus) -> u32 {
        self.tasks.values().filter(|t| t.status == status).count() as u32
    }
}
