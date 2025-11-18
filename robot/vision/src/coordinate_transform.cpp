#include "coordinate_transform.hpp"
#include <iostream>

namespace navign::robot::vision {

void CoordinateTransform::setCalibration(const cv::Mat& camera_matrix, const cv::Mat& dist_coeffs) {
    camera_matrix_ = camera_matrix.clone();
    dist_coeffs_ = dist_coeffs.clone();
    has_calibration_ = true;
}

void CoordinateTransform::setCameraPose(const cv::Mat& rotation, const cv::Mat& translation) {
    rotation_ = rotation.clone();
    translation_ = translation.clone();
    has_pose_ = true;
}

cv::Point3d CoordinateTransform::imageToWorld(const cv::Point2f& image_point, double z_plane) const {
    if (!has_calibration_ || !has_pose_) {
        std::cerr << "Missing calibration or pose information" << std::endl;
        return cv::Point3d(0, 0, 0);
    }

    // Step 1: Undistort and normalize the image point
    std::vector<cv::Point2f> points_in = {image_point};
    std::vector<cv::Point2f> points_out;
    cv::undistortPoints(points_in, points_out, camera_matrix_, dist_coeffs_);

    cv::Point2f normalized = points_out[0];

    // Step 2: Create ray in camera coordinates
    cv::Mat ray_camera = (cv::Mat_<double>(3, 1) << normalized.x, normalized.y, 1.0);

    // Step 3: Transform ray to world coordinates
    cv::Mat ray_world = rotation_ * ray_camera;

    // Normalize ray direction
    double norm = cv::norm(ray_world);
    ray_world /= norm;

    // Step 4: Camera position in world coordinates
    cv::Mat cam_pos_world = -rotation_.t() * translation_;

    // Step 5: Intersect ray with z = z_plane
    double cam_z = cam_pos_world.at<double>(2, 0);
    double ray_z = ray_world.at<double>(2, 0);

    if (std::abs(ray_z) < 1e-6) {
        std::cerr << "Ray is parallel to ground plane" << std::endl;
        return cv::Point3d(0, 0, 0);
    }

    double t = (z_plane - cam_z) / ray_z;

    if (t < 0) {
        std::cerr << "Intersection behind camera" << std::endl;
        return cv::Point3d(0, 0, 0);
    }

    // Step 6: Compute intersection point
    cv::Point3d world_point(
        cam_pos_world.at<double>(0, 0) + t * ray_world.at<double>(0, 0),
        cam_pos_world.at<double>(1, 0) + t * ray_world.at<double>(1, 0),
        z_plane
    );

    return world_point;
}

cv::Point2f CoordinateTransform::worldToImage(const cv::Point3d& world_point) const {
    if (!has_calibration_ || !has_pose_) {
        std::cerr << "Missing calibration or pose information" << std::endl;
        return cv::Point2f(0, 0);
    }

    // Convert world point to camera coordinates
    cv::Mat world_pt = (cv::Mat_<double>(3, 1) << world_point.x, world_point.y, world_point.z);
    cv::Mat camera_pt = rotation_.t() * (world_pt - translation_);

    // Project to image plane
    double x = camera_pt.at<double>(0, 0) / camera_pt.at<double>(2, 0);
    double y = camera_pt.at<double>(1, 0) / camera_pt.at<double>(2, 0);

    // Apply camera intrinsics
    double fx = camera_matrix_.at<double>(0, 0);
    double fy = camera_matrix_.at<double>(1, 1);
    double cx = camera_matrix_.at<double>(0, 2);
    double cy = camera_matrix_.at<double>(1, 2);

    cv::Point2f image_point(
        static_cast<float>(fx * x + cx),
        static_cast<float>(fy * y + cy)
    );

    return image_point;
}

cv::Point3d CoordinateTransform::getRayDirection(const cv::Point2f& image_point) const {
    if (!has_calibration_ || !has_pose_) {
        return cv::Point3d(0, 0, 0);
    }

    // Undistort and normalize
    std::vector<cv::Point2f> points_in = {image_point};
    std::vector<cv::Point2f> points_out;
    cv::undistortPoints(points_in, points_out, camera_matrix_, dist_coeffs_);

    cv::Point2f normalized = points_out[0];

    // Create ray in camera coordinates
    cv::Mat ray_camera = (cv::Mat_<double>(3, 1) << normalized.x, normalized.y, 1.0);

    // Transform to world coordinates
    cv::Mat ray_world = rotation_ * ray_camera;

    // Normalize
    double norm = cv::norm(ray_world);
    ray_world /= norm;

    return cv::Point3d(
        ray_world.at<double>(0, 0),
        ray_world.at<double>(1, 0),
        ray_world.at<double>(2, 0)
    );
}

std::optional<cv::Point3d> CoordinateTransform::intersectRayPlane(
    const cv::Point3d& ray_origin,
    const cv::Point3d& ray_direction,
    const cv::Point3d& plane_normal,
    const cv::Point3d& plane_point
) {
    // Ray: P = O + t * D
    // Plane: (P - P0) · N = 0
    // Solve for t: t = ((P0 - O) · N) / (D · N)

    double denominator = ray_direction.dot(plane_normal);

    if (std::abs(denominator) < 1e-6) {
        // Ray is parallel to plane
        return std::nullopt;
    }

    cv::Point3d diff = plane_point - ray_origin;
    double t = diff.dot(plane_normal) / denominator;

    if (t < 0) {
        // Intersection behind ray origin
        return std::nullopt;
    }

    cv::Point3d intersection(
        ray_origin.x + t * ray_direction.x,
        ray_origin.y + t * ray_direction.y,
        ray_origin.z + t * ray_direction.z
    );

    return intersection;
}

cv::Point3d CoordinateTransform::getCameraPosition() const {
    if (!has_pose_) {
        return cv::Point3d(0, 0, 0);
    }

    cv::Mat cam_pos_world = -rotation_.t() * translation_;

    return cv::Point3d(
        cam_pos_world.at<double>(0, 0),
        cam_pos_world.at<double>(1, 0),
        cam_pos_world.at<double>(2, 0)
    );
}

} // namespace navign::robot::vision
