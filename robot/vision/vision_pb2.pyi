import datetime

from google.protobuf import timestamp_pb2 as _timestamp_pb2
import common_pb2 as _common_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ImageFormat(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    IMAGE_FORMAT_UNSPECIFIED: _ClassVar[ImageFormat]
    IMAGE_FORMAT_RGB: _ClassVar[ImageFormat]
    IMAGE_FORMAT_BGR: _ClassVar[ImageFormat]
    IMAGE_FORMAT_GRAY: _ClassVar[ImageFormat]
    IMAGE_FORMAT_JPEG: _ClassVar[ImageFormat]
    IMAGE_FORMAT_PNG: _ClassVar[ImageFormat]

class CameraSource(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    CAMERA_SOURCE_UNSPECIFIED: _ClassVar[CameraSource]
    CAMERA_SOURCE_PRIMARY: _ClassVar[CameraSource]
    CAMERA_SOURCE_SECONDARY: _ClassVar[CameraSource]
    CAMERA_SOURCE_DEPTH: _ClassVar[CameraSource]

class DetectionMode(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DETECTION_MODE_UNSPECIFIED: _ClassVar[DetectionMode]
    DETECTION_MODE_FAST: _ClassVar[DetectionMode]
    DETECTION_MODE_ACCURATE: _ClassVar[DetectionMode]
    DETECTION_MODE_TRACKING: _ClassVar[DetectionMode]

class VisionDataType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    VISION_DATA_TYPE_UNSPECIFIED: _ClassVar[VisionDataType]
    VISION_DATA_TYPE_APRILTAGS: _ClassVar[VisionDataType]
    VISION_DATA_TYPE_OBJECTS: _ClassVar[VisionDataType]
    VISION_DATA_TYPE_RAW_IMAGE: _ClassVar[VisionDataType]
    VISION_DATA_TYPE_DEPTH: _ClassVar[VisionDataType]

class CoordinateFrame(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    COORDINATE_FRAME_UNSPECIFIED: _ClassVar[CoordinateFrame]
    COORDINATE_FRAME_CAMERA: _ClassVar[CoordinateFrame]
    COORDINATE_FRAME_WORLD: _ClassVar[CoordinateFrame]
    COORDINATE_FRAME_ROBOT: _ClassVar[CoordinateFrame]

IMAGE_FORMAT_UNSPECIFIED: ImageFormat
IMAGE_FORMAT_RGB: ImageFormat
IMAGE_FORMAT_BGR: ImageFormat
IMAGE_FORMAT_GRAY: ImageFormat
IMAGE_FORMAT_JPEG: ImageFormat
IMAGE_FORMAT_PNG: ImageFormat
CAMERA_SOURCE_UNSPECIFIED: CameraSource
CAMERA_SOURCE_PRIMARY: CameraSource
CAMERA_SOURCE_SECONDARY: CameraSource
CAMERA_SOURCE_DEPTH: CameraSource
DETECTION_MODE_UNSPECIFIED: DetectionMode
DETECTION_MODE_FAST: DetectionMode
DETECTION_MODE_ACCURATE: DetectionMode
DETECTION_MODE_TRACKING: DetectionMode
VISION_DATA_TYPE_UNSPECIFIED: VisionDataType
VISION_DATA_TYPE_APRILTAGS: VisionDataType
VISION_DATA_TYPE_OBJECTS: VisionDataType
VISION_DATA_TYPE_RAW_IMAGE: VisionDataType
VISION_DATA_TYPE_DEPTH: VisionDataType
COORDINATE_FRAME_UNSPECIFIED: CoordinateFrame
COORDINATE_FRAME_CAMERA: CoordinateFrame
COORDINATE_FRAME_WORLD: CoordinateFrame
COORDINATE_FRAME_ROBOT: CoordinateFrame

class AprilTagRequest(_message.Message):
    __slots__ = ("image_data", "format", "camera_id", "timestamp")
    IMAGE_DATA_FIELD_NUMBER: _ClassVar[int]
    FORMAT_FIELD_NUMBER: _ClassVar[int]
    CAMERA_ID_FIELD_NUMBER: _ClassVar[int]
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    image_data: bytes
    format: ImageFormat
    camera_id: CameraSource
    timestamp: _timestamp_pb2.Timestamp
    def __init__(
        self,
        image_data: _Optional[bytes] = ...,
        format: _Optional[_Union[ImageFormat, str]] = ...,
        camera_id: _Optional[_Union[CameraSource, str]] = ...,
        timestamp: _Optional[
            _Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]
        ] = ...,
    ) -> None: ...

class AprilTagResponse(_message.Message):
    __slots__ = ("tags", "timestamp", "frame_id", "status")
    TAGS_FIELD_NUMBER: _ClassVar[int]
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    FRAME_ID_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    tags: _containers.RepeatedCompositeFieldContainer[AprilTag]
    timestamp: _timestamp_pb2.Timestamp
    frame_id: int
    status: _common_pb2.Response
    def __init__(
        self,
        tags: _Optional[_Iterable[_Union[AprilTag, _Mapping]]] = ...,
        timestamp: _Optional[
            _Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]
        ] = ...,
        frame_id: _Optional[int] = ...,
        status: _Optional[_Union[_common_pb2.Response, _Mapping]] = ...,
    ) -> None: ...

class AprilTag(_message.Message):
    __slots__ = (
        "tag_id",
        "pose",
        "corners",
        "decision_margin",
        "hamming_distance",
        "tag_family",
        "center",
        "rotation",
        "translation",
    )
    TAG_ID_FIELD_NUMBER: _ClassVar[int]
    POSE_FIELD_NUMBER: _ClassVar[int]
    CORNERS_FIELD_NUMBER: _ClassVar[int]
    DECISION_MARGIN_FIELD_NUMBER: _ClassVar[int]
    HAMMING_DISTANCE_FIELD_NUMBER: _ClassVar[int]
    TAG_FAMILY_FIELD_NUMBER: _ClassVar[int]
    CENTER_FIELD_NUMBER: _ClassVar[int]
    ROTATION_FIELD_NUMBER: _ClassVar[int]
    TRANSLATION_FIELD_NUMBER: _ClassVar[int]
    tag_id: int
    pose: _common_pb2.Pose
    corners: _containers.RepeatedCompositeFieldContainer[Corner]
    decision_margin: float
    hamming_distance: int
    tag_family: str
    center: _common_pb2.Point3D
    rotation: RotationMatrix
    translation: TranslationVector
    def __init__(
        self,
        tag_id: _Optional[int] = ...,
        pose: _Optional[_Union[_common_pb2.Pose, _Mapping]] = ...,
        corners: _Optional[_Iterable[_Union[Corner, _Mapping]]] = ...,
        decision_margin: _Optional[float] = ...,
        hamming_distance: _Optional[int] = ...,
        tag_family: _Optional[str] = ...,
        center: _Optional[_Union[_common_pb2.Point3D, _Mapping]] = ...,
        rotation: _Optional[_Union[RotationMatrix, _Mapping]] = ...,
        translation: _Optional[_Union[TranslationVector, _Mapping]] = ...,
    ) -> None: ...

class Corner(_message.Message):
    __slots__ = ("x", "y")
    X_FIELD_NUMBER: _ClassVar[int]
    Y_FIELD_NUMBER: _ClassVar[int]
    x: float
    y: float
    def __init__(
        self, x: _Optional[float] = ..., y: _Optional[float] = ...
    ) -> None: ...

class RotationMatrix(_message.Message):
    __slots__ = ("elements",)
    ELEMENTS_FIELD_NUMBER: _ClassVar[int]
    elements: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, elements: _Optional[_Iterable[float]] = ...) -> None: ...

class TranslationVector(_message.Message):
    __slots__ = ("x", "y", "z")
    X_FIELD_NUMBER: _ClassVar[int]
    Y_FIELD_NUMBER: _ClassVar[int]
    Z_FIELD_NUMBER: _ClassVar[int]
    x: float
    y: float
    z: float
    def __init__(
        self,
        x: _Optional[float] = ...,
        y: _Optional[float] = ...,
        z: _Optional[float] = ...,
    ) -> None: ...

class ObjectDetectionRequest(_message.Message):
    __slots__ = (
        "image_data",
        "format",
        "camera_id",
        "mode",
        "confidence_threshold",
        "filter_classes",
        "timestamp",
    )
    IMAGE_DATA_FIELD_NUMBER: _ClassVar[int]
    FORMAT_FIELD_NUMBER: _ClassVar[int]
    CAMERA_ID_FIELD_NUMBER: _ClassVar[int]
    MODE_FIELD_NUMBER: _ClassVar[int]
    CONFIDENCE_THRESHOLD_FIELD_NUMBER: _ClassVar[int]
    FILTER_CLASSES_FIELD_NUMBER: _ClassVar[int]
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    image_data: bytes
    format: ImageFormat
    camera_id: CameraSource
    mode: DetectionMode
    confidence_threshold: float
    filter_classes: _containers.RepeatedScalarFieldContainer[str]
    timestamp: _timestamp_pb2.Timestamp
    def __init__(
        self,
        image_data: _Optional[bytes] = ...,
        format: _Optional[_Union[ImageFormat, str]] = ...,
        camera_id: _Optional[_Union[CameraSource, str]] = ...,
        mode: _Optional[_Union[DetectionMode, str]] = ...,
        confidence_threshold: _Optional[float] = ...,
        filter_classes: _Optional[_Iterable[str]] = ...,
        timestamp: _Optional[
            _Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]
        ] = ...,
    ) -> None: ...

