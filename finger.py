import mediapipe as mp
import cv2
import numpy as np
from locate import get_point_3d_place

mp_hands = mp.solutions.hands
hands = mp_hands.Hands(static_image_mode=False, max_num_hands=1)
mp_draw = mp.solutions.drawing_utils

def get_finger_direction(frame: cv2.typing.MatLike, Z0: float=0.0, camera_pos: np.ndarray=None, R: np.ndarray=None):
    rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
    results = hands.process(rgb)

    directions = []

    if results.multi_hand_landmarks:
        for hand_landmarks in results.multi_hand_landmarks:
            # index finger MCP
            base = hand_landmarks.landmark[5]
            # index finger tip
            tip = hand_landmarks.landmark[8]

            base = np.array([[[base.x + base.z / 2, base.y + base.z / 2]]], dtype=np.float32)
            tip = np.array([[[tip.x + tip.z / 2, tip.y + tip.z / 2]]], dtype=np.float32)

            base = get_point_3d_place(base, Z0=Z0, camera_pos=camera_pos, R=R)
            tip = get_point_3d_place(tip, Z0=Z0, camera_pos=camera_pos, R=R)

            direction = tip - base
            direction = direction / np.linalg.norm(direction)

            print("Pointing direction:", direction)

            directions.append((direction, base))
    return directions