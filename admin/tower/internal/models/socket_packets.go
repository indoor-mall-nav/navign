package models

// Socket.IO event names
const (
	EventConnect      = "connect"
	EventDisconnect   = "disconnect"
	EventRegister     = "register"
	EventTaskAssigned = "task_assigned"
	EventTaskUpdate   = "task_update"
	EventStatusUpdate = "status_update"
	EventKeepAlive    = "keep_alive"
	EventPing         = "ping"
	EventPong         = "pong"
)

// RegisterPacket is sent by robot when connecting
type RegisterPacket struct {
	RobotID   string  `json:"robot_id"`
	Name      string  `json:"name"`
	EntityID  string  `json:"entity_id"`
	Battery   float64 `json:"battery"`
	Timestamp int64   `json:"timestamp"`
}

// TaskAssignedPacket is sent to robot when a task is assigned
type TaskAssignedPacket struct {
	TaskID     string            `json:"task_id"`
	Type       string            `json:"type"`
	Sources    []LocationPacket  `json:"sources"`
	Terminals  []LocationPacket  `json:"terminals"`
	Priority   string            `json:"priority"`
	Metadata   map[string]string `json:"metadata,omitempty"`
	AssignedAt int64             `json:"assigned_at"`
}

// LocationPacket represents a location coordinate
type LocationPacket struct {
	X     float64 `json:"x"`
	Y     float64 `json:"y"`
	Z     float64 `json:"z"`
	Floor string  `json:"floor"`
}

// StatusUpdatePacket is sent by robot to report status
type StatusUpdatePacket struct {
	RobotID         string         `json:"robot_id"`
	State           string         `json:"state"` // idle, busy, charging, error
	CurrentLocation LocationPacket `json:"current_location"`
	Battery         float64        `json:"battery"`
	CurrentTaskID   string         `json:"current_task_id,omitempty"`
	Timestamp       int64          `json:"timestamp"`
}

// TaskUpdatePacket is sent by robot to report task progress
type TaskUpdatePacket struct {
	TaskID    string `json:"task_id"`
	RobotID   string `json:"robot_id"`
	Status    string `json:"status"`   // accepted, in_progress, completed, failed
	Progress  int    `json:"progress"` // 0-100
	Message   string `json:"message,omitempty"`
	Timestamp int64  `json:"timestamp"`
}

// KeepAlivePacket is exchanged to maintain connection
type KeepAlivePacket struct {
	RobotID   string `json:"robot_id"`
	Timestamp int64  `json:"timestamp"`
}

// PingPacket for latency measurement
type PingPacket struct {
	Timestamp int64 `json:"timestamp"`
}

// PongPacket response for ping
type PongPacket struct {
	Timestamp int64 `json:"timestamp"`
}
