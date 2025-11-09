"""Tests for the gRPC PlotService."""

import pytest
import grpc
import cv2
import numpy as np
from unittest.mock import Mock, patch

from proto import plot_pb2, plot_pb2_grpc
from server import PlotServicer


@pytest.mark.unit
class TestPlotServicer:
    """Test the PlotServicer implementation."""

    def test_health_check(self, mock_grpc_context):
        """Test the health check endpoint."""
        servicer = PlotServicer()
        request = plot_pb2.HealthCheckRequest()

        response = servicer.HealthCheck(request, mock_grpc_context)

        assert response.healthy is True
        assert response.version == "0.1.0"
        assert "running" in response.message.lower()

    def test_decode_image_success(self, sample_image_bytes):
        """Test successful image decoding."""
        servicer = PlotServicer()

        # Create Image message
        image_msg = plot_pb2.Image(
            data=sample_image_bytes,
            format="png",
            width=100,
            height=100,
            label="test"
        )

        decoded = servicer._decode_image(image_msg)

        assert decoded is not None
        assert decoded.shape == (100, 100, 3)

    def test_decode_image_failure(self):
        """Test image decoding with invalid data."""
        servicer = PlotServicer()

        # Create Image message with invalid data
        image_msg = plot_pb2.Image(
            data=b"invalid_image_data",
            format="png",
            width=100,
            height=100,
            label="test"
        )

        decoded = servicer._decode_image(image_msg)

        assert decoded is None

    def test_get_config_with_defaults_empty(self):
        """Test getting config with all defaults."""
        servicer = PlotServicer()
        config = plot_pb2.ExtractionConfig()

        result = servicer._get_config_with_defaults(config)

        assert result.blur_kernel_size == 5.0
        assert result.threshold_value == 127
        assert result.min_area == 100.0
        assert result.apply_morphology is True
        assert result.morph_kernel_size == 5

    def test_get_config_with_defaults_custom(self):
        """Test getting config with custom values."""
        servicer = PlotServicer()
        config = plot_pb2.ExtractionConfig(
            blur_kernel_size=7.0,
            threshold_value=150,
            min_area=200.0,
            apply_morphology=False
        )

        result = servicer._get_config_with_defaults(config)

        assert result.blur_kernel_size == 7.0
        assert result.threshold_value == 150
        assert result.min_area == 200.0
        assert result.apply_morphology is False

    def test_extract_polygons_invalid_image(self, mock_grpc_context):
        """Test polygon extraction with invalid image."""
        servicer = PlotServicer()

        request = plot_pb2.ExtractPolygonsRequest(
            entity_id="test_entity",
            floor_id="1",
            floor_plan=plot_pb2.Image(
                data=b"invalid",
                format="png",
                width=0,
                height=0,
                label="test"
            )
        )

        response = servicer.ExtractPolygons(request, mock_grpc_context)

        assert response.total_count == 0
        assert "Failed to decode" in response.error

    def test_extract_polygons_success(self, sample_image_bytes, mock_grpc_context):
        """Test successful polygon extraction."""
        servicer = PlotServicer()

        request = plot_pb2.ExtractPolygonsRequest(
            entity_id="test_entity",
            floor_id="1",
            floor_plan=plot_pb2.Image(
                data=sample_image_bytes,
                format="png",
                width=100,
                height=100,
                label="test"
            ),
            config=plot_pb2.ExtractionConfig()
        )

        response = servicer.ExtractPolygons(request, mock_grpc_context)

        # Should return successfully (even if no polygons found yet)
        assert response.error == ""
        assert response.stats.image_width == 100
        assert response.stats.image_height == 100
        assert response.stats.processing_time_ms > 0

    def test_batch_extract_empty(self, mock_grpc_context):
        """Test batch extraction with no floor plans."""
        servicer = PlotServicer()

        request = plot_pb2.BatchExtractRequest(
            entity_id="test_entity",
            floor_plans=[]
        )

        response = servicer.BatchExtract(request, mock_grpc_context)

        assert response.successful == 0
        assert response.failed == 0
        assert len(response.extractions) == 0

    def test_batch_extract_multiple(self, multiple_images, mock_grpc_context):
        """Test batch extraction with multiple floor plans."""
        servicer = PlotServicer()

        # Create floor plan inputs
        floor_plans = []
        for i, img_msg in enumerate(multiple_images):
            floor_plans.append(plot_pb2.FloorPlanInput(
                floor_id=str(i + 1),
                floor_plan=img_msg
            ))

        request = plot_pb2.BatchExtractRequest(
            entity_id="test_entity",
            floor_plans=floor_plans
        )

        response = servicer.BatchExtract(request, mock_grpc_context)

        assert len(response.extractions) == 3
        # All should succeed (even if no polygons found)
        assert response.successful == 3
        assert response.failed == 0


