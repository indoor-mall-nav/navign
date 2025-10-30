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

**Our Solution: GestureSpace** - An intelligent multimodal interaction system combining computer vision, gesture recognition, voice control, and BLE-based indoor navigation for accessible robotics

---

## 2. GestureSpace Core Techniques (2 minutes / 120 seconds)

### Overview
**GestureSpace** is a comprehensive computer vision and AI-based multimodal interaction system that enables intuitive, natural human-robot communication through gestures, voice, and spatial understanding. It serves as the intelligent interface layer for accessible robotics and indoor assistance.

### Core Technologies (6 key techniques)

#### 2.1 Hand Gesture Recognition (20 seconds)
**Technology Stack:**
- **MediaPipe Hands**: Real-time hand landmark detection
- **Custom CNN Classifier**: 4-class gesture recognition
  - Architecture: Conv2D → MaxPool2D → Fully Connected (40→64→4)
  - 70% dropout for regularization
  - Trained on augmented hand gesture dataset

**Key Features:**
- Detects 21 hand landmarks per hand
- Finger tip and MCP (metacarpophalangeal) joint tracking
- Supports rotation, flip augmentation for robust recognition

**Use Cases:**
- Point-and-command interface
- Direction indication for robot navigation
- Natural gesture-based control

#### 2.2 Finger Pointing Direction Detection (20 seconds)
**Implementation:**
- Extracts index finger landmarks (base MCP + tip)
- 3D spatial positioning using camera pose estimation
- Calculates normalized direction vectors in world coordinates

**Mathematical Process:**
```
1. Detect finger landmarks (u, v) in image space
2. Transform to 3D world coordinates using camera matrix K
3. Calculate direction vector: direction = (tip - base) / ||tip - base||
```

**Applications:**
- Point to objects for identification
- Indicate desired navigation direction
- Spatial interaction with environment

#### 2.3 Object Detection with YOLOv8 (15 seconds)
**Technology:**
- **Ultralytics YOLOv8 Large (yolo12l.pt)**
- Real-time object detection and classification
- Bounding box regression for object localization

**Outputs:**
- Object class names
- Confidence scores
- Bounding box coordinates (xyxy format)
- Center point calculation for 3D localization

#### 2.4 3D Localization & Camera Pose Estimation (25 seconds)
**AprilTag-Based Calibration:**
- 8 AprilTags (tag36h11 family) placed at known world positions
- Camera intrinsic parameters from chessboard calibration
- Real-time camera pose using solvePnP algorithm

**3D Point Reconstruction:**
```python
# Workflow:
1. Undistort image points using camera matrix K
2. Generate ray in camera coordinates
3. Transform ray to world coordinates using R_world
4. Intersect ray with ground plane (Z = Z0)
5. Calculate 3D world position
```

**Precision:**
- Camera position accuracy: ~2cm
- Object localization: ~5cm on ground plane

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
All components work together in real-time:
```
Camera Frame → [Hand Detection + Object Detection + AprilTag Localization]
                ↓
Audio Stream → [Wake Word] → [Speech Recognition]
                ↓
         Unified 3D Understanding
                ↓
         Robot Control Commands
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
1. **GestureSpace Core**: Natural, multimodal human-robot interaction through vision, voice, and gestures
2. **Navign Integration**: Precise indoor positioning and secure access control infrastructure
3. **Robotic Assistant**: Autonomous guidance and delivery powered by GestureSpace

### Impact
- **Accessibility**: Empowering 17.31 million visually impaired people in China
- **Cost-Effective**: BLE-based solution vs. expensive UWB or traditional robots
- **Scalable**: Applicable to malls, hospitals, offices, transportation hubs
- **Secure**: Military-grade cryptography for all communications
- **Natural Interaction**: Intuitive gesture and voice control anyone can use

### Technology Stack Highlights
- **Languages**: Rust (backend/embedded), Python (AI/vision), TypeScript (mobile), Swift (iOS)
- **Frameworks**: ROS2, MediaPipe, YOLOv8, Tauri, Vue 3, PyTorch, OpenCV
- **Hardware**: ESP32-C3, Raspberry Pi/Jetson, STM32, cameras, BLE beacons

---

## Presentation Tips

### Timing Breakdown
- **Market (30s)**: Start with shocking statistics (17.31M vs. 400 guide dogs), end with GestureSpace solution
- **GestureSpace Core (120s)**: Demonstrate each of 6 techniques with 1-2 sentence explanation + visual (20s each)
- **Navign Integration (60s)**: Show how GestureSpace leverages Navign for positioning and security
- **Robot System (90s)**: Emphasize how GestureSpace powers the robot, show architecture, demo modes

### Visual Aids Recommended
- Market data infographic (17.31M vs. 400 guide dogs ratio)
- **GestureSpace demos**: Live or pre-recorded video showing:
  - Hand tracking with MediaPipe
  - Finger pointing detection
  - Object recognition (YOLOv8)
  - AprilTag 3D localization
  - Voice wake word activation
- Navign: Beacon placement and pathfinding visualization
- Robot: ROS2 architecture diagram from animations/robot.py
- **Integration diagram**: GestureSpace controlling robot with Navign positioning

### Demo Suggestions (Focus on GestureSpace)
1. **Hand gesture demo**: Show real-time hand tracking and gesture classification
2. **Object detection + pointing**: User points at object, robot identifies it
3. **Voice command**: "Hey GestureSpace, bring me that bottle" + pointing
4. **Mobile app navigation**: Show Navign integration
5. **Robot delivery animation**: Full pipeline demonstration

### Q&A Preparation
- How does GestureSpace compare to other gesture control systems?
- Accuracy benchmarks for positioning (<2m) and localization (~5cm)
- Cost comparison: BLE beacons vs. UWB vs. traditional robots
- Scalability to different venue types and robot platforms
- Privacy: Is video/audio data stored? (No, processed locally)
- Security: Cryptographic protection details
- Timeline for deployment and current stage
