#pragma once

#include <atomic>
#include <memory>
#include <thread>
#include <opencv2/opencv.hpp>

// Forward declarations
namespace navign::robot::vision {
    class AprilTagDetector;
    class ObjectDetector;
    class CameraCalibration;
    class CoordinateTransform;
}

namespace navign::robot::vision {

/**
 * @brief Main vision service for robot perception
 *
 * Provides:
 * - AprilTag detection and pose estimation
 * - YOLO-based object detection
 * - MediaPipe hand tracking (optional)
 * - Camera calibration
 * - 2D-3D coordinate transformation
 */
class VisionService {
public:
    VisionService();
    ~VisionService();

    // Lifecycle
    bool start();
    void stop();
    bool isRunning() const { return running_.load(); }

    // Configuration
    void setCameraIndex(int index) { camera_index_ = index; }
    void setFrameRate(int fps) { target_fps_ = fps; }
    void setAprilTagSize(double size_meters) { apriltag_size_ = size_meters; }

    // Component access (for testing)
    AprilTagDetector* getAprilTagDetector() { return apriltag_detector_.get(); }
    ObjectDetector* getObjectDetector() { return object_detector_.get(); }
    CameraCalibration* getCameraCalibration() { return camera_calibration_.get(); }

private:
    // Processing loop
    void processingLoop();

    // Zenoh messaging
    bool initializeZenoh();
    void publishAprilTags();
    void publishObjects();
    void publishStatus();

    // Camera
    cv::VideoCapture camera_;
    int camera_index_ = 0;
    int target_fps_ = 30;

    // Components
    std::unique_ptr<AprilTagDetector> apriltag_detector_;
    std::unique_ptr<ObjectDetector> object_detector_;
    std::unique_ptr<CameraCalibration> camera_calibration_;
    std::unique_ptr<CoordinateTransform> coordinate_transform_;
    // TODO: Add hand_tracker_ when MediaPipe C++ is implemented

    // State
    std::atomic<bool> running_{false};
    std::thread processing_thread_;
    uint32_t frame_count_ = 0;
    double apriltag_size_ = 0.015;  // 15mm default

    // Metrics
    uint32_t total_frames_processed_ = 0;
    uint32_t total_tags_detected_ = 0;
    uint32_t total_objects_detected_ = 0;
};

} // namespace navign::robot::vision
