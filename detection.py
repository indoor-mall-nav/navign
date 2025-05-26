import cv2
import numpy as np
from ultralytics import YOLO

from finger import get_finger_direction
from local import generate_local_response
from locate import get_camera_pose, get_point_3d_place
from transformers import AutoTokenizer, AutoModelForCausalLM
import edge_tts
import os
import pygame

from remote import run_remote_response

interstices = np.load("camera_calibration_output.npz")

K = interstices["camera_matrix"]
dist = interstices["dist_coeffs"]
Z0 = 0.0

model = YOLO("yolo12l.pt")

cap = cv2.VideoCapture(0)

while True:
    ret, frame = cap.read()
    if not ret:
        print("Failed to grab frame")
        break

    image = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)

    detections = model(image)

    content = ""

    loc, pos = get_camera_pose(cap)

    points_3d = []

    for result in detections:
        xyxy = (
            result.boxes.xyxy
        )  # top-left-x, top-left-y, bottom-right-x, bottom-right-y
        names = [
            result.names[cls.item()] for cls in result.boxes.cls.int()
        ]  # class name of each box
        confs = result.boxes.conf  # confidence score of each box

        print(f"xyxy: {xyxy}, names: {names}, confs: {confs}")

        t_world, R_world = get_camera_pose(cap)

        for [x1, y1, x2, y2], name, conf in zip(xyxy, names, confs):
            u = ((x1 + x2) / 2).item()
            v = ((y1 + y2) / 2).item()
            point_world = get_point_3d_place(
                np.array([[[u, v]]], dtype=np.float32), Z0, t_world, R_world
            )
            points_3d.append(point_world)

        directions = get_finger_direction(frame, Z0, t_world, R_world)

        # --- Output ---
        for label, conf, pt in zip(names, confs, points_3d):
            if conf < 0.6:
                continue
            content += f"{label} at {pt[0]:.2f}, {pt[1]:.2f}, {pt[2]:.2f} (x, y, z). Confidence: {conf}\n"

    result = generate_local_response(content)

    if result == "<remote>":
        result = run_remote_response(content)

    if os.path.exists("output.mp3"):
        os.remove("output.mp3")

    communicate = edge_tts.Communicate(result, "en-GB-SoniaNeural")
    communicate.save_sync("output.mp3")

    pygame.init()
    pygame.mixer.init()

    pygame.mixer.music.load("output.mp3")  # or .wav
    pygame.mixer.music.play()

    while pygame.mixer.music.get_busy():
        pygame.time.Clock().tick(10)
    pygame.quit()
