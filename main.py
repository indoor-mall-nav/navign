import struct
import cv2
import numpy as np

from detection import model, Z0
from finger import get_finger_direction
from local import generate_response
from locate import get_camera_pose, get_point_3d_place
from objects import detect_objects
from play import play_audio
from recognition import record_audio, recognize_audio
from waking import stream, porcupine, pa

cap = cv2.VideoCapture(0)

try:
    print("Listening...")
    while True:
        pcm = stream.read(porcupine.frame_length, exception_on_overflow=False)
        pcm = struct.unpack_from("h" * porcupine.frame_length, pcm)
        keyword_index = porcupine.process(pcm)
        if keyword_index >= 0:
            print(f"Wake word detected! (index {keyword_index})")
            play_audio("We are ready to assist you. Please showcase the environment around you.")
            audio = record_audio()
            user_request = recognize_audio(audio)
            print(f"User request: {user_request}")
            ret, frame = cap.read()
            if not ret:
                print("Failed to grab frame")
                break

            image = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)

            detections = model(image)

            content = ""

            t_world, R_world = get_camera_pose(cap)

            points_3d = []

            objects = detect_objects(model, frame)

            directions = get_finger_direction(frame, Z0, t_world, R_world)

            for u, v, name, conf in objects:
                pt = get_point_3d_place(
                    np.array([[[u, v]]], dtype=np.float32), Z0, t_world, R_world
                )
                points_3d.append(pt)
                if conf < 0.6:
                    continue
                content += f"{name} at {pt[0]:.2f}, {pt[1]:.2f}, {pt[2]:.2f} (x, y, z). Confidence: {conf}\n"

            result = generate_response(content, user_request)

            play_audio(result)

except KeyboardInterrupt:
    pass
finally:
    stream.stop_stream()
    stream.close()
    pa.terminate()
    porcupine.delete()
    cap.release()
    cv2.destroyAllWindows()