package controller

import (
	"fmt"
	"log"
	"net"
	"net/http"
	"sync"

	socketio "github.com/googollee/go-socket.io"
	"github.com/googollee/go-socket.io/engineio"
	"google.golang.org/grpc"

	"github.com/indoor-mall-nav/navign/admin/tower/internal/grpc_server"
	"github.com/indoor-mall-nav/navign/admin/tower/internal/scheduler"
	"github.com/indoor-mall-nav/navign/admin/tower/internal/socket_server"
	pb "github.com/indoor-mall-nav/navign/admin/tower/proto"
)

type Controller struct {
	Entity string `json:"entity"`
	GRPC   string `json:"grpc"`
	Tower  string `json:"tower"`
}

var (
	instance       *controllerInstance
	instanceMutex  sync.Mutex
)

type controllerInstance struct {
	grpcServer   *grpc.Server
	httpServer   *http.Server
	socketServer *socket_server.Server
	scheduler    *scheduler.TaskScheduler
}

func Start(c *Controller) error {
	instanceMutex.Lock()
	defer instanceMutex.Unlock()

	if instance != nil {
		return fmt.Errorf("controller already running")
	}

	log.Printf("Starting controller for entity: %s", c.Entity)

	// Create scheduler
	sched := scheduler.NewTaskScheduler(c.Entity)

	// Create Socket.IO server
	socketIO := socketio.NewServer(&engineio.Options{})

	// Distribution change callback (reports to orchestrator)
	distributionCb := func() {
		log.Printf("Robot distribution changed, notifying orchestrator...")
		// In a real implementation, this would make a callback to the orchestrator
		// For now, we just log it
	}

	// Create socket server
	sockServer := socket_server.NewServer(socketIO, sched, distributionCb)

	// Task assignment callback (sends task to robot via Socket.IO)
	taskAssignedCb := func(robotID string, task *pb.Task) error {
		return sockServer.SendTaskToRobot(robotID, task)
	}

	// Create gRPC server
	grpcSrv := grpc_server.NewServer(sched, taskAssignedCb)

	// Start gRPC server
	lis, err := net.Listen("tcp", c.GRPC)
	if err != nil {
		return fmt.Errorf("failed to listen on %s: %v", c.GRPC, err)
	}

	grpcServer := grpc.NewServer()
	pb.RegisterTaskSchedulerServer(grpcServer, grpcSrv)

	go func() {
		log.Printf("Starting gRPC server on %s", c.GRPC)
		if err := grpcServer.Serve(lis); err != nil {
			log.Fatalf("gRPC server failed: %v", err)
		}
	}()

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

	// Store instance for cleanup
	instance = &controllerInstance{
		grpcServer:   grpcServer,
		httpServer:   httpServer,
		socketServer: sockServer,
		scheduler:    sched,
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

	// Gracefully stop gRPC server
	if instance.grpcServer != nil {
		instance.grpcServer.GracefulStop()
	}

	// Shutdown HTTP server
	if instance.httpServer != nil {
		instance.httpServer.Close()
	}

	instance = nil
	log.Println("Controller stopped")
}
