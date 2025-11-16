init:
  cargo install cargo-binstall
  cargo binstall cargo-deny cargo-shear typos-cli -y
  cargo clean
  corepack enable
  pnpm install
  cd animations && uv sync
  cd gesture_space && uv sync
  cargo check

fmt:
  pnpm run --filter mobile format
  cargo fmt
  uvx ruff format
  gofmt -w .

lint:
  taplo lint
  just check
  cd animations && uvx ruff check
  cd gesture_space && uvx ruff check
  cd robot/vision && uvx ruff check
  cd robot/audio && uvx ruff check
  cd admin/plot && uvx ruff check
  cd shared && cargo clippy -- -D warnings
  cd shared && cargo clippy --features heapless --no-default-features -- -D warnings
  cd shared && cargo clippy --features alloc --no-default-features -- -D warnings
  cd shared && cargo clippy --features crypto,heapless,serde --no-default-features -- -D warnings
  cd shared && cargo clippy --features base64,crypto,alloc,serde --no-default-features -- -D warnings
  cd shared && cargo clippy --features mongodb,serde,crypto
  cd shared && cargo clippy --features sql,serde,crypto
  cd shared && cargo clippy --features postgres,sql,serde,crypto
  cd shared && cargo clippy --features geo,alloc,serde -- -D warnings
  cd proc_macros && cargo clippy -- -D warnings
  cd firmware && cargo clippy -- -D warnings
  cd ts-schema && cargo clippy -- -D warnings
  pnpm run --filter mobile lint
  cd mobile/src-tauri && cargo clippy -- -D warnings
  cd server && cargo clippy --all-targets --all-features -- -D warnings
  cd admin/maintenance && cargo clippy --all-targets --all-features -- -D warnings
  cd robot/firmware && cargo clippy -- -D warnings
  cd robot/scheduler && cargo clippy -- -D warnings
  cd robot/network && cargo clippy -- -D warnings
  cd robot/serial && cargo clippy -- -D warnings

# Run firmware mock-based tests (fast, runs on host)
test-firmware-mocks:
  cd firmware && cargo test --test nonce_tests --target x86_64-unknown-linux-gnu --features std
  cd firmware && cargo test --test crypto_tests --target x86_64-unknown-linux-gnu --features std
  cd firmware && cargo test --test rate_limit_tests --target x86_64-unknown-linux-gnu --features std

# Run firmware tests in QEMU simulator (requires QEMU installation)
test-firmware-qemu:
  #!/usr/bin/env bash
  set -e
  echo "Building firmware for QEMU..."
  cd firmware && cargo build --release
  echo "Starting QEMU simulation..."
  cd firmware && ./tests/qemu_runner.sh

# Run all firmware tests (mocks + QEMU)
test-firmware-all:
  just test-firmware-mocks
  just test-firmware-qemu

fmt-check:
  taplo format --diff
  cargo fmt -- --check
  pnpm run --filter mobile format --check
  uvx ruff format --check
  test -z "$(gofmt -l .)" || (echo "Go code is not formatted:" && gofmt -d . && exit 1)

clean:
  cargo clean
  rm -rf mobile/.turbo
  rm -rf mobile/dist

clean-deps:
  just clean
  uvx cache clear
  rm -rf node_modules

ci-server:
  cd server && cargo check
  cd server && cargo fmt -- --check
  cd server && cargo clippy -- -D warnings
  cd server && cargo test -- --include-ignored

ci-firmware:
  cd firmware && cargo check --release
  cd firmware && cargo fmt -- --check
  cd firmware && cargo clippy --release -- -D warnings
  # Note: firmware mock tests disabled - need lib.rs extraction (see firmware/TESTING.md)
  # just test-firmware-mocks

ci-mobile:
  corepack enable
  pnpm install
  cd mobile && just check
  cd mobile && just fmt-check
  cd mobile && just lint
  cd mobile && just test

ci-desktop:
  echo "No desktop-specific CI tasks yet..."

ci-shared:
  cd shared && cargo check
  cd shared && cargo check --features heapless --no-default-features
  cd shared && cargo check --features alloc --no-default-features
  cd shared && cargo check --features heapless,serde,crypto --no-default-features
  cd shared && cargo check --features base64,alloc,serde,crypto --no-default-features
  cd shared && cargo check --features mongodb,serde,crypto
  cd shared && cargo check --features sql,serde,crypto
  cd shared && cargo check --features postgres,sql,serde,crypto
  cd shared && cargo check --features geo,alloc,serde
  cd shared && cargo fmt -- --check
  cd shared && cargo clippy -- -D warnings
  cd shared && cargo clippy --features heapless --no-default-features
  cd shared && cargo clippy --features alloc --no-default-features
  cd shared && cargo clippy --features heapless,serde,crypto --no-default-features
  cd shared && cargo clippy --features base64,alloc,serde,crypto --no-default-features
  cd shared && cargo clippy --features mongodb,serde,crypto
  cd shared && cargo clippy --features sql,serde,crypto
  cd shared && cargo clippy --features postgres,sql,serde,crypto
  cd shared && cargo clippy --features geo,alloc,serde -- -D warnings
  cd shared && cargo test
  cd shared && cargo test --features geo,alloc,serde
  cd shared && cargo test --features postgres,sql,serde,crypto

