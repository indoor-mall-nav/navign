#!/usr/bin/env python3
"""
Example usage of the robot audio module.

This script demonstrates how to use the various audio components:
- Wake word detection with Porcupine
- Speech recognition with Wav2Vec2
- Text-to-speech with Edge TTS
- Audio recording and playback
"""

import struct
import time

# Import audio modules
from waking import stream, porcupine, pa
from recognition import record_audio, recognize_audio
from play import play_audio

# Optional: Import config if you created one
try:
    import config
    DEBUG_MODE = config.DEBUG_MODE
    LOG_RECOGNITION_RESULTS = config.LOG_RECOGNITION_RESULTS
    ENABLE_PLAYBACK = config.ENABLE_PLAYBACK
except ImportError:
    # Use defaults if config.py doesn't exist
    DEBUG_MODE = True
    LOG_RECOGNITION_RESULTS = True
    ENABLE_PLAYBACK = True


def example_wake_word_detection():
    """Example 1: Wake word detection only."""
    print("=== Example 1: Wake Word Detection ===")
    print(f"Listening for wake word... (Press Ctrl+C to stop)")
    print("Say the wake word to trigger detection.\n")

    try:
        while True:
            pcm = stream.read(porcupine.frame_length, exception_on_overflow=False)
            pcm = struct.unpack_from("h" * porcupine.frame_length, pcm)

            keyword_index = porcupine.process(pcm)

            if keyword_index >= 0:
                print(f"✓ Wake word detected! (index: {keyword_index})")
                print("  Waiting 2 seconds before listening again...\n")
                time.sleep(2)

    except KeyboardInterrupt:
        print("\nStopped wake word detection.")


def example_speech_recognition():
    """Example 2: Speech recognition without wake word."""
    print("=== Example 2: Speech Recognition ===")
    print("Press Enter to start recording, or 'q' to quit.\n")

    try:
        while True:
            user_input = input("Press Enter to record (or 'q' to quit): ").strip()

            if user_input.lower() == 'q':
                break

            print("🎤 Recording... (speak now, will auto-stop after silence)")

            # Record audio
            audio = record_audio()

            print("🔍 Recognizing speech...")

            # Recognize speech
            transcription = recognize_audio(audio)

            if LOG_RECOGNITION_RESULTS:
                print(f"✓ Transcription: {transcription}\n")

    except KeyboardInterrupt:
        print("\nStopped speech recognition.")


def example_text_to_speech():
    """Example 3: Text-to-speech only."""
    print("=== Example 3: Text-to-Speech ===")
    print("Enter text to convert to speech, or 'q' to quit.\n")

    try:
        while True:
            text = input("Enter text: ").strip()

            if text.lower() == 'q':
                break

            if not text:
                continue

            print(f"🔊 Speaking: {text}")

            if ENABLE_PLAYBACK:
                play_audio(text)
            else:
                print("(Playback disabled in config)")

            print()

    except KeyboardInterrupt:
        print("\nStopped text-to-speech.")


def example_wake_word_and_recognition():
    """Example 4: Wake word detection followed by speech recognition."""
    print("=== Example 4: Wake Word + Speech Recognition ===")
    print(f"Listening for wake word... (Press Ctrl+C to stop)")
    print("Say the wake word, then speak your command.\n")

    try:
        while True:
            pcm = stream.read(porcupine.frame_length, exception_on_overflow=False)
            pcm = struct.unpack_from("h" * porcupine.frame_length, pcm)

            keyword_index = porcupine.process(pcm)

            if keyword_index >= 0:
                print(f"✓ Wake word detected! (index: {keyword_index})")
                print("🎤 Listening for your command...")

                # Record audio
                audio = record_audio()

                print("🔍 Recognizing speech...")

                # Recognize speech
                transcription = recognize_audio(audio)

                if LOG_RECOGNITION_RESULTS:
                    print(f"✓ You said: {transcription}\n")

                # Wait before listening for wake word again
                print("Waiting 2 seconds before listening again...\n")
                time.sleep(2)

    except KeyboardInterrupt:
        print("\nStopped wake word detection.")


