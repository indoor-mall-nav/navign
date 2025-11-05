package socket_server

import (
	"encoding/json"
	"fmt"
	"log"
	"time"

	socketio "github.com/googollee/go-socket.io"
	pb "github.com/indoor-mall-nav/navign/admin/tower/proto"
	"github.com/indoor-mall-nav/navign/admin/tower/internal/models"
	"github.com/indoor-mall-nav/navign/admin/tower/internal/robot"
)

// Server manages Socket.IO connections with robots
type Server struct {
	io            *socketio.Server
	robotManager  *robot.Manager
}

// NewServer creates a new Socket.IO server
func NewServer(io *socketio.Server, robotMgr *robot.Manager) *Server {
	s := &Server{
		io:           io,
		robotManager: robotMgr,
	}
	
	s.setupHandlers()
	return s
}

// setupHandlers configures Socket.IO event handlers
func (s *Server) setupHandlers() {
	s.io.OnConnect("/", func(conn socketio.Conn) error {
		log.Printf("Socket connected: %s", conn.ID())
		return nil
	})

	s.io.OnError("/", func(conn socketio.Conn, err error) {
		log.Printf("Socket error for %s: %v", conn.ID(), err)
	})

	s.io.OnDisconnect("/", func(conn socketio.Conn, reason string) {
		log.Printf("Socket disconnected: %s, reason: %s", conn.ID(), reason)
		s.handleDisconnect(conn)
	})

	// Handle robot registration
	s.io.OnEvent("/", models.EventRegister, func(conn socketio.Conn, data string) {
		var packet models.RegisterPacket
		if err := json.Unmarshal([]byte(data), &packet); err != nil {
			log.Printf("Failed to unmarshal register packet: %v", err)
			return
		}
		s.handleRegister(conn, &packet)
	})

	// Handle status updates from robots
	s.io.OnEvent("/", models.EventStatusUpdate, func(conn socketio.Conn, data string) {
		var packet models.StatusUpdatePacket
		if err := json.Unmarshal([]byte(data), &packet); err != nil {
			log.Printf("Failed to unmarshal status update: %v", err)
			return
		}
		s.handleStatusUpdate(&packet)
	})

	// Handle task updates from robots
	s.io.OnEvent("/", models.EventTaskUpdate, func(conn socketio.Conn, data string) {
		var packet models.TaskUpdatePacket
		if err := json.Unmarshal([]byte(data), &packet); err != nil {
			log.Printf("Failed to unmarshal task update: %v", err)
			return
		}
		s.handleTaskUpdate(&packet)
	})

	// Handle keep-alive/ping from robots
	s.io.OnEvent("/", models.EventPing, func(conn socketio.Conn, data string) {
		var packet models.PingPacket
		if err := json.Unmarshal([]byte(data), &packet); err != nil {
			log.Printf("Failed to unmarshal ping: %v", err)
			return
		}
		s.handlePing(conn, &packet)
	})
}

// handleRegister processes robot registration
func (s *Server) handleRegister(conn socketio.Conn, packet *models.RegisterPacket) {
	log.Printf("Robot registering: ID=%s, Name=%s, Entity=%s, Battery=%.1f%%",
		packet.RobotID, packet.Name, packet.EntityID, packet.Battery)

	// Register robot with manager (this starts the keep-alive goroutine)
	r := &robot.Robot{
		ID:       packet.RobotID,
		Name:     packet.Name,
		EntityID: packet.EntityID,
		State:    pb.RobotState_ROBOT_STATE_IDLE,
		Battery:  packet.Battery,
	}
	s.robotManager.RegisterRobot(r)

	// Store connection mapping
	conn.SetContext(packet.RobotID)
}

// handleDisconnect processes robot disconnection
func (s *Server) handleDisconnect(conn socketio.Conn) {
	robotID, ok := conn.Context().(string)
	if !ok {
		return
	}

	log.Printf("Robot disconnected: %s", robotID)

	// Unregister robot (this stops the keep-alive goroutine)
	s.robotManager.UnregisterRobot(robotID)
}

