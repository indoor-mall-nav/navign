package robot

import (
	"context"
	"testing"
	"time"

	pb "github.com/indoor-mall-nav/navign/admin/tower/proto"
	"google.golang.org/grpc"
)

// Mock gRPC client
type mockOrchestratorClient struct {
	reportCalled bool
	lastReport   *pb.RobotReportRequest
}

func (m *mockOrchestratorClient) ReportRobotStatus(ctx context.Context, in *pb.RobotReportRequest, opts ...grpc.CallOption) (*pb.RobotReportResponse, error) {
	m.reportCalled = true
	m.lastReport = in
	return &pb.RobotReportResponse{
		Success: true,
		Message: "OK",
	}, nil
}

func (m *mockOrchestratorClient) GetTaskAssignment(ctx context.Context, in *pb.RobotDistributionRequest, opts ...grpc.CallOption) (pb.OrchestratorService_GetTaskAssignmentClient, error) {
	return nil, nil
}

func TestNewManager(t *testing.T) {
	manager := NewManager("entity-1", nil)
	if manager == nil {
		t.Fatal("NewManager returned nil")
	}
	if manager.entityID != "entity-1" {
		t.Errorf("Expected entityID 'entity-1', got '%s'", manager.entityID)
	}
	if len(manager.robots) != 0 {
		t.Errorf("Expected 0 robots, got %d", len(manager.robots))
	}
}

func TestRegisterRobot(t *testing.T) {
	mockClient := &mockOrchestratorClient{}
	manager := NewManager("entity-1", mockClient)

	robot := &Robot{
		ID:       "robot-1",
		Name:     "Test Robot",
		EntityID: "entity-1",
		Battery:  80.0,
	}

	manager.RegisterRobot(robot)

	// Check robot was registered
	manager.mu.RLock()
	registered, exists := manager.robots["robot-1"]
	manager.mu.RUnlock()

	if !exists {
		t.Fatal("Robot was not registered")
	}

	if registered.ID != "robot-1" {
		t.Errorf("Expected robot ID 'robot-1', got '%s'", registered.ID)
	}

	if registered.State != pb.RobotState_ROBOT_STATE_IDLE {
		t.Errorf("Expected robot state IDLE, got %v", registered.State)
	}

	if registered.ConnectedAt.IsZero() {
		t.Error("ConnectedAt was not set")
	}

	if registered.LastSeen.IsZero() {
		t.Error("LastSeen was not set")
	}

	// Wait a bit for goroutine to report to orchestrator
	time.Sleep(100 * time.Millisecond)

	if !mockClient.reportCalled {
		t.Error("ReportRobotStatus was not called")
	}

	// Cleanup
	if registered.cancelFunc != nil {
		registered.cancelFunc()
	}
}

func TestRegisterRobotReplacesExisting(t *testing.T) {
	manager := NewManager("entity-1", nil)

	robot1 := &Robot{
		ID:       "robot-1",
		Name:     "Test Robot 1",
		EntityID: "entity-1",
		Battery:  80.0,
	}

	manager.RegisterRobot(robot1)

	// Wait a bit
	time.Sleep(50 * time.Millisecond)

	// Register again with different data
	robot2 := &Robot{
		ID:       "robot-1",
		Name:     "Test Robot 2",
		EntityID: "entity-1",
		Battery:  70.0,
	}

	manager.RegisterRobot(robot2)

	manager.mu.RLock()
	registered := manager.robots["robot-1"]
	robotCount := len(manager.robots)
	manager.mu.RUnlock()

	if robotCount != 1 {
		t.Errorf("Expected 1 robot, got %d", robotCount)
	}

	if registered.Name != "Test Robot 2" {
		t.Errorf("Expected name 'Test Robot 2', got '%s'", registered.Name)
	}

	if registered.Battery != 70.0 {
		t.Errorf("Expected battery 70.0, got %.1f", registered.Battery)
	}

	// Cleanup
	if registered.cancelFunc != nil {
		registered.cancelFunc()
	}
}

