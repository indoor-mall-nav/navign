import cv2
import mediapipe as mp
from cv2.typing import MatLike
import numpy as np
from PIL import Image

mp_hands = mp.solutions.hands
hands = mp_hands.Hands(
    static_image_mode=False, max_num_hands=1, min_detection_confidence=0.5
)
mp_draw = mp.solutions.drawing_utilsx


class ToNumpy:
    def __call__(self, image: Image):
        return np.array(image)


class CropToHand:
    def __call__(self, image: MatLike):
        global y_min, y_max, x_min, x_max, hand_image_resized
        image = cv2.cvtColor(image, cv2.COLOR_BGR2RGB)
        results = hands.process(image)
        if results.multi_hand_landmarks:
            for hand_landmarks in results.multi_hand_landmarks:
                # Get the bounding box coordinates of the hand
                image_height, image_width, _ = image.shape
                x_min = image_width
                y_min = image_height
                x_max = y_max = 0
                for landmark in hand_landmarks.landmark:
                    x = int(landmark.x * image_width)
                    y = int(landmark.y * image_height)
                    x_min = min(x_min, x)
                    y_min = min(y_min, y)
                    x_max = max(x_max, x)
                    y_max = max(y_max, y)

            # Crop the hand region from the frame
            hand_image = image[y_min:y_max, x_min:x_max]

            # Resize the cropped hand image to 256x256
            hand_image_resized = cv2.resize(hand_image, (256, 256))
            return hand_image_resized
        else:
            return image.reshape((256, 256))


class ExtractKeypoints:
    def __init__(self, hands: mp_hands.Hands):
        self.hands = hands

    def __call__(self, image: MatLike) -> np.ndarray:
        image = cv2.resize(image, (256, 256))
        results = hands.process(image)
        h, w, c = image.shape
        lst_lms = []
        x0, y0 = 0, 0
        if results.multi_hand_landmarks:
            for single_hand_marks in results.multi_hand_landmarks:
                for id, lm in enumerate(single_hand_marks.landmark):
                    if id == 0:
                        x0, y0 = int(w * lm.x), int(h * lm.y)
                    else:
                        x, y = int(w * lm.x) - x0, int(h * lm.y) - y0
                        lst_lms.append([id, x, y])

        return np.array(lst_lms, dtype=np.float32)


class HandleGestureDataset:
    def __init__(self):
        pass

    def __call__(self, matrix: np.ndarray):
        # assert matrix.shape == (20, 3)
        if matrix.shape == (0, 0) or matrix.shape == (0,):
            matrix = np.random.random((5, 4, 2))
            return matrix
        # Find the missed points
        if matrix.shape[0] < 20:
            print(matrix, matrix.shape)
            for i in range(1, 21):
                if i not in matrix[:, 0]:
                    matrix = np.insert(matrix, i, [i, 0, 0], axis=0)
        matrix = matrix[:, 1:]
        matrix = matrix.reshape(5, 4, 2)
        return matrix
