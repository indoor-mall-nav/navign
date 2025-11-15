# Robot Audio Module

Audio processing capabilities for the Navign robot, including wake word detection, speech recognition, and text-to-speech.

## Features

- **Wake Word Detection**: Porcupine-based wake word detection
- **Speech Recognition**: Wav2Vec2-based speech-to-text
- **Text-to-Speech**: Edge TTS for natural voice synthesis
- **Audio Recording**: Voice activity detection with automatic silence detection
- **Audio Playback**: Cross-platform audio playback with pygame

## Installation

```bash
cd robot/audio
uv sync
```

## Configuration

1. Copy the example configuration:
   ```bash
   cp config.example.py config.py
   ```

2. Get a Porcupine API key:
   - Visit https://console.picovoice.ai/
   - Create a free account
   - Copy your API key
   - Update `PORCUPINE_KEY` in `config.py`

3. (Optional) Create custom wake word:
   - Go to https://console.picovoice.ai/
   - Navigate to "Porcupine" → "Wake Word"
   - Train a custom wake word
   - Download the `.ppn` file
   - Save to `assets/` directory
   - Update `WAKE_WORD_MODEL_PATH` in `config.py`

4. Customize other settings in `config.py`:
   - Speech recognition model
   - TTS voice selection
   - Audio parameters
   - Silence detection thresholds

## Usage

### Quick Start

Run the example usage script:

```bash
uv run python example_usage.py
```

Select from available examples:
1. Wake Word Detection
2. Speech Recognition
3. Text-to-Speech
4. Wake Word + Recognition
5. Interactive Assistant
6. Continuous Listening
7. Audio Feedback

### Module Documentation

#### Wake Word Detection (`waking.py`)

```python
from waking import stream, porcupine
import struct

# Listen for wake word
pcm = stream.read(porcupine.frame_length, exception_on_overflow=False)
pcm = struct.unpack_from("h" * porcupine.frame_length, pcm)
keyword_index = porcupine.process(pcm)

if keyword_index >= 0:
    print("Wake word detected!")
```

#### Speech Recognition (`recognition.py`)

```python
from recognition import record_audio, recognize_audio

# Record audio with automatic silence detection
audio = record_audio()

# Recognize speech
transcription = recognize_audio(audio)
print(f"You said: {transcription}")
```

#### Text-to-Speech (`play.py`)

```python
from play import play_audio

# Convert text to speech and play
play_audio("Hello, I am your robot assistant.")
```

#### Complete Voice Interaction

```python
import struct
from waking import stream, porcupine
from recognition import record_audio, recognize_audio
from play import play_audio

# Listen for wake word
while True:
    pcm = stream.read(porcupine.frame_length, exception_on_overflow=False)
    pcm = struct.unpack_from("h" * porcupine.frame_length, pcm)

    if porcupine.process(pcm) >= 0:
        # Wake word detected
        play_audio("Yes?")

        # Record user request
        audio = record_audio()
        request = recognize_audio(audio)

        # Generate response (integrate with your LLM)
        response = generate_response(request)

        # Speak response
        play_audio(response)
```

## File Overview

| File | Purpose |
|------|---------|
| `waking.py` | Porcupine wake word detection initialization |
| `recognition.py` | Wav2Vec2 speech recognition |
| `play.py` | Edge TTS text-to-speech and playback |
| `config.example.py` | Example configuration file |
| `example_usage.py` | Usage examples and demos |

## Dependencies

- **pvporcupine**: Wake word detection
- **transformers**: Wav2Vec2 speech recognition
- **torch**: Neural network inference
- **pyaudio**: Audio input/output
- **edge-tts**: Text-to-speech synthesis
- **pygame**: Audio playback
- **numpy**: Audio processing

See `pyproject.toml` for complete dependency list.

## Integration with Robot System

The audio module can publish transcriptions and events to other robot components via Zenoh:

```python
import zenoh

# Example: Publishing transcriptions
z = zenoh.open()
pub = z.declare_publisher("robot/audio/transcription")

audio = record_audio()
transcription = recognize_audio(audio)
pub.put(transcription)
```

## Troubleshooting

### Porcupine Error: Invalid API Key
- Verify you've copied the correct API key from https://console.picovoice.ai/
- Update `PORCUPINE_KEY` in config.py
- Check that the key hasn't expired

### Wake Word Not Detected
- Speak clearly and directly to the microphone
- Adjust `WAKE_WORD_SENSITIVITY` in config.py (0.0-1.0)
- Check microphone input levels
- Ensure correct wake word model path

### Poor Speech Recognition
- Speak clearly and at a normal pace
- Reduce background noise
- Adjust `SILENCE_THRESHOLD` in config.py
- Try a larger Wav2Vec2 model

### Audio Input/Output Errors
- Check `INPUT_DEVICE_INDEX` in config.py
- List available devices:
  ```python
  import pyaudio
  p = pyaudio.PyAudio()
  for i in range(p.get_device_count()):
      print(f"{i}: {p.get_device_info_by_index(i)['name']}")
  ```
- Update config with correct device index

### TTS Not Playing
- Check `ENABLE_PLAYBACK` in config.py
- Verify pygame is installed correctly
- Check system audio output

### Recording Doesn't Stop
- Adjust `SILENCE_THRESHOLD` (try higher values like 30-35)
- Adjust `SILENCE_DURATION` (try longer like 3-4 seconds)
- Ensure quiet environment when not speaking

## Assets Required

Create an `assets/` directory with:
- Custom wake word `.ppn` files (optional)

Temporary files (auto-generated):
- `output.mp3`: TTS output (auto-deleted after playback)

## Available TTS Voices

Common English voices:
- `en-US-JennyNeural` (Female, US)
- `en-US-GuyNeural` (Male, US)
- `en-GB-SoniaNeural` (Female, UK)
- `en-GB-RyanNeural` (Male, UK)
- `en-AU-NatashaNeural` (Female, AU)

See full list: https://speech.microsoft.com/portal/voicegallery

## Performance Tips

1. **Wake Word Detection**:
   - Runs in real-time with minimal CPU usage
   - Lower sensitivity = fewer false positives

2. **Speech Recognition**:
   - Use `wav2vec2-base-960h` for faster, less accurate
   - Use `wav2vec2-large-960h` for slower, more accurate
   - Consider GPU acceleration for real-time performance

3. **TTS**:
   - Edge TTS generates audio quickly
   - Pre-generate common responses for instant playback
   - Cache frequently used phrases

## License

MIT License - Part of the Navign project
