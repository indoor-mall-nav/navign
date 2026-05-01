# Audio Service

Voice interaction system with wake word detection, speech recognition, and text-to-speech.

## Overview

**Language:** Python
**Location:** `robot/audio/`
**Technologies:** Porcupine, Wav2Vec2, Edge TTS

## Capabilities

- **Wake Word Detection:** Porcupine-based activation (migrating to OpenWakeWord)
- **Speech Recognition:** Wav2Vec2 speech-to-text
- **Text-to-Speech:** Edge TTS voice synthesis
- **Audio Recording:** Voice activity detection with silence detection
- **Audio Playback:** Cross-platform with pygame

## Architecture

```
  Microphone
      |
      v
Audio Service
  (Porcupine
   Wav2Vec2
   Edge TTS)
      |
      | Zenoh Topics
      |
      +-> robot/audio/wake_word
      +-> robot/audio/transcription
      +-> robot/audio/events
```

## Setup

```bash
cd robot/audio
uv sync
cp config.example.py config.py
```

Get Porcupine API key:

1. Visit https://console.picovoice.ai/
2. Create free account
3. Copy API key
4. Update `PORCUPINE_KEY` in `config.py`

## Zenoh Topics

### Published

- `robot/audio/wake_word` - Wake word detected events
- `robot/audio/transcription` - Speech recognition results
- `robot/audio/events` - Audio state changes

## Running

```bash
cd robot/audio
uv run python service.py
```

## Environment Variables

- `PORCUPINE_ACCESS_KEY` - Required for wake word detection

## Configuration

Edit `config.py`:

- Porcupine wake word model
- Wake word sensitivity (0.0-1.0)
- Speech recognition model
- TTS voice selection
- Audio device indices
- Silence detection thresholds

## Voice Interaction Example

```
1. Wait for wake word
2. Play acknowledgment: "Yes?"
3. Record user speech
4. Recognize: "Go to the cafeteria"
5. Process command
6. Respond: "Navigating to cafeteria"
```

## See Also

- [Audio README](/robot/audio/README.md)
- [Scheduler](scheduler.md)
- [Protocol Buffers](/robot/proto/audio.proto)