class ObjectDetectionResponse(_message.Message):
    __slots__ = ("objects", "timestamp", "frame_id", "processing_time_ms", "status")
    OBJECTS_FIELD_NUMBER: _ClassVar[int]
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    FRAME_ID_FIELD_NUMBER: _ClassVar[int]
    PROCESSING_TIME_MS_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    objects: _containers.RepeatedCompositeFieldContainer[DetectedObject]
    timestamp: _timestamp_pb2.Timestamp
    frame_id: int
    processing_time_ms: int
    status: _common_pb2.Response
    def __init__(
        self,
        objects: _Optional[_Iterable[_Union[DetectedObject, _Mapping]]] = ...,
        timestamp: _Optional[
            _Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]
        ] = ...,
        frame_id: _Optional[int] = ...,
        processing_time_ms: _Optional[int] = ...,
        status: _Optional[_Union[_common_pb2.Response, _Mapping]] = ...,
    ) -> None: ...

class DetectedObject(_message.Message):
    __slots__ = (
        "object_id",
        "class_name",
        "confidence",
        "bbox",
        "world_position",
        "distance_meters",
        "velocity",
        "attributes",
    )
    class AttributesEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(
            self, key: _Optional[str] = ..., value: _Optional[str] = ...
        ) -> None: ...

    OBJECT_ID_FIELD_NUMBER: _ClassVar[int]
    CLASS_NAME_FIELD_NUMBER: _ClassVar[int]
    CONFIDENCE_FIELD_NUMBER: _ClassVar[int]
    BBOX_FIELD_NUMBER: _ClassVar[int]
    WORLD_POSITION_FIELD_NUMBER: _ClassVar[int]
    DISTANCE_METERS_FIELD_NUMBER: _ClassVar[int]
    VELOCITY_FIELD_NUMBER: _ClassVar[int]
    ATTRIBUTES_FIELD_NUMBER: _ClassVar[int]
    object_id: int
    class_name: str
    confidence: float
    bbox: _common_pb2.BoundingBox
    world_position: _common_pb2.Point3D
    distance_meters: float
    velocity: Velocity
    attributes: _containers.ScalarMap[str, str]
    def __init__(
        self,
        object_id: _Optional[int] = ...,
        class_name: _Optional[str] = ...,
        confidence: _Optional[float] = ...,
        bbox: _Optional[_Union[_common_pb2.BoundingBox, _Mapping]] = ...,
        world_position: _Optional[_Union[_common_pb2.Point3D, _Mapping]] = ...,
        distance_meters: _Optional[float] = ...,
        velocity: _Optional[_Union[Velocity, _Mapping]] = ...,
        attributes: _Optional[_Mapping[str, str]] = ...,
    ) -> None: ...

