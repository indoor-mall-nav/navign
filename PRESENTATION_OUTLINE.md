# GestureSpace Project Presentation Outline

**Total Duration: 4.5 minutes (270 seconds)**

---

## 1. Market Current Situation (30 seconds)

### The Accessibility Crisis in China
- **17.31 million** visually impaired people in China
- Only **~400 guide dogs** available nationwide
- **1 guide dog per 40,000** visually impaired individuals
- Guide dogs require:
  - Extremely high training costs
  - Extended training periods (years)

### Indoor Navigation Challenges
**The Problem:**
- Indoor environments are more complex than outdoor (weak GPS, difficult terrain)
- Visually impaired people struggle to independently navigate:
  - Office buildings
  - Hospitals
  - Shopping malls
  - Multi-floor buildings

**Current Solutions Fall Short:**
- **UWB positioning**: High cost
- **Indoor delivery robots**: Cost tens of thousands, unstable, frequent malfunctions
- **Dog-type robots**: Lack pulling power and stability
- **Existing robots**: Cannot smoothly avoid crowds, limited functionality

**Our Solution: GestureSpace** - An intelligent camera pipeline system for spatial environment understanding, combining AprilTag pose estimation, YOLOv12 object detection, 3D coordinate transformation, voice control, and BLE-based indoor navigation for accessible robotics

---

## 2. GestureSpace Core Techniques (2 minutes / 120 seconds)

### Overview
**GestureSpace** is a comprehensive computer vision and AI-based multimodal interaction system that enables spatial understanding of indoor environments through camera pose estimation, object detection, 3D transformation, and voice control. It serves as the intelligent interface layer for accessible robotics and indoor assistance.

### Core Technologies (Camera Pipeline for Environment Understanding)

#### 2.1 Camera Pose Estimation with AprilTags (25 seconds)
**Technology Stack:**
- **AprilTag Detection**: tag36h11 family markers at known world positions
- **PnP Solving**: solvePnP algorithm for camera pose estimation
- **Camera Calibration**: Intrinsic parameters from chessboard calibration

**How It Works:**
1. Detect 8 AprilTags in camera frame at known world positions
2. Extract 2D image corners and match to 3D world coordinates
3. Solve Perspective-n-Point (PnP) problem to find camera rotation (R) and translation (t)
4. Achieve ~2cm camera position accuracy

**Applications:**
- Precise robot localization in indoor space
- Foundation for all 3D point reconstruction
- Integration with BLE positioning for map alignment

#### 2.2 3D Point Transformation Pipeline (25 seconds)
**Mathematical Process:**
```python
# Transform 2D image points to 3D world coordinates:
1. Undistort image points using camera matrix K and distortion coefficients
2. Generate ray in camera coordinates: ray_cam = [x, y, 1.0]
3. Transform ray to world coordinates: ray_world = R_world @ ray_cam
4. Intersect ray with ground plane (Z = Z0)
5. Calculate 3D world position: point_3d = camera_pos + s * ray_world
```

**Implementation:**
- Uses OpenCV's undistortPoints for lens correction
- Applies camera pose (R, t) from PnP solving
- Projects rays to ground plane for object localization

**Precision:**
- Object localization: ~5cm accuracy on ground plane
- Enables spatial understanding of environment

#### 2.3 Object Detection with YOLOv12 (20 seconds)
**Technology:**
- **Ultralytics YOLOv12 Large (yolo12l.pt)**
- **Transformer-based architecture** (not CNN-based)
- Real-time object detection and classification

**Pipeline:**
1. Process camera frame through YOLOv12 model
2. Extract bounding boxes (xyxy format) and class names
3. Calculate object center points (u, v) in image space
4. Transform center points to 3D world coordinates using camera pose
5. Output: Object name, 3D position (x, y, z), confidence score

**Applications:**
- Identify objects in environment ("bottle", "chair", etc.)
- Locate objects in 3D space for robot navigation
- Combined with voice commands for natural interaction

#### 2.4 Hand Landmark Detection & Finger Pointing (20 seconds)
**Technology:**
- **MediaPipe Hands**: Detects 21 hand landmarks in real-time
- **Note**: Used for pointing detection, NOT for gesture classification

**Finger Direction Pipeline:**
1. Extract index finger MCP (base) and tip landmarks from MediaPipe
2. Transform both points to 3D world coordinates using camera pose
3. Calculate normalized direction vector: direction = (tip - base) / ||tip - base||
4. Output: Pointing direction in 3D space

**Applications:**
- Point to objects for robot to identify and fetch
- Indicate desired navigation directions
- Spatial interaction with environment