// handleStatusUpdate processes status updates from robots
func (s *Server) handleStatusUpdate(packet *models.StatusUpdatePacket) {
	state := parseRobotState(packet.State)
	location := &pb.Location{
		X:     packet.CurrentLocation.X,
		Y:     packet.CurrentLocation.Y,
		Z:     packet.CurrentLocation.Z,
		Floor: packet.CurrentLocation.Floor,
	}

	s.robotManager.UpdateRobotStatus(
		packet.RobotID,
		state,
		location,
		packet.Battery,
		packet.CurrentTaskID,
	)
}

// handleTaskUpdate processes task updates from robots
func (s *Server) handleTaskUpdate(packet *models.TaskUpdatePacket) {
	log.Printf("Task update: TaskID=%s, RobotID=%s, Status=%s, Progress=%d%%",
		packet.TaskID, packet.RobotID, packet.Status, packet.Progress)

	// Update robot state based on task status
	if packet.Status == "completed" || packet.Status == "failed" {
		// Task finished, robot becomes idle
		s.robotManager.UpdateRobotStatus(
			packet.RobotID,
			pb.RobotState_ROBOT_STATE_IDLE,
			nil,
			0, // Don't update battery here
			"",
		)
	}
}

// handlePing responds to ping from robots
func (s *Server) handlePing(conn socketio.Conn, packet *models.PingPacket) {
	robotID, ok := conn.Context().(string)
	if ok {
		s.robotManager.UpdateHeartbeat(robotID)
	}

	// Send pong response
	pong := models.PongPacket{
		Timestamp: time.Now().Unix(),
	}
	data, _ := json.Marshal(pong)
	conn.Emit(models.EventPong, string(data))
}

// SendTaskToRobot sends a task assignment to a specific robot
func (s *Server) SendTaskToRobot(robotID string, task *pb.Task) error {
	log.Printf("Sending task %s to robot %s", task.Id, robotID)

	// Find the connection for this robot
	var targetConn socketio.Conn
	s.io.ForEach("/", "", func(conn socketio.Conn) {
		if id, ok := conn.Context().(string); ok && id == robotID {
			targetConn = conn
		}
	})

	if targetConn == nil {
		return fmt.Errorf("robot %s not found in active connections", robotID)
	}

	// Convert protobuf task to socket packet
	packet := models.TaskAssignedPacket{
		TaskID:     task.Id,
		Type:       taskTypeToString(task.Type),
		Sources:    convertLocations(task.Sources),
		Terminals:  convertLocations(task.Terminals),
		Priority:   priorityToString(task.Priority),
		Metadata:   task.Metadata,
		AssignedAt: time.Now().Unix(),
	}

	data, err := json.Marshal(packet)
	if err != nil {
		return err
	}

	targetConn.Emit(models.EventTaskAssigned, string(data))
	return nil
}

// Helper functions

func parseRobotState(state string) pb.RobotState {
	switch state {
	case "idle":
		return pb.RobotState_ROBOT_STATE_IDLE
	case "busy":
		return pb.RobotState_ROBOT_STATE_BUSY
	case "charging":
		return pb.RobotState_ROBOT_STATE_CHARGING
	case "error":
		return pb.RobotState_ROBOT_STATE_ERROR
	default:
		return pb.RobotState_ROBOT_STATE_UNSPECIFIED
	}
}

func taskTypeToString(taskType pb.TaskType) string {
	switch taskType {
	case pb.TaskType_TASK_TYPE_DELIVERY:
		return "delivery"
	case pb.TaskType_TASK_TYPE_PATROL:
		return "patrol"
	case pb.TaskType_TASK_TYPE_RETURN_HOME:
		return "return_home"
	case pb.TaskType_TASK_TYPE_EMERGENCY:
		return "emergency"
	default:
		return "unspecified"
	}
}

func priorityToString(priority pb.Priority) string {
	switch priority {
	case pb.Priority_PRIORITY_LOW:
		return "low"
	case pb.Priority_PRIORITY_NORMAL:
		return "normal"
	case pb.Priority_PRIORITY_HIGH:
		return "high"
	case pb.Priority_PRIORITY_URGENT:
		return "urgent"
	default:
		return "unspecified"
	}
}

func convertLocations(locations []*pb.Location) []models.LocationPacket {
	result := make([]models.LocationPacket, len(locations))
	for i, loc := range locations {
		result[i] = models.LocationPacket{
			X:     loc.X,
			Y:     loc.Y,
			Z:     loc.Z,
			Floor: loc.Floor,
		}
	}
	return result
}
