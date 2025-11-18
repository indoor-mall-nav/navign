#include "object_detector.hpp"
#include <fstream>
#include <iostream>
#include <algorithm>

namespace navign::robot::vision {

ObjectDetector::ObjectDetector() {
#ifdef USE_ONNXRUNTIME
    use_onnx_ = true;
    onnx_env_ = std::make_unique<Ort::Env>(ORT_LOGGING_LEVEL_WARNING, "NavignVision");
#endif
}

ObjectDetector::~ObjectDetector() = default;

bool ObjectDetector::loadModel(const std::string& model_path, const std::string& config_path) {
#ifdef USE_ONNXRUNTIME
    if (use_onnx_) {
        try {
            session_options_ = std::make_unique<Ort::SessionOptions>();
            session_options_->SetIntraOpNumThreads(4);
            session_options_->SetGraphOptimizationLevel(GraphOptimizationLevel::ORT_ENABLE_ALL);

            onnx_session_ = std::make_unique<Ort::Session>(*onnx_env_, model_path.c_str(), *session_options_);
            std::cout << "ONNX model loaded: " << model_path << std::endl;
            return true;
        } catch (const Ort::Exception& e) {
            std::cerr << "ONNX Runtime error: " << e.what() << std::endl;
            use_onnx_ = false;
            // Fall back to OpenCV DNN
        }
    }
#endif

    // Use OpenCV DNN backend
    try {
        net_ = cv::dnn::readNetFromONNX(model_path);
        if (net_.empty()) {
            std::cerr << "Failed to load model: " << model_path << std::endl;
            return false;
        }

        // Set backend and target
        net_.setPreferableBackend(cv::dnn::DNN_BACKEND_OPENCV);
        net_.setPreferableTarget(cv::dnn::DNN_TARGET_CPU);

        std::cout << "OpenCV DNN model loaded: " << model_path << std::endl;
        return true;
    } catch (const cv::Exception& e) {
        std::cerr << "OpenCV error: " << e.what() << std::endl;
        return false;
    }
}

bool ObjectDetector::loadClassNames(const std::string& names_file) {
    std::ifstream ifs(names_file);
    if (!ifs.is_open()) {
        std::cerr << "Failed to open class names file: " << names_file << std::endl;
        return false;
    }

    std::string line;
    while (std::getline(ifs, line)) {
        class_names_.push_back(line);
    }

    std::cout << "Loaded " << class_names_.size() << " class names" << std::endl;
    return true;
}

std::string ObjectDetector::getClassName(int class_id) const {
    if (class_id >= 0 && class_id < static_cast<int>(class_names_.size())) {
        return class_names_[class_id];
    }
    return "Unknown";
}

std::vector<DetectedObject> ObjectDetector::detect(
    const cv::Mat& image,
    float confidence_threshold,
    float nms_threshold
) {
    if (net_.empty() && !use_onnx_) {
        std::cerr << "Model not loaded" << std::endl;
        return {};
    }

    // Prepare input blob
    cv::Mat blob;
    cv::dnn::blobFromImage(image, blob, 1.0 / 255.0, input_size_, cv::Scalar(), true, false);

    // Forward pass
    net_.setInput(blob);
    std::vector<cv::Mat> outputs;
    net_.forward(outputs, net_.getUnconnectedOutLayersNames());

    // Post-process
    return postprocess(outputs, image, confidence_threshold, nms_threshold);
}

std::vector<DetectedObject> ObjectDetector::postprocess(
    const std::vector<cv::Mat>& outputs,
    const cv::Mat& image,
    float conf_threshold,
    float nms_threshold
) {
    std::vector<DetectedObject> results;
    std::vector<int> class_ids;
    std::vector<float> confidences;
    std::vector<cv::Rect> boxes;

    // Parse YOLO output
    // YOLOv8 output format: [batch, num_detections, 4 + num_classes]
    // Where 4 = [x, y, w, h]
    for (const auto& output : outputs) {
        const auto* data = (float*)output.data;
        const int num_detections = output.size[1];
        const int num_classes = output.size[2] - 4;

        for (int i = 0; i < num_detections; i++) {
            const float* detection = data + i * (4 + num_classes);

            // Get class scores
            float max_conf = 0.0f;
            int max_class_id = 0;

            for (int j = 0; j < num_classes; j++) {
                float conf = detection[4 + j];
                if (conf > max_conf) {
                    max_conf = conf;
                    max_class_id = j;
                }
            }

            if (max_conf > conf_threshold) {
                // Extract bounding box
                float cx = detection[0];
                float cy = detection[1];
                float w = detection[2];
                float h = detection[3];

                // Convert to pixel coordinates
                float scale_x = static_cast<float>(image.cols) / input_size_.width;
                float scale_y = static_cast<float>(image.rows) / input_size_.height;

                int left = static_cast<int>((cx - w / 2.0f) * scale_x);
                int top = static_cast<int>((cy - h / 2.0f) * scale_y);
                int width = static_cast<int>(w * scale_x);
                int height = static_cast<int>(h * scale_y);

                boxes.push_back(cv::Rect(left, top, width, height));
                confidences.push_back(max_conf);
                class_ids.push_back(max_class_id);
            }
        }
    }

    // Apply Non-Maximum Suppression
    std::vector<int> indices;
    cv::dnn::NMSBoxes(boxes, confidences, conf_threshold, nms_threshold, indices);

    // Create final results
    for (int idx : indices) {
        DetectedObject obj;
        obj.object_id = static_cast<uint32_t>(results.size());
        obj.class_name = getClassName(class_ids[idx]);
        obj.confidence = confidences[idx];
        obj.bbox = boxes[idx];
        obj.center = cv::Point2f(
            boxes[idx].x + boxes[idx].width / 2.0f,
            boxes[idx].y + boxes[idx].height / 2.0f
        );

        results.push_back(obj);
    }

    return results;
}

} // namespace navign::robot::vision
