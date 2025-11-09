import pytest
from proto import plot_pb2


@pytest.mark.unit
class TestImageMessage:
    def test_create_image_message(self):
        img = plot_pb2.Image(
            data=b"fake_image_data",
            format="png",
            width=100,
            height=100,
            label="test"
        )

        assert img.data == b"fake_image_data"
        assert img.format == "png"
        assert img.width == 100
        assert img.height == 100
        assert img.label == "test"

    def test_image_serialization(self, sample_plot_image):
        serialized = sample_plot_image.SerializeToString()
        assert isinstance(serialized, bytes)
        assert len(serialized) > 0

        deserialized = plot_pb2.Image()
        deserialized.ParseFromString(serialized)

        assert deserialized.format == sample_plot_image.format
        assert deserialized.width == sample_plot_image.width
        assert deserialized.height == sample_plot_image.height
        assert deserialized.label == sample_plot_image.label

    def test_image_with_empty_data(self):
        img = plot_pb2.Image(
            data=b"",
            format="png",
            width=0,
            height=0,
            label=""
        )
        assert img.data == b""
        assert img.width == 0
        assert img.height == 0


@pytest.mark.unit
class TestPlotRequestMessage:
    def test_create_plot_request(self):
        request = plot_pb2.PlotRequest(
            plot_id="plot_001",
            images=[],
            description="Test description"
        )

        assert request.plot_id == "plot_001"
        assert len(request.images) == 0
        assert request.description == "Test description"

    def test_plot_request_with_multiple_images(self, multiple_images):
        request = plot_pb2.PlotRequest(
            plot_id="plot_multi",
            images=multiple_images,
            description="Multiple images test"
        )

        assert len(request.images) == 3
        assert request.images[0].label == "image_0"
        assert request.images[1].label == "image_1"
        assert request.images[2].label == "image_2"

    def test_plot_request_serialization(self, sample_plot_request):
        serialized = sample_plot_request.SerializeToString()
        assert isinstance(serialized, bytes)

        deserialized = plot_pb2.PlotRequest()
        deserialized.ParseFromString(serialized)

        assert deserialized.plot_id == sample_plot_request.plot_id
        assert deserialized.description == sample_plot_request.description
        assert len(deserialized.images) == len(sample_plot_request.images)

    def test_add_image_to_request(self, sample_plot_request):
        initial_count = len(sample_plot_request.images)

        new_image = plot_pb2.Image(
            data=b"additional_data",
            format="jpeg",
            width=200,
            height=200,
            label="additional"
        )
        sample_plot_request.images.append(new_image)

        assert len(sample_plot_request.images) == initial_count + 1
        assert sample_plot_request.images[-1].label == "additional"

    def test_empty_plot_request(self):
        request = plot_pb2.PlotRequest()
        assert request.plot_id == ""
        assert len(request.images) == 0
        assert request.description == ""


@pytest.mark.unit
class TestImageDecoding:
    def test_decode_png_image(self, sample_plot_image):
        import cv2
        import numpy as np

        img_array = np.frombuffer(sample_plot_image.data, dtype=np.uint8)
        decoded = cv2.imdecode(img_array, cv2.IMREAD_COLOR)

        assert decoded is not None
        assert decoded.shape[0] == sample_plot_image.height
        assert decoded.shape[1] == sample_plot_image.width
        assert decoded.shape[2] == 3

    def test_decode_multiple_formats(self):
        import cv2
        import numpy as np

        formats = ['png', 'jpeg']
        for fmt in formats:
            img = np.zeros((10, 10, 3), dtype=np.uint8)
            success, encoded = cv2.imencode(f'.{fmt}', img)
            assert success

            proto_img = plot_pb2.Image(
                data=encoded.tobytes(),
                format=fmt,
                width=10,
                height=10,
                label=f"test_{fmt}"
            )

            img_array = np.frombuffer(proto_img.data, dtype=np.uint8)
            decoded = cv2.imdecode(img_array, cv2.IMREAD_COLOR)
            assert decoded is not None