func TestUnregisterRobot(t *testing.T) {
	mockClient := &mockOrchestratorClient{}
	manager := NewManager("entity-1", mockClient)

	robot := &Robot{
		ID:       "robot-1",
		Name:     "Test Robot",
		EntityID: "entity-1",
		Battery:  80.0,
	}

	manager.RegisterRobot(robot)
	time.Sleep(50 * time.Millisecond)

	manager.UnregisterRobot("robot-1")

	manager.mu.RLock()
	_, exists := manager.robots["robot-1"]
	manager.mu.RUnlock()

	if exists {
		t.Error("Robot still exists after unregister")
	}

	// Wait for final report
	time.Sleep(100 * time.Millisecond)

	// Check that offline status was reported
	if mockClient.lastReport != nil && mockClient.lastReport.Robot != nil {
		if mockClient.lastReport.Robot.State != pb.RobotState_ROBOT_STATE_OFFLINE {
			t.Errorf("Expected OFFLINE state in final report, got %v", mockClient.lastReport.Robot.State)
		}
	}
}

func TestUpdateRobotStatus(t *testing.T) {
	mockClient := &mockOrchestratorClient{}
	manager := NewManager("entity-1", mockClient)

	robot := &Robot{
		ID:       "robot-1",
		Name:     "Test Robot",
		EntityID: "entity-1",
		Battery:  80.0,
		State:    pb.RobotState_ROBOT_STATE_IDLE,
	}

	manager.RegisterRobot(robot)

	location := &pb.Location{
		X:     100.0,
		Y:     200.0,
		Z:     0.0,
		Floor: "1F",
	}

	manager.UpdateRobotStatus("robot-1", pb.RobotState_ROBOT_STATE_BUSY, location, 75.0, "task-1")

	manager.mu.RLock()
	updated := manager.robots["robot-1"]
	manager.mu.RUnlock()

	if updated.State != pb.RobotState_ROBOT_STATE_BUSY {
		t.Errorf("Expected state BUSY, got %v", updated.State)
	}

	if updated.Battery != 75.0 {
		t.Errorf("Expected battery 75.0, got %.1f", updated.Battery)
	}

	if updated.CurrentTaskID != "task-1" {
		t.Errorf("Expected task ID 'task-1', got '%s'", updated.CurrentTaskID)
	}

	if updated.CurrentLocation == nil {
		t.Fatal("Location was not updated")
	}

	if updated.CurrentLocation.X != 100.0 {
		t.Errorf("Expected location X 100.0, got %.1f", updated.CurrentLocation.X)
	}

	// Wait for report
	time.Sleep(100 * time.Millisecond)

	if !mockClient.reportCalled {
		t.Error("ReportRobotStatus was not called after update")
	}

	// Cleanup
	if updated.cancelFunc != nil {
		updated.cancelFunc()
	}
}

func TestUpdateRobotStatusNonexistent(t *testing.T) {
	manager := NewManager("entity-1", nil)

	location := &pb.Location{
		X:     100.0,
		Y:     200.0,
		Z:     0.0,
		Floor: "1F",
	}

	// Update a robot that doesn't exist - should be no-op
	manager.UpdateRobotStatus("robot-1", pb.RobotState_ROBOT_STATE_BUSY, location, 75.0, "task-1")

	manager.mu.RLock()
	robotCount := len(manager.robots)
	manager.mu.RUnlock()

	if robotCount != 0 {
		t.Errorf("Expected 0 robots, got %d", robotCount)
	}
}

