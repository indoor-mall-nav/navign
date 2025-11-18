#pragma once

#include <vector>
#include <opencv2/opencv.hpp>

namespace navign::robot::vision {

/**
 * @brief 2D-3D coordinate transformation utilities
 */
class CoordinateTransform {
public:
    CoordinateTransform() = default;
    ~CoordinateTransform() = default;

    /**
     * @brief Set camera calibration parameters
     */
    void setCalibration(const cv::Mat& camera_matrix, const cv::Mat& dist_coeffs);

    /**
     * @brief Set camera pose (extrinsics)
     * @param rotation 3x3 rotation matrix (camera to world)
     * @param translation 3x1 translation vector (camera to world)
     */
    void setCameraPose(const cv::Mat& rotation, const cv::Mat& translation);

    /**
     * @brief Project 2D image point to 3D world coordinates
     * @param image_point 2D point in image (u, v)
     * @param z_plane Z coordinate of ground plane in world coords
     * @return 3D point in world coordinates
     */
    cv::Point3d imageToWorld(const cv::Point2f& image_point, double z_plane = 0.0) const;

    /**
     * @brief Project 3D world point to 2D image coordinates
     * @param world_point 3D point in world coordinates
     * @return 2D point in image
     */
    cv::Point2f worldToImage(const cv::Point3d& world_point) const;

    /**
     * @brief Compute ray direction from camera through image point
     * @param image_point 2D point in image
     * @return Normalized ray direction in world coordinates
     */
    cv::Point3d getRayDirection(const cv::Point2f& image_point) const;

    /**
     * @brief Intersect ray with plane
     * @param ray_origin Camera position in world coords
     * @param ray_direction Ray direction (normalized)
     * @param plane_normal Plane normal vector
     * @param plane_point Point on plane
     * @return Intersection point (or empty if no intersection)
     */
    static std::optional<cv::Point3d> intersectRayPlane(
        const cv::Point3d& ray_origin,
        const cv::Point3d& ray_direction,
        const cv::Point3d& plane_normal,
        const cv::Point3d& plane_point
    );

    /**
     * @brief Get camera position in world coordinates
     */
    cv::Point3d getCameraPosition() const;

    /**
     * @brief Check if calibration is set
     */
    bool isCalibrated() const { return has_calibration_; }

    /**
     * @brief Check if pose is set
     */
    bool hasPose() const { return has_pose_; }

private:
    cv::Mat camera_matrix_;
    cv::Mat dist_coeffs_;
    cv::Mat rotation_;      // Camera to world
    cv::Mat translation_;   // Camera to world

    bool has_calibration_ = false;
    bool has_pose_ = false;
};

} // namespace navign::robot::vision