ci-proc-macros:
  cd proc_macros && cargo check
  cd proc_macros && cargo fmt -- --check
  cd proc_macros && cargo clippy -- -D warnings
  cd proc_macros && cargo test

ci-repo:
  taplo format --diff
  typos

ci-tower:
  cd admin/tower && go mod download
  cd admin/tower && go mod verify
  cd admin/tower && go build -v ./...
  cd admin/tower && go test -v ./...
  cd admin/tower && test -z "$(gofmt -l .)" || (echo "Go code is not formatted:" && gofmt -d . && exit 1)
  cd admin/tower && go vet ./...

ci-orchestrator:
  cd admin/orchestrator && cargo check
  cd admin/orchestrator && cargo fmt -- --check
  cd admin/orchestrator && cargo clippy -- -D warnings
  cd admin/orchestrator && cargo test

ci-plot:
  cd admin/plot && uv sync --extra dev
  cd admin/plot && uvx ruff format --check
  cd admin/plot && uvx ruff check
  cd admin/plot && uv run pytest

ci-maintenance:
  cd admin/maintenance && cargo check
  cd admin/maintenance && cargo fmt -- --check
  cd admin/maintenance && cargo clippy -- -D warnings
  cd admin/maintenance && cargo test

ci-gesture-space:
  cd gesture_space && uv sync
  cd gesture_space && uvx ruff format --check
  cd gesture_space && uvx ruff check
  cd gesture_space && uv run pytest tests/

ci-ts-schema:
  cd ts-schema && cargo check
  cd ts-schema && cargo fmt -- --check
  cd ts-schema && cargo clippy -- -D warnings
  # Note: ts-schema is a NAPI module (cdylib), tested via Node.js, not Rust unit tests
  echo "ts-schema is a NAPI module - use Node.js tests for integration testing"

ci-robot-firmware:
  cd robot/firmware && cargo check --release
  cd robot/firmware && cargo fmt -- --check
  cd robot/firmware && cargo clippy --release -- -D warnings
  # Note: Embedded testing requires hardware or QEMU setup
  echo "No tests for robot/firmware yet (requires hardware/QEMU)"

ci-robot-audio:
  cd robot/audio && uv sync
  cd robot/audio && uvx ruff check
  cd robot/audio && uvx ruff format --check
  # cd robot/audio && uv run pytest

ci-robot-vision:
  cd robot/vision && uv sync
  cd robot/vision && uvx ruff check
  cd robot/vision && uvx ruff format --check
  # cd robot/vision && uv run pytest

ci-robot-scheduler:
  cd robot/scheduler && cargo check
  cd robot/scheduler && cargo fmt -- --check
  cd robot/scheduler && cargo clippy -- -D warnings
  cd robot/scheduler && cargo test

ci-robot-network:
  cd robot/network && cargo check
  cd robot/network && cargo fmt -- --check
  cd robot/network && cargo clippy -- -D warnings
  cd robot/network && cargo test

ci-robot-serial:
  cd robot/serial && cargo check
  cd robot/serial && cargo fmt -- --check
  cd robot/serial && cargo clippy -- -D warnings
  cd robot/serial && cargo test

ci-robot-lower: ci-robot-firmware
ci-robot-upper: ci-robot-vision ci-robot-audio ci-robot-scheduler ci-robot-network ci-robot-serial

ci-robot: ci-robot-lower ci-robot-upper

ci-admin: ci-tower ci-orchestrator ci-plot ci-maintenance

roll:
  just fmt-check
  just lint
  just check

build-firmware:
  cargo install espup --locked
  espup install

  cd firmware && cargo build --release

proto-tower:
  cd admin/tower && protoc --proto_path=../proto --go_out=. --go_opt=paths=source_relative --go-grpc_out=. --go-grpc_opt=paths=source_relative ../proto/task.proto

proto-plot:
  cd admin/plot && python -m grpc_tools.protoc --proto_path=../proto --python_out=proto --grpc_python_out=proto --pyi_out=proto ../proto/plot.proto

proto-robot-python:
  cd robot/proto && ./generate_python.sh

proto-robot: proto-robot-python
  @echo "Robot Rust proto generation happens via build.rs during compilation"

proto: proto-tower proto-plot proto-robot

clean-proto:
  rm -f admin/tower/proto/*.pb.go
  rm -f admin/plot/proto/plot_pb2.py admin/plot/proto/plot_pb2_grpc.py admin/plot/proto/plot_pb2.pyi
  rm -f robot/vision/*_pb2.py robot/vision/*_pb2.pyi
  rm -f robot/audio/*_pb2.py robot/audio/*_pb2.pyi

# Generate TypeScript type definitions from Rust schemas
gen-ts-schema:
  @echo "Generating TypeScript definitions from Rust schemas..."
  cd shared && cargo test --features ts-rs --quiet
  @echo "Copying generated files to mobile/src/schema/generated/..."
  mkdir -p mobile/src/schema/generated
  cp ts-schema/bindings/generated/*.ts mobile/src/schema/generated/
  @echo "✓ TypeScript schemas generated successfully"
  @echo "  Output: mobile/src/schema/generated/"
  @ls mobile/src/schema/generated/ | wc -l | xargs echo "  Files:"