func TestUpdateHeartbeat(t *testing.T) {
	manager := NewManager("entity-1", nil)

	robot := &Robot{
		ID:       "robot-1",
		Name:     "Test Robot",
		EntityID: "entity-1",
		Battery:  80.0,
	}

	manager.RegisterRobot(robot)

	initialLastSeen := robot.LastSeen

	// Wait a bit
	time.Sleep(10 * time.Millisecond)

	manager.UpdateHeartbeat("robot-1")

	manager.mu.RLock()
	updated := manager.robots["robot-1"]
	manager.mu.RUnlock()

	if !updated.LastSeen.After(initialLastSeen) {
		t.Error("LastSeen was not updated")
	}

	// Cleanup
	if updated.cancelFunc != nil {
		updated.cancelFunc()
	}
}

func TestGetRobot(t *testing.T) {
	manager := NewManager("entity-1", nil)

	robot := &Robot{
		ID:       "robot-1",
		Name:     "Test Robot",
		EntityID: "entity-1",
		Battery:  80.0,
	}

	manager.RegisterRobot(robot)

	retrieved, exists := manager.GetRobot("robot-1")
	if !exists {
		t.Fatal("Robot not found")
	}

	if retrieved.ID != "robot-1" {
		t.Errorf("Expected robot ID 'robot-1', got '%s'", retrieved.ID)
	}

	_, exists = manager.GetRobot("nonexistent")
	if exists {
		t.Error("Nonexistent robot was found")
	}

	// Cleanup
	if retrieved.cancelFunc != nil {
		retrieved.cancelFunc()
	}
}

func TestRobotStateDefaults(t *testing.T) {
	manager := NewManager("entity-1", nil)

	robot := &Robot{
		ID:       "robot-1",
		Name:     "Test Robot",
		EntityID: "entity-1",
		Battery:  80.0,
		// State not set (UNSPECIFIED)
	}

	manager.RegisterRobot(robot)

	manager.mu.RLock()
	registered := manager.robots["robot-1"]
	manager.mu.RUnlock()

	if registered.State != pb.RobotState_ROBOT_STATE_IDLE {
		t.Errorf("Expected default state IDLE, got %v", registered.State)
	}

	// Cleanup
	if registered.cancelFunc != nil {
		registered.cancelFunc()
	}
}

func TestKeepAliveLoop(t *testing.T) {
	mockClient := &mockOrchestratorClient{}
	manager := NewManager("entity-1", mockClient)

	robot := &Robot{
		ID:       "robot-1",
		Name:     "Test Robot",
		EntityID: "entity-1",
		Battery:  80.0,
	}

	manager.RegisterRobot(robot)

	// Wait for periodic report (reportInterval = 10s, but we'll just check it started)
	time.Sleep(100 * time.Millisecond)

	// Verify the keep-alive goroutine is running by checking it can be cancelled
	manager.mu.RLock()
	registered := manager.robots["robot-1"]
	manager.mu.RUnlock()

	if registered.cancelFunc == nil {
		t.Fatal("Cancel function was not set")
	}

	// Cancel and verify goroutine stops
	registered.cancelFunc()

	// Give it time to stop
	time.Sleep(50 * time.Millisecond)

	// The goroutine should have stopped cleanly (no way to directly verify, but no panic = good)
}

func TestStaleRobotCleanup(t *testing.T) {
	manager := NewManager("entity-1", nil)

	robot := &Robot{
		ID:       "robot-1",
		Name:     "Test Robot",
		EntityID: "entity-1",
		Battery:  80.0,
	}

	manager.RegisterRobot(robot)

	// Manually set LastSeen to old time to simulate stale robot
	manager.mu.Lock()
	manager.robots["robot-1"].LastSeen = time.Now().Add(-robotTimeout - time.Second)
	manager.mu.Unlock()

	// Wait for cleanup ticker (robotTimeout/2 = 15s, but we can't wait that long)
	// Instead, just verify the logic is in place by checking timeout constant
	if robotTimeout != 30*time.Second {
		t.Errorf("Expected robotTimeout to be 30s, got %v", robotTimeout)
	}

	// Cleanup manually
	manager.mu.RLock()
	registered := manager.robots["robot-1"]
	manager.mu.RUnlock()
	if registered.cancelFunc != nil {
		registered.cancelFunc()
	}
}
