"""
Example configuration for robot audio module.

Copy this file to config.py and adjust the values for your setup.
"""

# ============================================================================
# Porcupine Wake Word Configuration
# ============================================================================

# Porcupine API key (get from https://console.picovoice.ai/)
# IMPORTANT: Replace with your actual API key
PORCUPINE_KEY = "YOUR_PORCUPINE_API_KEY_HERE"

# Path to custom wake word model file (.ppn)
# You can create custom wake words at https://console.picovoice.ai/
WAKE_WORD_MODEL_PATH = "./assets/Gesture-Space_en_mac_v3_0_0.ppn"

# Alternative: Use built-in wake words (uncomment to use)
# Built-in options: "porcupine", "picovoice", "bumblebee", "alexa",
#                   "americano", "blueberry", "computer", "grapefruit",
#                   "grasshopper", "hey google", "hey siri", "jarvis",
#                   "ok google", "terminator"
# WAKE_WORD_KEYWORDS = ["jarvis", "computer"]

# Wake word sensitivity (0.0 to 1.0, higher = more sensitive)
WAKE_WORD_SENSITIVITY = 0.5

# ============================================================================
# Audio Input Configuration
# ============================================================================

# Audio format
AUDIO_FORMAT = "paInt16"  # PyAudio format

# Number of audio channels
AUDIO_CHANNELS = 1  # Mono

# Sample rate (Hz)
# For Porcupine: must match porcupine.sample_rate (typically 16000)
# For Wav2Vec2: should be 16000
SAMPLE_RATE = 16000

# Audio chunk size (frames per buffer)
CHUNK_SIZE = 1024

# Input device index (None for default)
INPUT_DEVICE_INDEX = None

# ============================================================================
# Speech Recognition Configuration
# ============================================================================

# Wav2Vec2 model for speech recognition
# Options:
# - "facebook/wav2vec2-base-960h" (smaller, faster)
# - "facebook/wav2vec2-large-960h" (larger, more accurate)
# - "facebook/wav2vec2-large-960h-lv60-self" (best quality)
SPEECH_MODEL = "facebook/wav2vec2-large-960h"

# Device for speech recognition model
SPEECH_DEVICE = "cpu"  # or "cuda:0" or "mps" for Apple Silicon

# Speech recognition confidence threshold
SPEECH_CONFIDENCE = 0.5

# ============================================================================
# Recording Configuration
# ============================================================================

# Silence detection threshold (dB)
# Adjust based on your microphone sensitivity
SILENCE_THRESHOLD = 25

# Silence duration before stopping recording (seconds)
SILENCE_DURATION = 2

# Maximum recording duration (seconds)
MAX_RECORDING_DURATION = 10

# Minimum recording duration (seconds)
MIN_RECORDING_DURATION = 0.5

# Enable recording indicator (console output)
SHOW_RECORDING_STATUS = True

# ============================================================================
# Text-to-Speech Configuration
# ============================================================================

# Edge TTS voice
# Options for English:
# - "en-US-JennyNeural" (Female, US)
# - "en-US-GuyNeural" (Male, US)
# - "en-GB-SoniaNeural" (Female, UK)
# - "en-GB-RyanNeural" (Male, UK)
# - "en-AU-NatashaNeural" (Female, AU)
# See full list: https://speech.microsoft.com/portal/voicegallery
TTS_VOICE = "en-GB-SoniaNeural"

# TTS output file (temporary)
TTS_OUTPUT_FILE = "output.mp3"

# TTS speech rate (percentage, 100 = normal)
TTS_RATE = "+0%"

# TTS volume (percentage, 100 = normal)
TTS_VOLUME = "+0%"

# TTS pitch (percentage, 0 = normal)
TTS_PITCH = "+0Hz"

# ============================================================================
# Audio Playback Configuration
# ============================================================================

# Enable audio playback
ENABLE_PLAYBACK = True

# Playback volume (0.0 to 1.0)
PLAYBACK_VOLUME = 0.8

# Automatically delete TTS files after playback
AUTO_DELETE_TTS_FILES = True

# ============================================================================
# Audio Processing Configuration
# ============================================================================

# Enable noise suppression
ENABLE_NOISE_SUPPRESSION = False

# Enable automatic gain control
ENABLE_AGC = False

# Enable echo cancellation
ENABLE_ECHO_CANCELLATION = False

# ============================================================================
# Performance Configuration
# ============================================================================

# Buffer size for audio processing
AUDIO_BUFFER_SIZE = 4096

# Enable audio caching
ENABLE_AUDIO_CACHE = True
AUDIO_CACHE_DIR = "cache/audio"

# ============================================================================
# Logging Configuration
# ============================================================================

# Enable debug logging
DEBUG_MODE = True

# Log audio levels to console
LOG_AUDIO_LEVELS = False

# Log recognition results
LOG_RECOGNITION_RESULTS = True

# Save audio recordings
SAVE_RECORDINGS = False
RECORDING_OUTPUT_DIR = "output/recordings"

# ============================================================================
# Integration Configuration
# ============================================================================

# Zenoh topic for publishing transcriptions
ZENOH_TRANSCRIPTION_TOPIC = "robot/audio/transcription"

# Zenoh topic for publishing wake word events
ZENOH_WAKE_WORD_TOPIC = "robot/audio/wake_word"

# Data publishing rate (Hz)
PUBLISH_RATE = 1

# ============================================================================
# Advanced Configuration
# ============================================================================

# Wake word detection timeout (seconds)
# Stop listening after this duration if no wake word detected
WAKE_WORD_TIMEOUT = 0  # 0 = no timeout

# Multi-wake word mode (detect multiple wake words)
MULTI_WAKE_WORD = False

# Voice activity detection (VAD) threshold
VAD_THRESHOLD = 0.5

# Pre-recording buffer (seconds)
# Capture audio before wake word detection
PRE_RECORDING_BUFFER = 0.5

# Post-recording buffer (seconds)
# Continue recording after silence detected
POST_RECORDING_BUFFER = 0.3
