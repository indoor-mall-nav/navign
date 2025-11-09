#!/usr/bin/env python3
"""
gRPC client example for the PlotService.

This client demonstrates how to call the polygon extraction service
from Python code.
"""

import cv2
import grpc
import logging
from typing import Optional

from proto import plot_pb2, plot_pb2_grpc


# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class PlotClient:
    """
    Client for connecting to the PlotService gRPC server.
    """

    def __init__(self, server_address: str = 'localhost:50052'):
        """
        Initialize the PlotClient.

        Args:
            server_address: Address of the PlotService server (default: localhost:50052)
        """
        self.server_address = server_address
        self.channel: Optional[grpc.Channel] = None
        self.stub: Optional[plot_pb2_grpc.PlotServiceStub] = None

    def connect(self):
        """Establish connection to the gRPC server."""
        logger.info(f"Connecting to PlotService at {self.server_address}")
        self.channel = grpc.insecure_channel(self.server_address)
        self.stub = plot_pb2_grpc.PlotServiceStub(self.channel)
        logger.info("Connected successfully")

    def close(self):
        """Close the connection to the gRPC server."""
        if self.channel:
            self.channel.close()
            logger.info("Connection closed")

    def health_check(self) -> plot_pb2.HealthCheckResponse:
        """
        Perform a health check on the server.

        Returns:
            HealthCheckResponse with server status
        """
        if not self.stub:
            raise RuntimeError("Client not connected. Call connect() first.")

        request = plot_pb2.HealthCheckRequest()
        response = self.stub.HealthCheck(request)
        logger.info(
            f"Health check: healthy={response.healthy}, "
            f"version={response.version}, message={response.message}"
        )
        return response

    def extract_polygons_from_file(
        self,
        image_path: str,
        entity_id: str,
        floor_id: str,
        config: Optional[plot_pb2.ExtractionConfig] = None
    ) -> plot_pb2.ExtractPolygonsResponse:
        """
        Extract polygons from a floor plan image file.

        Args:
            image_path: Path to the floor plan image file
            entity_id: Entity ID (building/mall identifier)
            floor_id: Floor identifier (e.g., "1", "2", "B1")
            config: Optional extraction configuration

        Returns:
            ExtractPolygonsResponse with extracted polygons
        """
        if not self.stub:
            raise RuntimeError("Client not connected. Call connect() first.")

        # Read and encode image
        image = cv2.imread(image_path)
        if image is None:
            raise ValueError(f"Failed to read image from {image_path}")

        success, encoded = cv2.imencode('.png', image)
        if not success:
            raise ValueError("Failed to encode image")

        # Create Image message
        height, width = image.shape[:2]
        image_msg = plot_pb2.Image(
            data=encoded.tobytes(),
            format='png',
            width=width,
            height=height,
            label=f"{entity_id}_{floor_id}"
        )

        # Create request
        request = plot_pb2.ExtractPolygonsRequest(
            entity_id=entity_id,
            floor_id=floor_id,
            floor_plan=image_msg,
            config=config if config else plot_pb2.ExtractionConfig()
        )

        # Call service
        logger.info(f"Extracting polygons from {image_path}")
        response = self.stub.ExtractPolygons(request)

        if response.error:
            logger.error(f"Extraction failed: {response.error}")
        else:
            logger.info(
                f"Extracted {response.total_count} polygons in "
                f"{response.stats.processing_time_ms:.2f}ms"
            )

        return response

    def extract_polygons(
        self,
        image_data: bytes,
        entity_id: str,
        floor_id: str,
        image_format: str = 'png',
        config: Optional[plot_pb2.ExtractionConfig] = None
    ) -> plot_pb2.ExtractPolygonsResponse:
        """
        Extract polygons from floor plan image data.

        Args:
            image_data: Encoded image data (PNG/JPEG)
            entity_id: Entity ID (building/mall identifier)
            floor_id: Floor identifier
            image_format: Image format ('png' or 'jpeg')
            config: Optional extraction configuration

        Returns:
            ExtractPolygonsResponse with extracted polygons
        """
        if not self.stub:
            raise RuntimeError("Client not connected. Call connect() first.")

        # Decode to get dimensions
        import numpy as np
        img_array = np.frombuffer(image_data, dtype=np.uint8)
        image = cv2.imdecode(img_array, cv2.IMREAD_COLOR)
        if image is None:
            raise ValueError("Failed to decode image data")

        height, width = image.shape[:2]

        # Create Image message
        image_msg = plot_pb2.Image(
            data=image_data,
            format=image_format,
            width=width,
            height=height,
            label=f"{entity_id}_{floor_id}"
        )

        # Create request
        request = plot_pb2.ExtractPolygonsRequest(
            entity_id=entity_id,
            floor_id=floor_id,
            floor_plan=image_msg,
            config=config if config else plot_pb2.ExtractionConfig()
        )

        # Call service
        logger.info(f"Extracting polygons for entity={entity_id}, floor={floor_id}")
        response = self.stub.ExtractPolygons(request)

        if response.error:
            logger.error(f"Extraction failed: {response.error}")
        else:
            logger.info(
                f"Extracted {response.total_count} polygons in "
                f"{response.stats.processing_time_ms:.2f}ms"
            )

        return response

    def batch_extract(
        self,
        floor_plans: list[tuple[str, str]],  # [(floor_id, image_path)]
        entity_id: str,
        config: Optional[plot_pb2.ExtractionConfig] = None
    ) -> plot_pb2.BatchExtractResponse:
        """
        Extract polygons from multiple floor plans in batch.

        Args:
            floor_plans: List of (floor_id, image_path) tuples
            entity_id: Entity ID
            config: Optional extraction configuration

        Returns:
            BatchExtractResponse with results for each floor
        """
        if not self.stub:
            raise RuntimeError("Client not connected. Call connect() first.")

        # Prepare floor plan inputs
        floor_inputs = []
        for floor_id, image_path in floor_plans:
            # Read and encode image
            image = cv2.imread(image_path)
            if image is None:
                logger.warning(f"Failed to read image from {image_path}, skipping")
                continue

            success, encoded = cv2.imencode('.png', image)
            if not success:
                logger.warning(f"Failed to encode image {image_path}, skipping")
                continue

            height, width = image.shape[:2]
            image_msg = plot_pb2.Image(
                data=encoded.tobytes(),
                format='png',
                width=width,
                height=height,
                label=f"{entity_id}_{floor_id}"
            )

            floor_inputs.append(plot_pb2.FloorPlanInput(
                floor_id=floor_id,
                floor_plan=image_msg
            ))

        # Create batch request
        request = plot_pb2.BatchExtractRequest(
            entity_id=entity_id,
            floor_plans=floor_inputs,
            config=config if config else plot_pb2.ExtractionConfig()
        )

        # Call service
        logger.info(f"Batch extracting {len(floor_inputs)} floor plans")
        response = self.stub.BatchExtract(request)
        logger.info(
            f"Batch complete - successful: {response.successful}, "
            f"failed: {response.failed}"
        )

        return response

    def __enter__(self):
        """Context manager entry."""
        self.connect()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()


