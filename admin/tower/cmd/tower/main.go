package main

import (
	"flag"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/indoor-mall-nav/navign/admin/tower/internal/controller"
)

func main() {
	entityID := flag.String("entity-id", "", "Entity ID")
	grpcAddr := flag.String("grpc", "localhost:50051", "Rust orchestrator gRPC address")
	atcAddr := flag.String("atc", "http://[::1]:8080", "Controller server address")
	flag.Parse()

	if *entityID == "" {
		log.Fatal("entity-id is required")
	}

	startPayload := &controller.Controller{
		Entity: *entityID,
		GRPC:   *grpcAddr,
		Tower:  *atcAddr,
	}

	// Start controller
	if err := controller.Start(startPayload); err != nil {
		log.Fatalf("Failed to start tower: %v", err)
	}

	log.Printf("Tower running for entity: %s", *entityID)

	// Graceful shutdown
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, os.Interrupt, syscall.SIGTERM)
	<-sigCh

	log.Println("Shutting down tower...")
	controller.Stop()
}
