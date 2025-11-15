"""
Example configuration for robot vision module.

Copy this file to config.py and adjust the values for your setup.
"""

import numpy as np

# ============================================================================
# Camera Configuration
# ============================================================================

# Camera device index (0 for default camera)
CAMERA_INDEX = 0

# Camera resolution
CAMERA_WIDTH = 640
CAMERA_HEIGHT = 480

# Camera intrinsic parameters (obtained from calibrate.py)
# These are example values - run calibrate.py to get your actual values
CAMERA_MATRIX = np.array([
    [800.0, 0.0, 320.0],
    [0.0, 800.0, 240.0],
    [0.0, 0.0, 1.0]
], dtype=np.float32)

# Distortion coefficients (k1, k2, p1, p2, k3)
DIST_COEFFS = np.array([0.1, -0.2, 0.0, 0.0, 0.0], dtype=np.float32)

# Path to saved calibration file
CALIBRATION_FILE = "assets/interstices.npz"

# ============================================================================
# YOLO Object Detection Configuration
# ============================================================================

# YOLO model path (download from ultralytics)
# Options: yolo12n.pt, yolo12s.pt, yolo12m.pt, yolo12l.pt, yolo12x.pt
YOLO_MODEL = "yolo12l.pt"

# Detection confidence threshold
DETECTION_CONFIDENCE = 0.6

# Enable GPU acceleration (if available)
YOLO_DEVICE = "cuda:0"  # or "cpu" or "mps" for Apple Silicon

# Maximum detections per frame
MAX_DETECTIONS = 100

# ============================================================================
# AprilTag Configuration
# ============================================================================

# AprilTag family (tag36h11, tag25h9, tag16h5, tagCircle21h7, tagStandard41h12)
APRILTAG_FAMILY = "tag36h11"

# Tag size in meters (physical dimension of the tag)
TAG_SIZE = 0.015  # 15mm

# Known tag positions in world coordinates (x, y) in meters
# This defines the spatial layout of your AprilTag markers
KNOWN_TAG_POSITIONS = {
    # tag_id: (x, y) in meters (world coordinates of tag centers)
    0: (0.0, 1.0),
    1: (0.0, 0.5),
    2: (0.0, 0.0),
    3: (0.5, 1.0),
    4: (1.0, 0.5),
    5: (1.0, 1.0),
    6: (1.0, 0.0),
    7: (0.5, 0.0),
}

# Minimum number of tags required for pose estimation
MIN_TAGS_FOR_POSE = 6

# ============================================================================
# MediaPipe Hand Tracking Configuration
# ============================================================================

# Hand detection confidence threshold
HAND_DETECTION_CONFIDENCE = 0.5

# Hand tracking confidence threshold
HAND_TRACKING_CONFIDENCE = 0.5

# Maximum number of hands to detect
MAX_NUM_HANDS = 1

# Static image mode (False for video stream)
HAND_STATIC_IMAGE_MODE = False

# ============================================================================
# Gesture Classification Configuration
# ============================================================================

# Path to trained gesture classifier model
GESTURE_MODEL_PATH = "assets/gesture_classifier.pth"

# Gesture classes (update based on your training data)
GESTURE_CLASSES = ["peace", "fist", "palm", "point"]

# Gesture confidence threshold
GESTURE_CONFIDENCE = 0.7

# Device for gesture model
GESTURE_DEVICE = "cpu"  # or "cuda:0" or "mps"

# ============================================================================
# 3D Localization Configuration
# ============================================================================

# Default Z-plane for 3D point projection (meters)
DEFAULT_Z_PLANE = 0.0

# Maximum distance for valid object detection (meters)
MAX_OBJECT_DISTANCE = 5.0

# ============================================================================
# Visualization Configuration
# ============================================================================

# Enable visualization windows (set to False for headless operation)
ENABLE_VISUALIZATION = True

# Show AprilTag annotations
SHOW_APRILTAG_ANNOTATIONS = True

# Show object detection bounding boxes
SHOW_DETECTION_BOXES = True

# Show hand landmarks
SHOW_HAND_LANDMARKS = True

# FPS display
SHOW_FPS = True

# ============================================================================
# Performance Configuration
# ============================================================================

# Frame rate limit (0 for unlimited)
FPS_LIMIT = 30

# Enable frame skipping for performance
ENABLE_FRAME_SKIP = False
FRAME_SKIP_INTERVAL = 2  # Process every Nth frame

# ============================================================================
# Logging Configuration
# ============================================================================

# Enable debug logging
DEBUG_MODE = True

# Log detection results to console
LOG_DETECTIONS = True

# Save annotated frames to disk
SAVE_FRAMES = False
FRAME_OUTPUT_DIR = "output/frames"

# ============================================================================
# Integration Configuration
# ============================================================================

# Zenoh topic for publishing vision data
ZENOH_VISION_TOPIC = "robot/vision/detections"

# Zenoh topic for publishing camera pose
ZENOH_POSE_TOPIC = "robot/vision/pose"

# Data publishing rate (Hz)
PUBLISH_RATE = 10
