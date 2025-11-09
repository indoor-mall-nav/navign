from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class TaskType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    TASK_TYPE_UNSPECIFIED: _ClassVar[TaskType]
    TASK_TYPE_DELIVERY: _ClassVar[TaskType]
    TASK_TYPE_PATROL: _ClassVar[TaskType]
    TASK_TYPE_RETURN_HOME: _ClassVar[TaskType]
    TASK_TYPE_EMERGENCY: _ClassVar[TaskType]

class Priority(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PRIORITY_UNSPECIFIED: _ClassVar[Priority]
    PRIORITY_LOW: _ClassVar[Priority]
    PRIORITY_NORMAL: _ClassVar[Priority]
    PRIORITY_HIGH: _ClassVar[Priority]
    PRIORITY_URGENT: _ClassVar[Priority]

class RobotState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    ROBOT_STATE_UNSPECIFIED: _ClassVar[RobotState]
    ROBOT_STATE_IDLE: _ClassVar[RobotState]
    ROBOT_STATE_BUSY: _ClassVar[RobotState]
    ROBOT_STATE_CHARGING: _ClassVar[RobotState]
    ROBOT_STATE_ERROR: _ClassVar[RobotState]
    ROBOT_STATE_OFFLINE: _ClassVar[RobotState]
TASK_TYPE_UNSPECIFIED: TaskType
TASK_TYPE_DELIVERY: TaskType
TASK_TYPE_PATROL: TaskType
TASK_TYPE_RETURN_HOME: TaskType
TASK_TYPE_EMERGENCY: TaskType
PRIORITY_UNSPECIFIED: Priority
PRIORITY_LOW: Priority
PRIORITY_NORMAL: Priority
PRIORITY_HIGH: Priority
PRIORITY_URGENT: Priority
ROBOT_STATE_UNSPECIFIED: RobotState
ROBOT_STATE_IDLE: RobotState
ROBOT_STATE_BUSY: RobotState
ROBOT_STATE_CHARGING: RobotState
ROBOT_STATE_ERROR: RobotState
ROBOT_STATE_OFFLINE: RobotState

class Location(_message.Message):
    __slots__ = ("x", "y", "z", "floor")
    X_FIELD_NUMBER: _ClassVar[int]
    Y_FIELD_NUMBER: _ClassVar[int]
    Z_FIELD_NUMBER: _ClassVar[int]
    FLOOR_FIELD_NUMBER: _ClassVar[int]
    x: float
    y: float
    z: float
    floor: str
    def __init__(self, x: _Optional[float] = ..., y: _Optional[float] = ..., z: _Optional[float] = ..., floor: _Optional[str] = ...) -> None: ...

class Task(_message.Message):
    __slots__ = ("id", "type", "sources", "terminals", "priority", "created_at", "entity_id", "metadata")
    class MetadataEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    ID_FIELD_NUMBER: _ClassVar[int]
    TYPE_FIELD_NUMBER: _ClassVar[int]
    SOURCES_FIELD_NUMBER: _ClassVar[int]
    TERMINALS_FIELD_NUMBER: _ClassVar[int]
    PRIORITY_FIELD_NUMBER: _ClassVar[int]
    CREATED_AT_FIELD_NUMBER: _ClassVar[int]
    ENTITY_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    id: str
    type: TaskType
    sources: _containers.RepeatedCompositeFieldContainer[Location]
    terminals: _containers.RepeatedCompositeFieldContainer[Location]
    priority: Priority
    created_at: int
    entity_id: str
    metadata: _containers.ScalarMap[str, str]
    def __init__(self, id: _Optional[str] = ..., type: _Optional[_Union[TaskType, str]] = ..., sources: _Optional[_Iterable[_Union[Location, _Mapping]]] = ..., terminals: _Optional[_Iterable[_Union[Location, _Mapping]]] = ..., priority: _Optional[_Union[Priority, str]] = ..., created_at: _Optional[int] = ..., entity_id: _Optional[str] = ..., metadata: _Optional[_Mapping[str, str]] = ...) -> None: ...

class TaskRequest(_message.Message):
    __slots__ = ("task",)
    TASK_FIELD_NUMBER: _ClassVar[int]
    task: Task
    def __init__(self, task: _Optional[_Union[Task, _Mapping]] = ...) -> None: ...

class TaskResponse(_message.Message):
    __slots__ = ("accepted", "robot_id", "message", "estimated_completion_time")
    ACCEPTED_FIELD_NUMBER: _ClassVar[int]
    ROBOT_ID_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    ESTIMATED_COMPLETION_TIME_FIELD_NUMBER: _ClassVar[int]
    accepted: bool
    robot_id: str
    message: str
    estimated_completion_time: int
    def __init__(self, accepted: bool = ..., robot_id: _Optional[str] = ..., message: _Optional[str] = ..., estimated_completion_time: _Optional[int] = ...) -> None: ...

class RobotInfo(_message.Message):
    __slots__ = ("id", "name", "state", "current_location", "battery_level", "current_task_id", "last_seen", "entity_id")
    ID_FIELD_NUMBER: _ClassVar[int]
    NAME_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    CURRENT_LOCATION_FIELD_NUMBER: _ClassVar[int]
    BATTERY_LEVEL_FIELD_NUMBER: _ClassVar[int]
    CURRENT_TASK_ID_FIELD_NUMBER: _ClassVar[int]
    LAST_SEEN_FIELD_NUMBER: _ClassVar[int]
    ENTITY_ID_FIELD_NUMBER: _ClassVar[int]
    id: str
    name: str
    state: RobotState
    current_location: Location
    battery_level: float
    current_task_id: str
    last_seen: int
    entity_id: str
    def __init__(self, id: _Optional[str] = ..., name: _Optional[str] = ..., state: _Optional[_Union[RobotState, str]] = ..., current_location: _Optional[_Union[Location, _Mapping]] = ..., battery_level: _Optional[float] = ..., current_task_id: _Optional[str] = ..., last_seen: _Optional[int] = ..., entity_id: _Optional[str] = ...) -> None: ...

class RobotDistributionRequest(_message.Message):
    __slots__ = ("entity_id",)
    ENTITY_ID_FIELD_NUMBER: _ClassVar[int]
    entity_id: str
    def __init__(self, entity_id: _Optional[str] = ...) -> None: ...

class RobotDistributionResponse(_message.Message):
    __slots__ = ("robots", "total_count", "idle_count", "busy_count")
    ROBOTS_FIELD_NUMBER: _ClassVar[int]
    TOTAL_COUNT_FIELD_NUMBER: _ClassVar[int]
    IDLE_COUNT_FIELD_NUMBER: _ClassVar[int]
    BUSY_COUNT_FIELD_NUMBER: _ClassVar[int]
    robots: _containers.RepeatedCompositeFieldContainer[RobotInfo]
    total_count: int
    idle_count: int
    busy_count: int
    def __init__(self, robots: _Optional[_Iterable[_Union[RobotInfo, _Mapping]]] = ..., total_count: _Optional[int] = ..., idle_count: _Optional[int] = ..., busy_count: _Optional[int] = ...) -> None: ...

class RobotReportRequest(_message.Message):
    __slots__ = ("robot",)
    ROBOT_FIELD_NUMBER: _ClassVar[int]
    robot: RobotInfo
    def __init__(self, robot: _Optional[_Union[RobotInfo, _Mapping]] = ...) -> None: ...

class RobotReportResponse(_message.Message):
    __slots__ = ("success", "message")
    SUCCESS_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    success: bool
    message: str
    def __init__(self, success: bool = ..., message: _Optional[str] = ...) -> None: ...

class TaskAssignment(_message.Message):
    __slots__ = ("robot_id", "task")
    ROBOT_ID_FIELD_NUMBER: _ClassVar[int]
    TASK_FIELD_NUMBER: _ClassVar[int]
    robot_id: str
    task: Task
    def __init__(self, robot_id: _Optional[str] = ..., task: _Optional[_Union[Task, _Mapping]] = ...) -> None: ...

class TaskAssignmentResponse(_message.Message):
    __slots__ = ("accepted", "message")
    ACCEPTED_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    accepted: bool
    message: str
    def __init__(self, accepted: bool = ..., message: _Optional[str] = ...) -> None: ...

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

class FloorPlanImage(_message.Message):
    __slots__ = ("data", "format", "width", "height")
    DATA_FIELD_NUMBER: _ClassVar[int]
    FORMAT_FIELD_NUMBER: _ClassVar[int]
    WIDTH_FIELD_NUMBER: _ClassVar[int]
    HEIGHT_FIELD_NUMBER: _ClassVar[int]
    data: bytes
    format: str
    width: int
    height: int
    def __init__(self, data: _Optional[bytes] = ..., format: _Optional[str] = ..., width: _Optional[int] = ..., height: _Optional[int] = ...) -> None: ...

class PlotExtractionConfig(_message.Message):
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
    floor_plan: FloorPlanImage
    config: PlotExtractionConfig
    def __init__(self, entity_id: _Optional[str] = ..., floor_id: _Optional[str] = ..., floor_plan: _Optional[_Union[FloorPlanImage, _Mapping]] = ..., config: _Optional[_Union[PlotExtractionConfig, _Mapping]] = ...) -> None: ...

class ExtractPolygonsResponse(_message.Message):
    __slots__ = ("polygons", "total_count", "error", "stats")
    POLYGONS_FIELD_NUMBER: _ClassVar[int]
    TOTAL_COUNT_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    STATS_FIELD_NUMBER: _ClassVar[int]
    polygons: _containers.RepeatedCompositeFieldContainer[Polygon]
    total_count: int
    error: str
    stats: PlotProcessingStats
    def __init__(self, polygons: _Optional[_Iterable[_Union[Polygon, _Mapping]]] = ..., total_count: _Optional[int] = ..., error: _Optional[str] = ..., stats: _Optional[_Union[PlotProcessingStats, _Mapping]] = ...) -> None: ...

class PlotProcessingStats(_message.Message):
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
    floor_plans: _containers.RepeatedCompositeFieldContainer[FloorPlanBatch]
    config: PlotExtractionConfig
    def __init__(self, entity_id: _Optional[str] = ..., floor_plans: _Optional[_Iterable[_Union[FloorPlanBatch, _Mapping]]] = ..., config: _Optional[_Union[PlotExtractionConfig, _Mapping]] = ...) -> None: ...

class FloorPlanBatch(_message.Message):
    __slots__ = ("floor_id", "floor_plan")
    FLOOR_ID_FIELD_NUMBER: _ClassVar[int]
    FLOOR_PLAN_FIELD_NUMBER: _ClassVar[int]
    floor_id: str
    floor_plan: FloorPlanImage
    def __init__(self, floor_id: _Optional[str] = ..., floor_plan: _Optional[_Union[FloorPlanImage, _Mapping]] = ...) -> None: ...

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
    stats: PlotProcessingStats
    def __init__(self, floor_id: _Optional[str] = ..., polygons: _Optional[_Iterable[_Union[Polygon, _Mapping]]] = ..., error: _Optional[str] = ..., stats: _Optional[_Union[PlotProcessingStats, _Mapping]] = ...) -> None: ...
