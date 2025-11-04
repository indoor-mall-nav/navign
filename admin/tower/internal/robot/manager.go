package robot

import (
	"context"
	"log"
	"sync"
	"time"

	pb "github.com/indoor-mall-nav/navign/admin/tower/proto"
)

const (
	keepAliveInterval = 5 * time.Second
	reportInterval    = 10 * time.Second
	robotTimeout      = 30 * time.Second
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
	cancelFunc      context.CancelFunc
}

// Manager manages robot connections and reports to orchestrator
type Manager struct {
	robots     map[string]*Robot
	mu         sync.RWMutex
	entityID   string
	grpcClient pb.OrchestratorServiceClient
}

// NewManager creates a new robot manager
func NewManager(entityID string, grpcClient pb.OrchestratorServiceClient) *Manager {
	return &Manager{
		robots:     make(map[string]*Robot),
		entityID:   entityID,
		grpcClient: grpcClient,
	}
}

// RegisterRobot adds a robot and starts its keep-alive goroutine
func (m *Manager) RegisterRobot(robot *Robot) {
	m.mu.Lock()
	defer m.mu.Unlock()

	robot.ConnectedAt = time.Now()
	robot.LastSeen = time.Now()
	if robot.State == pb.RobotState_ROBOT_STATE_UNSPECIFIED {
		robot.State = pb.RobotState_ROBOT_STATE_IDLE
	}

	// Cancel existing goroutine if robot reconnects
	if existing, ok := m.robots[robot.ID]; ok && existing.cancelFunc != nil {
		existing.cancelFunc()
	}

	m.robots[robot.ID] = robot

	// Start keep-alive goroutine for this robot
	ctx, cancel := context.WithCancel(context.Background())
	robot.cancelFunc = cancel

	go m.keepAliveLoop(ctx, robot.ID)

	log.Printf("Robot registered: %s (Entity: %s, Battery: %.1f%%)", 
		robot.ID, robot.EntityID, robot.Battery)

	// Report to orchestrator
	m.reportToOrchestrator(robot)
}

// UnregisterRobot removes a robot and stops its goroutine
func (m *Manager) UnregisterRobot(robotID string) {
	m.mu.Lock()
	defer m.mu.Unlock()

	if robot, exists := m.robots[robotID]; exists {
		if robot.cancelFunc != nil {
			robot.cancelFunc()
		}
		robot.State = pb.RobotState_ROBOT_STATE_OFFLINE
		
		// Report offline status to orchestrator
		m.reportToOrchestrator(robot)
		
		delete(m.robots, robotID)
		log.Printf("Robot unregistered: %s", robotID)
	}
}

// UpdateRobotStatus updates robot status
func (m *Manager) UpdateRobotStatus(robotID string, state pb.RobotState, location *pb.Location, battery float64, taskID string) {
	m.mu.Lock()
	defer m.mu.Unlock()

	robot, exists := m.robots[robotID]
	if !exists {
		return
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

	// Report to orchestrator
	go m.reportToOrchestrator(robot)
}

// UpdateHeartbeat updates the last seen time
func (m *Manager) UpdateHeartbeat(robotID string) {
	m.mu.Lock()
	defer m.mu.Unlock()

	if robot, exists := m.robots[robotID]; exists {
		robot.LastSeen = time.Now()
	}
}

// GetRobot returns a robot by ID
func (m *Manager) GetRobot(robotID string) (*Robot, bool) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	robot, exists := m.robots[robotID]
	return robot, exists
}

// keepAliveLoop runs in a goroutine for each robot
func (m *Manager) keepAliveLoop(ctx context.Context, robotID string) {
	keepAliveTicker := time.NewTicker(keepAliveInterval)
	reportTicker := time.NewTicker(reportInterval)
	cleanupTicker := time.NewTicker(robotTimeout / 2)
	
	defer keepAliveTicker.Stop()
	defer reportTicker.Stop()
	defer cleanupTicker.Stop()

	log.Printf("Keep-alive loop started for robot: %s", robotID)

	for {
		select {
		case <-ctx.Done():
			log.Printf("Keep-alive loop stopped for robot: %s", robotID)
			return

		case <-keepAliveTicker.C:
			// Keep-alive is handled by Socket.IO ping/pong in socket_server
			// This goroutine exists mainly for periodic reporting

		case <-reportTicker.C:
			// Periodically report to orchestrator
			m.mu.RLock()
			robot, exists := m.robots[robotID]
			m.mu.RUnlock()
			
			if exists {
				m.reportToOrchestrator(robot)
			}

		case <-cleanupTicker.C:
			// Check if robot is stale
			m.mu.RLock()
			robot, exists := m.robots[robotID]
			m.mu.RUnlock()
			
			if exists && time.Since(robot.LastSeen) > robotTimeout {
				log.Printf("Robot %s is stale, cleaning up", robotID)
				m.UnregisterRobot(robotID)
				return
			}
		}
	}
}

// reportToOrchestrator sends robot status to Rust orchestrator via gRPC
func (m *Manager) reportToOrchestrator(robot *Robot) {
	if m.grpcClient == nil {
		return
	}

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

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

	req := &pb.RobotReportRequest{
		Robot: robotInfo,
	}

	resp, err := m.grpcClient.ReportRobotStatus(ctx, req)
	if err != nil {
		log.Printf("Failed to report robot status to orchestrator: %v", err)
		return
	}

	if resp.Success {
		log.Printf("Robot status reported: %s", robot.ID)
	} else {
		log.Printf("Orchestrator rejected robot report: %s", resp.Message)
	}
}
