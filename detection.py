import cv2
import numpy as np
from ultralytics import YOLO
from finger import get_finger_direction
from local import generate_response
from locate import get_camera_pose, get_point_3d_place
import edge_tts
import os
import pygame

from objects import detect_objects
from play import play_audio
from remote import run_remote_response

interstices = np.load("assets/interstices.npz")

K = interstices["camera_matrix"]
dist = interstices["dist_coeffs"]
Z0 = 0.0

model = YOLO("yolo12l.pt")
