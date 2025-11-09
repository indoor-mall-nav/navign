#!/bin/bash
# Generate Python code from protobuf definitions

set -e

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Generating Python protobuf code from orchestrator task.proto..."

# Generate Python protobuf and gRPC code from orchestrator's task.proto
python -m grpc_tools.protoc \
    --proto_path=../tower/proto \
    --python_out=proto \
    --grpc_python_out=proto \
    --pyi_out=proto \
    ../tower/proto/task.proto

echo "✓ Generated proto/task_pb2.py (from orchestrator)"
echo "✓ Generated proto/task_pb2_grpc.py (from orchestrator)"
echo "✓ Generated proto/task_pb2.pyi (from orchestrator)"
echo ""
echo "Protobuf generation complete!"
