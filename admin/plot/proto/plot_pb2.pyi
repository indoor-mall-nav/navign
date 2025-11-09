from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Image(_message.Message):
    __slots__ = ("data", "format", "width", "height", "label")
    DATA_FIELD_NUMBER: _ClassVar[int]
    FORMAT_FIELD_NUMBER: _ClassVar[int]
    WIDTH_FIELD_NUMBER: _ClassVar[int]
    HEIGHT_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    data: bytes
    format: str
    width: int
    height: int
    label: str
    def __init__(self, data: _Optional[bytes] = ..., format: _Optional[str] = ..., width: _Optional[int] = ..., height: _Optional[int] = ..., label: _Optional[str] = ...) -> None: ...

class PlotRequest(_message.Message):
    __slots__ = ("plot_id", "images", "description")
    PLOT_ID_FIELD_NUMBER: _ClassVar[int]
    IMAGES_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    plot_id: str
    images: _containers.RepeatedCompositeFieldContainer[Image]
    description: str
    def __init__(self, plot_id: _Optional[str] = ..., images: _Optional[_Iterable[_Union[Image, _Mapping]]] = ..., description: _Optional[str] = ...) -> None: ...

class Point(_message.Message):
    __slots__ = ("x", "y")
    X_FIELD_NUMBER: _ClassVar[int]
    Y_FIELD_NUMBER: _ClassVar[int]
    x: float
    y: float
    def __init__(self, x: _Optional[float] = ..., y: _Optional[float] = ...) -> None: ...

class Polygon(_message.Message):
    __slots__ = ("vertices", "label", "area", "centroid")
    VERTICES_FIELD_NUMBER: _ClassVar[int]
    LABEL_FIELD_NUMBER: _ClassVar[int]
    AREA_FIELD_NUMBER: _ClassVar[int]
    CENTROID_FIELD_NUMBER: _ClassVar[int]
    vertices: _containers.RepeatedCompositeFieldContainer[Point]
    label: str
    area: float
    centroid: Point
    def __init__(self, vertices: _Optional[_Iterable[_Union[Point, _Mapping]]] = ..., label: _Optional[str] = ..., area: _Optional[float] = ..., centroid: _Optional[_Union[Point, _Mapping]] = ...) -> None: ...

class ExtractionConfig(_message.Message):
    __slots__ = ("blur_kernel_size", "threshold_value", "threshold_type", "min_area", "max_area", "epsilon_factor", "apply_morphology", "morph_kernel_size", "use_canny", "canny_low", "canny_high")
    BLUR_KERNEL_SIZE_FIELD_NUMBER: _ClassVar[int]
    THRESHOLD_VALUE_FIELD_NUMBER: _ClassVar[int]
    THRESHOLD_TYPE_FIELD_NUMBER: _ClassVar[int]
    MIN_AREA_FIELD_NUMBER: _ClassVar[int]
    MAX_AREA_FIELD_NUMBER: _ClassVar[int]
    EPSILON_FACTOR_FIELD_NUMBER: _ClassVar[int]
    APPLY_MORPHOLOGY_FIELD_NUMBER: _ClassVar[int]
    MORPH_KERNEL_SIZE_FIELD_NUMBER: _ClassVar[int]
    USE_CANNY_FIELD_NUMBER: _ClassVar[int]
    CANNY_LOW_FIELD_NUMBER: _ClassVar[int]
    CANNY_HIGH_FIELD_NUMBER: _ClassVar[int]
    blur_kernel_size: float
    threshold_value: int
    threshold_type: int
    min_area: float
    max_area: float
    epsilon_factor: float
    apply_morphology: bool
    morph_kernel_size: int
    use_canny: bool
    canny_low: int
    canny_high: int
    def __init__(self, blur_kernel_size: _Optional[float] = ..., threshold_value: _Optional[int] = ..., threshold_type: _Optional[int] = ..., min_area: _Optional[float] = ..., max_area: _Optional[float] = ..., epsilon_factor: _Optional[float] = ..., apply_morphology: bool = ..., morph_kernel_size: _Optional[int] = ..., use_canny: bool = ..., canny_low: _Optional[int] = ..., canny_high: _Optional[int] = ...) -> None: ...

class ExtractPolygonsRequest(_message.Message):
    __slots__ = ("entity_id", "floor_id", "floor_plan", "config")
    ENTITY_ID_FIELD_NUMBER: _ClassVar[int]
    FLOOR_ID_FIELD_NUMBER: _ClassVar[int]
    FLOOR_PLAN_FIELD_NUMBER: _ClassVar[int]
    CONFIG_FIELD_NUMBER: _ClassVar[int]
    entity_id: str
    floor_id: str
    floor_plan: Image
    config: ExtractionConfig
    def __init__(self, entity_id: _Optional[str] = ..., floor_id: _Optional[str] = ..., floor_plan: _Optional[_Union[Image, _Mapping]] = ..., config: _Optional[_Union[ExtractionConfig, _Mapping]] = ...) -> None: ...

