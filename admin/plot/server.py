#!/usr/bin/env python3
"""
gRPC server for polygon extraction from floor plans.

This server provides the PlotService implementation for extracting polygons
from floor plan images using OpenCV and computer vision algorithms.
"""

import time
import logging
from concurrent import futures
from typing import Optional

import grpc
import cv2
import numpy as np

from proto import plot_pb2, plot_pb2_grpc


# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class PlotServicer(plot_pb2_grpc.PlotServiceServicer):
    """
    Implementation of the PlotService gRPC service.

    This servicer handles polygon extraction from floor plan images using
    OpenCV-based computer vision algorithms.
    """

    VERSION = "0.1.0"

    def ExtractPolygons(
        self,
        request: plot_pb2.ExtractPolygonsRequest,
        context: grpc.ServicerContext
    ) -> plot_pb2.ExtractPolygonsResponse:
        """
        Extract polygons from a single floor plan image.

        Args:
            request: ExtractPolygonsRequest containing floor plan image and config
            context: gRPC context

        Returns:
            ExtractPolygonsResponse with extracted polygons and statistics
        """
        start_time = time.time()

        try:
            logger.info(
                f"Processing floor plan extraction - "
                f"entity_id: {request.entity_id}, floor_id: {request.floor_id}"
            )

            # Decode image from protobuf
            image = self._decode_image(request.floor_plan)
            if image is None:
                error_msg = "Failed to decode floor plan image"
                logger.error(error_msg)
                return plot_pb2.ExtractPolygonsResponse(
                    polygons=[],
                    total_count=0,
                    error=error_msg,
                    stats=plot_pb2.ProcessingStats(
                        contours_found=0,
                        contours_filtered=0,
                        processing_time_ms=0,
                        image_width=0,
                        image_height=0
                    )
                )

            # Get configuration with defaults
            config = self._get_config_with_defaults(request.config)

            # Extract polygons using OpenCV
            polygons, stats = self._extract_polygons_opencv(image, config)

            # Calculate processing time
            processing_time = (time.time() - start_time) * 1000
            stats.processing_time_ms = processing_time

            logger.info(
                f"Extraction complete - found {len(polygons)} polygons "
                f"in {processing_time:.2f}ms"
            )

            return plot_pb2.ExtractPolygonsResponse(
                polygons=polygons,
                total_count=len(polygons),
                error="",
                stats=stats
            )

        except Exception as e:
            error_msg = f"Error during polygon extraction: {str(e)}"
            logger.error(error_msg, exc_info=True)
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(error_msg)
            return plot_pb2.ExtractPolygonsResponse(
                polygons=[],
                total_count=0,
                error=error_msg,
                stats=plot_pb2.ProcessingStats()
            )

    def BatchExtract(
        self,
        request: plot_pb2.BatchExtractRequest,
        context: grpc.ServicerContext
    ) -> plot_pb2.BatchExtractResponse:
        """
        Extract polygons from multiple floor plans in batch.

        Args:
            request: BatchExtractRequest with multiple floor plans
            context: gRPC context

        Returns:
            BatchExtractResponse with results for each floor
        """
        logger.info(
            f"Processing batch extraction - "
            f"entity_id: {request.entity_id}, "
            f"floors: {len(request.floor_plans)}"
        )

        extractions = []
        successful = 0
        failed = 0

        for floor_input in request.floor_plans:
            try:
                # Create individual request
                individual_request = plot_pb2.ExtractPolygonsRequest(
                    entity_id=request.entity_id,
                    floor_id=floor_input.floor_id,
                    floor_plan=floor_input.floor_plan,
                    config=request.config
                )

                # Extract polygons
                response = self.ExtractPolygons(individual_request, context)

                # Create floor extraction result
                floor_extraction = plot_pb2.FloorExtraction(
                    floor_id=floor_input.floor_id,
                    polygons=response.polygons,
                    error=response.error,
                    stats=response.stats
                )

                extractions.append(floor_extraction)

                if response.error:
                    failed += 1
                else:
                    successful += 1

            except Exception as e:
                error_msg = f"Error processing floor {floor_input.floor_id}: {str(e)}"
                logger.error(error_msg, exc_info=True)
                extractions.append(plot_pb2.FloorExtraction(
                    floor_id=floor_input.floor_id,
                    polygons=[],
                    error=error_msg,
                    stats=plot_pb2.ProcessingStats()
                ))
                failed += 1

        logger.info(
            f"Batch extraction complete - "
            f"successful: {successful}, failed: {failed}"
        )

        return plot_pb2.BatchExtractResponse(
            extractions=extractions,
            successful=successful,
            failed=failed
        )

    def HealthCheck(
        self,
        request: plot_pb2.HealthCheckRequest,
        context: grpc.ServicerContext
    ) -> plot_pb2.HealthCheckResponse:
        """
        Health check endpoint.

        Args:
            request: HealthCheckRequest (empty)
            context: gRPC context

        Returns:
            HealthCheckResponse with service status
        """
        return plot_pb2.HealthCheckResponse(
            healthy=True,
            version=self.VERSION,
            message="PlotService is running"
        )

    def _decode_image(self, image_msg: plot_pb2.Image) -> Optional[np.ndarray]:
        """
        Decode image from protobuf message to OpenCV format.

        Args:
            image_msg: Image protobuf message

        Returns:
            OpenCV image (numpy array) or None if decoding fails
        """
        try:
            # Convert bytes to numpy array
            img_array = np.frombuffer(image_msg.data, dtype=np.uint8)
            # Decode image
            image = cv2.imdecode(img_array, cv2.IMREAD_COLOR)
            return image
        except Exception as e:
            logger.error(f"Failed to decode image: {e}")
            return None

    def _get_config_with_defaults(
        self,
        config: plot_pb2.ExtractionConfig
    ) -> plot_pb2.ExtractionConfig:
        """
        Get configuration with default values for unset fields.

        Args:
            config: User-provided configuration (may be empty)

        Returns:
            Configuration with defaults applied
        """
        # Create new config with defaults
        default_config = plot_pb2.ExtractionConfig(
            blur_kernel_size=5.0,
            threshold_value=127,
            threshold_type=0,
            min_area=100.0,
            max_area=0.0,  # 0 means no limit
            epsilon_factor=0.01,
            apply_morphology=True,
            morph_kernel_size=5,
            use_canny=False,
            canny_low=50,
            canny_high=150
        )

        # Override with user-provided values
        if config.blur_kernel_size > 0:
            default_config.blur_kernel_size = config.blur_kernel_size
        if config.threshold_value > 0:
            default_config.threshold_value = config.threshold_value
        if config.threshold_type > 0:
            default_config.threshold_type = config.threshold_type
        if config.min_area > 0:
            default_config.min_area = config.min_area
        if config.max_area > 0:
            default_config.max_area = config.max_area
        if config.epsilon_factor > 0:
            default_config.epsilon_factor = config.epsilon_factor
        if config.morph_kernel_size > 0:
            default_config.morph_kernel_size = config.morph_kernel_size
        if config.canny_low > 0:
            default_config.canny_low = config.canny_low
        if config.canny_high > 0:
            default_config.canny_high = config.canny_high

        # Boolean fields
        default_config.apply_morphology = config.apply_morphology
        default_config.use_canny = config.use_canny

        return default_config

    def _extract_polygons_opencv(
        self,
        image: np.ndarray,
        config: plot_pb2.ExtractionConfig
    ) -> tuple[list[plot_pb2.Polygon], plot_pb2.ProcessingStats]:
        """
        Extract polygons from image using OpenCV.

        This method contains the placeholder for the actual polygon extraction
        algorithm. The user will implement the detailed algorithm.

        Args:
            image: OpenCV image (numpy array)
            config: Extraction configuration

        Returns:
            Tuple of (polygons list, processing stats)
        """
        height, width = image.shape[:2]
        stats = plot_pb2.ProcessingStats(
            contours_found=0,
            contours_filtered=0,
            image_width=width,
            image_height=height
        )

        # TODO: Implement the actual polygon extraction algorithm
        # This is a placeholder that returns an empty list
        # The user will implement the detailed OpenCV algorithm here

        # Example structure (to be implemented):
        # 1. Preprocess image (grayscale, blur, threshold)
        # 2. Apply morphological operations if enabled
        # 3. Detect edges (Canny if enabled)
        # 4. Find contours
        # 5. Approximate contours to polygons
        # 6. Filter by area
        # 7. Calculate centroids
        # 8. Convert to protobuf format

        polygons = []

        logger.warning(
            "Polygon extraction algorithm not yet implemented. "
            "Returning empty result."
        )

        return polygons, stats


def serve(port: int = 50052, max_workers: int = 10):
    """
    Start the gRPC server.

    Args:
        port: Port to listen on (default: 50052)
        max_workers: Maximum number of worker threads (default: 10)
    """
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=max_workers))
    plot_pb2_grpc.add_PlotServiceServicer_to_server(PlotServicer(), server)

    server_address = f'[::]:{port}'
    server.add_insecure_port(server_address)

    logger.info(f"Starting PlotService gRPC server on {server_address}")
    server.start()
    logger.info("Server started successfully. Press Ctrl+C to stop.")

    try:
        server.wait_for_termination()
    except KeyboardInterrupt:
        logger.info("Shutting down server...")
        server.stop(grace=5)
        logger.info("Server stopped")


if __name__ == '__main__':
    serve()