@pytest.mark.unit
class TestExtractionConfig:
    """Test the ExtractionConfig message."""

    def test_create_config_with_defaults(self):
        """Test creating config with default values."""
        config = plot_pb2.ExtractionConfig()

        # Protobuf defaults
        assert config.blur_kernel_size == 0.0
        assert config.threshold_value == 0
        assert config.min_area == 0.0

    def test_create_config_custom(self):
        """Test creating config with custom values."""
        config = plot_pb2.ExtractionConfig(
            blur_kernel_size=7.0,
            threshold_value=150,
            threshold_type=1,
            min_area=200.0,
            max_area=10000.0,
            epsilon_factor=0.02,
            apply_morphology=False,
            morph_kernel_size=3,
            use_canny=True,
            canny_low=100,
            canny_high=200
        )

        assert config.blur_kernel_size == 7.0
        assert config.threshold_value == 150
        assert config.threshold_type == 1
        assert config.min_area == 200.0
        assert config.max_area == 10000.0
        assert config.epsilon_factor == 0.02
        assert config.apply_morphology is False
        assert config.morph_kernel_size == 3
        assert config.use_canny is True
        assert config.canny_low == 100
        assert config.canny_high == 200


@pytest.mark.unit
class TestPolygonMessage:
    """Test the Polygon message."""

    def test_create_polygon(self):
        """Test creating a polygon."""
        vertices = [
            plot_pb2.Point(x=0.0, y=0.0),
            plot_pb2.Point(x=100.0, y=0.0),
            plot_pb2.Point(x=100.0, y=100.0),
            plot_pb2.Point(x=0.0, y=100.0),
        ]

        polygon = plot_pb2.Polygon(
            vertices=vertices,
            label="room_1",
            area=10000.0,
            centroid=plot_pb2.Point(x=50.0, y=50.0)
        )

        assert len(polygon.vertices) == 4
        assert polygon.label == "room_1"
        assert polygon.area == 10000.0
        assert polygon.centroid.x == 50.0
        assert polygon.centroid.y == 50.0

    def test_polygon_serialization(self):
        """Test polygon serialization and deserialization."""
        vertices = [
            plot_pb2.Point(x=10.0, y=20.0),
            plot_pb2.Point(x=30.0, y=40.0),
            plot_pb2.Point(x=50.0, y=60.0),
        ]

        original = plot_pb2.Polygon(
            vertices=vertices,
            label="test",
            area=500.0,
            centroid=plot_pb2.Point(x=25.0, y=35.0)
        )

        # Serialize
        serialized = original.SerializeToString()
        assert isinstance(serialized, bytes)

        # Deserialize
        deserialized = plot_pb2.Polygon()
        deserialized.ParseFromString(serialized)

        assert len(deserialized.vertices) == 3
        assert deserialized.label == "test"
        assert deserialized.area == 500.0


@pytest.mark.unit
class TestExtractPolygonsRequest:
    """Test the ExtractPolygonsRequest message."""

    def test_create_request(self, sample_plot_image):
        """Test creating extraction request."""
        config = plot_pb2.ExtractionConfig(
            blur_kernel_size=5.0,
            threshold_value=127
        )

        request = plot_pb2.ExtractPolygonsRequest(
            entity_id="mall_001",
            floor_id="1",
            floor_plan=sample_plot_image,
            config=config
        )

        assert request.entity_id == "mall_001"
        assert request.floor_id == "1"
        assert request.floor_plan.width == 50
        assert request.floor_plan.height == 50
        assert request.config.blur_kernel_size == 5.0

    def test_request_without_config(self, sample_plot_image):
        """Test creating request without config."""
        request = plot_pb2.ExtractPolygonsRequest(
            entity_id="mall_001",
            floor_id="2",
            floor_plan=sample_plot_image
        )

        assert request.entity_id == "mall_001"
        assert request.floor_id == "2"
        # Config should use defaults
        assert request.config.blur_kernel_size == 0.0


@pytest.mark.unit
class TestProcessingStats:
    """Test the ProcessingStats message."""

    def test_create_stats(self):
        """Test creating processing stats."""
        stats = plot_pb2.ProcessingStats(
            contours_found=50,
            contours_filtered=30,
            processing_time_ms=123.45,
            image_width=1024,
            image_height=768
        )

        assert stats.contours_found == 50
        assert stats.contours_filtered == 30
        assert stats.processing_time_ms == 123.45
        assert stats.image_width == 1024
        assert stats.image_height == 768
