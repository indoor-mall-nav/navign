import cv2
import numpy as np

# Lists to store calibration points
object_points = []  # 3D points in real-world space
image_points = []  # 2D points in image plane

# Set chessboard dimensions
chessboard_size = (9, 6)  # 9x7 internal corners
criteria = (cv2.TermCriteria_EPS + cv2.TermCriteria_MAX_ITER, 15, 0.001)

# Prepare object points (0,0,0), (1,0,0), ..., (8,5,0)
objp = np.zeros((chessboard_size[0] * chessboard_size[1], 3), np.float32)
objp[:, :2] = np.mgrid[0 : chessboard_size[0], 0 : chessboard_size[1]].T.reshape(-1, 2)

# Open default camera
cap = cv2.VideoCapture(0)

if not cap.isOpened():
    print("Cannot open camera")
    exit()

while True:
    ret, frame = cap.read()
    if not ret:
        print("Failed to grab frame")
        break

    gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)

    # Find the chessboard corners
    ret, corners = cv2.findChessboardCorners(gray, chessboard_size, None)

    if ret:
        # Refine corner locations
        corners_subpix = cv2.cornerSubPix(gray, corners, (11, 11), (-1, -1), criteria)

        # Draw corners
        cv2.drawChessboardCorners(frame, chessboard_size, corners_subpix, ret)

    # If corners were found, save object and image points
    if ret:
        object_points.append(objp)
        image_points.append(corners_subpix)

        # Auto-exit after 20 valid frames
        if len(object_points) >= 20:
            print("Collected 20 calibration frames. Stopping...")
            break

    # Show the frame
    cv2.imshow("Chessboard Detection", frame)

    # Press 'q' to quit
    if cv2.waitKey(1) & 0xFF == ord("q"):
        break


# After the loop, perform calibration if points were collected
if object_points and image_points:
    ret, camera_matrix, dist_coeffs, rvecs, tvecs = cv2.calibrateCamera(
        object_points, image_points, gray.shape[::-1], None, None
    )
    np.savez(
        "camera_calibration_output.npz",
        camera_matrix=camera_matrix,
        dist_coeffs=dist_coeffs,
        rvecs=rvecs,
        tvecs=tvecs,
    )
    print("Calibration complete. Parameters saved to 'camera_calibration_output.npz'.")

cap.release()
cv2.destroyAllWindows()
