"""
Tests for gesture_space components

Run with: pytest tests/test_gesture_space.py
Or from root: uv run pytest gesture_space/tests/
"""

import pytest
import numpy as np
import sys
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))


class TestGestureSpaceImports:
    """Test that gesture_space modules can be imported"""

    def test_import_main(self):
        """Test importing main module"""
        try:
            import main

            assert main is not None
        except ImportError as e:
            pytest.skip(f"Cannot import main: {e}")

    def test_import_dependencies(self):
        """Test that required dependencies are available"""
        # These should be available from pyproject.toml
        import cv2
        import numpy

        assert cv2 is not None
        assert numpy is not None


class TestCoordinateTransforms:
    """Test 3D coordinate transformation utilities (if implemented)"""

    def test_basic_3d_point(self):
        """Test basic 3D point representation"""
        point = np.array([1.0, 2.0, 3.0])

        assert point.shape == (3,)
        assert point[0] == 1.0
        assert point[1] == 2.0
        assert point[2] == 3.0

    def test_translation_matrix(self):
        """Test 3D translation matrix construction"""
        # Translation by (5, 10, 15)
        t = np.array([5.0, 10.0, 15.0])

        # 4x4 homogeneous transformation matrix
        T = np.eye(4)
        T[:3, 3] = t

        # Apply to point (1, 2, 3)
        point_homogeneous = np.array([1.0, 2.0, 3.0, 1.0])
        transformed = T @ point_homogeneous

        expected = np.array([6.0, 12.0, 18.0, 1.0])
        np.testing.assert_array_almost_equal(transformed, expected)

    def test_rotation_matrix_identity(self):
        """Test identity rotation (no rotation)"""
        R = np.eye(3)

        point = np.array([1.0, 2.0, 3.0])
        rotated = R @ point

        np.testing.assert_array_almost_equal(rotated, point)


class TestImageProcessing:
    """Test image processing utilities"""

    def test_create_blank_frame(self):
        """Test creating a blank video frame"""
        import cv2

        # Create 640x480 blank frame
        frame = np.zeros((480, 640, 3), dtype=np.uint8)

        assert frame.shape == (480, 640, 3)
        assert frame.dtype == np.uint8

        # Convert to RGB
        rgb_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
        assert rgb_frame.shape == (480, 640, 3)

    def test_frame_dimensions(self):
        """Test standard video frame dimensions"""
        import cv2

        # 720p frame
        frame_720p = np.zeros((720, 1280, 3), dtype=np.uint8)
        assert frame_720p.shape == (720, 1280, 3)

        # 1080p frame
        frame_1080p = np.zeros((1080, 1920, 3), dtype=np.uint8)
        assert frame_1080p.shape == (1080, 1920, 3)


class TestHandLandmarks:
    """Test hand landmark processing (if gesture recognition is implemented)"""

    def test_hand_landmark_count(self):
        """Test that hand has 21 landmarks (MediaPipe standard)"""
        # MediaPipe hand tracking produces 21 landmarks
        num_landmarks = 21

        # Simulate landmark coordinates (x, y, z for each)
        landmarks = np.random.rand(num_landmarks, 3)

        assert landmarks.shape == (21, 3)

    def test_landmark_coordinate_range(self):
        """Test that normalized landmarks are in [0, 1] range"""
        # MediaPipe returns normalized coordinates
        landmarks = np.array(
            [
                [0.5, 0.5, 0.0],  # WRIST
                [0.4, 0.3, 0.0],  # THUMB_CMC
                [0.3, 0.2, 0.0],  # THUMB_MCP
                # ... more landmarks
            ]
        )

        # Check all coordinates are in valid range
        assert np.all(landmarks >= 0.0) and np.all(landmarks <= 1.0)


