import pytest
import numpy as np
import cv2


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
def mock_grpc_context(mocker):
    """Create a mock gRPC context for testing."""
    context = mocker.Mock()
    context.set_code = mocker.Mock()
    context.set_details = mocker.Mock()
    return context
