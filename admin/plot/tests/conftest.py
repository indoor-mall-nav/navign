import pytest
import numpy as np
import cv2
from proto import task_pb2


@pytest.fixture
def sample_image():
    """Create a sample OpenCV image for testing."""
    img = np.zeros((100, 100, 3), dtype=np.uint8)
    cv2.rectangle(img, (25, 25), (75, 75), (0, 0, 255), -1)
    return img


@pytest.fixture
def sample_image_bytes(sample_image):
    """Create sample image bytes in PNG format."""
    success, encoded = cv2.imencode(".png", sample_image)
    assert success
    return encoded.tobytes()


@pytest.fixture
def sample_floor_plan_image(sample_image_bytes):
    """Create a sample FloorPlanImage protobuf message."""
    return task_pb2.FloorPlanImage(
        data=sample_image_bytes, format="png", width=100, height=100
    )


@pytest.fixture
def sample_extraction_config():
    """Create a sample PlotExtractionConfig."""
    return task_pb2.PlotExtractionConfig(
        blur_kernel_size=5.0,
        threshold_value=127,
        min_area=100.0,
        apply_morphology=True,
        morph_kernel_size=5,
    )


@pytest.fixture
def mock_grpc_context(mocker):
    """Create a mock gRPC context for testing."""
    context = mocker.Mock()
    context.set_code = mocker.Mock()
    context.set_details = mocker.Mock()
    return context
