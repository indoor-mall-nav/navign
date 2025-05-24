import cv2
import numpy as np
from ultralytics import YOLO
from locate import get_camera_pose
from transformers import AutoTokenizer, AutoModelForCausalLM
import edge_tts
import os
import pygame

model_name = 'Qwen/Qwen3-0.6B'

tokenizer = AutoTokenizer.from_pretrained(model_name)
llm = AutoModelForCausalLM.from_pretrained(
    model_name,
    torch_dtype="auto",
    device_map="auto"
)

prompt = ('<|im_start|>/no_think You are talking to a person who is unable to get the whole sight to the room. You will be given a list of objects and their 3D coordinates. Your task is to describe the scene with your text to a blind in the room. Suppose the person is in (0,0,0) in this scene. You should not involve any coordinate, including exact number, but use "far" or "near."'
          f'Now the user is asking: {input()}\n')  # basic system prompt
prompt_suffix = '\n<|im_end|><|im_start|>\n'

content = ''

interstices = np.load('camera_calibration_output.npz')

K = interstices["camera_matrix"]
dist = interstices["dist_coeffs"]
Z0 = 0.0

model = YOLO('yolo12l.pt')

cap = cv2.VideoCapture(0)

ret, frame = cap.read()

image = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)

detections = model(image)

loc, pos = get_camera_pose(cap)

points_3d = []

for result in detections:
    xyxy = result.boxes.xyxy  # top-left-x, top-left-y, bottom-right-x, bottom-right-y
    names = [result.names[cls.item()] for cls in result.boxes.cls.int()]  # class name of each box
    confs = result.boxes.conf  # confidence score of each box

    print(f"xyxy: {xyxy}, names: {names}, confs: {confs}")

    t_world, R_world = get_camera_pose(cap)

    for [x1, y1, x2, y2], name, conf in zip(xyxy, names, confs):
        print(x1, y1, x2, y2, name, conf)

        u = ((x1 + x2) / 2).item()
        v = ((y1 + y2) / 2).item()
        uv = np.array([[[u, v]]], dtype=np.float32)

        # Step 1: undistort & normalize
        norm = cv2.undistortPoints(uv, K, dist)
        x, y = norm[0][0]
        ray_cam = np.array([x, y, 1.0])

        if R_world is None or t_world is None:
            print("Camera pose not found.")
            continue

        print(R_world, ray_cam)

        # Step 2: transform to the world
        ray_world = R_world @ ray_cam.T
        ray_world /= np.linalg.norm(ray_world)
        cam_world = t_world.flatten()

        # Step 3: intersect with Z = Z0
        s = (Z0 - cam_world[2]) / ray_world[2]
        point_world = cam_world + s * ray_world
        points_3d.append(point_world)

    # --- Output ---
    for label, conf, pt in zip(names, confs, points_3d):
        if conf < 0.6:
            continue
        content += f"{label} at {pt[0]:.2f}, {pt[1]:.2f}, {pt[2]:.2f} (x, y, z). Confidence: {conf}\n"

message = prompt + content + prompt_suffix

text = tokenizer.apply_chat_template(
    [{"role": "user", "content": message}],
    tokenize=False,
    add_generation_prompt=True,
    enable_thinking=False # Switches between thinking and non-thinking modes. Default is True.
)
model_inputs = tokenizer([text], return_tensors="pt").to(llm.device)

# conduct text completion
generated_ids = llm.generate(
    **model_inputs,
    max_new_tokens=1024
)
output_ids = generated_ids[0][len(model_inputs.input_ids[0]):].tolist()

result = tokenizer.decode(output_ids, skip_special_tokens=True).strip("\n")


if os.path.exists('output.mp3'):
    os.remove('output.mp3')

communicate = edge_tts.Communicate(result, 'en-GB-SoniaNeural')
communicate.save_sync('output.mp3')

pygame.init()
pygame.mixer.init()

pygame.mixer.music.load("output.mp3")  # or .wav
pygame.mixer.music.play()

while pygame.mixer.music.get_busy():
    pygame.time.Clock().tick(10)
pygame.quit()