def main():
    """Example usage of the PlotClient."""
    import sys

    # Example: Connect and health check
    with PlotClient('localhost:50052') as client:
        # Health check
        health = client.health_check()
        print(f"Server health: {health.healthy}, version: {health.version}")

        # Example: Extract polygons from an image file
        if len(sys.argv) > 1:
            image_path = sys.argv[1]
            entity_id = sys.argv[2] if len(sys.argv) > 2 else "test_entity"
            floor_id = sys.argv[3] if len(sys.argv) > 3 else "1"

            # Create custom configuration
            config = plot_pb2.ExtractionConfig(
                blur_kernel_size=5.0,
                threshold_value=127,
                min_area=100.0,
                apply_morphology=True,
                morph_kernel_size=5
            )

            response = client.extract_polygons_from_file(
                image_path,
                entity_id,
                floor_id,
                config
            )

            if not response.error:
                print(f"Successfully extracted {response.total_count} polygons:")
                for i, polygon in enumerate(response.polygons):
                    print(f"  Polygon {i+1}: {len(polygon.vertices)} vertices, "
                          f"area={polygon.area:.2f}, label={polygon.label}")
            else:
                print(f"Error: {response.error}")
        else:
            print("Usage: python client.py <image_path> [entity_id] [floor_id]")


if __name__ == '__main__':
    main()
