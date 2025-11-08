use std::collections::HashMap;
use tokio::sync::mpsc;
use tonic::Status;

pub mod task {
    tonic::include_proto!("task");
}

pub use task::{
    Priority, RobotDistributionRequest, RobotInfo, RobotReportRequest, RobotReportResponse,
    RobotState, Task, TaskAssignment, TaskType,
};

pub type TaskChannelMap = HashMap<String, mpsc::Sender<Result<TaskAssignment, Status>>>;
