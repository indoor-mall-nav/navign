#!/usr/bin/env python3
"""
Plot client for connecting to the Orchestrator gRPC server.

This client implements polygon extraction from floor plans and submits
the results to the orchestrator.
"""

import time
import logging
from typing import Optional
import cv2
import numpy as np
import grpc

from proto import task_pb2, task_pb2_grpc


# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class PlotExtractionClient:
    """
    Client for connecting to the Orchestrator and performing plot extraction.

    This client connects to the Rust orchestrator's gRPC server and provides
    polygon extraction functionality for floor plans using OpenCV.
    """

    def __init__(self, orchestrator_address: str = 'localhost:50051'):
        """
        Initialize the plot extraction client.

        Args:
            orchestrator_address: Address of the orchestrator server (default: localhost:50051)
        """
        self.orchestrator_address = orchestrator_address
        self.channel: Optional[grpc.Channel] = None
        self.stub: Optional[task_pb2_grpc.OrchestratorServiceStub] = None

    def connect(self):
        """Establish connection to the orchestrator gRPC server."""
        logger.info(f"Connecting to Orchestrator at {self.orchestrator_address}")
        self.channel = grpc.insecure_channel(self.orchestrator_address)
        self.stub = task_pb2_grpc.OrchestratorServiceStub(self.channel)
        logger.info("Connected to orchestrator successfully")

    def close(self):
        """Close the connection to the orchestrator."""
        if self.channel:
            self.channel.close()
            logger.info("Connection closed")

    def extract_polygons_from_file(
        self,
        image_path: str,
        entity_id: str,
        floor_id: str,
        config: Optional[task_pb2.PlotExtractionConfig] = None
    ) -> task_pb2.ExtractPolygonsResponse:
        """
        Extract polygons from a floor plan image file.

        Args:
            image_path: Path to the floor plan image
            entity_id: Entity/building identifier
            floor_id: Floor identifier
            config: Optional extraction configuration

        Returns:
            ExtractPolygonsResponse with extracted polygons
        """
        if not self.stub:
            raise RuntimeError("Client not connected. Call connect() first.")

        # Read image
        image = cv2.imread(image_path)
        if image is None:
            raise ValueError(f"Failed to read image from {image_path}")

        # Encode image
        success, encoded = cv2.imencode('.png', image)
        if not success:
            raise ValueError("Failed to encode image")

        height, width = image.shape[:2]

        # Create FloorPlanImage message
        floor_plan_image = task_pb2.FloorPlanImage(
            data=encoded.tobytes(),
            format='png',
            width=width,
            height=height
        )

        # Create request
        request = task_pb2.ExtractPolygonsRequest(
            entity_id=entity_id,
            floor_id=floor_id,
            floor_plan=floor_plan_image,
            config=config if config else task_pb2.PlotExtractionConfig()
        )

        # Perform local extraction (Python implementation)
        logger.info(f"Extracting polygons from {image_path} locally")
        response = self._perform_local_extraction(image, entity_id, floor_id, config)

        # TODO: Optionally send to orchestrator for storage/coordination
        # orchestrator_response = self.stub.ExtractPolygons(request)

        return response

    def extract_polygons(
        self,
        image_data: bytes,
        entity_id: str,
        floor_id: str,
        image_format: str = 'png',
        config: Optional[task_pb2.PlotExtractionConfig] = None
    ) -> task_pb2.ExtractPolygonsResponse:
        """
        Extract polygons from floor plan image data.

        Args:
            image_data: Encoded image data
            entity_id: Entity identifier
            floor_id: Floor identifier
            image_format: Image format ('png' or 'jpeg')
            config: Optional extraction configuration

        Returns:
            ExtractPolygonsResponse with extracted polygons
        """
        if not self.stub:
            raise RuntimeError("Client not connected. Call connect() first.")

        # Decode image
        img_array = np.frombuffer(image_data, dtype=np.uint8)
        image = cv2.imdecode(img_array, cv2.IMREAD_COLOR)
        if image is None:
            raise ValueError("Failed to decode image data")

        # Perform extraction
        response = self._perform_local_extraction(image, entity_id, floor_id, config)
        return response

    def batch_extract(
        self,
        floor_plans: list[tuple[str, str]],  # [(floor_id, image_path)]
        entity_id: str,
        config: Optional[task_pb2.PlotExtractionConfig] = None
    ) -> task_pb2.BatchExtractResponse:
        """
        Extract polygons from multiple floor plans in batch.

        Args:
            floor_plans: List of (floor_id, image_path) tuples
            entity_id: Entity identifier
            config: Optional extraction configuration

        Returns:
            BatchExtractResponse with results for each floor
        """
        if not self.stub:
            raise RuntimeError("Client not connected. Call connect() first.")

        extractions = []
        successful = 0
        failed = 0

        for floor_id, image_path in floor_plans:
            try:
                response = self.extract_polygons_from_file(
                    image_path, entity_id, floor_id, config
                )

                extraction = task_pb2.FloorExtraction(
                    floor_id=floor_id,
                    polygons=response.polygons,
                    error=response.error,
                    stats=response.stats
                )
                extractions.append(extraction)

                if response.error:
                    failed += 1
                else:
                    successful += 1

            except Exception as e:
                logger.error(f"Failed to extract floor {floor_id}: {e}")
                extraction = task_pb2.FloorExtraction(
                    floor_id=floor_id,
                    polygons=[],
                    error=str(e),
                    stats=task_pb2.PlotProcessingStats()
                )
                extractions.append(extraction)
                failed += 1

        return task_pb2.BatchExtractResponse(
            extractions=extractions,
            successful=successful,
            failed=failed
        )

    def _perform_local_extraction(
        self,
        image: np.ndarray,
        entity_id: str,
        floor_id: str,
        config: Optional[task_pb2.PlotExtractionConfig] = None
    ) -> task_pb2.ExtractPolygonsResponse:
        """
        Perform polygon extraction using local OpenCV implementation.

        This method contains the actual computer vision algorithm for extracting
        polygons from floor plans.

        Args:
            image: OpenCV image (numpy array)
            entity_id: Entity identifier
            floor_id: Floor identifier
            config: Extraction configuration

        Returns:
            ExtractPolygonsResponse with extracted polygons
        """
        start_time = time.time()

        try:
            # Get config with defaults
            cfg = self._get_config_with_defaults(config)

            # Extract polygons using OpenCV
            polygons, stats = self._extract_polygons_opencv(image, cfg)

            # Calculate processing time
            processing_time = (time.time() - start_time) * 1000
            stats.processing_time_ms = processing_time

            logger.info(
                f"Extracted {len(polygons)} polygons for {entity_id}/{floor_id} "
                f"in {processing_time:.2f}ms"
            )

            return task_pb2.ExtractPolygonsResponse(
                polygons=polygons,
                total_count=len(polygons),
                error="",
                stats=stats
            )

        except Exception as e:
            logger.error(f"Extraction failed: {e}", exc_info=True)
            return task_pb2.ExtractPolygonsResponse(
                polygons=[],
                total_count=0,
                error=str(e),
                stats=task_pb2.PlotProcessingStats()
            )

    def _get_config_with_defaults(
        self,
        config: Optional[task_pb2.PlotExtractionConfig]
    ) -> task_pb2.PlotExtractionConfig:
        """
        Get configuration with defaults applied.

        Args:
            config: User-provided configuration (may be None)

        Returns:
            Configuration with defaults
        """
        if config is None:
            config = task_pb2.PlotExtractionConfig()

        # Apply defaults for zero values
        default_config = task_pb2.PlotExtractionConfig(
            blur_kernel_size=config.blur_kernel_size if config.blur_kernel_size > 0 else 5.0,
            threshold_value=config.threshold_value if config.threshold_value > 0 else 127,
            threshold_type=config.threshold_type,
            min_area=config.min_area if config.min_area > 0 else 100.0,
            max_area=config.max_area,
            epsilon_factor=config.epsilon_factor if config.epsilon_factor > 0 else 0.01,
            apply_morphology=config.apply_morphology if config.HasField('apply_morphology') else True,
            morph_kernel_size=config.morph_kernel_size if config.morph_kernel_size > 0 else 5,
            use_canny=config.use_canny,
            canny_low=config.canny_low if config.canny_low > 0 else 50,
            canny_high=config.canny_high if config.canny_high > 0 else 150
        )

        return default_config

    def _extract_polygons_opencv(
        self,
        image: np.ndarray,
        config: task_pb2.PlotExtractionConfig
    ) -> tuple[list[task_pb2.Polygon], task_pb2.PlotProcessingStats]:
        """
        Extract polygons from image using OpenCV.

        TODO: This is a placeholder implementation. The user will implement
        the actual computer vision algorithm using OpenCV.

        The algorithm should:
        1. Preprocess image (grayscale, blur, threshold)
        2. Apply morphological operations if enabled
        3. Detect edges (Canny if enabled)
        4. Find contours
        5. Approximate contours to polygons
        6. Filter by area
        7. Calculate centroids and areas
        8. Convert to protobuf format

        Args:
            image: OpenCV image
            config: Extraction configuration

        Returns:
            Tuple of (polygons, processing stats)
        """
        height, width = image.shape[:2]

        stats = task_pb2.PlotProcessingStats(
            contours_found=0,
            contours_filtered=0,
            image_width=width,
            image_height=height
        )

        # TODO: Implement actual polygon extraction algorithm
        # This is a placeholder that returns empty results

        polygons = []

        logger.warning(
            "Polygon extraction algorithm not yet implemented. "
            "This is a placeholder. Implement _extract_polygons_opencv() "
            "with OpenCV contour detection and polygon approximation."
        )

        return polygons, stats

    def __enter__(self):
        """Context manager entry."""
        self.connect()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()


