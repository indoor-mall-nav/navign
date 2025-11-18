#pragma once

#include <vector>
#include <string>
#include <opencv2/opencv.hpp>

namespace navign::robot::vision {

/**
 * @brief Camera calibration data
 */
struct CalibrationData {
    cv::Mat camera_matrix;      // 3x3 intrinsic matrix
    cv::Mat dist_coeffs;        // Distortion coefficients (k1, k2, p1, p2, k3)
    cv::Size image_size;

    // Extrinsic parameters (camera to world)
    cv::Mat rotation;           // 3x3 rotation matrix
    cv::Mat translation;        // 3x1 translation vector

    bool is_valid = false;
    double reprojection_error = 0.0;
};

/**
 * @brief Camera calibration using chessboard pattern
 */
class CameraCalibration {
public:
    CameraCalibration();
    ~CameraCalibration() = default;

    /**
     * @brief Calibrate camera using chessboard pattern
     * @param images Vector of calibration images
     * @param pattern_size Chessboard pattern size (cols, rows) - internal corners
     * @param square_size Physical size of chessboard square in meters
     * @return true if calibration successful
     */
    bool calibrate(
        const std::vector<cv::Mat>& images,
        cv::Size pattern_size,
        double square_size
    );

    /**
     * @brief Calibrate from live camera feed
     * @param camera_index Camera device index
     * @param pattern_size Chessboard pattern size
     * @param square_size Physical square size in meters
     * @param num_frames Number of calibration frames to collect
     * @return true if calibration successful
     */
    bool calibrateFromCamera(
        int camera_index,
        cv::Size pattern_size,
        double square_size,
        int num_frames = 20
    );

    /**
     * @brief Save calibration to file
     * @param filename Path to save (e.g., "calibration.yml")
     */
    bool save(const std::string& filename) const;

    /**
     * @brief Load calibration from file
     * @param filename Path to load
     */
    bool load(const std::string& filename);

    /**
     * @brief Get calibration data
     */
    const CalibrationData& getCalibration() const { return calibration_; }

    /**
     * @brief Check if calibration is valid
     */
    bool isValid() const { return calibration_.is_valid; }

    /**
     * @brief Undistort an image
     */
    cv::Mat undistort(const cv::Mat& image) const;

    /**
     * @brief Get optimal new camera matrix
     */
    cv::Mat getOptimalCameraMatrix(double alpha = 1.0) const;

private:
    CalibrationData calibration_;

    // Helper: detect chessboard corners
    bool detectChessboard(
        const cv::Mat& image,
        cv::Size pattern_size,
        std::vector<cv::Point2f>& corners
    );
};

} // namespace navign::robot::vision
