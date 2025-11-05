package controller

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"sync"
	"time"

	socketio "github.com/googollee/go-socket.io"
	"github.com/googollee/go-socket.io/engineio"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	"github.com/indoor-mall-nav/navign/admin/tower/internal/robot"
	"github.com/indoor-mall-nav/navign/admin/tower/internal/socket_server"
	pb "github.com/indoor-mall-nav/navign/admin/tower/proto"
)

type Controller struct {
	Entity string `json:"entity"`
	GRPC   string `json:"grpc"`  // Orchestrator gRPC address (we connect as client)
	Tower  string `json:"tower"` // Our Socket.IO server address
}

var (
	instance      *controllerInstance
	instanceMutex sync.Mutex
)

type controllerInstance struct {
	grpcConn     *grpc.ClientConn
	grpcClient   pb.OrchestratorServiceClient
	httpServer   *http.Server
	socketServer *socket_server.Server
	robotManager *robot.Manager
	taskStream   pb.OrchestratorService_GetTaskAssignmentClient
	stopChan     chan struct{}
}

func Start(c *Controller) error {
	instanceMutex.Lock()
	defer instanceMutex.Unlock()

	if instance != nil {
		return fmt.Errorf("controller already running")
	}

	log.Printf("Starting controller for entity: %s", c.Entity)

	// Connect to Rust orchestrator as gRPC client
	creds := grpc.WithTransportCredentials(insecure.NewCredentials())
	grpcConn, err := grpc.NewClient(c.GRPC, creds)
	if err != nil {
		return fmt.Errorf("failed to connect to orchestrator at %s: %v", c.GRPC, err)
	}

	grpcClient := pb.NewOrchestratorServiceClient(grpcConn)
	log.Printf("Connected to orchestrator at %s", c.GRPC)

	// Create robot manager
	robotMgr := robot.NewManager(c.Entity, grpcClient)

	// Create Socket.IO server
	socketIO := socketio.NewServer(&engineio.Options{})

	// Create socket server
	sockServer := socket_server.NewServer(socketIO, robotMgr)

	// Start Socket.IO HTTP server
	mux := http.NewServeMux()
	mux.Handle("/socket.io/", socketIO)

	httpServer := &http.Server{
		Addr:    c.Tower,
		Handler: mux,
	}

	go func() {
		log.Printf("Starting Socket.IO server on %s", c.Tower)
		if err := httpServer.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("HTTP server failed: %v", err)
		}
	}()

	// Start task assignment stream from orchestrator
	stopChan := make(chan struct{})
	taskStream, err := grpcClient.GetTaskAssignment(context.Background(), &pb.RobotDistributionRequest{
		EntityId: c.Entity,
	})
	if err != nil {
		grpcConn.Close()
		httpServer.Close()
		return fmt.Errorf("failed to get task assignment stream: %v", err)
	}

	// Start goroutine to receive task assignments from orchestrator
	go func() {
		log.Println("Listening for task assignments from orchestrator...")
		for {
			select {
			case <-stopChan:
				return
			default:
				assignment, err := taskStream.Recv()
				if err != nil {
					log.Printf("Task stream error: %v", err)
					time.Sleep(5 * time.Second) // Wait before potential reconnect
					continue
				}

				if assignment.Task != nil {
					log.Printf("Received task assignment: %s for robot %s",
						assignment.Task.Id, assignment.RobotId)

					// Send task to robot via Socket.IO
					if err := sockServer.SendTaskToRobot(assignment.RobotId, assignment.Task); err != nil {
						log.Printf("Failed to send task to robot: %v", err)
					}
				}
			}
		}
	}()

	// Store instance for cleanup
	instance = &controllerInstance{
		grpcConn:     grpcConn,
		grpcClient:   grpcClient,
		httpServer:   httpServer,
		socketServer: sockServer,
		robotManager: robotMgr,
		taskStream:   taskStream,
		stopChan:     stopChan,
	}

	log.Println("Controller started successfully")
	return nil
}

func Stop() {
	instanceMutex.Lock()
	defer instanceMutex.Unlock()

	if instance == nil {
		return
	}

	log.Println("Stopping controller...")

	// Stop task stream listener
	close(instance.stopChan)

	// Shutdown HTTP server
	if instance.httpServer != nil {
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		instance.httpServer.Shutdown(ctx)
	}

	// Close gRPC connection
	if instance.grpcConn != nil {
		instance.grpcConn.Close()
	}

	instance = nil
	log.Println("Controller stopped")
}
