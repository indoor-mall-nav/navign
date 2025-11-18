#include "apriltag_detector.hpp"
#include <iostream>

namespace navign::robot::vision {

AprilTagDetector::AprilTagDetector() {
    // Create detector
    detector_ = apriltag_detector_create();

    // Create tag family (tag36h11)
    tag_family_ = tag36h11_create();
    apriltag_detector_add_family(detector_, tag_family_);

    // Default settings
    detector_->quad_decimate = 2.0;
    detector_->quad_sigma = 0.0;
    detector_->nthreads = 4;
    detector_->refine_edges = 1;
    detector_->decode_sharpening = 0.25;
}

AprilTagDetector::~AprilTagDetector() {
    if (detector_) {
        apriltag_detector_destroy(detector_);
    }
    if (tag_family_) {
        tag36h11_destroy(tag_family_);
    }
}

std::vector<AprilTagResult> AprilTagDetector::detect(
    const cv::Mat& image,
    const cv::Mat& camera_matrix,
    const cv::Mat& dist_coeffs,
    double tag_size
) {
    std::vector<AprilTagResult> results;

    // Convert to grayscale if needed
    cv::Mat gray;
    if (image.channels() == 3) {
        cv::cvtColor(image, gray, cv::COLOR_BGR2GRAY);
    } else {
        gray = image.clone();
    }

    // Create image_u8 structure for apriltag
    image_u8_t im = {
        .width = gray.cols,
        .height = gray.rows,
        .stride = gray.cols,
        .buf = gray.data
    };

    // Detect tags
    zarray_t* detections = apriltag_detector_detect(detector_, &im);

    // Process detections
    for (int i = 0; i < zarray_size(detections); i++) {
        apriltag_detection_t* det;
        zarray_get(detections, i, &det);

        AprilTagResult result;
        result.tag_id = det->id;
        result.center = cv::Point2d(det->c[0], det->c[1]);
        result.decision_margin = det->decision_margin;
        result.hamming_distance = det->hamming;

        // Extract corners
        for (int j = 0; j < 4; j++) {
            result.corners.push_back(cv::Point2d(det->p[j][0], det->p[j][1]));
        }

        // Estimate pose if calibration provided
        if (!camera_matrix.empty()) {
            result.pose_valid = estimatePose(detections, i, camera_matrix, tag_size, result);
        }

        results.push_back(result);
    }

    // Cleanup
    apriltag_detections_destroy(detections);

    return results;
}

bool AprilTagDetector::estimatePose(
    zarray_t* detections,
    int idx,
    const cv::Mat& camera_matrix,
    double tag_size,
    AprilTagResult& result
) {
    apriltag_detection_t* det;
    zarray_get(detections, idx, &det);

    // Prepare detection info for pose estimation
    apriltag_detection_info_t info;
    info.det = det;
    info.tagsize = tag_size;
    info.fx = camera_matrix.at<double>(0, 0);
    info.fy = camera_matrix.at<double>(1, 1);
    info.cx = camera_matrix.at<double>(0, 2);
    info.cy = camera_matrix.at<double>(1, 2);

    // Estimate pose
    apriltag_pose_t pose;
    double err = estimate_tag_pose(&info, &pose);

    // Convert to OpenCV format
    result.rotation = cv::Mat(3, 3, CV_64F);
    result.translation = cv::Mat(3, 1, CV_64F);

    for (int i = 0; i < 3; i++) {
        for (int j = 0; j < 3; j++) {
            result.rotation.at<double>(i, j) = MATD_EL(pose.R, i, j);
        }
        result.translation.at<double>(i, 0) = MATD_EL(pose.t, i, 0);
    }

    // Extract position
    result.position = cv::Point3d(
        result.translation.at<double>(0, 0),
        result.translation.at<double>(1, 0),
        result.translation.at<double>(2, 0)
    );

    // Cleanup
    matd_destroy(pose.R);
    matd_destroy(pose.t);

    return true;
}

void AprilTagDetector::setNumThreads(int threads) {
    detector_->nthreads = threads;
}

void AprilTagDetector::setQuadDecimate(float decimate) {
    detector_->quad_decimate = decimate;
}

void AprilTagDetector::setQuadSigma(float sigma) {
    detector_->quad_sigma = sigma;
}

void AprilTagDetector::setRefineEdges(bool refine) {
    detector_->refine_edges = refine ? 1 : 0;
}

void AprilTagDetector::setDecodeSharpening(double sharpening) {
    detector_->decode_sharpening = sharpening;
}

} // namespace navign::robot::vision
