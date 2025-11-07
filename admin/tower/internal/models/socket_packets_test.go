package models

import (
	"encoding/json"
	"testing"
)

func TestRegisterPacketJSON(t *testing.T) {
	packet := RegisterPacket{
		RobotID:   "robot-1",
		Name:      "Test Robot",
		EntityID:  "entity-1",
		Battery:   80.5,
		Timestamp: 1234567890,
	}

	data, err := json.Marshal(packet)
	if err != nil {
		t.Fatalf("Failed to marshal RegisterPacket: %v", err)
	}

	var decoded RegisterPacket
	err = json.Unmarshal(data, &decoded)
	if err != nil {
		t.Fatalf("Failed to unmarshal RegisterPacket: %v", err)
	}

	if decoded.RobotID != packet.RobotID {
		t.Errorf("Expected RobotID '%s', got '%s'", packet.RobotID, decoded.RobotID)
	}
	if decoded.Name != packet.Name {
		t.Errorf("Expected Name '%s', got '%s'", packet.Name, decoded.Name)
	}
	if decoded.EntityID != packet.EntityID {
		t.Errorf("Expected EntityID '%s', got '%s'", packet.EntityID, decoded.EntityID)
	}
	if decoded.Battery != packet.Battery {
		t.Errorf("Expected Battery %.1f, got %.1f", packet.Battery, decoded.Battery)
	}
	if decoded.Timestamp != packet.Timestamp {
		t.Errorf("Expected Timestamp %d, got %d", packet.Timestamp, decoded.Timestamp)
	}
}

func TestTaskAssignedPacketJSON(t *testing.T) {
	packet := TaskAssignedPacket{
		TaskID: "task-1",
		Type:   "delivery",
		Sources: []LocationPacket{
			{X: 100.0, Y: 200.0, Z: 0.0, Floor: "1F"},
		},
		Terminals: []LocationPacket{
			{X: 300.0, Y: 400.0, Z: 0.0, Floor: "2F"},
		},
		Priority: "high",
		Metadata: map[string]string{
			"customer": "Alice",
			"item":     "Package",
		},
		AssignedAt: 1234567890,
	}

	data, err := json.Marshal(packet)
	if err != nil {
		t.Fatalf("Failed to marshal TaskAssignedPacket: %v", err)
	}

	var decoded TaskAssignedPacket
	err = json.Unmarshal(data, &decoded)
	if err != nil {
		t.Fatalf("Failed to unmarshal TaskAssignedPacket: %v", err)
	}

	if decoded.TaskID != packet.TaskID {
		t.Errorf("Expected TaskID '%s', got '%s'", packet.TaskID, decoded.TaskID)
	}
	if decoded.Type != packet.Type {
		t.Errorf("Expected Type '%s', got '%s'", packet.Type, decoded.Type)
	}
	if len(decoded.Sources) != 1 {
		t.Errorf("Expected 1 source, got %d", len(decoded.Sources))
	}
	if len(decoded.Terminals) != 1 {
		t.Errorf("Expected 1 terminal, got %d", len(decoded.Terminals))
	}
	if decoded.Priority != packet.Priority {
		t.Errorf("Expected Priority '%s', got '%s'", packet.Priority, decoded.Priority)
	}
	if len(decoded.Metadata) != 2 {
		t.Errorf("Expected 2 metadata entries, got %d", len(decoded.Metadata))
	}
}

func TestLocationPacketJSON(t *testing.T) {
	packet := LocationPacket{
		X:     123.45,
		Y:     678.90,
		Z:     1.5,
		Floor: "3F",
	}

	data, err := json.Marshal(packet)
	if err != nil {
		t.Fatalf("Failed to marshal LocationPacket: %v", err)
	}

	var decoded LocationPacket
	err = json.Unmarshal(data, &decoded)
	if err != nil {
		t.Fatalf("Failed to unmarshal LocationPacket: %v", err)
	}

	if decoded.X != packet.X {
		t.Errorf("Expected X %.2f, got %.2f", packet.X, decoded.X)
	}
	if decoded.Y != packet.Y {
		t.Errorf("Expected Y %.2f, got %.2f", packet.Y, decoded.Y)
	}
	if decoded.Z != packet.Z {
		t.Errorf("Expected Z %.2f, got %.2f", packet.Z, decoded.Z)
	}
	if decoded.Floor != packet.Floor {
		t.Errorf("Expected Floor '%s', got '%s'", packet.Floor, decoded.Floor)
	}
}

func TestStatusUpdatePacketJSON(t *testing.T) {
	packet := StatusUpdatePacket{
		RobotID: "robot-1",
		State:   "busy",
		CurrentLocation: LocationPacket{
			X:     100.0,
			Y:     200.0,
			Z:     0.0,
			Floor: "1F",
		},
		Battery:       75.5,
		CurrentTaskID: "task-1",
		Timestamp:     1234567890,
	}

	data, err := json.Marshal(packet)
	if err != nil {
		t.Fatalf("Failed to marshal StatusUpdatePacket: %v", err)
	}

	var decoded StatusUpdatePacket
	err = json.Unmarshal(data, &decoded)
	if err != nil {
		t.Fatalf("Failed to unmarshal StatusUpdatePacket: %v", err)
	}

	if decoded.RobotID != packet.RobotID {
		t.Errorf("Expected RobotID '%s', got '%s'", packet.RobotID, decoded.RobotID)
	}
	if decoded.State != packet.State {
		t.Errorf("Expected State '%s', got '%s'", packet.State, decoded.State)
	}
	if decoded.Battery != packet.Battery {
		t.Errorf("Expected Battery %.1f, got %.1f", packet.Battery, decoded.Battery)
	}
	if decoded.CurrentTaskID != packet.CurrentTaskID {
		t.Errorf("Expected CurrentTaskID '%s', got '%s'", packet.CurrentTaskID, decoded.CurrentTaskID)
	}
}

