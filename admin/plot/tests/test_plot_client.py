"""
Tests for plot_client.py

Run with: pytest tests/test_plot_client.py
Or from root: uv run pytest admin/plot/tests/
"""

import pytest
import numpy as np
import cv2
from pathlib import Path
import sys

# Add parent directory to path to import plot_client
sys.path.insert(0, str(Path(__file__).parent.parent))

from plot_client import PlotExtractionClient
from proto import task_pb2


class TestPlotExtractionClient:
    """Test PlotExtractionClient functionality"""

    def test_client_creation(self):
        """Test client can be created with default address"""
        client = PlotExtractionClient()
        assert client.orchestrator_address == "localhost:50051"

    def test_client_creation_custom_address(self):
        """Test client with custom orchestrator address"""
        custom_addr = "192.168.1.100:50051"
        client = PlotExtractionClient(custom_addr)
        assert client.orchestrator_address == custom_addr

    def test_context_manager(self):
        """Test client works as context manager"""
        with PlotExtractionClient() as client:
            assert client is not None
            # Connection would be established here in real scenario
            # For now, just verify the client object exists

    def test_config_with_defaults_none(self):
        """Test _get_config_with_defaults with None config"""
        client = PlotExtractionClient()
        config = client._get_config_with_defaults(None)

        assert config.blur_kernel_size == 5.0
        assert config.threshold_value == 127
        assert config.min_area == 100.0
        assert config.epsilon_factor == 0.01
        assert config.canny_low == 50
        assert config.canny_high == 150

    def test_config_with_defaults_partial(self):
        """Test _get_config_with_defaults with partial config"""
        client = PlotExtractionClient()

        partial_config = task_pb2.PlotExtractionConfig(
            blur_kernel_size=7.0, threshold_value=100, min_area=200.0
        )

        config = client._get_config_with_defaults(partial_config)

        assert config.blur_kernel_size == 7.0  # Custom value
        assert config.threshold_value == 100  # Custom value
        assert config.min_area == 200.0  # Custom value
        assert config.epsilon_factor == 0.01  # Default value

    def test_extract_polygons_opencv_placeholder(self):
        """Test _extract_polygons_opencv returns empty results (placeholder)"""
        client = PlotExtractionClient()

        # Create a simple test image (white rectangle on black background)
        image = np.zeros((500, 500, 3), dtype=np.uint8)
        cv2.rectangle(image, (100, 100), (400, 400), (255, 255, 255), -1)

        config = task_pb2.PlotExtractionConfig()

        polygons, stats = client._extract_polygons_opencv(image, config)

        # Placeholder implementation returns empty results
        assert len(polygons) == 0
        assert stats.contours_found == 0
        assert stats.image_width == 500
        assert stats.image_height == 500

    def test_perform_local_extraction_handles_errors(self):
        """Test _perform_local_extraction error handling"""
        client = PlotExtractionClient()

        # Create invalid image data (should still handle gracefully)
        invalid_image = np.array([])  # Empty array
        config = task_pb2.PlotExtractionConfig()

        # This should return an error response, not crash
        response = client._perform_local_extraction(
            invalid_image, "entity-1", "floor-1", config
        )

        assert response is not None

    def test_extract_polygons_with_numpy_image(self):
        """Test extract_polygons with numpy array image"""
        client = PlotExtractionClient()

        # Create test image
        image = np.zeros((300, 300, 3), dtype=np.uint8)
        cv2.rectangle(image, (50, 50), (250, 250), (255, 255, 255), -1)

        # Encode as PNG
        success, encoded = cv2.imencode(".png", image)
        assert success

        response = client.extract_polygons(
            image_data=encoded.tobytes(),
            entity_id="test-entity",
            floor_id="1",
            image_format="png",
        )

        # Should return a valid response (even if polygons are empty in placeholder)
        assert response is not None
        assert response.total_count >= 0

    def test_batch_extract_empty_list(self):
        """Test batch_extract with empty floor plans list"""
        client = PlotExtractionClient()

        response = client.batch_extract(
            floor_plans=[], entity_id="test-entity", config=None
        )

        assert response.successful == 0
        assert response.failed == 0
        assert len(response.extractions) == 0

    @pytest.mark.skipif(
        not Path("test_data/sample_floor_plan.png").exists(),
        reason="Test image not available",
    )
    def test_extract_polygons_from_file(self):
        """Test extracting polygons from an actual image file (if available)"""
        client = PlotExtractionClient()

        image_path = "test_data/sample_floor_plan.png"

        response = client.extract_polygons_from_file(
            image_path=image_path, entity_id="test-entity", floor_id="1", config=None
        )

        # Should return valid response
        assert response is not None
        assert response.total_count >= 0
        assert response.stats is not None