def example_interactive_assistant():
    """Example 5: Interactive voice assistant with responses."""
    print("=== Example 5: Interactive Voice Assistant ===")
    print(f"Listening for wake word... (Press Ctrl+C to stop)")
    print("Say the wake word, ask a question, and get a response.\n")

    # Simple response generator (replace with your LLM integration)
    def generate_response(user_input: str) -> str:
        """Simple rule-based responses for demonstration."""
        user_input = user_input.lower()

        if "hello" in user_input or "hi" in user_input:
            return "Hello! How can I help you today?"
        elif "time" in user_input:
            return f"The current time is {time.strftime('%I:%M %p')}."
        elif "date" in user_input:
            return f"Today is {time.strftime('%B %d, %Y')}."
        elif "weather" in user_input:
            return "I'm sorry, I don't have access to weather data yet."
        elif "thank" in user_input:
            return "You're welcome! Happy to help."
        elif "name" in user_input:
            return "I'm your robot assistant."
        elif "help" in user_input:
            return "You can ask me about the time, date, or just say hello!"
        else:
            return "I heard you, but I'm not sure how to respond to that yet."

    try:
        while True:
            pcm = stream.read(porcupine.frame_length, exception_on_overflow=False)
            pcm = struct.unpack_from("h" * porcupine.frame_length, pcm)

            keyword_index = porcupine.process(pcm)

            if keyword_index >= 0:
                print(f"✓ Wake word detected!")

                if ENABLE_PLAYBACK:
                    play_audio("Yes?")

                print("🎤 Listening for your question...")

                # Record audio
                audio = record_audio()

                print("🔍 Recognizing speech...")

                # Recognize speech
                user_request = recognize_audio(audio)

                if LOG_RECOGNITION_RESULTS:
                    print(f"✓ You asked: {user_request}")

                # Generate response
                print("💭 Generating response...")
                response = generate_response(user_request)

                print(f"🤖 Response: {response}")

                # Speak response
                if ENABLE_PLAYBACK:
                    play_audio(response)

                print("\nWaiting 2 seconds before listening again...\n")
                time.sleep(2)

    except KeyboardInterrupt:
        print("\nStopped interactive assistant.")


def example_continuous_listening():
    """Example 6: Continuous listening with voice activity detection."""
    print("=== Example 6: Continuous Listening ===")
    print("The system will continuously listen and transcribe speech.")
    print("Press Ctrl+C to stop.\n")

    try:
        print("🎤 Recording continuously...")

        while True:
            # Record audio (will auto-stop after silence)
            audio = record_audio()

            # Recognize speech
            transcription = recognize_audio(audio)

            # Only print non-empty transcriptions
            if transcription.strip():
                timestamp = time.strftime('%H:%M:%S')
                print(f"[{timestamp}] {transcription}")

    except KeyboardInterrupt:
        print("\nStopped continuous listening.")


def example_audio_feedback():
    """Example 7: Audio feedback for user actions."""
    print("=== Example 7: Audio Feedback ===")
    print("Demonstrates providing audio feedback for various events.\n")

    events = [
        ("System starting up", "Robot system is now online and ready."),
        ("Battery low warning", "Warning: Battery level is low. Please charge soon."),
        ("Task completed", "Task completed successfully."),
        ("Error occurred", "An error has occurred. Please check the system."),
        ("Shutdown", "Shutting down. Goodbye!"),
    ]

    try:
        for event_name, message in events:
            print(f"\nEvent: {event_name}")
            print(f"Message: {message}")

            if ENABLE_PLAYBACK:
                play_audio(message)

            time.sleep(1)

    except KeyboardInterrupt:
        print("\nStopped audio feedback demo.")


if __name__ == "__main__":
    import sys

    examples = {
        "1": ("Wake Word Detection", example_wake_word_detection),
        "2": ("Speech Recognition", example_speech_recognition),
        "3": ("Text-to-Speech", example_text_to_speech),
        "4": ("Wake Word + Recognition", example_wake_word_and_recognition),
        "5": ("Interactive Assistant", example_interactive_assistant),
        "6": ("Continuous Listening", example_continuous_listening),
        "7": ("Audio Feedback", example_audio_feedback),
    }

    print("\n" + "="*60)
    print("Robot Audio Module - Example Usage")
    print("="*60)
    print("\nAvailable examples:")
    for key, (name, _) in examples.items():
        print(f"  {key}. {name}")
    print("\nPress Ctrl+C to exit any example.\n")

    if len(sys.argv) > 1:
        choice = sys.argv[1]
    else:
        choice = input("Select example (1-7): ").strip()

    if choice in examples:
        _, func = examples[choice]
        print()
        func()
    else:
        print("Invalid choice. Please select 1-7.")

    # Cleanup
    print("\nCleaning up...")
    stream.stop_stream()
    stream.close()
    pa.terminate()
    porcupine.delete()
    print("Done!")
