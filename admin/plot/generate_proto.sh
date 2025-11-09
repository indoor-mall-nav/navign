#!/bin/bash
# Generate Python code from protobuf definitions

set -e

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Generating Python protobuf code from admin plot.proto..."

# Generate Python protobuf and gRPC code from admin's plot.proto
python -m grpc_tools.protoc \
    --proto_path=../proto \
    --python_out=proto \
    --grpc_python_out=proto \
    --pyi_out=proto \
    ../proto/plot.proto

echo "✓ Generated proto/plot_pb2.py"
echo "✓ Generated proto/plot_pb2_grpc.py"
echo "✓ Generated proto/plot_pb2.pyi"
echo ""
echo "Protobuf generation complete!"