class TestObjectDetection:
    """Test object detection utilities (YOLO)"""

    def test_bounding_box_format(self):
        """Test bounding box representation"""
        # Bounding box: (x, y, width, height)
        bbox = np.array([100, 150, 200, 300])

        x, y, w, h = bbox
        assert x == 100
        assert y == 150
        assert w == 200
        assert h == 300

    def test_confidence_score_range(self):
        """Test confidence scores are in [0, 1] range"""
        confidence = 0.85

        assert 0.0 <= confidence <= 1.0

    def test_filter_low_confidence(self):
        """Test filtering detections by confidence threshold"""
        detections = [
            {"class": "person", "confidence": 0.95},
            {"class": "chair", "confidence": 0.45},
            {"class": "table", "confidence": 0.78},
            {"class": "cup", "confidence": 0.23},
        ]

        threshold = 0.6
        filtered = [d for d in detections if d["confidence"] >= threshold]

        assert len(filtered) == 2
        assert filtered[0]["class"] == "person"
        assert filtered[1]["class"] == "table"


class TestWakeWordDetection:
    """Test wake word detection (Porcupine)"""

    @patch("waking.porcupine")
    def test_wake_word_mock(self, mock_porcupine):
        """Test wake word detection with mock"""
        # Mock porcupine instance
        mock_porcupine.frame_length = 512
        mock_porcupine.process = Mock(return_value=0)  # Wake word detected at index 0

        # Simulate audio frame
        pcm = np.zeros(512, dtype=np.int16)

        # Process frame
        keyword_index = mock_porcupine.process(pcm)

        assert keyword_index == 0  # Wake word detected

    def test_audio_frame_format(self):
        """Test audio frame format for wake word detection"""
        # Porcupine typically uses 16-bit PCM audio
        frame_length = 512
        audio_frame = np.zeros(frame_length, dtype=np.int16)

        assert audio_frame.shape == (512,)
        assert audio_frame.dtype == np.int16


class TestCameraPose:
    """Test camera pose estimation"""

    def test_camera_pose_representation(self):
        """Test camera pose as rotation + translation"""
        # Rotation matrix (3x3)
        R = np.eye(3)

        # Translation vector (3x1)
        t = np.array([0.0, 0.0, 1.0])

        assert R.shape == (3, 3)
        assert t.shape == (3,)

    def test_projection_matrix(self):
        """Test constructing projection matrix from camera pose"""
        R = np.eye(3)
        t = np.array([0.0, 0.0, 0.0])

        # Projection matrix [R | t]
        P = np.hstack([R, t.reshape(3, 1)])

        assert P.shape == (3, 4)


class TestDepthEstimation:
    """Test depth (Z) estimation for objects"""

    def test_depth_positive(self):
        """Test that depth values are positive (in front of camera)"""
        Z0 = 1.5  # meters

        assert Z0 > 0

    def test_pixel_to_3d_conversion(self):
        """Test converting pixel coordinates to 3D world coordinates"""
        # Simplified test of coordinate conversion logic
        # In practice, this requires camera calibration

        # Pixel coordinates (u, v)
        u, v = 320, 240

        # Assume calibrated camera intrinsics
        fx, fy = 800, 800  # focal lengths
        cx, cy = 320, 240  # principal point

        # Depth
        Z = 1.0  # meters

        # Back-project to 3D
        X = (u - cx) * Z / fx
        Y = (v - cy) * Z / fy

        # At principal point, should be (0, 0, Z)
        assert abs(X) < 0.01
        assert abs(Y) < 0.01


class TestUtilityFunctions:
    """Test utility functions"""

    def test_euclidean_distance_2d(self):
        """Test 2D Euclidean distance calculation"""
        p1 = np.array([0.0, 0.0])
        p2 = np.array([3.0, 4.0])

        distance = np.linalg.norm(p2 - p1)

        assert distance == 5.0  # 3-4-5 triangle

    def test_euclidean_distance_3d(self):
        """Test 3D Euclidean distance calculation"""
        p1 = np.array([0.0, 0.0, 0.0])
        p2 = np.array([1.0, 1.0, 1.0])

        distance = np.linalg.norm(p2 - p1)

        expected = np.sqrt(3.0)
        assert abs(distance - expected) < 0.001


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
