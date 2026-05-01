# Vision Service

Computer vision processing for object detection, pose estimation, and hand tracking.

## Overview

**Language:** Python
**Location:** `robot/vision/`
**Technologies:** YOLOv12, AprilTag, MediaPipe

## Capabilities

- **Object Detection:** YOLOv12 real-time detection
- **Pose Estimation:** AprilTag-based camera localization
- **Hand Tracking:** MediaPipe hand landmarks (21 points per hand)
- **Finger Pointing:** 3D direction detection
- **Gesture Recognition:** Neural network classification
- **3D Localization:** 2D→3D coordinate transformation

## Architecture

```
   Camera
      |
      v
Vision Service
  (YOLOv12
   AprilTag
   MediaPipe)
      |
      | Zenoh Topics
      |
      +-> robot/vision/objects
      +-> robot/vision/pose
      +-> robot/vision/gestures
      +-> robot/vision/pointing
```

## Setup

```bash
cd robot/vision
uv sync
cp config.example.py config.py
```

### Camera Calibration

```bash
uv run python calibrate.py
# Uses chessboard pattern
# Generates assets/interstices.npz
```

## Zenoh Topics

### Published

- `robot/vision/objects` - Detected objects with bounding boxes
- `robot/vision/pose` - Camera pose (position + rotation)
- `robot/vision/gestures` - Classified hand gestures
- `robot/vision/pointing` - Finger directions in 3D space

## Running

```bash
cd robot/vision
uv run python service.py
```

## Environment Variables

- `CAMERA_INDEX` - Default: `0`
- `YOLO_MODEL` - Default: `yolo12n.pt`

## Configuration

Edit `config.py`:

- Camera index and resolution
- YOLO model selection
- AprilTag positions (known world coordinates)
- MediaPipe hand tracking settings
- Visualization options

## See Also

- [Vision README](/robot/vision/README.md)
- [Scheduler](scheduler.md)
- [Protocol Buffers](/robot/proto/vision.proto)
