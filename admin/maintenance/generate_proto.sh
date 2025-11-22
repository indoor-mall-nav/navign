#!/usr/bin/env bash
# Generate Python gRPC code from protobuf definitions

uv sync

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROTO_DIR="$SCRIPT_DIR/../proto"
OUTPUT_DIR="$SCRIPT_DIR/proto"

echo "Generating Python protobuf code..."
echo "Proto dir: $PROTO_DIR"
echo "Output dir: $OUTPUT_DIR"

# Create output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"

# Create __init__.py
touch "$OUTPUT_DIR/__init__.py"

# Generate Python code for sync.proto (includes beacon registration)
uv run python3 -m grpc_tools.protoc \
  --proto_path="$PROTO_DIR" \
  --python_out="$OUTPUT_DIR" \
  --grpc_python_out="$OUTPUT_DIR" \
  --pyi_out="$OUTPUT_DIR" \
  "$PROTO_DIR/sync.proto"

echo "✅ Protobuf code generated successfully!"
echo "Generated files:"
ls -lh "$OUTPUT_DIR"/*.py "$OUTPUT_DIR"/*.pyi 2>/dev/null || true
