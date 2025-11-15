#!/usr/bin/env python3
"""
Example usage of the robot vision module.

This script demonstrates how to use the various vision components:
- Camera calibration
- Object detection with YOLO
- AprilTag-based pose estimation
- Hand gesture recognition
- Finger direction detection
- 3D point localization
"""

import cv2
import numpy as np

# Import vision modules
from detection import model, K, dist, Z0
from objects import detect_objects
from locate import get_camera_pose, get_point_3d_place
from finger import get_finger_direction

# Optional: Import config if you created one
try:
    import config
    CAMERA_INDEX = config.CAMERA_INDEX
    DETECTION_CONFIDENCE = config.DETECTION_CONFIDENCE
    ENABLE_VISUALIZATION = config.ENABLE_VISUALIZATION
except ImportError:
    # Use defaults if config.py doesn't exist
    CAMERA_INDEX = 0
    DETECTION_CONFIDENCE = 0.6
    ENABLE_VISUALIZATION = True


def example_basic_detection():
    """Example 1: Basic object detection with YOLO."""
    print("=== Example 1: Basic Object Detection ===")

    cap = cv2.VideoCapture(CAMERA_INDEX)

    try:
        while True:
            ret, frame = cap.read()
            if not ret:
                print("Failed to grab frame")
                break

            # Detect objects in the frame
            objects = detect_objects(model, frame)

            # Print detected objects
            for u, v, name, conf in objects:
                if conf >= DETECTION_CONFIDENCE:
                    print(f"Detected: {name} at ({u:.0f}, {v:.0f}) - Confidence: {conf:.2f}")

                    # Draw bounding box
                    if ENABLE_VISUALIZATION:
                        cv2.circle(frame, (int(u), int(v)), 5, (0, 255, 0), -1)
                        cv2.putText(frame, f"{name} {conf:.2f}",
                                  (int(u), int(v) - 10),
                                  cv2.FONT_HERSHEY_SIMPLEX, 0.5, (0, 255, 0), 2)

            if ENABLE_VISUALIZATION:
                cv2.imshow("Object Detection", frame)

            # Press 'q' to quit
            if cv2.waitKey(1) & 0xFF == ord('q'):
                break

    finally:
        cap.release()
        cv2.destroyAllWindows()


def example_pose_estimation():
    """Example 2: Camera pose estimation using AprilTags."""
    print("=== Example 2: Camera Pose Estimation ===")

    cap = cv2.VideoCapture(CAMERA_INDEX)

    try:
        while True:
            ret, frame = cap.read()
            if not ret:
                print("Failed to grab frame")
                break

            # Get camera pose from AprilTags
            camera_pos, R = get_camera_pose(frame)

            if camera_pos is not None:
                print(f"Camera position: X={camera_pos[0]:.3f}m, "
                      f"Y={camera_pos[1]:.3f}m, Z={camera_pos[2]:.3f}m")

            if ENABLE_VISUALIZATION:
                cv2.imshow("AprilTag Pose Estimation", frame)

            # Press 'q' to quit
            if cv2.waitKey(1) & 0xFF == ord('q'):
                break

    finally:
        cap.release()
        cv2.destroyAllWindows()


def example_3d_localization():
    """Example 3: 3D object localization in world coordinates."""
    print("=== Example 3: 3D Object Localization ===")

    cap = cv2.VideoCapture(CAMERA_INDEX)

    try:
        while True:
            ret, frame = cap.read()
            if not ret:
                print("Failed to grab frame")
                break

            # Get camera pose
            camera_pos, R = get_camera_pose(frame)

            if camera_pos is None or R is None:
                print("Camera pose not available, need AprilTags in view")
                if ENABLE_VISUALIZATION:
                    cv2.imshow("3D Localization", frame)
                if cv2.waitKey(1) & 0xFF == ord('q'):
                    break
                continue

            # Detect objects
            objects = detect_objects(model, frame)

            # Localize objects in 3D
            for u, v, name, conf in objects:
                if conf < DETECTION_CONFIDENCE:
                    continue

                # Convert 2D image point to 3D world coordinates
                point_2d = np.array([[[u, v]]], dtype=np.float32)
                point_3d = get_point_3d_place(
                    point_2d,
                    Z0=Z0,
                    camera_pos=camera_pos,
                    R=R
                )

                if point_3d is not None:
                    print(f"{name}: 3D position = "
                          f"X={point_3d[0]:.2f}m, "
                          f"Y={point_3d[1]:.2f}m, "
                          f"Z={point_3d[2]:.2f}m")

                    if ENABLE_VISUALIZATION:
                        cv2.circle(frame, (int(u), int(v)), 5, (255, 0, 0), -1)
                        cv2.putText(frame,
                                  f"{name} ({point_3d[0]:.1f}, {point_3d[1]:.1f})",
                                  (int(u), int(v) - 10),
                                  cv2.FONT_HERSHEY_SIMPLEX, 0.4, (255, 0, 0), 1)

            if ENABLE_VISUALIZATION:
                cv2.imshow("3D Localization", frame)

            # Press 'q' to quit
            if cv2.waitKey(1) & 0xFF == ord('q'):
                break

    finally:
        cap.release()
        cv2.destroyAllWindows()