class ExtractPolygonsResponse(_message.Message):
    __slots__ = ("polygons", "total_count", "error", "stats")
    POLYGONS_FIELD_NUMBER: _ClassVar[int]
    TOTAL_COUNT_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    STATS_FIELD_NUMBER: _ClassVar[int]
    polygons: _containers.RepeatedCompositeFieldContainer[Polygon]
    total_count: int
    error: str
    stats: ProcessingStats
    def __init__(self, polygons: _Optional[_Iterable[_Union[Polygon, _Mapping]]] = ..., total_count: _Optional[int] = ..., error: _Optional[str] = ..., stats: _Optional[_Union[ProcessingStats, _Mapping]] = ...) -> None: ...

class ProcessingStats(_message.Message):
    __slots__ = ("contours_found", "contours_filtered", "processing_time_ms", "image_width", "image_height")
    CONTOURS_FOUND_FIELD_NUMBER: _ClassVar[int]
    CONTOURS_FILTERED_FIELD_NUMBER: _ClassVar[int]
    PROCESSING_TIME_MS_FIELD_NUMBER: _ClassVar[int]
    IMAGE_WIDTH_FIELD_NUMBER: _ClassVar[int]
    IMAGE_HEIGHT_FIELD_NUMBER: _ClassVar[int]
    contours_found: int
    contours_filtered: int
    processing_time_ms: float
    image_width: int
    image_height: int
    def __init__(self, contours_found: _Optional[int] = ..., contours_filtered: _Optional[int] = ..., processing_time_ms: _Optional[float] = ..., image_width: _Optional[int] = ..., image_height: _Optional[int] = ...) -> None: ...

class BatchExtractRequest(_message.Message):
    __slots__ = ("entity_id", "floor_plans", "config")
    ENTITY_ID_FIELD_NUMBER: _ClassVar[int]
    FLOOR_PLANS_FIELD_NUMBER: _ClassVar[int]
    CONFIG_FIELD_NUMBER: _ClassVar[int]
    entity_id: str
    floor_plans: _containers.RepeatedCompositeFieldContainer[FloorPlanInput]
    config: ExtractionConfig
    def __init__(self, entity_id: _Optional[str] = ..., floor_plans: _Optional[_Iterable[_Union[FloorPlanInput, _Mapping]]] = ..., config: _Optional[_Union[ExtractionConfig, _Mapping]] = ...) -> None: ...

class FloorPlanInput(_message.Message):
    __slots__ = ("floor_id", "floor_plan")
    FLOOR_ID_FIELD_NUMBER: _ClassVar[int]
    FLOOR_PLAN_FIELD_NUMBER: _ClassVar[int]
    floor_id: str
    floor_plan: Image
    def __init__(self, floor_id: _Optional[str] = ..., floor_plan: _Optional[_Union[Image, _Mapping]] = ...) -> None: ...

class BatchExtractResponse(_message.Message):
    __slots__ = ("extractions", "successful", "failed")
    EXTRACTIONS_FIELD_NUMBER: _ClassVar[int]
    SUCCESSFUL_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    extractions: _containers.RepeatedCompositeFieldContainer[FloorExtraction]
    successful: int
    failed: int
    def __init__(self, extractions: _Optional[_Iterable[_Union[FloorExtraction, _Mapping]]] = ..., successful: _Optional[int] = ..., failed: _Optional[int] = ...) -> None: ...

class FloorExtraction(_message.Message):
    __slots__ = ("floor_id", "polygons", "error", "stats")
    FLOOR_ID_FIELD_NUMBER: _ClassVar[int]
    POLYGONS_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    STATS_FIELD_NUMBER: _ClassVar[int]
    floor_id: str
    polygons: _containers.RepeatedCompositeFieldContainer[Polygon]
    error: str
    stats: ProcessingStats
    def __init__(self, floor_id: _Optional[str] = ..., polygons: _Optional[_Iterable[_Union[Polygon, _Mapping]]] = ..., error: _Optional[str] = ..., stats: _Optional[_Union[ProcessingStats, _Mapping]] = ...) -> None: ...

class HealthCheckRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class HealthCheckResponse(_message.Message):
    __slots__ = ("healthy", "version", "message")
    HEALTHY_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    healthy: bool
    version: str
    message: str
    def __init__(self, healthy: bool = ..., version: _Optional[str] = ..., message: _Optional[str] = ...) -> None: ...