class Velocity(_message.Message):
    __slots__ = ("vx", "vy", "vz")
    VX_FIELD_NUMBER: _ClassVar[int]
    VY_FIELD_NUMBER: _ClassVar[int]
    VZ_FIELD_NUMBER: _ClassVar[int]
    vx: float
    vy: float
    vz: float
    def __init__(
        self,
        vx: _Optional[float] = ...,
        vy: _Optional[float] = ...,
        vz: _Optional[float] = ...,
    ) -> None: ...

class CalibrationRequest(_message.Message):
    __slots__ = ("camera_id",)
    CAMERA_ID_FIELD_NUMBER: _ClassVar[int]
    camera_id: CameraSource
    def __init__(
        self, camera_id: _Optional[_Union[CameraSource, str]] = ...
    ) -> None: ...

class CalibrationResponse(_message.Message):
    __slots__ = ("intrinsics", "extrinsics", "distortion", "status")
    INTRINSICS_FIELD_NUMBER: _ClassVar[int]
    EXTRINSICS_FIELD_NUMBER: _ClassVar[int]
    DISTORTION_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    intrinsics: CameraIntrinsics
    extrinsics: CameraExtrinsics
    distortion: DistortionCoefficients
    status: _common_pb2.Response
    def __init__(
        self,
        intrinsics: _Optional[_Union[CameraIntrinsics, _Mapping]] = ...,
        extrinsics: _Optional[_Union[CameraExtrinsics, _Mapping]] = ...,
        distortion: _Optional[_Union[DistortionCoefficients, _Mapping]] = ...,
        status: _Optional[_Union[_common_pb2.Response, _Mapping]] = ...,
    ) -> None: ...

