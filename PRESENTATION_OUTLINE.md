# Navign Project Presentation Outline

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

**Our Solution:** Bluetooth Low Energy (BLE) based indoor navigation system combining gesture control and robotics

---

## 2. Gesture Space Techniques (2 minutes / 120 seconds)

### Overview
Computer vision-based multimodal interaction system for intuitive human-robot communication

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

## 3. Navign Project (1 minute / 60 seconds)

### Core System Architecture

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

## 4. Integrated Robot System (1.5 minutes / 90 seconds)

### Robot Architecture Overview (20 seconds)

#### Dual-Layer Design
**Upper Layer (Raspberry Pi / Jetson Nano):**
- ROS2 core for high-level coordination
- 6 subsystems: Vision, Audio, Bluetooth, Navign, Tasks, Serial

**Lower Layer (STM32 + Embassy Rust):**
- Real-time motor control
- 12-DOF servo management (3 per leg × 4 legs)
- Hardware abstraction layer
- Emergency stop and safety systems

### Integration with Gesture Space (25 seconds)

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

### The Complete Solution
1. **Gesture Space**: Natural, multimodal human-robot interaction
2. **Navign**: Precise indoor positioning and secure access control
3. **Integrated Robot**: Autonomous assistance and delivery

### Impact
- **Accessibility**: Empowering 17.31 million visually impaired people
- **Cost-Effective**: BLE-based solution vs. expensive alternatives
- **Scalable**: Applicable to malls, hospitals, offices, transportation hubs
- **Secure**: Military-grade cryptography for all communications

### Technology Stack Highlights
- **Languages**: Rust (backend/embedded), Python (AI/vision), TypeScript (mobile), Swift (iOS)
- **Frameworks**: ROS2, Tauri, Vue 3, MediaPipe, YOLOv8
- **Hardware**: ESP32-C3, Raspberry Pi, STM32, cameras, BLE beacons

---

## Presentation Tips

### Timing Breakdown
- **Market (30s)**: Start with shocking statistics, end with problem statement
- **Gesture Space (120s)**: Demonstrate each technique with 1-2 sentence explanation + visual
- **Navign (60s)**: Focus on positioning accuracy and security features
- **Robot (90s)**: Show architecture diagram, emphasize integration, demo delivery mode

### Visual Aids Recommended
- Market data infographic (17.31M vs. 400 guide dogs)
- Gesture Space: Live demo or pre-recorded video of hand tracking
- Navign: Interactive map with beacon placement and pathfinding
- Robot: Architecture diagram from animations/robot.py
- Integration diagram: All three systems working together

### Demo Suggestions
1. Show hand gesture controlling virtual robot
2. Display object detection + finger pointing in real-time
3. Demonstrate mobile app navigation
4. Play robot delivery animation

### Q&A Preparation
- Cost comparison with traditional solutions
- Accuracy benchmarks for positioning
- Scalability to different venue types
- Privacy and security considerations
- Timeline for deployment
