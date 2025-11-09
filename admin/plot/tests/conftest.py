import pytest
import numpy as np
import cv2
from proto import plot_pb2


@pytest.fixture
def sample_image():
    img = np.zeros((100, 100, 3), dtype=np.uint8)
    cv2.rectangle(img, (25, 25), (75, 75), (0, 0, 255), -1)
    return img


@pytest.fixture
def sample_image_bytes(sample_image):
    success, encoded = cv2.imencode('.png', sample_image)
    assert success
    return encoded.tobytes()


@pytest.fixture
def sample_plot_image():
    img = np.zeros((50, 50, 3), dtype=np.uint8)
    cv2.rectangle(img, (10, 10), (40, 40), (255, 0, 0), -1)
    success, encoded = cv2.imencode('.png', img)
    assert success

    return plot_pb2.Image(
        data=encoded.tobytes(),
        format="png",
        width=50,
        height=50,
        label="test_image"
    )


@pytest.fixture
def sample_plot_request(sample_plot_image):
    return plot_pb2.PlotRequest(
        plot_id="test_plot_001",
        images=[sample_plot_image],
        description="Test plot request"
    )


@pytest.fixture
def multiple_images():
    images = []
    colors = [(255, 0, 0), (0, 255, 0), (0, 0, 255)]

    for idx, color in enumerate(colors):
        img = np.zeros((30, 30, 3), dtype=np.uint8)
        cv2.circle(img, (15, 15), 10, color, -1)
        success, encoded = cv2.imencode('.jpeg', img)
        assert success

        images.append(plot_pb2.Image(
            data=encoded.tobytes(),
            format="jpeg",
            width=30,
            height=30,
            label=f"image_{idx}"
        ))

    return images


@pytest.fixture
def mock_grpc_context(mocker):
    context = mocker.Mock()
    context.set_code = mocker.Mock()
    context.set_details = mocker.Mock()
    return context
