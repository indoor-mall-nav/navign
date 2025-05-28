import os
import edge_tts
import pygame


def play_audio(text: str):
    if os.path.exists("output.mp3"):
        os.remove("output.mp3")

    communicate = edge_tts.Communicate(text, "en-GB-SoniaNeural")
    communicate.save_sync("output.mp3")

    pygame.init()
    pygame.mixer.init()

    pygame.mixer.music.load("output.mp3")  # or .wav
    pygame.mixer.music.play()

    while pygame.mixer.music.get_busy():
        pygame.time.Clock().tick(10)
    pygame.quit()
