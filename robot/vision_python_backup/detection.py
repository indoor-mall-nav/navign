import numpy as np
from ultralytics import YOLO

interstices = np.load("assets/interstices.npz")

K = interstices["camera_matrix"]
dist = interstices["dist_coeffs"]
Z0 = 0.0

model = YOLO("yolo12l.pt")
