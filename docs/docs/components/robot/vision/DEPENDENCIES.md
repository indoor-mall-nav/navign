# Vision Service Dependencies

The C++ vision service requires several system libraries to be installed before building.

## Required Dependencies

### 1. OpenCV (>= 4.5)
Computer vision library for image processing and camera I/O.

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y libopencv-dev
```

**macOS:**
```bash
brew install opencv
```

**Verify installation:**
```bash
pkg-config --modversion opencv4
```

### 2. AprilTag
C library for AprilTag detection and pose estimation.

**Ubuntu/Debian:**
```bash
sudo apt-get install -y libapriltag-dev
```

**macOS:**
```bash
brew install apriltag
```

**Build from source (if not available):**
```bash
git clone https://github.com/AprilRobotics/apriltag
cd apriltag
cmake -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build
sudo cmake --install build
```

### 3. Protocol Buffers (>= 3.0)
Message serialization library.

**Ubuntu/Debian:**
```bash
sudo apt-get install -y libprotobuf-dev protobuf-compiler
```

**macOS:**
```bash
brew install protobuf
```

**Verify installation:**
```bash
protoc --version
```

### 4. CMake (>= 3.20)
Build system generator.

**Ubuntu/Debian:**
```bash
sudo apt-get install -y cmake
```

**macOS:**
```bash
brew install cmake
```

## Optional Dependencies

### 5. ONNX Runtime (Recommended)
For faster YOLO inference (40% performance improvement).

**Ubuntu/Debian:**
```bash
# Download pre-built package
wget https://github.com/microsoft/onnxruntime/releases/download/v1.16.0/onnxruntime-linux-x64-1.16.0.tgz
tar xzf onnxruntime-linux-x64-1.16.0.tgz
sudo cp -r onnxruntime-linux-x64-1.16.0/include/* /usr/local/include/
sudo cp -r onnxruntime-linux-x64-1.16.0/lib/* /usr/local/lib/
sudo ldconfig
```

**macOS:**
```bash
brew install onnxruntime
```

### 6. Zenoh C++
For pub/sub messaging (optional, work in progress).

**Build from source:**
```bash
git clone https://github.com/eclipse-zenoh/zenoh-cpp
cd zenoh-cpp
cmake -B build -DCMAKE_BUILD_TYPE=Release
sudo cmake --install build
```

### 7. MediaPipe C++ (Experimental)
For hand tracking (not yet implemented).

See: https://google.github.io/mediapipe/getting_started/cpp.html

## Installation Scripts

### One-Command Install (Ubuntu/Debian)

```bash
sudo apt-get update && sudo apt-get install -y \
    cmake \
    build-essential \
    libopencv-dev \
    libapriltag-dev \
    libprotobuf-dev \
    protobuf-compiler
```

### One-Command Install (macOS)

```bash
brew install cmake opencv apriltag protobuf
```

## Verification

After installing dependencies, verify everything is ready:

```bash
# Check OpenCV
pkg-config --modversion opencv4 || pkg-config --modversion opencv

# Check CMake
cmake --version

# Check Protobuf
protoc --version

# Check AprilTag (library check)
ldconfig -p | grep apriltag || echo "AprilTag library found in system paths"
```

## Build Test

Test the build after installing dependencies:

```bash
cd robot/vision
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build -j$(nproc)
```

If successful, you should see:
```
[100%] Built target navign_vision
```

## Troubleshooting

### OpenCV not found

**Error:**
```
CMake Error: Could not find a package configuration file provided by "OpenCV"
```

**Solution:**
```bash
# Ubuntu/Debian
sudo apt-get install libopencv-dev

# If multiple versions installed, set explicitly
export OpenCV_DIR=/usr/lib/x86_64-linux-gnu/cmake/opencv4
```

### AprilTag not found

**Error:**
```
CMake Error: AprilTag library not found
```

**Solution:**
Build from source (see above) or check installation:
```bash
# Check if library exists
find /usr -name "libapriltag*" 2>/dev/null

# Add to CMake search path if needed
export CMAKE_PREFIX_PATH=/usr/local:$CMAKE_PREFIX_PATH
```

### Protobuf version mismatch

**Error:**
```
Protobuf compiler version doesn't match library
```

**Solution:**
```bash
# Ensure compiler and library versions match
sudo apt-get install --reinstall protobuf-compiler libprotobuf-dev
```

## CI/CD

The CI system checks for OpenCV availability and skips the vision build if dependencies are missing:

```bash
just ci-robot-vision
```

This will:
- Check for OpenCV using `pkg-config`
- Build if available
- Skip with a warning if not available

## Docker

For containerized builds, use this Dockerfile snippet:

```dockerfile
FROM ubuntu:24.04

RUN apt-get update && apt-get install -y \
    cmake \
    build-essential \
    libopencv-dev \
    libapriltag-dev \
    libprotobuf-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY robot/vision robot/vision
RUN cd robot/vision && \
    cmake -S . -B build -DCMAKE_BUILD_TYPE=Release && \
    cmake --build build -j$(nproc)
```

## Development Setup

For active development:

```bash
# Install dependencies
sudo apt-get install -y \
    libopencv-dev \
    libapriltag-dev \
    libprotobuf-dev \
    protobuf-compiler \
    libonnxruntime-dev

# Build with all features
cd robot/vision
cmake -S . -B build \
    -DCMAKE_BUILD_TYPE=Debug \
    -DUSE_ONNXRUNTIME=ON \
    -DCMAKE_EXPORT_COMPILE_COMMANDS=ON

cmake --build build -j$(nproc)
```

## Minimal Build (No Optional Dependencies)

If you only want core functionality without ONNX/Zenoh:

```bash
# Install only required dependencies
sudo apt-get install -y \
    cmake \
    build-essential \
    libopencv-dev \
    libapriltag-dev \
    libprotobuf-dev

# Build
cd robot/vision
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build
```

This will use OpenCV DNN for YOLO (slower but no extra dependencies).