#### 2.5 Voice Wake Word Detection (15 seconds)
**Technology:**
- **Porcupine Wake Word Engine**
- Always-listening mode with low CPU usage
- Keyword index-based trigger system

**Workflow:**
1. Continuous audio stream processing
2. Wake word detection triggers interaction
3. Seamless handoff to speech recognition

#### 2.6 Speech Recognition & Generation (25 seconds)
**Audio Pipeline:**
- Audio recording on wake word trigger
- Speech-to-text recognition
- Natural language understanding for user requests

**Multimodal Integration:**
- Combines voice commands with visual context
- "Show me [object]" + finger pointing
- "Navigate to [location]" + gesture direction

**Response Generation:**
- Context-aware responses using local LLM
- Audio feedback via text-to-speech
- Confirmation of detected objects/directions

### System Integration
Camera pipeline processes environment in real-time:
```
Camera Frame → [AprilTag Detection → PnP Solving (Camera Pose R, t)]
                         ↓
            [YOLOv12 Object Detection + Hand Landmarks]
                         ↓
            [3D Transformation Pipeline]
                         ↓
              3D Environment Model (Objects + Positions)
                         ↓
Audio Stream → [Wake Word + Speech Recognition]
                         ↓
         Robot Control + Navigation (with BLE positioning)
```

---

## 3. GestureSpace Navigation System (Navign Integration) (1 minute / 60 seconds)

### Overview
GestureSpace integrates with **Navign**, our custom-built indoor positioning infrastructure, to provide precise location awareness and navigation capabilities for the gesture-controlled robot system.

#### 3.1 BLE Beacon-Based Indoor Positioning (15 seconds)
**Hardware:**
- ESP32-C3 microcontrollers with BLE capability
- Four beacon types: Merchant, Pathway, Connection, Turnstile
- Secure cryptographic communication (P-256 ECDSA signatures)

**Positioning:**
- RSSI-based triangulation
- Real-time distance calculation from multiple beacons
- Accuracy: <2 meters in typical mall environments

#### 3.2 Advanced Pathfinding Algorithm (15 seconds)
**Backend (Rust):**
- Dijkstra algorithm with bump allocation for ultra-fast routing
- Multi-floor navigation support (elevators, escalators, stairs)
- Dynamic area connectivity graph generation
- Point-to-point and merchant-based routing

**Performance:**
- Sub-millisecond pathfinding for typical mall layouts
- Handles complex multi-level buildings

#### 3.3 Secure Access Control (15 seconds)
**Cryptographic Security:**
- P-256 ECDSA signatures for all beacon communications
- TOTP-based authentication
- Nonce-based challenge-response (replay attack prevention)
- Hardware-secured keys in ESP32-C3 efuse storage

**Access Scenarios:**
- Door unlocking via mobile app
- Turnstile authentication
- Merchant space entry authorization

#### 3.4 Cross-Platform Mobile Experience (15 seconds)
**Technology:**
- Vue 3 + Tauri 2.0 framework
- Supports: iOS, Android, macOS, Windows, Linux
- MapLibre GL + Konva canvas for interactive maps
- Biometric authentication (Face ID, Touch ID, fingerprint)

**Features:**
- Real-time navigation overlay
- Turn-by-turn directions
- Merchant information and search
- Secure credential storage (Tauri Stronghold)

---

## 4. GestureSpace-Powered Robotic Assistant (1.5 minutes / 90 seconds)

### Robot Architecture Overview (20 seconds)

GestureSpace controls a quadruped robotic assistant designed as an accessible, affordable alternative to guide dogs.

#### Dual-Layer Design
**Upper Layer (Raspberry Pi / Jetson Nano):**
- ROS2 core for high-level coordination
- 6 subsystems integrated with GestureSpace: Vision, Audio, Bluetooth, Navign, Tasks, Serial
- **GestureSpace runs here** - processing gestures, voice, and spatial understanding

**Lower Layer (STM32 + Embassy Rust):**
- Real-time motor control
- 12-DOF servo management (3 per leg × 4 legs)
- Hardware abstraction layer
- Emergency stop and safety systems

### How GestureSpace Controls the Robot (25 seconds)

#### Vision-Based Control
- **Object Recognition**: "Bring me the bottle" → YOLOv8 detection
- **Gesture Commands**: Point to destination → Robot navigates
- **Voice + Gesture**: "Go there" + pointing → Combined input

#### Autonomous Navigation
- Combines Navign BLE positioning with camera localization
- AprilTag landmarks for precise pose correction
- Obstacle avoidance using object detection

#### Multimodal Feedback
- Audio confirmation of commands
- Visual LED indicators on robot
- Real-time status updates to mobile app

### Navigation System Integration (25 seconds)

