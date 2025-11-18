#include "vision_service.hpp"
#include "apriltag_detector.hpp"
#include "object_detector.hpp"
#include "camera_calibration.hpp"
#include "coordinate_transform.hpp"

#include <iostream>
#include <chrono>
#include <thread>

namespace navign::robot::vision {

VisionService::VisionService() {
    // Initialize components
    apriltag_detector_ = std::make_unique<AprilTagDetector>();
    object_detector_ = std::make_unique<ObjectDetector>();
    camera_calibration_ = std::make_unique<CameraCalibration>();
    coordinate_transform_ = std::make_unique<CoordinateTransform>();
}

VisionService::~VisionService() {
    stop();
}

bool VisionService::start() {
    if (running_.load()) {
        std::cerr << "Vision service already running" << std::endl;
        return false;
    }

    std::cout << "Starting Vision service..." << std::endl;

    // Initialize camera
    std::cout << "Opening camera " << camera_index_ << "..." << std::endl;
    camera_.open(camera_index_);

    if (!camera_.isOpened()) {
        std::cerr << "Failed to open camera " << camera_index_ << std::endl;
        return false;
    }

    // Set camera properties
    camera_.set(cv::CAP_PROP_FRAME_WIDTH, 640);
    camera_.set(cv::CAP_PROP_FRAME_HEIGHT, 480);
    camera_.set(cv::CAP_PROP_FPS, target_fps_);

    // Load camera calibration if available
    if (camera_calibration_->load("calibration.yml")) {
        std::cout << "Camera calibration loaded" << std::endl;
        const auto& calib = camera_calibration_->getCalibration();
        coordinate_transform_->setCalibration(calib.camera_matrix, calib.dist_coeffs);
    } else {
        std::cout << "No calibration file found - pose estimation will be less accurate" << std::endl;
    }

    // Load YOLO model
    std::cout << "Loading YOLO model..." << std::endl;
    if (!object_detector_->loadModel("yolov8n.onnx")) {
        std::cerr << "Warning: Failed to load YOLO model - object detection disabled" << std::endl;
    }

    // Load class names
    if (!object_detector_->loadClassNames("coco.names")) {
        std::cerr << "Warning: Failed to load class names" << std::endl;
    }

    // Initialize Zenoh
    if (!initializeZenoh()) {
        std::cerr << "Warning: Zenoh initialization failed - pub/sub disabled" << std::endl;
    }

    // Start processing loop
    running_.store(true);
    processing_thread_ = std::thread(&VisionService::processingLoop, this);

    std::cout << "Vision service started successfully" << std::endl;
    return true;
}

void VisionService::stop() {
    if (!running_.load()) {
        return;
    }

    std::cout << "Stopping Vision service..." << std::endl;
    running_.store(false);

    if (processing_thread_.joinable()) {
        processing_thread_.join();
    }

    if (camera_.isOpened()) {
        camera_.release();
    }

    std::cout << "Vision service stopped" << std::endl;
}

void VisionService::processingLoop() {
    const auto frame_duration = std::chrono::milliseconds(1000 / target_fps_);

    while (running_.load()) {
        auto start_time = std::chrono::steady_clock::now();

        cv::Mat frame;
        camera_ >> frame;

        if (frame.empty()) {
            std::cerr << "Failed to read frame from camera" << std::endl;
            std::this_thread::sleep_for(std::chrono::milliseconds(100));
            continue;
        }

        frame_count_++;
        total_frames_processed_++;

        // Detect AprilTags
        cv::Mat camera_matrix, dist_coeffs;
        if (camera_calibration_->isValid()) {
            const auto& calib = camera_calibration_->getCalibration();
            camera_matrix = calib.camera_matrix;
            dist_coeffs = calib.dist_coeffs;
        }

        auto tags = apriltag_detector_->detect(frame, camera_matrix, dist_coeffs, apriltag_size_);
        total_tags_detected_ += tags.size();

        if (!tags.empty()) {
            std::cout << "Detected " << tags.size() << " AprilTags" << std::endl;
            for (const auto& tag : tags) {
                std::cout << "  Tag ID " << tag.tag_id << " at ("
                          << tag.center.x << ", " << tag.center.y << ")" << std::endl;
                if (tag.pose_valid) {
                    std::cout << "    Position: (" << tag.position.x << ", "
                              << tag.position.y << ", " << tag.position.z << ")" << std::endl;
                }
            }
        }

        // Detect objects with YOLO
        auto objects = object_detector_->detect(frame, 0.5f, 0.4f);
        total_objects_detected_ += objects.size();

        if (!objects.empty()) {
            std::cout << "Detected " << objects.size() << " objects" << std::endl;
            for (const auto& obj : objects) {
                std::cout << "  " << obj.class_name << " ("
                          << obj.confidence << ") at ("
                          << obj.center.x << ", " << obj.center.y << ")" << std::endl;
            }
        }

        // Publish updates (if Zenoh is available)
        // TODO: Implement Zenoh publishing

        // Publish status periodically
        if (frame_count_ % 100 == 0) {
            publishStatus();
        }

        // Control frame rate
        auto end_time = std::chrono::steady_clock::now();
        auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(end_time - start_time);
        auto sleep_time = frame_duration - elapsed;

        if (sleep_time.count() > 0) {
            std::this_thread::sleep_for(sleep_time);
        }
    }
}

bool VisionService::initializeZenoh() {
    // TODO: Initialize Zenoh session
    // This requires zenoh-cpp library
    std::cout << "Zenoh initialization not yet implemented" << std::endl;
    return false;
}

void VisionService::publishAprilTags() {
    // TODO: Publish AprilTag detections via Zenoh
}

void VisionService::publishObjects() {
    // TODO: Publish object detections via Zenoh
}

void VisionService::publishStatus() {
    std::cout << "Vision Status:" << std::endl;
    std::cout << "  Frames processed: " << total_frames_processed_ << std::endl;
    std::cout << "  Tags detected: " << total_tags_detected_ << std::endl;
    std::cout << "  Objects detected: " << total_objects_detected_ << std::endl;
    std::cout << "  Average FPS: " << (total_frames_processed_ / (frame_count_ / static_cast<float>(target_fps_))) << std::endl;
}

} // namespace navign::robot::vision