def main():
    """Example usage of the PlotExtractionClient."""
    import sys

    if len(sys.argv) < 2:
        print("Usage: python plot_client.py <image_path> [entity_id] [floor_id]")
        print("\nExample:")
        print("  python plot_client.py floor_plan.png mall_001 1")
        return

    image_path = sys.argv[1]
    entity_id = sys.argv[2] if len(sys.argv) > 2 else "test_entity"
    floor_id = sys.argv[3] if len(sys.argv) > 3 else "1"

    # Create client and connect
    with PlotExtractionClient('localhost:50051') as client:
        # Create custom configuration
        config = task_pb2.PlotExtractionConfig(
            blur_kernel_size=5.0,
            threshold_value=127,
            min_area=100.0,
            apply_morphology=True,
            morph_kernel_size=5
        )

        # Extract polygons
        response = client.extract_polygons_from_file(
            image_path,
            entity_id,
            floor_id,
            config
        )

        # Print results
        if response.error:
            print(f"Error: {response.error}")
        else:
            print(f"Successfully extracted {response.total_count} polygons")
            print(f"Processing time: {response.stats.processing_time_ms:.2f}ms")
            print(f"Contours found: {response.stats.contours_found}")
            print(f"Contours filtered: {response.stats.contours_filtered}")

            for i, polygon in enumerate(response.polygons):
                print(f"\nPolygon {i+1}:")
                print(f"  Vertices: {len(polygon.vertices)}")
                print(f"  Area: {polygon.area:.2f} sq px")
                print(f"  Centroid: ({polygon.centroid.x:.2f}, {polygon.centroid.y:.2f})")
                if polygon.label:
                    print(f"  Label: {polygon.label}")


if __name__ == '__main__':
    main()
