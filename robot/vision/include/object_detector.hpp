#pragma once

#include <vector>
#include <string>
#include <opencv2/opencv.hpp>
#include <opencv2/dnn.hpp>

#ifdef USE_ONNXRUNTIME
#include <onnxruntime_cxx_api.h>
#endif

namespace navign::robot::vision {

/**
 * @brief Detected object result
 */
struct DetectedObject {
    uint32_t object_id;
    std::string class_name;
    float confidence;
    cv::Rect bbox;
    cv::Point2f center;

    // 3D information (if available)
    bool has_3d = false;
    cv::Point3d world_position;
    double distance_meters = 0.0;
};

/**
 * @brief Object detector using YOLO via OpenCV DNN or ONNX Runtime
 */
class ObjectDetector {
public:
    ObjectDetector();
    ~ObjectDetector();

    /**
     * @brief Load YOLO model
     * @param model_path Path to ONNX model file (e.g., yolov8n.onnx)
     * @param config_path Path to model config (optional)
     * @return true if loaded successfully
     */
    bool loadModel(const std::string& model_path, const std::string& config_path = "");

    /**
     * @brief Detect objects in an image
     * @param image Input image (BGR)
     * @param confidence_threshold Minimum confidence (0.0-1.0)
     * @param nms_threshold Non-maximum suppression threshold
     * @return Vector of detected objects
     */
    std::vector<DetectedObject> detect(
        const cv::Mat& image,
        float confidence_threshold = 0.5f,
        float nms_threshold = 0.4f
    );

    /**
     * @brief Load COCO class names
     */
    bool loadClassNames(const std::string& names_file);

    /**
     * @brief Get class name by index
     */
    std::string getClassName(int class_id) const;

private:
    // OpenCV DNN backend
    cv::dnn::Net net_;
    std::vector<std::string> class_names_;
    cv::Size input_size_{640, 640};

#ifdef USE_ONNXRUNTIME
    // ONNX Runtime backend (faster inference)
    std::unique_ptr<Ort::Env> onnx_env_;
    std::unique_ptr<Ort::Session> onnx_session_;
    std::unique_ptr<Ort::SessionOptions> session_options_;
#endif

    bool use_onnx_ = false;

    // Post-processing
    std::vector<DetectedObject> postprocess(
        const std::vector<cv::Mat>& outputs,
        const cv::Mat& image,
        float conf_threshold,
        float nms_threshold
    );
};

} // namespace navign::robot::vision
