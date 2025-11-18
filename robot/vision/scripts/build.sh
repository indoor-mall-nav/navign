#!/bin/bash
set -e

# Build script for Navign Vision C++

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_DIR="$SCRIPT_DIR/.."
BUILD_DIR="$PROJECT_DIR/build"

# Parse arguments
BUILD_TYPE="Release"
USE_ONNX="OFF"
USE_MEDIAPIPE="OFF"
CLEAN=false
JOBS=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

while [[ $# -gt 0 ]]; do
    case $1 in
        --debug)
            BUILD_TYPE="Debug"
            shift
            ;;
        --onnx)
            USE_ONNX="ON"
            shift
            ;;
        --mediapipe)
            USE_MEDIAPIPE="ON"
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --jobs|-j)
            JOBS="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --debug           Build in debug mode (default: Release)"
            echo "  --onnx            Enable ONNX Runtime support"
            echo "  --mediapipe       Enable MediaPipe support"
            echo "  --clean           Clean build directory before building"
            echo "  --jobs, -j N      Number of parallel jobs (default: $JOBS)"
            echo "  --help, -h        Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "========================================="
echo "Navign Vision C++ Build Script"
echo "========================================="
echo "Build type: $BUILD_TYPE"
echo "ONNX Runtime: $USE_ONNX"
echo "MediaPipe: $USE_MEDIAPIPE"
echo "Parallel jobs: $JOBS"
echo "========================================="

# Clean if requested
if [ "$CLEAN" = true ]; then
    echo "Cleaning build directory..."
    rm -rf "$BUILD_DIR"
fi

# Create build directory
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

# Configure
echo "Configuring..."
cmake .. \
    -DCMAKE_BUILD_TYPE="$BUILD_TYPE" \
    -DUSE_ONNXRUNTIME="$USE_ONNX" \
    -DUSE_MEDIAPIPE="$USE_MEDIAPIPE"

# Build
echo "Building..."
cmake --build . -j "$JOBS"

echo "========================================="
echo "Build complete!"
echo "Binary: $BUILD_DIR/navign_vision"
echo "========================================="
