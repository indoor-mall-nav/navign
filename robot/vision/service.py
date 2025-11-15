#!/usr/bin/env python3
"""Vision service for AprilTag detection and object recognition using Zenoh."""

import asyncio
import logging
import time
from typing import Optional

import cv2
import numpy as np
import zenoh
from pupil_apriltags import Detector
from ultralytics import YOLO

# TODO: Import generated protobuf messages
# from vision_pb2 import *

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class VisionService:
    """Vision service for AprilTag and object detection."""

    def __init__(self):
        """Initialize vision service."""
        self.session: Optional[zenoh.Session] = None
        self.apriltag_detector: Optional[Detector] = None
        self.yolo_model: Optional[YOLO] = None
        self.camera: Optional[cv2.VideoCapture] = None
        self.running = False

    async def start(self):
        """Start the vision service."""
        logger.info("Starting Vision service...")

        # Initialize Zenoh session
        logger.info("Connecting to Zenoh...")
        conf = zenoh.Config()
        self.session = zenoh.open(conf)

        # Initialize AprilTag detector
        logger.info("Initializing AprilTag detector...")
        self.apriltag_detector = Detector(
            families="tag36h11",
            nthreads=4,
            quad_decimate=2.0,
            quad_sigma=0.0,
            refine_edges=1,
            decode_sharpening=0.25,
            debug=0,
        )

        # Initialize YOLO model
        logger.info("Loading YOLO model...")
        self.yolo_model = YOLO("yolov8n.pt")  # Nano model for speed

        # Initialize camera
        logger.info("Opening camera...")
        self.camera = cv2.VideoCapture(0)
        if not self.camera.isOpened():
            logger.error("Failed to open camera")
            return

        # Subscribe to vision requests
        await self.subscribe_requests()

        # Start publishing vision updates
        self.running = True
        await self.publish_updates()

        logger.info("Vision service started successfully")

    async def subscribe_requests(self):
        """Subscribe to vision-related requests."""

        def apriltag_callback(sample):
            """Handle AprilTag detection requests."""
            logger.info("Received AprilTag detection request")
            # TODO: Decode request, perform detection, encode and publish response

        def object_detection_callback(sample):
            """Handle object detection requests."""
            logger.info("Received object detection request")
            # TODO: Decode request, perform detection, encode and publish response

        # Subscribe to AprilTag requests
        self.session.declare_subscriber(
            "robot/vision/apriltag/request",
            apriltag_callback
        )

        # Subscribe to object detection requests
        self.session.declare_subscriber(
            "robot/vision/object/request",
            object_detection_callback
        )

    async def publish_updates(self):
        """Publish continuous vision updates."""
        frame_count = 0

        while self.running:
            ret, frame = self.camera.read()
            if not ret:
                logger.warning("Failed to read frame from camera")
                await asyncio.sleep(0.1)
                continue

            frame_count += 1

            # Convert to grayscale for AprilTag detection
            gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)

            # Detect AprilTags
            tags = self.apriltag_detector.detect(
                gray,
                estimate_tag_pose=True,
                camera_params=[800, 800, 320, 240],  # TODO: Use calibrated params
                tag_size=0.16,  # TODO: Configure tag size
            )

            # Detect objects with YOLO
            results = self.yolo_model(frame, verbose=False)

            # TODO: Encode vision update protobuf message
            # For now, just log
            if tags:
                logger.debug(f"Detected {len(tags)} AprilTags")
            if results and len(results[0].boxes) > 0:
                logger.debug(f"Detected {len(results[0].boxes)} objects")

            # Publish status update
            if frame_count % 100 == 0:
                self.publish_status()

            # Control frame rate (30 FPS)
            await asyncio.sleep(1.0 / 30.0)

    def publish_status(self):
        """Publish component status."""
        # TODO: Encode and publish status protobuf message
        logger.debug("Publishing vision status")

    async def stop(self):
        """Stop the vision service."""
        logger.info("Stopping Vision service...")
        self.running = False

        if self.camera:
            self.camera.release()

        if self.session:
            self.session.close()

        logger.info("Vision service stopped")


async def main():
    """Main entry point."""
    service = VisionService()

    try:
        await service.start()

        # Keep running until interrupted
        while True:
            await asyncio.sleep(1)

    except KeyboardInterrupt:
        logger.info("Received keyboard interrupt")
    finally:
        await service.stop()


if __name__ == "__main__":
    asyncio.run(main())
