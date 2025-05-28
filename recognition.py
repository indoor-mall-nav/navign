import time
import wave
from datetime import datetime

import numpy as np
import torch
from pyaudio import Stream
from transformers import AutoModelForCTC, AutoProcessor
import pyaudio

from waking import stream

FORMAT = pyaudio.paInt16
CHANNELS = 1
RATE = 16000
CHUNK = 1024
SILENCE_THRESHOLD = 25
SILENCE_DURATION = 2

processor = AutoProcessor.from_pretrained("facebook/wav2vec2-large-960h")
audio_model = AutoModelForCTC.from_pretrained("facebook/wav2vec2-large-960h")


def record_audio():
    audio = pyaudio.PyAudio()
    stream = audio.open(format=FORMAT, channels=CHANNELS,
                        rate=RATE, input=True,
                        frames_per_buffer=CHUNK)
    audio_buffer = np.empty((0,), dtype=np.int16)
    silent_chunks = 0
    recording = True

    while recording:
        data = np.frombuffer(stream.read(CHUNK), dtype=np.int16)
        audio_buffer = np.concatenate((audio_buffer, data))

        if db_level(data) > SILENCE_THRESHOLD:
            silent_chunks += 1

            if silent_chunks > (SILENCE_DURATION * RATE / CHUNK):
                recording = False
        else:
            silent_chunks = 0

    print(f"Recording: {len(audio_buffer) / RATE:.2f}s", end='\r')
    stream.stop_stream()
    stream.close()
    return audio_buffer


def db_level(data):
    """Calculate the dB level of the audio data."""
    rms = np.sqrt(np.mean(data ** 2))
    if rms > 0:
        return 20 * np.log10(rms)
    else:
        return -np.inf  # Return negative infinity for silence


def recognize_audio(audio: np.ndarray) -> str:
    inputs = processor(audio, return_tensors="pt", sampling_rate=16000).input_values.float().to(device)
    with torch.no_grad():
        logits = audio_model(inputs).logits
        predicted_ids = torch.argmax(logits, dim=-1)
        transcription = processor.batch_decode(predicted_ids)[0]

        return transcription.lower()
