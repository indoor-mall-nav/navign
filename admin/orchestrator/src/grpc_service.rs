use crate::robot_registry::RobotRegistry;
use crate::types::task::orchestrator_service_server::OrchestratorService;
use crate::types::{
    RobotDistributionRequest, RobotReportRequest, RobotReportResponse, TaskAssignment,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

pub struct OrchestratorServiceImpl {
    pub registry: RobotRegistry,
}

impl OrchestratorServiceImpl {
    pub fn new() -> Self {
        Self {
            registry: RobotRegistry::new(),
        }
    }
}

impl Default for OrchestratorServiceImpl {
    fn default() -> Self {
        Self::new()
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
        tracing::info!("Task assignment stream requested for entity: {}", entity_id);

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        // Register this channel for task assignments
        self.registry
            .register_task_channel(entity_id.clone(), tx)
            .await;

        // Convert mpsc::Receiver to ReceiverStream for tonic
        let stream = ReceiverStream::new(rx);

        tracing::info!("Task assignment stream active for entity: {}", entity_id);

        Ok(Response::new(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_test_robot;
    use tonic::Request;

    #[tokio::test]
    async fn test_orchestrator_service_report_robot_status() {
        let service = OrchestratorServiceImpl::new();

        let robot = create_test_robot("robot-1", "entity-1", crate::types::RobotState::Idle, 80.0);
        let request = Request::new(RobotReportRequest { robot: Some(robot) });

        let response = service.report_robot_status(request).await;
        assert!(response.is_ok());

        let report = response.unwrap().into_inner();
        assert!(report.success);

        // Verify robot was registered
        assert_eq!(service.registry.robot_count().await, 1);
    }

    #[tokio::test]
    async fn test_orchestrator_service_report_robot_status_missing_robot() {
        let service = OrchestratorServiceImpl::new();

        let request = Request::new(RobotReportRequest { robot: None });

        let response = service.report_robot_status(request).await;
        assert!(response.is_err());
        assert_eq!(response.unwrap_err().code(), tonic::Code::InvalidArgument);
    }
}
