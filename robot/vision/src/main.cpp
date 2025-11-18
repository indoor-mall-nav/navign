#include "vision_service.hpp"
#include <iostream>
#include <csignal>
#include <atomic>

std::atomic<bool> keep_running{true};

void signalHandler(int signal) {
    std::cout << "\nReceived signal " << signal << ", shutting down..." << std::endl;
    keep_running.store(false);
}

int main(int argc, char** argv) {
    std::cout << "========================================" << std::endl;
    std::cout << "Navign Vision Service (C++)" << std::endl;
    std::cout << "Version 0.1.0" << std::endl;
    std::cout << "========================================" << std::endl;

    // Register signal handlers
    std::signal(SIGINT, signalHandler);
    std::signal(SIGTERM, signalHandler);

    // Parse command-line arguments
    int camera_index = 0;
    int fps = 30;
    double apriltag_size = 0.015; // 15mm

    for (int i = 1; i < argc; i++) {
        std::string arg = argv[i];
        if (arg == "--camera" && i + 1 < argc) {
            camera_index = std::atoi(argv[++i]);
        } else if (arg == "--fps" && i + 1 < argc) {
            fps = std::atoi(argv[++i]);
        } else if (arg == "--tag-size" && i + 1 < argc) {
            apriltag_size = std::atof(argv[++i]);
        } else if (arg == "--help") {
            std::cout << "Usage: " << argv[0] << " [options]\n";
            std::cout << "Options:\n";
            std::cout << "  --camera <index>       Camera device index (default: 0)\n";
            std::cout << "  --fps <fps>            Target frame rate (default: 30)\n";
            std::cout << "  --tag-size <meters>    AprilTag physical size in meters (default: 0.015)\n";
            std::cout << "  --help                 Show this help message\n";
            return 0;
        }
    }

    // Create and configure vision service
    navign::robot::vision::VisionService service;
    service.setCameraIndex(camera_index);
    service.setFrameRate(fps);
    service.setAprilTagSize(apriltag_size);

    // Start service
    if (!service.start()) {
        std::cerr << "Failed to start vision service" << std::endl;
        return 1;
    }

    // Keep running until signal received
    std::cout << "Vision service running... Press Ctrl+C to stop" << std::endl;
    while (keep_running.load()) {
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
    }

    // Stop service
    service.stop();

    std::cout << "Vision service shutdown complete" << std::endl;
    return 0;
}