def example_finger_pointing():
    """Example 4: Finger pointing direction detection."""
    print("=== Example 4: Finger Pointing Detection ===")

    cap = cv2.VideoCapture(CAMERA_INDEX)

    try:
        while True:
            ret, frame = cap.read()
            if not ret:
                print("Failed to grab frame")
                break

            # Get camera pose
            camera_pos, R = get_camera_pose(frame)

            if camera_pos is None or R is None:
                print("Camera pose not available, need AprilTags in view")
                if ENABLE_VISUALIZATION:
                    cv2.imshow("Finger Pointing", frame)
                if cv2.waitKey(1) & 0xFF == ord('q'):
                    break
                continue

            # Get finger direction
            directions = get_finger_direction(frame, Z0, camera_pos, R)

            for direction, base in directions:
                print(f"Pointing direction: [{direction[0]:.2f}, "
                      f"{direction[1]:.2f}, {direction[2]:.2f}]")
                print(f"From position: [{base[0]:.2f}, {base[1]:.2f}, {base[2]:.2f}]")

            if ENABLE_VISUALIZATION:
                cv2.imshow("Finger Pointing", frame)

            # Press 'q' to quit
            if cv2.waitKey(1) & 0xFF == ord('q'):
                break

    finally:
        cap.release()
        cv2.destroyAllWindows()


def example_integrated_scene_understanding():
    """Example 5: Integrated scene understanding (objects + pointing)."""
    print("=== Example 5: Integrated Scene Understanding ===")
    print("Point at objects to identify them!")

    cap = cv2.VideoCapture(CAMERA_INDEX)

    try:
        while True:
            ret, frame = cap.read()
            if not ret:
                print("Failed to grab frame")
                break

            # Get camera pose
            camera_pos, R = get_camera_pose(frame)

            if camera_pos is None or R is None:
                if ENABLE_VISUALIZATION:
                    cv2.imshow("Scene Understanding", frame)
                if cv2.waitKey(1) & 0xFF == ord('q'):
                    break
                continue

            # Detect objects
            objects = detect_objects(model, frame)
            objects_3d = []

            for u, v, name, conf in objects:
                if conf < DETECTION_CONFIDENCE:
                    continue

                point_2d = np.array([[[u, v]]], dtype=np.float32)
                point_3d = get_point_3d_place(point_2d, Z0, camera_pos, R)

                if point_3d is not None:
                    objects_3d.append((name, point_3d, conf))

                    if ENABLE_VISUALIZATION:
                        cv2.circle(frame, (int(u), int(v)), 5, (0, 255, 0), -1)

            # Get finger direction
            directions = get_finger_direction(frame, Z0, camera_pos, R)

            # Find objects in pointing direction
            for direction, base in directions:
                for name, obj_pos, conf in objects_3d:
                    # Vector from finger base to object
                    to_object = obj_pos - base
                    to_object_norm = to_object / np.linalg.norm(to_object)

                    # Calculate angle between pointing direction and object direction
                    dot_product = np.dot(direction, to_object_norm)
                    angle = np.arccos(np.clip(dot_product, -1.0, 1.0))
                    angle_deg = np.degrees(angle)

                    # If pointing roughly towards the object (within 30 degrees)
                    if angle_deg < 30:
                        distance = np.linalg.norm(to_object)
                        print(f"👉 Pointing at: {name} "
                              f"(distance: {distance:.2f}m, confidence: {conf:.2f})")

            if ENABLE_VISUALIZATION:
                cv2.imshow("Scene Understanding", frame)

            # Press 'q' to quit
            if cv2.waitKey(1) & 0xFF == ord('q'):
                break

    finally:
        cap.release()
        cv2.destroyAllWindows()


if __name__ == "__main__":
    import sys

    examples = {
        "1": ("Basic Object Detection", example_basic_detection),
        "2": ("Camera Pose Estimation", example_pose_estimation),
        "3": ("3D Object Localization", example_3d_localization),
        "4": ("Finger Pointing Detection", example_finger_pointing),
        "5": ("Integrated Scene Understanding", example_integrated_scene_understanding),
    }

    print("\n" + "="*60)
    print("Robot Vision Module - Example Usage")
    print("="*60)
    print("\nAvailable examples:")
    for key, (name, _) in examples.items():
        print(f"  {key}. {name}")
    print("\nPress 'q' in the video window to exit any example.\n")

    if len(sys.argv) > 1:
        choice = sys.argv[1]
    else:
        choice = input("Select example (1-5): ").strip()

    if choice in examples:
        _, func = examples[choice]
        func()
    else:
        print("Invalid choice. Please select 1-5.")