class CameraIntrinsics(_message.Message):
    __slots__ = ("fx", "fy", "cx", "cy", "image_width", "image_height")
    FX_FIELD_NUMBER: _ClassVar[int]
    FY_FIELD_NUMBER: _ClassVar[int]
    CX_FIELD_NUMBER: _ClassVar[int]
    CY_FIELD_NUMBER: _ClassVar[int]
    IMAGE_WIDTH_FIELD_NUMBER: _ClassVar[int]
    IMAGE_HEIGHT_FIELD_NUMBER: _ClassVar[int]
    fx: float
    fy: float
    cx: float
    cy: float
    image_width: int
    image_height: int
    def __init__(
        self,
        fx: _Optional[float] = ...,
        fy: _Optional[float] = ...,
        cx: _Optional[float] = ...,
        cy: _Optional[float] = ...,
        image_width: _Optional[int] = ...,
        image_height: _Optional[int] = ...,
    ) -> None: ...

class CameraExtrinsics(_message.Message):
    __slots__ = ("rotation", "translation")
    ROTATION_FIELD_NUMBER: _ClassVar[int]
    TRANSLATION_FIELD_NUMBER: _ClassVar[int]
    rotation: RotationMatrix
    translation: TranslationVector
    def __init__(
        self,
        rotation: _Optional[_Union[RotationMatrix, _Mapping]] = ...,
        translation: _Optional[_Union[TranslationVector, _Mapping]] = ...,
    ) -> None: ...

class DistortionCoefficients(_message.Message):
    __slots__ = ("radial", "tangential")
    RADIAL_FIELD_NUMBER: _ClassVar[int]
    TANGENTIAL_FIELD_NUMBER: _ClassVar[int]
    radial: _containers.RepeatedScalarFieldContainer[float]
    tangential: _containers.RepeatedScalarFieldContainer[float]
    def __init__(
        self,
        radial: _Optional[_Iterable[float]] = ...,
        tangential: _Optional[_Iterable[float]] = ...,
    ) -> None: ...

class VisionStreamRequest(_message.Message):
    __slots__ = ("data_types", "fps", "camera_id")
    DATA_TYPES_FIELD_NUMBER: _ClassVar[int]
    FPS_FIELD_NUMBER: _ClassVar[int]
    CAMERA_ID_FIELD_NUMBER: _ClassVar[int]
    data_types: _containers.RepeatedScalarFieldContainer[VisionDataType]
    fps: int
    camera_id: CameraSource
    def __init__(
        self,
        data_types: _Optional[_Iterable[_Union[VisionDataType, str]]] = ...,
        fps: _Optional[int] = ...,
        camera_id: _Optional[_Union[CameraSource, str]] = ...,
    ) -> None: ...

class VisionUpdate(_message.Message):
    __slots__ = (
        "timestamp",
        "frame_id",
        "apriltag_data",
        "object_data",
        "raw_image",
        "depth_image",
    )
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    FRAME_ID_FIELD_NUMBER: _ClassVar[int]
    APRILTAG_DATA_FIELD_NUMBER: _ClassVar[int]
    OBJECT_DATA_FIELD_NUMBER: _ClassVar[int]
    RAW_IMAGE_FIELD_NUMBER: _ClassVar[int]
    DEPTH_IMAGE_FIELD_NUMBER: _ClassVar[int]
    timestamp: _timestamp_pb2.Timestamp
    frame_id: int
    apriltag_data: AprilTagResponse
    object_data: ObjectDetectionResponse
    raw_image: bytes
    depth_image: bytes
    def __init__(
        self,
        timestamp: _Optional[
            _Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]
        ] = ...,
        frame_id: _Optional[int] = ...,
        apriltag_data: _Optional[_Union[AprilTagResponse, _Mapping]] = ...,
        object_data: _Optional[_Union[ObjectDetectionResponse, _Mapping]] = ...,
        raw_image: _Optional[bytes] = ...,
        depth_image: _Optional[bytes] = ...,
    ) -> None: ...

