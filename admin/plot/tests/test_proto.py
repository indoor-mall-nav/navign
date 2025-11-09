"""Tests for the protobuf messages from task.proto."""

import pytest
from proto import task_pb2


@pytest.mark.unit
class TestFloorPlanImage:
    """Test the FloorPlanImage message."""

    def test_create_floor_plan_image(self, sample_floor_plan_image):
        """Test creating a FloorPlanImage."""
        assert sample_floor_plan_image.format == "png"
        assert sample_floor_plan_image.width == 100
        assert sample_floor_plan_image.height == 100
        assert len(sample_floor_plan_image.data) > 0

    def test_floor_plan_serialization(self, sample_floor_plan_image):
        """Test FloorPlanImage serialization."""
        serialized = sample_floor_plan_image.SerializeToString()
        assert isinstance(serialized, bytes)

        deserialized = task_pb2.FloorPlanImage()
        deserialized.ParseFromString(serialized)

        assert deserialized.format == sample_floor_plan_image.format
        assert deserialized.width == sample_floor_plan_image.width
        assert deserialized.height == sample_floor_plan_image.height


@pytest.mark.unit
class TestPlotExtractionConfig:
    """Test the PlotExtractionConfig message."""

    def test_create_config(self, sample_extraction_config):
        """Test creating extraction config."""
        assert sample_extraction_config.blur_kernel_size == 5.0
        assert sample_extraction_config.threshold_value == 127
        assert sample_extraction_config.min_area == 100.0
        assert sample_extraction_config.apply_morphology is True

    def test_config_serialization(self, sample_extraction_config):
        """Test config serialization."""
        serialized = sample_extraction_config.SerializeToString()
        assert isinstance(serialized, bytes)

        deserialized = task_pb2.PlotExtractionConfig()
        deserialized.ParseFromString(serialized)

        assert deserialized.blur_kernel_size == sample_extraction_config.blur_kernel_size
        assert deserialized.threshold_value == sample_extraction_config.threshold_value


@pytest.mark.unit
class TestPolygonMessage:
    """Test the Polygon message."""

    def test_create_polygon(self):
        """Test creating a polygon."""
        vertices = [
            task_pb2.Point(x=0.0, y=0.0),
            task_pb2.Point(x=100.0, y=0.0),
            task_pb2.Point(x=100.0, y=100.0),
            task_pb2.Point(x=0.0, y=100.0),
        ]

        polygon = task_pb2.Polygon(
            vertices=vertices,
            label="room_1",
            area=10000.0,
            centroid=task_pb2.Point(x=50.0, y=50.0),
        )

        assert len(polygon.vertices) == 4
        assert polygon.label == "room_1"
        assert polygon.area == 10000.0
        assert polygon.centroid.x == 50.0
        assert polygon.centroid.y == 50.0

    def test_polygon_serialization(self):
        """Test polygon serialization."""
        vertices = [
            task_pb2.Point(x=10.0, y=20.0),
            task_pb2.Point(x=30.0, y=40.0),
            task_pb2.Point(x=50.0, y=60.0),
        ]

        original = task_pb2.Polygon(
            vertices=vertices,
            label="test",
            area=500.0,
            centroid=task_pb2.Point(x=25.0, y=35.0),
        )

        serialized = original.SerializeToString()
        assert isinstance(serialized, bytes)

        deserialized = task_pb2.Polygon()
        deserialized.ParseFromString(serialized)

        assert len(deserialized.vertices) == 3
        assert deserialized.label == "test"
        assert deserialized.area == 500.0


@pytest.mark.unit
class TestExtractPolygonsRequest:
    """Test the ExtractPolygonsRequest message."""

    def test_create_request(self, sample_floor_plan_image, sample_extraction_config):
        """Test creating extraction request."""
        request = task_pb2.ExtractPolygonsRequest(
            entity_id="mall_001",
            floor_id="1",
            floor_plan=sample_floor_plan_image,
            config=sample_extraction_config,
        )

        assert request.entity_id == "mall_001"
        assert request.floor_id == "1"
        assert request.floor_plan.width == 100
        assert request.floor_plan.height == 100
        assert request.config.blur_kernel_size == 5.0

    def test_request_serialization(
        self, sample_floor_plan_image, sample_extraction_config
    ):
        """Test request serialization."""
        request = task_pb2.ExtractPolygonsRequest(
            entity_id="mall_001",
            floor_id="1",
            floor_plan=sample_floor_plan_image,
            config=sample_extraction_config,
        )

        serialized = request.SerializeToString()
        assert isinstance(serialized, bytes)

        deserialized = task_pb2.ExtractPolygonsRequest()
        deserialized.ParseFromString(serialized)

        assert deserialized.entity_id == "mall_001"
        assert deserialized.floor_id == "1"


@pytest.mark.unit
class TestProcessingStats:
    """Test the PlotProcessingStats message."""

    def test_create_stats(self):
        """Test creating processing stats."""
        stats = task_pb2.PlotProcessingStats(
            contours_found=50,
            contours_filtered=30,
            processing_time_ms=123.45,
            image_width=1024,
            image_height=768,
        )

        assert stats.contours_found == 50
        assert stats.contours_filtered == 30
        assert stats.processing_time_ms == 123.45
        assert stats.image_width == 1024
        assert stats.image_height == 768
