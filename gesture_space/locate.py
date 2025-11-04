import cv2
import numpy as np
from pupil_apriltags import Detector

# --- Hyperparameters ---
TAG_SIZE = 0.015  # in meters
KNOWN_TAG_POSITIONS = {
    # tag_id: (x, y) in meters (world coordinates of tag centers)
    0: (0.0, 1.0),
    1: (0.0, 0.5),
    2: (0.0, 0.0),
    3: (0.5, 1.0),
    4: (1.0, 0.5),
    5: (1.0, 1.0),
    6: (1.0, 0.0),
    7: (0.5, 0.0),
}

# --- Load camera intrinsics ---
calib = np.load("assets/interstices.npz")
K = calib["camera_matrix"]
dist = calib["dist_coeffs"]

# --- AprilTag Detector ---
detector = Detector(
    families="tag36h11",
)


# --- Utilities ---
def get_tag_object_corners(tag_center, tag_size):
    s = tag_size / 2
    corners = np.array(
        [[-s, -s, 0], [s, -s, 0], [s, s, 0], [-s, s, 0]], dtype=np.float32
    )
    corners[:, 0] += tag_center[0]
    corners[:, 1] += tag_center[1]
    return corners


def get_camera_pose(frame: cv2.typing.MatLike):
    gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
    tags = detector.detect(gray)

    object_points = []
    image_points = []

    for tag in tags:
        if tag.tag_id >= 8:
            continue
        print(f"Detected tag {tag.tag_id}")
        world_corners = get_tag_object_corners(
            KNOWN_TAG_POSITIONS[tag.tag_id], TAG_SIZE
        )
        object_points.append(world_corners)
        image_points.append(tag.corners.astype(np.float32))

        # Annotate tag
        for i in range(4):
            pt1 = tuple(tag.corners[i].astype(int))
            pt2 = tuple(tag.corners[(i + 1) % 4].astype(int))
            cv2.line(frame, pt1, pt2, (0, 255, 0), 2)
        center = tuple(tag.center.astype(int))
        cv2.circle(frame, center, 4, (0, 0, 255), -1)
        cv2.putText(
            frame,
            f"ID: {tag.tag_id}",
            (center[0] + 5, center[1] - 5),
            cv2.FONT_HERSHEY_SIMPLEX,
            0.5,
            (255, 0, 0),
            1,
        )

    cv2.imshow("AprilTags", frame)

    if len(object_points) < 6:
        print("Not enough tags detected.")
        return None, None

    if object_points:
        obj_pts = np.concatenate(object_points, axis=0)
        img_pts = np.concatenate(image_points, axis=0)

        success, rvec, tvec = cv2.solvePnP(obj_pts, img_pts, K, dist)
        if success:
            R, _ = cv2.Rodrigues(rvec)
            camera_pos = -R.T @ tvec
            print("Camera position (world coords):", camera_pos.flatten())
            print("Camera rotation (world coords):", R)
            return camera_pos.flatten(), R
        else:
            print("Pose estimation failed.")
            return None, None
    else:
        print("No object points found.")
        return None, None


def get_point_3d_place(
    point: np.ndarray,
    Z0: float,
    camera_pos: np.ndarray,
    R: np.ndarray,
):
    t_world, R_world = camera_pos, R

    if R_world is None or t_world is None:
        print("Camera pose not found.")
        return None
    
    # Step 1: undistort & normalize
    norm = cv2.undistortPoints(point, K, dist)
    x, y = norm[0][0]
    ray_cam = np.array([x, y, 1.0])

    print(R_world, ray_cam)

    # Step 2: transform to the world
    ray_world = R_world @ ray_cam.T
    ray_world /= np.linalg.norm(ray_world)
    cam_world = t_world.flatten()

    # Step 3: intersect with Z = Z0
    s = (Z0 - cam_world[2]) / ray_world[2]
    point_world = cam_world + s * ray_world
    return point_world


if __name__ == "__main__":
    cap = cv2.VideoCapture(0)
    while True:
        ret, frame = cap.read()
        camera_pos, R = get_camera_pose(frame)
        if camera_pos is not None:
            break
