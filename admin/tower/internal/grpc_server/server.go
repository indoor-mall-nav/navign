package grpc_server

import (
	"context"
	"fmt"
	"log"
	"time"

	pb "github.com/indoor-mall-nav/navign/admin/tower/proto"
	"github.com/indoor-mall-nav/navign/admin/tower/internal/scheduler"
)

// Server implements the TaskScheduler gRPC service
type Server struct {
	pb.UnimplementedTaskSchedulerServer
	scheduler      *scheduler.TaskScheduler
	taskAssignedCb func(string, *pb.Task) error // Callback to notify socket server
}

// NewServer creates a new gRPC server instance
func NewServer(sched *scheduler.TaskScheduler, taskAssignedCb func(string, *pb.Task) error) *Server {
	return &Server{
		scheduler:      sched,
		taskAssignedCb: taskAssignedCb,
	}
}

// SubmitTask handles task submission from the Rust orchestrator
func (s *Server) SubmitTask(ctx context.Context, req *pb.TaskRequest) (*pb.TaskResponse, error) {
	task := req.Task
	if task == nil {
		return &pb.TaskResponse{
			Accepted: false,
			Message:  "Task is nil",
		}, fmt.Errorf("task is nil")
	}

	log.Printf("Received task: ID=%s, Type=%s, Priority=%s, Entity=%s",
		task.Id, task.Type, task.Priority, task.EntityId)

	// Set created timestamp if not set
	if task.CreatedAt == 0 {
		task.CreatedAt = time.Now().Unix()
	}

	// Assign task to best available robot
	robotID, err := s.scheduler.AssignTask(task)
	if err != nil {
		log.Printf("Failed to assign task %s: %v", task.Id, err)
		return &pb.TaskResponse{
			Accepted: false,
			Message:  fmt.Sprintf("No robots available: %v", err),
		}, nil
	}

	log.Printf("Task %s assigned to robot %s", task.Id, robotID)

	// Notify the socket server to send task to robot
	if s.taskAssignedCb != nil {
		if err := s.taskAssignedCb(robotID, task); err != nil {
			log.Printf("Failed to send task to robot %s: %v", robotID, err)
			return &pb.TaskResponse{
				Accepted: false,
				RobotId:  robotID,
				Message:  fmt.Sprintf("Failed to send task to robot: %v", err),
			}, nil
		}
	}

	// Estimate completion time (simplified - in reality would be more complex)
	estimatedTime := time.Now().Add(10 * time.Minute).Unix()

	return &pb.TaskResponse{
		Accepted:                true,
		RobotId:                 robotID,
		Message:                 "Task assigned successfully",
		EstimatedCompletionTime: estimatedTime,
	}, nil
}

// GetRobotDistribution returns current robot status
func (s *Server) GetRobotDistribution(ctx context.Context, req *pb.RobotDistributionRequest) (*pb.RobotDistributionResponse, error) {
	log.Printf("Robot distribution requested for entity: %s", req.EntityId)
	
	distribution := s.scheduler.GetRobotDistribution()
	
	log.Printf("Robot distribution: Total=%d, Idle=%d, Busy=%d",
		distribution.TotalCount, distribution.IdleCount, distribution.BusyCount)
	
	return distribution, nil
}
