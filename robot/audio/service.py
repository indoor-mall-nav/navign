#!/usr/bin/env python3
"""Audio service for wake word detection and text-to-speech using Zenoh."""

import asyncio
import logging
from typing import Optional
import struct

import edge_tts
import pvporcupine
import pyaudio
import zenoh

# TODO: Import generated protobuf messages
# from audio_pb2 import *

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class AudioService:
    """Audio service for wake word detection and TTS."""

    def __init__(self):
        """Initialize audio service."""
        self.session: Optional[zenoh.Session] = None
        self.porcupine: Optional[pvporcupine.Porcupine] = None
        self.audio: Optional[pyaudio.PyAudio] = None
        self.audio_stream: Optional[pyaudio.Stream] = None
        self.running = False
        self.wake_word_active = False

    async def start(self):
        """Start the audio service."""
        logger.info("Starting Audio service...")

        # Initialize Zenoh session
        logger.info("Connecting to Zenoh...")
        conf = zenoh.Config()
        self.session = zenoh.open(conf)

        # Initialize wake word detector
        # NOTE: Requires Porcupine access key - get from environment
        # logger.info("Initializing wake word detector...")
        # self.porcupine = pvporcupine.create(
        #     access_key=os.getenv("PORCUPINE_ACCESS_KEY"),
        #     keywords=["jarvis"]  # Built-in wake word
        # )

        # Initialize PyAudio
        logger.info("Initializing audio system...")
        self.audio = pyaudio.PyAudio()

        # Subscribe to TTS requests
        await self.subscribe_requests()

        # Start wake word detection (if enabled)
        # self.running = True
        # await self.detect_wake_word()

        logger.info("Audio service started successfully")

    async def subscribe_requests(self):
        """Subscribe to audio-related requests."""

        def speak_callback(sample):
            """Handle TTS speak requests."""
            logger.info("Received TTS speak request")
            # TODO: Decode request, perform TTS, publish completion event
            asyncio.create_task(self.speak("Hello, I am the robot"))

        def wake_word_config_callback(sample):
            """Handle wake word configuration requests."""
            logger.info("Received wake word config request")
            # TODO: Decode request and update wake word settings

        # Subscribe to TTS requests
        self.session.declare_subscriber(
            "robot/audio/speak/request",
            speak_callback
        )

        # Subscribe to wake word config
        self.session.declare_subscriber(
            "robot/audio/wakeword/config",
            wake_word_config_callback
        )

    async def detect_wake_word(self):
        """Detect wake words from microphone input."""
        if not self.porcupine:
            logger.warning("Wake word detector not initialized")
            return

        # Open audio stream
        self.audio_stream = self.audio.open(
            rate=self.porcupine.sample_rate,
            channels=1,
            format=pyaudio.paInt16,
            input=True,
            frames_per_buffer=self.porcupine.frame_length,
        )

        logger.info("Wake word detection started")

        while self.running:
            pcm = self.audio_stream.read(self.porcupine.frame_length, exception_on_overflow=False)
            pcm = struct.unpack_from("h" * self.porcupine.frame_length, pcm)

            keyword_index = self.porcupine.process(pcm)

            if keyword_index >= 0:
                logger.info(f"Wake word detected! Index: {keyword_index}")
                # TODO: Publish wake word detected event
                self.publish_wake_word_event()

            await asyncio.sleep(0.01)

    def publish_wake_word_event(self):
        """Publish wake word detection event."""
        # TODO: Encode and publish wake word event protobuf message
        logger.info("Publishing wake word detected event")
        self.session.put("robot/audio/events", b"wake_word_detected")

    async def speak(self, text: str, voice: str = "en-US-AriaNeural"):
        """Perform text-to-speech."""
        logger.info(f"Speaking: {text}")

        try:
            # Generate TTS audio
            communicate = edge_tts.Communicate(text, voice)

            # Save to temporary file and play
            temp_file = "/tmp/tts_output.mp3"
            await communicate.save(temp_file)

            # TODO: Play audio file using pygame or other player
            # TODO: Publish speech completion event
            logger.info("TTS completed")

        except Exception as e:
            logger.error(f"TTS failed: {e}")

    def publish_status(self):
        """Publish component status."""
        # TODO: Encode and publish status protobuf message
        logger.debug("Publishing audio status")

    async def stop(self):
        """Stop the audio service."""
        logger.info("Stopping Audio service...")
        self.running = False

        if self.audio_stream:
            self.audio_stream.close()

        if self.audio:
            self.audio.terminate()

        if self.porcupine:
            self.porcupine.delete()

        if self.session:
            self.session.close()

        logger.info("Audio service stopped")


async def main():
    """Main entry point."""
    service = AudioService()

    try:
        await service.start()

        # Publish status periodically
        while True:
            service.publish_status()
            await asyncio.sleep(5)

    except KeyboardInterrupt:
        logger.info("Received keyboard interrupt")
    finally:
        await service.stop()


if __name__ == "__main__":
    asyncio.run(main())