class TestPlotExtractionConfig:
    """Test PlotExtractionConfig protobuf message"""

    def test_config_creation_defaults(self):
        """Test creating config with default values"""
        config = task_pb2.PlotExtractionConfig()

        # Protobuf defaults are zero values
        assert config.blur_kernel_size == 0.0
        assert config.threshold_value == 0
        assert config.min_area == 0.0

    def test_config_creation_custom_values(self):
        """Test creating config with custom values"""
        config = task_pb2.PlotExtractionConfig(
            blur_kernel_size=7.0,
            threshold_value=150,
            min_area=500.0,
            max_area=10000.0,
            epsilon_factor=0.02,
            apply_morphology=True,
            morph_kernel_size=3,
            use_canny=True,
            canny_low=100,
            canny_high=200,
        )

        assert config.blur_kernel_size == 7.0
        assert config.threshold_value == 150
        assert config.min_area == 500.0
        assert config.max_area == 10000.0
        assert config.epsilon_factor == 0.02
        assert config.apply_morphology is True
        assert config.morph_kernel_size == 3
        assert config.use_canny is True
        assert config.canny_low == 100
        assert config.canny_high == 200


class TestImageProcessing:
    """Test image processing utilities"""

    def test_create_simple_test_image(self):
        """Test creating a test image for polygon extraction"""
        # Create image with simple shapes
        image = np.zeros((400, 400, 3), dtype=np.uint8)

        # Draw a rectangle
        cv2.rectangle(image, (50, 50), (150, 150), (255, 255, 255), -1)

        # Draw a circle
        cv2.circle(image, (300, 300), 40, (255, 255, 255), -1)

        # Verify image properties
        assert image.shape == (400, 400, 3)
        assert image.dtype == np.uint8

        # Verify shapes are drawn (non-zero pixels exist)
        gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY)
        non_zero = np.count_nonzero(gray)
        assert non_zero > 0

    def test_image_encoding_decoding(self):
        """Test image encoding and decoding"""
        # Create test image
        original = np.random.randint(0, 256, (100, 100, 3), dtype=np.uint8)

        # Encode
        success, encoded = cv2.imencode(".png", original)
        assert success
        assert len(encoded) > 0

        # Decode
        decoded = cv2.imdecode(encoded, cv2.IMREAD_COLOR)
        assert decoded is not None
        assert decoded.shape == original.shape

        # Should be identical (PNG is lossless)
        np.testing.assert_array_equal(decoded, original)


class TestPolygonMessage:
    """Test Polygon protobuf message"""

    def test_polygon_creation(self):
        """Test creating polygon with vertices"""
        polygon = task_pb2.Polygon(
            vertices=[
                task_pb2.Point(x=0.0, y=0.0),
                task_pb2.Point(x=100.0, y=0.0),
                task_pb2.Point(x=100.0, y=100.0),
                task_pb2.Point(x=0.0, y=100.0),
            ],
            area=10000.0,
            centroid=task_pb2.Point(x=50.0, y=50.0),
            label="room_123",
        )

        assert len(polygon.vertices) == 4
        assert polygon.area == 10000.0
        assert polygon.centroid.x == 50.0
        assert polygon.centroid.y == 50.0
        assert polygon.label == "room_123"

    def test_polygon_empty(self):
        """Test creating empty polygon"""
        polygon = task_pb2.Polygon()

        assert len(polygon.vertices) == 0
        assert polygon.area == 0.0
        assert polygon.label == ""


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