func TestTaskUpdatePacketJSON(t *testing.T) {
	packet := TaskUpdatePacket{
		TaskID:    "task-1",
		RobotID:   "robot-1",
		Status:    "in_progress",
		Progress:  50,
		Message:   "Halfway there",
		Timestamp: 1234567890,
	}

	data, err := json.Marshal(packet)
	if err != nil {
		t.Fatalf("Failed to marshal TaskUpdatePacket: %v", err)
	}

	var decoded TaskUpdatePacket
	err = json.Unmarshal(data, &decoded)
	if err != nil {
		t.Fatalf("Failed to unmarshal TaskUpdatePacket: %v", err)
	}

	if decoded.TaskID != packet.TaskID {
		t.Errorf("Expected TaskID '%s', got '%s'", packet.TaskID, decoded.TaskID)
	}
	if decoded.RobotID != packet.RobotID {
		t.Errorf("Expected RobotID '%s', got '%s'", packet.RobotID, decoded.RobotID)
	}
	if decoded.Status != packet.Status {
		t.Errorf("Expected Status '%s', got '%s'", packet.Status, decoded.Status)
	}
	if decoded.Progress != packet.Progress {
		t.Errorf("Expected Progress %d, got %d", packet.Progress, decoded.Progress)
	}
	if decoded.Message != packet.Message {
		t.Errorf("Expected Message '%s', got '%s'", packet.Message, decoded.Message)
	}
}

func TestKeepAlivePacketJSON(t *testing.T) {
	packet := KeepAlivePacket{
		RobotID:   "robot-1",
		Timestamp: 1234567890,
	}

	data, err := json.Marshal(packet)
	if err != nil {
		t.Fatalf("Failed to marshal KeepAlivePacket: %v", err)
	}

	var decoded KeepAlivePacket
	err = json.Unmarshal(data, &decoded)
	if err != nil {
		t.Fatalf("Failed to unmarshal KeepAlivePacket: %v", err)
	}

	if decoded.RobotID != packet.RobotID {
		t.Errorf("Expected RobotID '%s', got '%s'", packet.RobotID, decoded.RobotID)
	}
	if decoded.Timestamp != packet.Timestamp {
		t.Errorf("Expected Timestamp %d, got %d", packet.Timestamp, decoded.Timestamp)
	}
}

func TestPingPongPacketsJSON(t *testing.T) {
	pingPacket := PingPacket{
		Timestamp: 1234567890,
	}

	data, err := json.Marshal(pingPacket)
	if err != nil {
		t.Fatalf("Failed to marshal PingPacket: %v", err)
	}

	var decodedPing PingPacket
	err = json.Unmarshal(data, &decodedPing)
	if err != nil {
		t.Fatalf("Failed to unmarshal PingPacket: %v", err)
	}

	if decodedPing.Timestamp != pingPacket.Timestamp {
		t.Errorf("Expected Timestamp %d, got %d", pingPacket.Timestamp, decodedPing.Timestamp)
	}

	pongPacket := PongPacket{
		Timestamp: 1234567891,
	}

	data, err = json.Marshal(pongPacket)
	if err != nil {
		t.Fatalf("Failed to marshal PongPacket: %v", err)
	}

	var decodedPong PongPacket
	err = json.Unmarshal(data, &decodedPong)
	if err != nil {
		t.Fatalf("Failed to unmarshal PongPacket: %v", err)
	}

	if decodedPong.Timestamp != pongPacket.Timestamp {
		t.Errorf("Expected Timestamp %d, got %d", pongPacket.Timestamp, decodedPong.Timestamp)
	}
}

func TestEventConstants(t *testing.T) {
	expectedEvents := map[string]string{
		"EventConnect":      "connect",
		"EventDisconnect":   "disconnect",
		"EventRegister":     "register",
		"EventTaskAssigned": "task_assigned",
		"EventTaskUpdate":   "task_update",
		"EventStatusUpdate": "status_update",
		"EventKeepAlive":    "keep_alive",
		"EventPing":         "ping",
		"EventPong":         "pong",
	}

	actualEvents := map[string]string{
		"EventConnect":      EventConnect,
		"EventDisconnect":   EventDisconnect,
		"EventRegister":     EventRegister,
		"EventTaskAssigned": EventTaskAssigned,
		"EventTaskUpdate":   EventTaskUpdate,
		"EventStatusUpdate": EventStatusUpdate,
		"EventKeepAlive":    EventKeepAlive,
		"EventPing":         EventPing,
		"EventPong":         EventPong,
	}

	for name, expected := range expectedEvents {
		actual := actualEvents[name]
		if actual != expected {
			t.Errorf("Expected %s to be '%s', got '%s'", name, expected, actual)
		}
	}
}
