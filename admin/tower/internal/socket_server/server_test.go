package socket_server

import (
	"testing"

	"github.com/indoor-mall-nav/navign/admin/tower/internal/models"
	pb "github.com/indoor-mall-nav/navign/admin/tower/proto"
)

func TestParseRobotState(t *testing.T) {
	tests := []struct {
		input    string
		expected pb.RobotState
	}{
		{"idle", pb.RobotState_ROBOT_STATE_IDLE},
		{"busy", pb.RobotState_ROBOT_STATE_BUSY},
		{"charging", pb.RobotState_ROBOT_STATE_CHARGING},
		{"error", pb.RobotState_ROBOT_STATE_ERROR},
		{"unknown", pb.RobotState_ROBOT_STATE_UNSPECIFIED},
		{"", pb.RobotState_ROBOT_STATE_UNSPECIFIED},
	}

	for _, test := range tests {
		result := parseRobotState(test.input)
		if result != test.expected {
			t.Errorf("parseRobotState(%q) = %v, want %v", test.input, result, test.expected)
		}
	}
}

func TestTaskTypeToString(t *testing.T) {
	tests := []struct {
		input    pb.TaskType
		expected string
	}{
		{pb.TaskType_TASK_TYPE_DELIVERY, "delivery"},
		{pb.TaskType_TASK_TYPE_PATROL, "patrol"},
		{pb.TaskType_TASK_TYPE_RETURN_HOME, "return_home"},
		{pb.TaskType_TASK_TYPE_EMERGENCY, "emergency"},
		{pb.TaskType_TASK_TYPE_UNSPECIFIED, "unspecified"},
	}

	for _, test := range tests {
		result := taskTypeToString(test.input)
		if result != test.expected {
			t.Errorf("taskTypeToString(%v) = %q, want %q", test.input, result, test.expected)
		}
	}
}

func TestPriorityToString(t *testing.T) {
	tests := []struct {
		input    pb.Priority
		expected string
	}{
		{pb.Priority_PRIORITY_LOW, "low"},
		{pb.Priority_PRIORITY_NORMAL, "normal"},
		{pb.Priority_PRIORITY_HIGH, "high"},
		{pb.Priority_PRIORITY_URGENT, "urgent"},
		{pb.Priority_PRIORITY_UNSPECIFIED, "unspecified"},
	}

	for _, test := range tests {
		result := priorityToString(test.input)
		if result != test.expected {
			t.Errorf("priorityToString(%v) = %q, want %q", test.input, result, test.expected)
		}
	}
}

func TestConvertLocations(t *testing.T) {
	pbLocations := []*pb.Location{
		{
			X:     100.0,
			Y:     200.0,
			Z:     0.0,
			Floor: "1F",
		},
		{
			X:     300.0,
			Y:     400.0,
			Z:     1.5,
			Floor: "2F",
		},
	}

	result := convertLocations(pbLocations)

	if len(result) != 2 {
		t.Fatalf("Expected 2 locations, got %d", len(result))
	}

	// Check first location
	if result[0].X != 100.0 {
		t.Errorf("Expected X=100.0, got %.1f", result[0].X)
	}
	if result[0].Y != 200.0 {
		t.Errorf("Expected Y=200.0, got %.1f", result[0].Y)
	}
	if result[0].Z != 0.0 {
		t.Errorf("Expected Z=0.0, got %.1f", result[0].Z)
	}
	if result[0].Floor != "1F" {
		t.Errorf("Expected Floor='1F', got '%s'", result[0].Floor)
	}

	// Check second location
	if result[1].X != 300.0 {
		t.Errorf("Expected X=300.0, got %.1f", result[1].X)
	}
	if result[1].Y != 400.0 {
		t.Errorf("Expected Y=400.0, got %.1f", result[1].Y)
	}
	if result[1].Z != 1.5 {
		t.Errorf("Expected Z=1.5, got %.1f", result[1].Z)
	}
	if result[1].Floor != "2F" {
		t.Errorf("Expected Floor='2F', got '%s'", result[1].Floor)
	}
}

