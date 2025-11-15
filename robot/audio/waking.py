import pvporcupine
from config import PORCUPINE_KEY
import pyaudio

pa = pyaudio.PyAudio()

porcupine = pvporcupine.create(
    access_key=PORCUPINE_KEY, keyword_paths=["./assets/Gesture-Space_en_mac_v3_0_0.ppn"]
)

stream = pa.open(
    rate=porcupine.sample_rate,
    channels=1,
    format=pyaudio.paInt16,
    input=True,
    frames_per_buffer=porcupine.frame_length,
)
