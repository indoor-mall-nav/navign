#include "camera_calibration.hpp"
#include <iostream>

namespace navign::robot::vision {

CameraCalibration::CameraCalibration() = default;

bool CameraCalibration::calibrate(
    const std::vector<cv::Mat>& images,
    cv::Size pattern_size,
    double square_size
) {
    std::vector<std::vector<cv::Point2f>> image_points;
    std::vector<std::vector<cv::Point3f>> object_points;

    // Generate object points for chessboard
    std::vector<cv::Point3f> obj_pts;
    for (int i = 0; i < pattern_size.height; i++) {
        for (int j = 0; j < pattern_size.width; j++) {
            obj_pts.push_back(cv::Point3f(j * square_size, i * square_size, 0.0));
        }
    }

    // Find chessboard corners in all images
    for (const auto& image : images) {
        std::vector<cv::Point2f> corners;
        if (detectChessboard(image, pattern_size, corners)) {
            image_points.push_back(corners);
            object_points.push_back(obj_pts);
        }
    }

    if (image_points.size() < 3) {
        std::cerr << "Not enough valid calibration images (need at least 3)" << std::endl;
        return false;
    }

    std::cout << "Calibrating with " << image_points.size() << " images..." << std::endl;

    // Calibrate camera
    cv::Mat camera_matrix = cv::Mat::eye(3, 3, CV_64F);
    cv::Mat dist_coeffs = cv::Mat::zeros(5, 1, CV_64F);
    std::vector<cv::Mat> rvecs, tvecs;

    calibration_.image_size = images[0].size();

    double rms_error = cv::calibrateCamera(
        object_points,
        image_points,
        calibration_.image_size,
        camera_matrix,
        dist_coeffs,
        rvecs,
        tvecs,
        cv::CALIB_FIX_K3
    );

    calibration_.camera_matrix = camera_matrix;
    calibration_.dist_coeffs = dist_coeffs;
    calibration_.reprojection_error = rms_error;
    calibration_.is_valid = true;

    std::cout << "Calibration complete!" << std::endl;
    std::cout << "RMS reprojection error: " << rms_error << " pixels" << std::endl;
    std::cout << "Camera matrix:\n" << camera_matrix << std::endl;
    std::cout << "Distortion coefficients:\n" << dist_coeffs << std::endl;

    return true;
}

bool CameraCalibration::calibrateFromCamera(
    int camera_index,
    cv::Size pattern_size,
    double square_size,
    int num_frames
) {
    cv::VideoCapture camera(camera_index);
    if (!camera.isOpened()) {
        std::cerr << "Failed to open camera " << camera_index << std::endl;
        return false;
    }

    std::vector<cv::Mat> calibration_images;
    int frames_collected = 0;

    std::cout << "Collecting calibration images..." << std::endl;
    std::cout << "Press SPACE to capture, ESC to cancel" << std::endl;

    while (frames_collected < num_frames) {
        cv::Mat frame;
        camera >> frame;

        if (frame.empty()) {
            continue;
        }

        // Try to detect chessboard
        std::vector<cv::Point2f> corners;
        bool found = detectChessboard(frame, pattern_size, corners);

        // Draw corners
        if (found) {
            cv::drawChessboardCorners(frame, pattern_size, corners, found);
        }

        // Display instructions
        cv::putText(frame,
                    "Frames: " + std::to_string(frames_collected) + "/" + std::to_string(num_frames),
                    cv::Point(10, 30),
                    cv::FONT_HERSHEY_SIMPLEX,
                    1.0,
                    cv::Scalar(0, 255, 0),
                    2);

        cv::imshow("Camera Calibration", frame);

        int key = cv::waitKey(30);
        if (key == 27) { // ESC
            std::cout << "Calibration cancelled" << std::endl;
            return false;
        } else if (key == ' ' && found) { // SPACE
            calibration_images.push_back(frame.clone());
            frames_collected++;
            std::cout << "Captured frame " << frames_collected << "/" << num_frames << std::endl;
        }
    }

    cv::destroyAllWindows();

    // Calibrate using collected images
    return calibrate(calibration_images, pattern_size, square_size);
}

bool CameraCalibration::save(const std::string& filename) const {
    if (!calibration_.is_valid) {
        std::cerr << "Cannot save invalid calibration" << std::endl;
        return false;
    }

    cv::FileStorage fs(filename, cv::FileStorage::WRITE);
    if (!fs.isOpened()) {
        std::cerr << "Failed to open file for writing: " << filename << std::endl;
        return false;
    }

    fs << "camera_matrix" << calibration_.camera_matrix;
    fs << "dist_coeffs" << calibration_.dist_coeffs;
    fs << "image_width" << calibration_.image_size.width;
    fs << "image_height" << calibration_.image_size.height;
    fs << "reprojection_error" << calibration_.reprojection_error;

    if (!calibration_.rotation.empty()) {
        fs << "rotation" << calibration_.rotation;
        fs << "translation" << calibration_.translation;
    }

    fs.release();

    std::cout << "Calibration saved to: " << filename << std::endl;
    return true;
}

bool CameraCalibration::load(const std::string& filename) {
    cv::FileStorage fs(filename, cv::FileStorage::READ);
    if (!fs.isOpened()) {
        std::cerr << "Failed to open calibration file: " << filename << std::endl;
        return false;
    }

    fs["camera_matrix"] >> calibration_.camera_matrix;
    fs["dist_coeffs"] >> calibration_.dist_coeffs;

    int width, height;
    fs["image_width"] >> width;
    fs["image_height"] >> height;
    calibration_.image_size = cv::Size(width, height);

    fs["reprojection_error"] >> calibration_.reprojection_error;

    // Optional extrinsics
    if (!fs["rotation"].empty()) {
        fs["rotation"] >> calibration_.rotation;
        fs["translation"] >> calibration_.translation;
    }

    fs.release();

    calibration_.is_valid = !calibration_.camera_matrix.empty();

    if (calibration_.is_valid) {
        std::cout << "Calibration loaded from: " << filename << std::endl;
    }

    return calibration_.is_valid;
}

cv::Mat CameraCalibration::undistort(const cv::Mat& image) const {
    if (!calibration_.is_valid) {
        return image;
    }

    cv::Mat undistorted;
    cv::undistort(image, undistorted, calibration_.camera_matrix, calibration_.dist_coeffs);
    return undistorted;
}

cv::Mat CameraCalibration::getOptimalCameraMatrix(double alpha) const {
    if (!calibration_.is_valid) {
        return cv::Mat();
    }

    return cv::getOptimalNewCameraMatrix(
        calibration_.camera_matrix,
        calibration_.dist_coeffs,
        calibration_.image_size,
        alpha
    );
}

bool CameraCalibration::detectChessboard(
    const cv::Mat& image,
    cv::Size pattern_size,
    std::vector<cv::Point2f>& corners
) {
    cv::Mat gray;
    if (image.channels() == 3) {
        cv::cvtColor(image, gray, cv::COLOR_BGR2GRAY);
    } else {
        gray = image;
    }

    bool found = cv::findChessboardCorners(
        gray,
        pattern_size,
        corners,
        cv::CALIB_CB_ADAPTIVE_THRESH | cv::CALIB_CB_NORMALIZE_IMAGE
    );

    if (found) {
        cv::cornerSubPix(
            gray,
            corners,
            cv::Size(11, 11),
            cv::Size(-1, -1),
            cv::TermCriteria(cv::TermCriteria::EPS + cv::TermCriteria::COUNT, 30, 0.1)
        );
    }

    return found;
}

} // namespace navign::robot::vision
