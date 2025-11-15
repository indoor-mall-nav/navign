# Robot Vision Module

Computer vision capabilities for the Navign robot, including object detection, pose estimation, hand tracking, and 3D localization.

## Features

- **Object Detection**: Real-time object detection using YOLOv12
- **Camera Pose Estimation**: AprilTag-based camera localization in 3D space
- **Hand Tracking**: MediaPipe-based hand landmark detection
- **Finger Pointing**: Detect and track finger pointing direction in 3D
- **Gesture Recognition**: Neural network-based gesture classification
- **3D Localization**: Convert 2D image points to 3D world coordinates
- **Camera Calibration**: Calibrate camera intrinsic parameters

## Installation

```bash
cd robot/vision
uv sync
```

## Configuration

1. Copy the example configuration:
   ```bash
   cp config.example.py config.py
   ```

2. Edit `config.py` to customize settings:
   - Camera parameters
   - YOLO model selection
   - AprilTag tag positions
   - MediaPipe hand tracking settings
   - Visualization options

3. Run camera calibration (first time setup):
   ```bash
   uv run python calibrate.py
   ```

   This will:
   - Open your camera
   - Detect a chessboard pattern (9x6 internal corners)
   - Collect 20 calibration frames
   - Save camera matrix and distortion coefficients to `assets/interstices.npz`

4. Print and place AprilTags in your environment:
   - Use tag family: `tag36h11`
   - Tag IDs: 0-7
   - Update `KNOWN_TAG_POSITIONS` in `config.py` with physical positions

## Usage

### Quick Start

Run the example usage script:

```bash
uv run python example_usage.py
```

Select from available examples:
1. Basic Object Detection
2. Camera Pose Estimation
3. 3D Object Localization
4. Finger Pointing Detection
5. Integrated Scene Understanding

### Module Documentation

#### Object Detection (`detection.py`, `objects.py`)

```python
from detection import model
from objects import detect_objects

# Detect objects in a frame
objects = detect_objects(model, frame)

for u, v, name, conf in objects:
    print(f"{name} at ({u}, {v}) - confidence: {conf}")
```

#### Camera Pose Estimation (`locate.py`)

```python
from locate import get_camera_pose

# Get camera position and rotation from AprilTags
camera_pos, R = get_camera_pose(frame)

if camera_pos is not None:
    print(f"Camera at: {camera_pos}")
```

#### 3D Localization (`locate.py`)

```python
from locate import get_point_3d_place
import numpy as np

# Convert 2D image point to 3D world coordinates
point_2d = np.array([[[320, 240]]], dtype=np.float32)
point_3d = get_point_3d_place(
    point_2d,
    Z0=0.0,  # Ground plane
    camera_pos=camera_pos,
    R=R
)
```

#### Finger Direction (`finger.py`)

```python
from finger import get_finger_direction

# Get finger pointing directions
directions = get_finger_direction(frame, Z0=0.0, camera_pos=camera_pos, R=R)

for direction, base in directions:
    print(f"Pointing: {direction} from {base}")
```

#### Gesture Recognition (`gesture.py`)

```python
from gesture import model, eval_transform
from PIL import Image

# Classify hand gesture
image = Image.fromarray(frame)
tensor = eval_transform(image).unsqueeze(0)
output = model(tensor)
gesture_class = output.argmax(dim=1).item()
```

## File Overview

| File | Purpose |
|------|---------|
| `calibrate.py` | Camera calibration using chessboard pattern |
| `detection.py` | YOLO model initialization and camera parameters |
| `objects.py` | Object detection wrapper function |
| `locate.py` | AprilTag pose estimation and 3D localization |
| `finger.py` | Hand/finger direction detection |
| `gesture.py` | Gesture classification neural network |
| `transform.py` | MediaPipe hand landmark transforms |
| `config.example.py` | Example configuration file |
| `example_usage.py` | Usage examples and demos |

## Dependencies

- **opencv-contrib-python**: Computer vision operations
- **mediapipe**: Hand landmark detection
- **ultralytics**: YOLOv12 object detection
- **torch**: Neural network inference
- **pupil-apriltags**: AprilTag detection and pose estimation
- **pillow**: Image processing

See `pyproject.toml` for complete dependency list.

## Integration with Robot System

The vision module can publish data to other robot components via Zenoh:

```python
import zenoh

# Example: Publishing object detections
z = zenoh.open()
pub = z.declare_publisher("robot/vision/detections")

objects = detect_objects(model, frame)
pub.put(json.dumps(objects))
```

## Troubleshooting

### Camera not opening
- Check `CAMERA_INDEX` in config.py
- Try different indices: 0, 1, 2
- Verify camera permissions

### AprilTags not detected
- Ensure good lighting conditions
- Print tags at correct size (15mm default)
- Check `KNOWN_TAG_POSITIONS` matches physical layout
- Need at least 6 tags in view for pose estimation

### Poor object detection accuracy
- Lower `DETECTION_CONFIDENCE` threshold
- Use larger YOLO model (yolo12l.pt or yolo12x.pt)
- Ensure proper lighting and camera focus

### Hand tracking not working
- Increase `HAND_DETECTION_CONFIDENCE`
- Ensure hand is clearly visible
- Check lighting conditions

## Assets Required

Create an `assets/` directory with:
- `interstices.npz`: Camera calibration (generated by calibrate.py)
- `gesture_classifier.pth`: Trained gesture model (optional)
- `yolo12l.pt`: YOLO model weights (auto-downloaded by ultralytics)

## License

MIT License - Part of the Navign project
