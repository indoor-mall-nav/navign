package scheduler

import (
	"errors"
	"sync"
	"time"

	pb "github.com/indoor-mall-nav/navign/admin/tower/proto"
)

var (
	ErrNoRobotsAvailable = errors.New("no robots available for task")
	ErrRobotNotFound     = errors.New("robot not found")
)

// Robot represents a connected robot
type Robot struct {
	ID              string
	Name            string
	EntityID        string
	State           pb.RobotState
	CurrentLocation *pb.Location
	Battery         float64
	CurrentTaskID   string
	LastSeen        time.Time
	ConnectedAt     time.Time
}

// TaskScheduler manages robots and task assignments
type TaskScheduler struct {
	robots     map[string]*Robot
	tasks      map[string]*pb.Task
	mu         sync.RWMutex
	entityID   string
}

// NewTaskScheduler creates a new scheduler instance
func NewTaskScheduler(entityID string) *TaskScheduler {
	return &TaskScheduler{
		robots:   make(map[string]*Robot),
		tasks:    make(map[string]*pb.Task),
		entityID: entityID,
	}
}

// RegisterRobot adds a robot to the scheduler
func (s *TaskScheduler) RegisterRobot(robot *Robot) {
	s.mu.Lock()
	defer s.mu.Unlock()
	
	robot.ConnectedAt = time.Now()
	robot.LastSeen = time.Now()
	if robot.State == pb.RobotState_ROBOT_STATE_UNSPECIFIED {
		robot.State = pb.RobotState_ROBOT_STATE_IDLE
	}
	s.robots[robot.ID] = robot
}

// UnregisterRobot removes a robot from the scheduler
func (s *TaskScheduler) UnregisterRobot(robotID string) {
	s.mu.Lock()
	defer s.mu.Unlock()
	
	if robot, exists := s.robots[robotID]; exists {
		robot.State = pb.RobotState_ROBOT_STATE_OFFLINE
		delete(s.robots, robotID)
	}
}

// UpdateRobotStatus updates robot status information
func (s *TaskScheduler) UpdateRobotStatus(robotID string, state pb.RobotState, location *pb.Location, battery float64, taskID string) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	
	robot, exists := s.robots[robotID]
	if !exists {
		return ErrRobotNotFound
	}
	
	robot.State = state
	robot.Battery = battery
	robot.LastSeen = time.Now()
	if location != nil {
		robot.CurrentLocation = location
	}
	if taskID != "" {
		robot.CurrentTaskID = taskID
	}
	
	return nil
}

// UpdateRobotHeartbeat updates the last seen time for a robot
func (s *TaskScheduler) UpdateRobotHeartbeat(robotID string) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	
	robot, exists := s.robots[robotID]
	if !exists {
		return ErrRobotNotFound
	}
	
	robot.LastSeen = time.Now()
	return nil
}

// AssignTask assigns a task to the best available robot
func (s *TaskScheduler) AssignTask(task *pb.Task) (string, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	
	// Store the task
	s.tasks[task.Id] = task
	
	// Find the best robot for this task
	var bestRobot *Robot
	var bestScore float64
	
	for _, robot := range s.robots {
		// Only consider robots in the same entity and in idle state
		if robot.EntityID != task.EntityId || robot.State != pb.RobotState_ROBOT_STATE_IDLE {
			continue
		}
		
		// Calculate score based on battery level and proximity
		// Higher battery and closer proximity = higher score
		score := robot.Battery / 100.0
		
		// If we have location info, factor in distance
		if robot.CurrentLocation != nil && len(task.Sources) > 0 {
			// Simple Euclidean distance (in real implementation, use actual pathfinding)
			source := task.Sources[0]
			dx := robot.CurrentLocation.X - source.X
			dy := robot.CurrentLocation.Y - source.Y
			distance := dx*dx + dy*dy
			
			// Closer robots get higher scores (inverse distance)
			if distance > 0 {
				score += 1.0 / (1.0 + distance/1000.0)
			}
		}
		
		if bestRobot == nil || score > bestScore {
			bestRobot = robot
			bestScore = score
		}
	}
	
	if bestRobot == nil {
		return "", ErrNoRobotsAvailable
	}
	
	// Mark robot as busy and assign task
	bestRobot.State = pb.RobotState_ROBOT_STATE_BUSY
	bestRobot.CurrentTaskID = task.Id
	
	return bestRobot.ID, nil
}

// GetRobotDistribution returns information about all robots
func (s *TaskScheduler) GetRobotDistribution() *pb.RobotDistributionResponse {
	s.mu.RLock()
	defer s.mu.RUnlock()
	
	robots := make([]*pb.RobotInfo, 0, len(s.robots))
	idleCount := int32(0)
	busyCount := int32(0)
	
	for _, robot := range s.robots {
		robotInfo := &pb.RobotInfo{
			Id:              robot.ID,
			Name:            robot.Name,
			State:           robot.State,
			CurrentLocation: robot.CurrentLocation,
			BatteryLevel:    robot.Battery,
			CurrentTaskId:   robot.CurrentTaskID,
			LastSeen:        robot.LastSeen.Unix(),
			EntityId:        robot.EntityID,
		}
		robots = append(robots, robotInfo)
		
		if robot.State == pb.RobotState_ROBOT_STATE_IDLE {
			idleCount++
		} else if robot.State == pb.RobotState_ROBOT_STATE_BUSY {
			busyCount++
		}
	}
	
	return &pb.RobotDistributionResponse{
		Robots:     robots,
		TotalCount: int32(len(robots)),
		IdleCount:  idleCount,
		BusyCount:  busyCount,
	}
}

// GetRobot returns a robot by ID
func (s *TaskScheduler) GetRobot(robotID string) (*Robot, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	
	robot, exists := s.robots[robotID]
	if !exists {
		return nil, ErrRobotNotFound
	}
	
	return robot, nil
}

// CleanupStaleRobots removes robots that haven't sent a heartbeat in a while
func (s *TaskScheduler) CleanupStaleRobots(timeout time.Duration) []string {
	s.mu.Lock()
	defer s.mu.Unlock()
	
	now := time.Now()
	staleRobots := []string{}
	
	for robotID, robot := range s.robots {
		if now.Sub(robot.LastSeen) > timeout {
			robot.State = pb.RobotState_ROBOT_STATE_OFFLINE
			staleRobots = append(staleRobots, robotID)
			delete(s.robots, robotID)
		}
	}
	
	return staleRobots
}
