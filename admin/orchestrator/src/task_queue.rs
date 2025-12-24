use crate::types::Task;
use std::collections::BinaryHeap;

/// Wrapper type for Task to implement Ord for priority queue
/// Higher priority values should be processed first (max-heap behavior)
#[derive(Debug, Clone)]
struct PriorityTask(Task);

impl PartialEq for PriorityTask {
    fn eq(&self, other: &Self) -> bool {
        self.0.priority == other.0.priority
    }
}

impl Eq for PriorityTask {}

impl PartialOrd for PriorityTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare by priority (higher priority = higher value)
        // For equal priorities, we could add secondary sorting by created_at
        // but for now we just compare by priority
        self.0.priority.cmp(&other.0.priority)
    }
}

#[derive(Debug)]
pub struct TaskQueue {
    pending: BinaryHeap<PriorityTask>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            pending: BinaryHeap::new(),
        }
    }

    /// Add a task to the queue with O(log n) complexity
    pub fn add_task(&mut self, task: Task) {
        self.pending.push(PriorityTask(task));
    }

    /// Get the next highest priority task with O(log n) complexity
    pub fn get_next_task(&mut self) -> Option<Task> {
        self.pending.pop().map(|pt| pt.0)
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
    use crate::test_utils::{create_test_task};
    use crate::types::Priority;

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
