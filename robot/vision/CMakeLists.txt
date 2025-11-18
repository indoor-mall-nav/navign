cmake_minimum_required(VERSION 3.20)
project(navign_vision VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Options
option(BUILD_TESTS "Build tests" ON)
option(USE_MEDIAPIPE "Enable MediaPipe hand tracking" ON)

# Find required packages
find_package(OpenCV QUIET)
if(NOT OpenCV_FOUND)
    message(FATAL_ERROR "OpenCV not found. Please install:\n"
                        "  Ubuntu/Debian: sudo apt-get install libopencv-dev\n"
                        "  macOS: brew install opencv\n"
                        "  Or set OpenCV_DIR to your OpenCV installation")
endif()

find_package(Protobuf QUIET)
if(NOT Protobuf_FOUND)
    message(FATAL_ERROR "Protobuf not found. Please install:\n"
                        "  Ubuntu/Debian: sudo apt-get install libprotobuf-dev protobuf-compiler\n"
                        "  macOS: brew install protobuf")
endif()

find_package(Threads REQUIRED)

# Zenoh C++ (header-only or find_package if installed)
find_package(zenohcxx QUIET)
if(NOT zenohcxx_FOUND)
    message(STATUS "Zenoh C++ not found, will use header-only from submodule")
    # TODO: Add zenoh-cpp as git submodule or FetchContent
endif()

# AprilTag library
find_library(APRILTAG_LIB apriltag)
find_path(APRILTAG_INCLUDE_DIR apriltag/apriltag.h)

if(NOT APRILTAG_LIB OR NOT APRILTAG_INCLUDE_DIR)
    message(FATAL_ERROR "AprilTag library not found. Please install:\n"
                        "  Ubuntu/Debian: sudo apt-get install libapriltag-dev\n"
                        "  macOS: brew install apriltag\n"
                        "  Or build from source: https://github.com/AprilRobotics/apriltag")
endif()

# ONNX Runtime for YOLO (alternative to PyTorch)
find_package(onnxruntime QUIET)
if(NOT onnxruntime_FOUND)
    message(WARNING "ONNX Runtime not found - YOLO detection will use OpenCV DNN")
endif()

# MediaPipe (optional)
if(USE_MEDIAPIPE)
    find_package(mediapipe QUIET)
    if(NOT mediapipe_FOUND)
        message(WARNING "MediaPipe not found - hand tracking will be disabled")
        set(USE_MEDIAPIPE OFF)
    endif()
endif()

# Generate protobuf files
set(PROTO_FILES
    ${CMAKE_CURRENT_SOURCE_DIR}/../proto/common.proto
    ${CMAKE_CURRENT_SOURCE_DIR}/../proto/vision.proto
)

protobuf_generate_cpp(PROTO_SRCS PROTO_HDRS ${PROTO_FILES})

# Include directories
include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${CMAKE_CURRENT_BINARY_DIR}  # For generated protobuf headers
    ${OpenCV_INCLUDE_DIRS}
    ${APRILTAG_INCLUDE_DIR}
    ${Protobuf_INCLUDE_DIRS}
)

# Source files
set(VISION_SOURCES
    src/main.cpp
    src/vision_service.cpp
    src/apriltag_detector.cpp
    src/object_detector.cpp
    src/camera_calibration.cpp
    src/coordinate_transform.cpp
    ${PROTO_SRCS}
)

if(USE_MEDIAPIPE)
    list(APPEND VISION_SOURCES src/hand_tracker.cpp)
    add_compile_definitions(USE_MEDIAPIPE)
endif()

# Main executable
add_executable(navign_vision ${VISION_SOURCES})

# Link libraries
target_link_libraries(navign_vision
    ${OpenCV_LIBS}
    ${APRILTAG_LIB}
    ${Protobuf_LIBRARIES}
    Threads::Threads
)

if(zenohcxx_FOUND)
    target_link_libraries(navign_vision zenohcxx::zenohc)
endif()

if(USE_MEDIAPIPE)
    target_link_libraries(navign_vision mediapipe::mediapipe)
endif()

if(onnxruntime_FOUND)
    target_link_libraries(navign_vision onnxruntime)
    add_compile_definitions(USE_ONNXRUNTIME)
endif()

# Install
install(TARGETS navign_vision DESTINATION bin)

# Tests
if(BUILD_TESTS)
    enable_testing()
    add_subdirectory(tests)
endif()

# Print configuration summary
message(STATUS "==============================================")
message(STATUS "Navign Vision C++ Configuration:")
message(STATUS "  OpenCV version: ${OpenCV_VERSION}")
message(STATUS "  Protobuf version: ${Protobuf_VERSION}")
message(STATUS "  AprilTag: ${APRILTAG_LIB}")
message(STATUS "  MediaPipe: ${USE_MEDIAPIPE}")
message(STATUS "  ONNX Runtime: ${onnxruntime_FOUND}")
message(STATUS "  Zenoh C++: ${zenohcxx_FOUND}")
message(STATUS "==============================================")