class CoordinateTransformRequest(_message.Message):
    __slots__ = ("points", "source_frame", "target_frame", "camera_id")
    POINTS_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FRAME_FIELD_NUMBER: _ClassVar[int]
    TARGET_FRAME_FIELD_NUMBER: _ClassVar[int]
    CAMERA_ID_FIELD_NUMBER: _ClassVar[int]
    points: _containers.RepeatedCompositeFieldContainer[_common_pb2.Point3D]
    source_frame: CoordinateFrame
    target_frame: CoordinateFrame
    camera_id: CameraSource
    def __init__(
        self,
        points: _Optional[_Iterable[_Union[_common_pb2.Point3D, _Mapping]]] = ...,
        source_frame: _Optional[_Union[CoordinateFrame, str]] = ...,
        target_frame: _Optional[_Union[CoordinateFrame, str]] = ...,
        camera_id: _Optional[_Union[CameraSource, str]] = ...,
    ) -> None: ...

class CoordinateTransformResponse(_message.Message):
    __slots__ = ("transformed_points", "status")
    TRANSFORMED_POINTS_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    transformed_points: _containers.RepeatedCompositeFieldContainer[_common_pb2.Point3D]
    status: _common_pb2.Response
    def __init__(
        self,
        transformed_points: _Optional[
            _Iterable[_Union[_common_pb2.Point3D, _Mapping]]
        ] = ...,
        status: _Optional[_Union[_common_pb2.Response, _Mapping]] = ...,
    ) -> None: ...

class StatusRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class StatusResponse(_message.Message):
    __slots__ = ("component", "metrics", "cameras")
    COMPONENT_FIELD_NUMBER: _ClassVar[int]
    METRICS_FIELD_NUMBER: _ClassVar[int]
    CAMERAS_FIELD_NUMBER: _ClassVar[int]
    component: _common_pb2.ComponentInfo
    metrics: VisionMetrics
    cameras: _containers.RepeatedCompositeFieldContainer[CameraStatus]
    def __init__(
        self,
        component: _Optional[_Union[_common_pb2.ComponentInfo, _Mapping]] = ...,
        metrics: _Optional[_Union[VisionMetrics, _Mapping]] = ...,
        cameras: _Optional[_Iterable[_Union[CameraStatus, _Mapping]]] = ...,
    ) -> None: ...

class VisionMetrics(_message.Message):
    __slots__ = (
        "frames_processed",
        "average_fps",
        "tags_detected",
        "objects_detected",
        "processing_queue_size",
        "average_latency_ms",
    )
    FRAMES_PROCESSED_FIELD_NUMBER: _ClassVar[int]
    AVERAGE_FPS_FIELD_NUMBER: _ClassVar[int]
    TAGS_DETECTED_FIELD_NUMBER: _ClassVar[int]
    OBJECTS_DETECTED_FIELD_NUMBER: _ClassVar[int]
    PROCESSING_QUEUE_SIZE_FIELD_NUMBER: _ClassVar[int]
    AVERAGE_LATENCY_MS_FIELD_NUMBER: _ClassVar[int]
    frames_processed: int
    average_fps: float
    tags_detected: int
    objects_detected: int
    processing_queue_size: int
    average_latency_ms: float
    def __init__(
        self,
        frames_processed: _Optional[int] = ...,
        average_fps: _Optional[float] = ...,
        tags_detected: _Optional[int] = ...,
        objects_detected: _Optional[int] = ...,
        processing_queue_size: _Optional[int] = ...,
        average_latency_ms: _Optional[float] = ...,
    ) -> None: ...

class CameraStatus(_message.Message):
    __slots__ = (
        "camera_id",
        "connected",
        "width",
        "height",
        "current_fps",
        "error_message",
    )
    CAMERA_ID_FIELD_NUMBER: _ClassVar[int]
    CONNECTED_FIELD_NUMBER: _ClassVar[int]
    WIDTH_FIELD_NUMBER: _ClassVar[int]
    HEIGHT_FIELD_NUMBER: _ClassVar[int]
    CURRENT_FPS_FIELD_NUMBER: _ClassVar[int]
    ERROR_MESSAGE_FIELD_NUMBER: _ClassVar[int]
    camera_id: CameraSource
    connected: bool
    width: int
    height: int
    current_fps: float
    error_message: str
    def __init__(
        self,
        camera_id: _Optional[_Union[CameraSource, str]] = ...,
        connected: bool = ...,
        width: _Optional[int] = ...,
        height: _Optional[int] = ...,
        current_fps: _Optional[float] = ...,
        error_message: _Optional[str] = ...,
    ) -> None: ...
