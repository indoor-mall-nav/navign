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

class WakeWordEngine(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    WAKE_WORD_ENGINE_UNSPECIFIED: _ClassVar[WakeWordEngine]
    WAKE_WORD_ENGINE_PORCUPINE: _ClassVar[WakeWordEngine]
    WAKE_WORD_ENGINE_OPENWAKEWORD: _ClassVar[WakeWordEngine]

class Priority(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    PRIORITY_UNSPECIFIED: _ClassVar[Priority]
    PRIORITY_LOW: _ClassVar[Priority]
    PRIORITY_NORMAL: _ClassVar[Priority]
    PRIORITY_HIGH: _ClassVar[Priority]
    PRIORITY_URGENT: _ClassVar[Priority]

class TTSEngine(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    TTS_ENGINE_UNSPECIFIED: _ClassVar[TTSEngine]
    TTS_ENGINE_EDGE_TTS: _ClassVar[TTSEngine]
    TTS_ENGINE_PYTTSX3: _ClassVar[TTSEngine]
    TTS_ENGINE_ESPEAK: _ClassVar[TTSEngine]

class TTSStatus(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    TTS_STATUS_UNSPECIFIED: _ClassVar[TTSStatus]
    TTS_STATUS_QUEUED: _ClassVar[TTSStatus]
    TTS_STATUS_PLAYING: _ClassVar[TTSStatus]
    TTS_STATUS_COMPLETED: _ClassVar[TTSStatus]
    TTS_STATUS_INTERRUPTED: _ClassVar[TTSStatus]
    TTS_STATUS_ERROR: _ClassVar[TTSStatus]

class AudioEventType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    AUDIO_EVENT_TYPE_UNSPECIFIED: _ClassVar[AudioEventType]
    AUDIO_EVENT_TYPE_WAKE_WORD: _ClassVar[AudioEventType]
    AUDIO_EVENT_TYPE_SPEECH_STARTED: _ClassVar[AudioEventType]
    AUDIO_EVENT_TYPE_SPEECH_COMPLETED: _ClassVar[AudioEventType]
    AUDIO_EVENT_TYPE_ERROR: _ClassVar[AudioEventType]

WAKE_WORD_ENGINE_UNSPECIFIED: WakeWordEngine
WAKE_WORD_ENGINE_PORCUPINE: WakeWordEngine
WAKE_WORD_ENGINE_OPENWAKEWORD: WakeWordEngine
PRIORITY_UNSPECIFIED: Priority
PRIORITY_LOW: Priority
PRIORITY_NORMAL: Priority
PRIORITY_HIGH: Priority
PRIORITY_URGENT: Priority
TTS_ENGINE_UNSPECIFIED: TTSEngine
TTS_ENGINE_EDGE_TTS: TTSEngine
TTS_ENGINE_PYTTSX3: TTSEngine
TTS_ENGINE_ESPEAK: TTSEngine
TTS_STATUS_UNSPECIFIED: TTSStatus
TTS_STATUS_QUEUED: TTSStatus
TTS_STATUS_PLAYING: TTSStatus
TTS_STATUS_COMPLETED: TTSStatus
TTS_STATUS_INTERRUPTED: TTSStatus
TTS_STATUS_ERROR: TTSStatus
AUDIO_EVENT_TYPE_UNSPECIFIED: AudioEventType
AUDIO_EVENT_TYPE_WAKE_WORD: AudioEventType
AUDIO_EVENT_TYPE_SPEECH_STARTED: AudioEventType
AUDIO_EVENT_TYPE_SPEECH_COMPLETED: AudioEventType
AUDIO_EVENT_TYPE_ERROR: AudioEventType

class WakeWordConfig(_message.Message):
    __slots__ = ("wake_word", "sensitivity", "engine", "engine_config")
    class EngineConfigEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(
            self, key: _Optional[str] = ..., value: _Optional[str] = ...
        ) -> None: ...

    WAKE_WORD_FIELD_NUMBER: _ClassVar[int]
    SENSITIVITY_FIELD_NUMBER: _ClassVar[int]
    ENGINE_FIELD_NUMBER: _ClassVar[int]
    ENGINE_CONFIG_FIELD_NUMBER: _ClassVar[int]
    wake_word: str
    sensitivity: float
    engine: WakeWordEngine
    engine_config: _containers.ScalarMap[str, str]
    def __init__(
        self,
        wake_word: _Optional[str] = ...,
        sensitivity: _Optional[float] = ...,
        engine: _Optional[_Union[WakeWordEngine, str]] = ...,
        engine_config: _Optional[_Mapping[str, str]] = ...,
    ) -> None: ...

class WakeWordResponse(_message.Message):
    __slots__ = ("success", "message", "active_wake_word", "engine")
    SUCCESS_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    ACTIVE_WAKE_WORD_FIELD_NUMBER: _ClassVar[int]
    ENGINE_FIELD_NUMBER: _ClassVar[int]
    success: bool
    message: str
    active_wake_word: str
    engine: WakeWordEngine
    def __init__(
        self,
        success: bool = ...,
        message: _Optional[str] = ...,
        active_wake_word: _Optional[str] = ...,
        engine: _Optional[_Union[WakeWordEngine, str]] = ...,
    ) -> None: ...

class WakeWordDetectedEvent(_message.Message):
    __slots__ = ("wake_word", "confidence", "timestamp", "audio_frame_index")
    WAKE_WORD_FIELD_NUMBER: _ClassVar[int]
    CONFIDENCE_FIELD_NUMBER: _ClassVar[int]
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    AUDIO_FRAME_INDEX_FIELD_NUMBER: _ClassVar[int]
    wake_word: str
    confidence: float
    timestamp: _timestamp_pb2.Timestamp
    audio_frame_index: int
    def __init__(
        self,
        wake_word: _Optional[str] = ...,
        confidence: _Optional[float] = ...,
        timestamp: _Optional[
            _Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]
        ] = ...,
        audio_frame_index: _Optional[int] = ...,
    ) -> None: ...

class StopRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SpeakRequest(_message.Message):
    __slots__ = ("text", "config", "priority", "interrupt_current", "request_id")
    TEXT_FIELD_NUMBER: _ClassVar[int]
    CONFIG_FIELD_NUMBER: _ClassVar[int]
    PRIORITY_FIELD_NUMBER: _ClassVar[int]
    INTERRUPT_CURRENT_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    text: str
    config: TTSConfig
    priority: Priority
    interrupt_current: bool
    request_id: str
    def __init__(
        self,
        text: _Optional[str] = ...,
        config: _Optional[_Union[TTSConfig, _Mapping]] = ...,
        priority: _Optional[_Union[Priority, str]] = ...,
        interrupt_current: bool = ...,
        request_id: _Optional[str] = ...,
    ) -> None: ...

class TTSConfig(_message.Message):
    __slots__ = ("voice", "language", "rate", "volume", "pitch", "engine")
    VOICE_FIELD_NUMBER: _ClassVar[int]
    LANGUAGE_FIELD_NUMBER: _ClassVar[int]
    RATE_FIELD_NUMBER: _ClassVar[int]
    VOLUME_FIELD_NUMBER: _ClassVar[int]
    PITCH_FIELD_NUMBER: _ClassVar[int]
    ENGINE_FIELD_NUMBER: _ClassVar[int]
    voice: str
    language: str
    rate: float
    volume: float
    pitch: float
    engine: TTSEngine
    def __init__(
        self,
        voice: _Optional[str] = ...,
        language: _Optional[str] = ...,
        rate: _Optional[float] = ...,
        volume: _Optional[float] = ...,
        pitch: _Optional[float] = ...,
        engine: _Optional[_Union[TTSEngine, str]] = ...,
    ) -> None: ...

class SpeakResponse(_message.Message):
    __slots__ = ("success", "message", "request_id", "estimated_duration_ms", "status")
    SUCCESS_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    ESTIMATED_DURATION_MS_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    success: bool
    message: str
    request_id: str
    estimated_duration_ms: int
    status: TTSStatus
    def __init__(
        self,
        success: bool = ...,
        message: _Optional[str] = ...,
        request_id: _Optional[str] = ...,
        estimated_duration_ms: _Optional[int] = ...,
        status: _Optional[_Union[TTSStatus, str]] = ...,
    ) -> None: ...

class SpeakCompletedEvent(_message.Message):
    __slots__ = ("request_id", "status", "actual_duration_ms", "completed_at")
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    ACTUAL_DURATION_MS_FIELD_NUMBER: _ClassVar[int]
    COMPLETED_AT_FIELD_NUMBER: _ClassVar[int]
    request_id: str
    status: TTSStatus
    actual_duration_ms: int
    completed_at: _timestamp_pb2.Timestamp
    def __init__(
        self,
        request_id: _Optional[str] = ...,
        status: _Optional[_Union[TTSStatus, str]] = ...,
        actual_duration_ms: _Optional[int] = ...,
        completed_at: _Optional[
            _Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]
        ] = ...,
    ) -> None: ...

class AudioStreamRequest(_message.Message):
    __slots__ = ("event_types",)
    EVENT_TYPES_FIELD_NUMBER: _ClassVar[int]
    event_types: _containers.RepeatedScalarFieldContainer[AudioEventType]
    def __init__(
        self, event_types: _Optional[_Iterable[_Union[AudioEventType, str]]] = ...
    ) -> None: ...

class AudioEvent(_message.Message):
    __slots__ = ("type", "timestamp", "wake_word", "speech_completed", "error")
    TYPE_FIELD_NUMBER: _ClassVar[int]
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    WAKE_WORD_FIELD_NUMBER: _ClassVar[int]
    SPEECH_COMPLETED_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    type: AudioEventType
    timestamp: _timestamp_pb2.Timestamp
    wake_word: WakeWordDetectedEvent
    speech_completed: SpeakCompletedEvent
    error: _common_pb2.ErrorInfo
    def __init__(
        self,
        type: _Optional[_Union[AudioEventType, str]] = ...,
        timestamp: _Optional[
            _Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]
        ] = ...,
        wake_word: _Optional[_Union[WakeWordDetectedEvent, _Mapping]] = ...,
        speech_completed: _Optional[_Union[SpeakCompletedEvent, _Mapping]] = ...,
        error: _Optional[_Union[_common_pb2.ErrorInfo, _Mapping]] = ...,
    ) -> None: ...

class StatusRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class StatusResponse(_message.Message):
    __slots__ = ("component", "metrics", "devices", "wake_word_status", "tts_status")
    COMPONENT_FIELD_NUMBER: _ClassVar[int]
    METRICS_FIELD_NUMBER: _ClassVar[int]
    DEVICES_FIELD_NUMBER: _ClassVar[int]
    WAKE_WORD_STATUS_FIELD_NUMBER: _ClassVar[int]
    TTS_STATUS_FIELD_NUMBER: _ClassVar[int]
    component: _common_pb2.ComponentInfo
    metrics: AudioMetrics
    devices: AudioDeviceStatus
    wake_word_status: WakeWordStatus
    tts_status: TTSStatus
    def __init__(
        self,
        component: _Optional[_Union[_common_pb2.ComponentInfo, _Mapping]] = ...,
        metrics: _Optional[_Union[AudioMetrics, _Mapping]] = ...,
        devices: _Optional[_Union[AudioDeviceStatus, _Mapping]] = ...,
        wake_word_status: _Optional[_Union[WakeWordStatus, _Mapping]] = ...,
        tts_status: _Optional[_Union[TTSStatus, str]] = ...,
    ) -> None: ...

class AudioMetrics(_message.Message):
    __slots__ = (
        "wake_words_detected",
        "tts_requests_completed",
        "tts_queue_size",
        "average_tts_duration_ms",
        "uptime_seconds",
    )
    WAKE_WORDS_DETECTED_FIELD_NUMBER: _ClassVar[int]
    TTS_REQUESTS_COMPLETED_FIELD_NUMBER: _ClassVar[int]
    TTS_QUEUE_SIZE_FIELD_NUMBER: _ClassVar[int]
    AVERAGE_TTS_DURATION_MS_FIELD_NUMBER: _ClassVar[int]
    UPTIME_SECONDS_FIELD_NUMBER: _ClassVar[int]
    wake_words_detected: int
    tts_requests_completed: int
    tts_queue_size: int
    average_tts_duration_ms: float
    uptime_seconds: int
    def __init__(
        self,
        wake_words_detected: _Optional[int] = ...,
        tts_requests_completed: _Optional[int] = ...,
        tts_queue_size: _Optional[int] = ...,
        average_tts_duration_ms: _Optional[float] = ...,
        uptime_seconds: _Optional[int] = ...,
    ) -> None: ...

class AudioDeviceStatus(_message.Message):
    __slots__ = (
        "microphone_available",
        "speaker_available",
        "microphone_name",
        "speaker_name",
        "current_volume",
        "is_muted",
    )
    MICROPHONE_AVAILABLE_FIELD_NUMBER: _ClassVar[int]
    SPEAKER_AVAILABLE_FIELD_NUMBER: _ClassVar[int]
    MICROPHONE_NAME_FIELD_NUMBER: _ClassVar[int]
    SPEAKER_NAME_FIELD_NUMBER: _ClassVar[int]
    CURRENT_VOLUME_FIELD_NUMBER: _ClassVar[int]
    IS_MUTED_FIELD_NUMBER: _ClassVar[int]
    microphone_available: bool
    speaker_available: bool
    microphone_name: str
    speaker_name: str
    current_volume: float
    is_muted: bool
    def __init__(
        self,
        microphone_available: bool = ...,
        speaker_available: bool = ...,
        microphone_name: _Optional[str] = ...,
        speaker_name: _Optional[str] = ...,
        current_volume: _Optional[float] = ...,
        is_muted: bool = ...,
    ) -> None: ...

class WakeWordStatus(_message.Message):
    __slots__ = ("active", "wake_word", "engine", "sensitivity")
    ACTIVE_FIELD_NUMBER: _ClassVar[int]
    WAKE_WORD_FIELD_NUMBER: _ClassVar[int]
    ENGINE_FIELD_NUMBER: _ClassVar[int]
    SENSITIVITY_FIELD_NUMBER: _ClassVar[int]
    active: bool
    wake_word: str
    engine: WakeWordEngine
    sensitivity: float
    def __init__(
        self,
        active: bool = ...,
        wake_word: _Optional[str] = ...,
        engine: _Optional[_Union[WakeWordEngine, str]] = ...,
        sensitivity: _Optional[float] = ...,
    ) -> None: ...
