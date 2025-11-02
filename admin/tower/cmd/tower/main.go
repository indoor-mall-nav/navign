package main

import (
	"flag"
	"log"
	"os"
	"os/signal"
	"syscall"

	socketio "github.com/googollee/go-socket.io"
	"github.com/googollee/go-socket.io/engineio"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	"github.com/indoor-mall-nav/navign/admin/tower/internal/controller"
)

func main() {
	entityID := flag.String("entity-id", "", "Entity ID")
	grpcAddr := flag.String("grpc", "localhost:50051", "Rust orchestrator gRPC address")
	towerAddr := flag.String("tower", "http://[::1]:8080", "Controller server address")
	flag.Parse()

	creds := grpc.WithTransportCredentials(insecure.NewCredentials())

	grpc_client, err := grpc.NewClient(*grpcAddr, creds)

	if err != nil {
		log.Fatalf("Failed to create gRPC client: %v", err)
	}

	defer grpc_client.Close()

	socket := socketio.NewServer(&engineio.Options{})

	defer socket.Close()

	if *entityID == "" {
		log.Fatal("entity-id is required")
	}

	startPayload := &controller.Controller{
		Entity: *entityID,
		GRPC:   *grpcAddr,
		Tower:  *towerAddr,
	}

	// Start controller
	if err := controller.Start(startPayload); err != nil {
		log.Fatalf("Failed to start controller: %v", err)
	}

	log.Printf("Controller running for entity: %s", *entityID)

	// Graceful shutdown
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, os.Interrupt, syscall.SIGTERM)
	<-sigCh

	log.Println("Shutting down controller...")
	controller.Stop()
}