func TestConvertLocationsEmpty(t *testing.T) {
	pbLocations := []*pb.Location{}
	result := convertLocations(pbLocations)

	if len(result) != 0 {
		t.Errorf("Expected 0 locations, got %d", len(result))
	}
}

func TestConvertLocationsNil(t *testing.T) {
	result := convertLocations(nil)

	if len(result) != 0 {
		t.Errorf("Expected 0 locations, got %d", len(result))
	}
}

func TestTaskConversionIntegration(t *testing.T) {
	// Create a protobuf task
	pbTask := &pb.Task{
		Id:   "task-123",
		Type: pb.TaskType_TASK_TYPE_DELIVERY,
		Sources: []*pb.Location{
			{X: 100.0, Y: 200.0, Z: 0.0, Floor: "1F"},
		},
		Terminals: []*pb.Location{
			{X: 500.0, Y: 600.0, Z: 0.0, Floor: "3F"},
		},
		Priority: pb.Priority_PRIORITY_HIGH,
		EntityId: "entity-1",
		Metadata: map[string]string{
			"customer": "Bob",
			"package":  "Electronics",
		},
	}

	// Convert to socket packet (simulating what SendTaskToRobot does)
	packet := models.TaskAssignedPacket{
		TaskID:     pbTask.Id,
		Type:       taskTypeToString(pbTask.Type),
		Sources:    convertLocations(pbTask.Sources),
		Terminals:  convertLocations(pbTask.Terminals),
		Priority:   priorityToString(pbTask.Priority),
		Metadata:   pbTask.Metadata,
		AssignedAt: 1234567890,
	}

	// Verify conversion
	if packet.TaskID != "task-123" {
		t.Errorf("Expected TaskID 'task-123', got '%s'", packet.TaskID)
	}
	if packet.Type != "delivery" {
		t.Errorf("Expected Type 'delivery', got '%s'", packet.Type)
	}
	if packet.Priority != "high" {
		t.Errorf("Expected Priority 'high', got '%s'", packet.Priority)
	}
	if len(packet.Sources) != 1 {
		t.Errorf("Expected 1 source, got %d", len(packet.Sources))
	}
	if len(packet.Terminals) != 1 {
		t.Errorf("Expected 1 terminal, got %d", len(packet.Terminals))
	}
	if packet.Metadata["customer"] != "Bob" {
		t.Errorf("Expected customer 'Bob', got '%s'", packet.Metadata["customer"])
	}
}

func TestStatusUpdateConversion(t *testing.T) {
	// Simulate receiving a status update packet from robot
	packet := models.StatusUpdatePacket{
		RobotID: "robot-1",
		State:   "busy",
		CurrentLocation: models.LocationPacket{
			X:     100.0,
			Y:     200.0,
			Z:     0.0,
			Floor: "1F",
		},
		Battery:       75.5,
		CurrentTaskID: "task-1",
		Timestamp:     1234567890,
	}

	// Convert to protobuf (simulating what handleStatusUpdate does)
	state := parseRobotState(packet.State)
	location := &pb.Location{
		X:     packet.CurrentLocation.X,
		Y:     packet.CurrentLocation.Y,
		Z:     packet.CurrentLocation.Z,
		Floor: packet.CurrentLocation.Floor,
	}

	// Verify conversion
	if state != pb.RobotState_ROBOT_STATE_BUSY {
		t.Errorf("Expected state BUSY, got %v", state)
	}
	if location.X != 100.0 {
		t.Errorf("Expected X=100.0, got %.1f", location.X)
	}
	if location.Y != 200.0 {
		t.Errorf("Expected Y=200.0, got %.1f", location.Y)
	}
	if location.Floor != "1F" {
		t.Errorf("Expected Floor='1F', got '%s'", location.Floor)
	}
}
