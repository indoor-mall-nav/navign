#!/usr/bin/env bash
# Generate Python protobuf code from proto files

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROTO_DIR="$SCRIPT_DIR"

# Output directories
VISION_OUT="../vision"
AUDIO_OUT="../audio"

echo "Generating Python protobuf code..."

# Generate for vision
python -m grpc_tools.protoc \
    -I"$PROTO_DIR" \
    --python_out="$VISION_OUT" \
    --pyi_out="$VISION_OUT" \
    "$PROTO_DIR/common.proto" \
    "$PROTO_DIR/vision.proto"

# Generate for audio
python -m grpc_tools.protoc \
    -I"$PROTO_DIR" \
    --python_out="$AUDIO_OUT" \
    --pyi_out="$AUDIO_OUT" \
    "$PROTO_DIR/common.proto" \
    "$PROTO_DIR/audio.proto"

echo "Python protobuf code generated successfully"
