import datetime

from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ComponentStatus(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    COMPONENT_STATUS_UNSPECIFIED: _ClassVar[ComponentStatus]
    COMPONENT_STATUS_INITIALIZING: _ClassVar[ComponentStatus]
    COMPONENT_STATUS_READY: _ClassVar[ComponentStatus]
    COMPONENT_STATUS_BUSY: _ClassVar[ComponentStatus]
    COMPONENT_STATUS_ERROR: _ClassVar[ComponentStatus]
    COMPONENT_STATUS_OFFLINE: _ClassVar[ComponentStatus]

class ComponentType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    COMPONENT_TYPE_UNSPECIFIED: _ClassVar[ComponentType]
    COMPONENT_TYPE_VISION: _ClassVar[ComponentType]
    COMPONENT_TYPE_AUDIO: _ClassVar[ComponentType]
    COMPONENT_TYPE_SCHEDULER: _ClassVar[ComponentType]
    COMPONENT_TYPE_SERIAL: _ClassVar[ComponentType]
    COMPONENT_TYPE_NETWORK: _ClassVar[ComponentType]
    COMPONENT_TYPE_INTELLIGENCE: _ClassVar[ComponentType]

COMPONENT_STATUS_UNSPECIFIED: ComponentStatus
COMPONENT_STATUS_INITIALIZING: ComponentStatus
COMPONENT_STATUS_READY: ComponentStatus
COMPONENT_STATUS_BUSY: ComponentStatus
COMPONENT_STATUS_ERROR: ComponentStatus
COMPONENT_STATUS_OFFLINE: ComponentStatus
COMPONENT_TYPE_UNSPECIFIED: ComponentType
COMPONENT_TYPE_VISION: ComponentType
COMPONENT_TYPE_AUDIO: ComponentType
COMPONENT_TYPE_SCHEDULER: ComponentType
COMPONENT_TYPE_SERIAL: ComponentType
COMPONENT_TYPE_NETWORK: ComponentType
COMPONENT_TYPE_INTELLIGENCE: ComponentType

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
    def __init__(
        self,
        x: _Optional[float] = ...,
        y: _Optional[float] = ...,
        z: _Optional[float] = ...,
        floor: _Optional[str] = ...,
    ) -> None: ...

class Pose(_message.Message):
    __slots__ = ("position", "orientation")
    POSITION_FIELD_NUMBER: _ClassVar[int]
    ORIENTATION_FIELD_NUMBER: _ClassVar[int]
    position: Location
    orientation: Quaternion
    def __init__(
        self,
        position: _Optional[_Union[Location, _Mapping]] = ...,
        orientation: _Optional[_Union[Quaternion, _Mapping]] = ...,
    ) -> None: ...

class Quaternion(_message.Message):
    __slots__ = ("x", "y", "z", "w")
    X_FIELD_NUMBER: _ClassVar[int]
    Y_FIELD_NUMBER: _ClassVar[int]
    Z_FIELD_NUMBER: _ClassVar[int]
    W_FIELD_NUMBER: _ClassVar[int]
    x: float
    y: float
    z: float
    w: float
    def __init__(
        self,
        x: _Optional[float] = ...,
        y: _Optional[float] = ...,
        z: _Optional[float] = ...,
        w: _Optional[float] = ...,
    ) -> None: ...

class BoundingBox(_message.Message):
    __slots__ = ("x_min", "y_min", "x_max", "y_max", "confidence")
    X_MIN_FIELD_NUMBER: _ClassVar[int]
    Y_MIN_FIELD_NUMBER: _ClassVar[int]
    X_MAX_FIELD_NUMBER: _ClassVar[int]
    Y_MAX_FIELD_NUMBER: _ClassVar[int]
    CONFIDENCE_FIELD_NUMBER: _ClassVar[int]
    x_min: float
    y_min: float
    x_max: float
    y_max: float
    confidence: float
    def __init__(
        self,
        x_min: _Optional[float] = ...,
        y_min: _Optional[float] = ...,
        x_max: _Optional[float] = ...,
        y_max: _Optional[float] = ...,
        confidence: _Optional[float] = ...,
    ) -> None: ...

class Point3D(_message.Message):
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

class ComponentInfo(_message.Message):
    __slots__ = ("component_id", "type", "status", "timestamp", "metadata")
    class MetadataEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(
            self, key: _Optional[str] = ..., value: _Optional[str] = ...
        ) -> None: ...

    COMPONENT_ID_FIELD_NUMBER: _ClassVar[int]
    TYPE_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    component_id: str
    type: ComponentType
    status: ComponentStatus
    timestamp: _timestamp_pb2.Timestamp
    metadata: _containers.ScalarMap[str, str]
    def __init__(
        self,
        component_id: _Optional[str] = ...,
        type: _Optional[_Union[ComponentType, str]] = ...,
        status: _Optional[_Union[ComponentStatus, str]] = ...,
        timestamp: _Optional[
            _Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]
        ] = ...,
        metadata: _Optional[_Mapping[str, str]] = ...,
    ) -> None: ...

class ErrorInfo(_message.Message):
    __slots__ = ("code", "message", "component", "timestamp", "details")
    class DetailsEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(
            self, key: _Optional[str] = ..., value: _Optional[str] = ...
        ) -> None: ...

    CODE_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    COMPONENT_FIELD_NUMBER: _ClassVar[int]
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    DETAILS_FIELD_NUMBER: _ClassVar[int]
    code: str
    message: str
    component: str
    timestamp: _timestamp_pb2.Timestamp
    details: _containers.ScalarMap[str, str]
    def __init__(
        self,
        code: _Optional[str] = ...,
        message: _Optional[str] = ...,
        component: _Optional[str] = ...,
        timestamp: _Optional[
            _Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]
        ] = ...,
        details: _Optional[_Mapping[str, str]] = ...,
    ) -> None: ...

class Response(_message.Message):
    __slots__ = ("success", "message", "error")
    SUCCESS_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    success: bool
    message: str
    error: ErrorInfo
    def __init__(
        self,
        success: bool = ...,
        message: _Optional[str] = ...,
        error: _Optional[_Union[ErrorInfo, _Mapping]] = ...,
    ) -> None: ...
