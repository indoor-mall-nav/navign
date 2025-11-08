use crate::types::Task;

#[derive(Debug)]
pub struct TaskQueue {
    pending: Vec<Task>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.pending.push(task);
        // Sort by priority (higher priority first)
        self.pending.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub fn get_next_task(&mut self) -> Option<Task> {
        if self.pending.is_empty() {
            None
        } else {
            Some(self.pending.remove(0))
        }
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn test_task_queue_new() {
        let queue = TaskQueue::new();
        assert_eq!(queue.pending_count(), 0);
    }

    #[test]
    fn test_task_queue_add_and_get() {
        let mut queue = TaskQueue::new();

        let task = create_test_task("task-1", Priority::Normal);
        queue.add_task(task);

        assert_eq!(queue.pending_count(), 1);

        let retrieved = queue.get_next_task();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "task-1");
        assert_eq!(queue.pending_count(), 0);
    }

    #[test]
    fn test_task_queue_priority_sorting() {
        let mut queue = TaskQueue::new();

        // Add tasks in mixed priority order
        queue.add_task(create_test_task("normal", Priority::Normal));
        queue.add_task(create_test_task("urgent", Priority::Urgent));
        queue.add_task(create_test_task("low", Priority::Low));
        queue.add_task(create_test_task("high", Priority::High));

        // Should be retrieved in priority order: Urgent > High > Normal > Low
        assert_eq!(queue.get_next_task().unwrap().id, "urgent");
        assert_eq!(queue.get_next_task().unwrap().id, "high");
        assert_eq!(queue.get_next_task().unwrap().id, "normal");
        assert_eq!(queue.get_next_task().unwrap().id, "low");
        assert!(queue.get_next_task().is_none());
    }

    #[test]
    fn test_task_queue_empty() {
        let mut queue = TaskQueue::new();
        assert!(queue.get_next_task().is_none());
    }
}