#### BLE-Based Localization
- Robot equipped with BLE scanner
- Receives beacon signals for indoor positioning
- Synchronizes position with Navign server

#### Pathfinding & Execution
- Server sends optimal path to robot
- Robot follows waypoints with local obstacle avoidance
- Dynamic re-routing on path blockage

#### Multi-Floor Capability
- Autonomous elevator usage (future work)
- Stair/escalator detection and avoidance
- Floor transition coordination with Navign system

### Delivery & Assistance Features (20 seconds)

#### Guide Mode for Visually Impaired
- Robot acts as robotic guide dog
- Voice-guided navigation
- Obstacle detection and warning
- Physical guidance via haptic handle (future)

#### Delivery Mode
- Item transport in cargo bay
- Autonomous navigation to destination
- Secure delivery confirmation via app
- Return to charging station

#### Interaction Modes
- **Passive Following**: Robot follows user with BLE tracking
- **Active Guidance**: Robot leads user along optimal path
- **Fetch & Retrieve**: Voice command → Find object → Bring back

---

## Key Takeaways (Final Summary)

### The Complete GestureSpace Solution
1. **GestureSpace Core**: Camera pipeline for environment understanding (AprilTags + PnP solving + YOLOv12 + 3D transforms)
2. **Navign Integration**: BLE-based indoor positioning and secure access control infrastructure
3. **Robotic Assistant**: Autonomous guidance and delivery powered by GestureSpace spatial understanding

### Impact
- **Accessibility**: Empowering 17.31 million visually impaired people in China
- **Cost-Effective**: BLE-based solution vs. expensive UWB or traditional robots
- **Scalable**: Applicable to malls, hospitals, offices, transportation hubs
- **Secure**: Military-grade cryptography for all communications
- **Spatial Understanding**: Precise 3D environment mapping with ~5cm object localization

### Technology Stack Highlights
- **Languages**: Rust (backend/embedded), Python (AI/vision), TypeScript (mobile), Swift (iOS)
- **Frameworks**: ROS2, MediaPipe, YOLOv12 (Transformer-based), Tauri, Vue 3, PyTorch, OpenCV
- **Hardware**: ESP32-C3, Raspberry Pi/Jetson, STM32, cameras, AprilTags, BLE beacons

---

## Presentation Tips

### Timing Breakdown
- **Market (30s)**: Start with shocking statistics (17.31M vs. 400 guide dogs), end with GestureSpace solution
- **GestureSpace Core (120s)**: Focus on camera pipeline for environment understanding
  - Camera Pose (AprilTags + PnP): 25s
  - 3D Transformation: 25s
  - YOLOv12 Object Detection: 20s
  - Hand Landmarks & Pointing: 20s
  - Voice Wake Word: 15s
  - Speech Recognition: 15s
- **Navign Integration (60s)**: Show how GestureSpace leverages Navign BLE for positioning and security
- **Robot System (90s)**: Emphasize how GestureSpace powers the robot, show architecture, demo modes

### Visual Aids Recommended
- Market data infographic (17.31M vs. 400 guide dogs ratio)
- **GestureSpace camera pipeline demos**: Live or pre-recorded video showing:
  - AprilTag detection and camera pose estimation
  - 3D point transformation (2D image → 3D world coords)
  - YOLOv12 object detection with 3D localization
  - Hand landmark detection and finger pointing
  - Voice wake word activation
- Navign: BLE beacon placement and pathfinding visualization
- Robot: ROS2 architecture diagram from animations/robot.py
- **Integration diagram**: Camera pipeline + BLE positioning for robot control

### Demo Suggestions (Focus on Camera Pipeline)
1. **AprilTag pose estimation**: Show camera position tracking in real-time
2. **Object detection + 3D localization**: YOLOv12 detects object, system outputs 3D coordinates
3. **Finger pointing**: User points, system shows 3D direction vector
4. **Voice command**: "Hey GestureSpace, bring me that bottle" + object localization
5. **Mobile app navigation**: Show Navign BLE integration
6. **Robot delivery animation**: Full pipeline demonstration

### Q&A Preparation
- How does GestureSpace's camera pipeline work? (AprilTags + PnP solving + 3D transforms)
- What's the difference between YOLOv12 and YOLOv8? (Transformer-based vs. CNN-based)
- Is gesture classification used? (No, only hand landmark detection for pointing)
- Accuracy benchmarks: Camera pose (~2cm), object localization (~5cm), BLE positioning (<2m)
- Cost comparison: BLE beacons vs. UWB vs. traditional robots
- Scalability to different venue types and robot platforms
- Privacy: Is video/audio data stored? (No, processed locally)
- Security: Cryptographic protection details (P-256 ECDSA)
- Timeline for deployment and current stage
