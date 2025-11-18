#pragma once

#include <vector>
#include <opencv2/opencv.hpp>
#include <apriltag/apriltag.h>
#include <apriltag/tag36h11.h>
#include <apriltag/apriltag_pose.h>

namespace navign::robot::vision {

/**
 * @brief Result of AprilTag detection
 */
struct AprilTagResult {
    uint32_t tag_id;
    cv::Point2d center;
    std::vector<cv::Point2d> corners;  // 4 corners
    double decision_margin;
    int hamming_distance;

    // Pose estimation (if calibration available)
    bool pose_valid = false;
    cv::Mat rotation;     // 3x3 rotation matrix
    cv::Mat translation;  // 3x1 translation vector
    cv::Point3d position; // Tag position in world coordinates
};

/**
 * @brief AprilTag detector using apriltag C library
 */
class AprilTagDetector {
public:
    AprilTagDetector();
    ~AprilTagDetector();

    /**
     * @brief Detect AprilTags in an image
     * @param image Input image (grayscale or BGR)
     * @param camera_matrix Camera intrinsic matrix (3x3) for pose estimation
     * @param dist_coeffs Distortion coefficients (optional)
     * @param tag_size Physical tag size in meters (for pose estimation)
     * @return Vector of detected tags
     */
    std::vector<AprilTagResult> detect(
        const cv::Mat& image,
        const cv::Mat& camera_matrix = cv::Mat(),
        const cv::Mat& dist_coeffs = cv::Mat(),
        double tag_size = 0.015
    );

    /**
     * @brief Set detection parameters
     */
    void setNumThreads(int threads);
    void setQuadDecimate(float decimate);
    void setQuadSigma(float sigma);
    void setRefineEdges(bool refine);
    void setDecodeSharpening(double sharpening);

private:
    apriltag_detector_t* detector_ = nullptr;
    apriltag_family_t* tag_family_ = nullptr;

    // Estimate pose for a single tag
    bool estimatePose(
        zarray_t* detections,
        int idx,
        const cv::Mat& camera_matrix,
        double tag_size,
        AprilTagResult& result
    );
};

} // namespace navign::robot::vision
