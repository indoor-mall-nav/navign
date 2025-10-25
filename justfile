init:
  cd animations && uv sync
  cd gesture_space && uv sync
  cd beacon && cargo build
  cd mobile && just init
  cd server && cargo build
  cd maintenance-tool && cargo build

fmt:
  cd animations && uvx ruff format
  cd gesture_space && uvx ruff format
  cd beacon && cargo fmt
  cd mobile && just fmt
  cd server && cargo fmt
  cd maintenance-tool && cargo fmt

lint:
  cd animations && uvx ruff check
  cd gesture_space && uvx ruff check
  cd beacon && cargo clippy -- -D warnings
  cd mobile && just lint
  cd server && cargo clippy --all-targets --all-features -- -D warnings
  cd maintenance-tool && cargo clippy --all-targets --all-features -- -D warnings

fix:
  cd animations && uvx ruff fix
  cd gesture_space && uvx ruff fix
  cd beacon && cargo fix --allow-dirty
  cd mobile && just fix
  cd server && cargo fix --allow-dirty --all-targets --all-features
  cd maintenance-tool && cargo fix --allow-dirty --all-targets --all-features

test:
  echo "No tests for beacons yet..."
  cd mobile && just test
  cd server && cargo test
  cd maintenance-tool && cargo test

fmt-check:
  cd animations && uvx ruff check --diff
  cd gesture_space && uvx ruff check --diff
  cd beacon && cargo fmt -- --check
  cd mobile && just fmt-check
  cd server && cargo fmt -- --check
  cd maintenance-tool && cargo fmt -- --check

clean:
  cargo clean
  rm -rf mobile/.turbo
  rm -rf mobile/dist

check:
  cd animations && uvx ty check
  # FIXME: Enable gesture_space type checking
  # cd gesture_space && uvx ty check
  cd beacon && cargo check
  cd mobile && just check
  cd server && cargo check
  cd maintenance-tool && cargo check
