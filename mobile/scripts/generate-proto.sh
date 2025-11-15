#!/bin/bash

# Script to generate TypeScript code from proto files for gRPC-Web

set -e

# Directories
PROTO_DIR="../admin/proto"
OUT_DIR="src/lib/grpc"

# Create output directory if it doesn't exist
mkdir -p "$OUT_DIR"

# Find protoc
if command -v protoc &> /dev/null; then
    PROTOC=protoc
elif [ -f "node_modules/.bin/grpc_tools_node_protoc" ]; then
    PROTOC="node_modules/.bin/grpc_tools_node_protoc"
else
    echo "❌ protoc not found! Please install protobuf compiler or grpc-tools"
    exit 1
fi

# Generate JavaScript code + TypeScript definitions with gRPC-Web
$PROTOC \
  --js_out=import_style=commonjs,binary:"$OUT_DIR" \
  --grpc-web_out=import_style=typescript,mode=grpcwebtext:"$OUT_DIR" \
  --proto_path="$PROTO_DIR" \
  --proto_path="/usr/include" \
  --proto_path="/usr/local/include" \
  "$PROTO_DIR/admin.proto"

echo "✅ Proto code generation complete!"
echo "Generated files in $OUT_DIR/"
ls -lh "$OUT_DIR"
