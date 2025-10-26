init:
  cargo install cargo-binstall
  cargo binstall cargo-deny cargo-shear typos-cli -y
  cargo clean
  corepack enable
  pnpm install
  cd animations && uv sync
  cd gesture_space && uv sync
  cd shared && cargo build
  cd beacon && cargo build --release
  cd mobile/src-tauri && cargo build
  cd server && cargo build
  cd maintenance-tool && cargo build

fmt:
  taplo format
  cd animations && uvx ruff format
  cd gesture_space && uvx ruff format
  cd shared && cargo fmt
  cd beacon && cargo fmt
  pnpm run --filter mobile format
  cd mobile/src-tauri && cargo fmt
  cd server && cargo fmt
  cd maintenance-tool && cargo fmt
  pnpm run --filter ts-schema format

lint:
  taplo lint
  just check
  cd animations && uvx ruff check
  cd gesture_space && uvx ruff check
  cd shared && cargo clippy -- -D warnings
  cd shared && cargo clippy --features heapless --no-default-features -- -D warnings
  cd shared && cargo clippy --features alloc --no-default-features -- -D warnings
  cd shared && cargo clippy --features crypto --features heapless --features serde --no-default-features -- -D warnings
  cd shared && cargo clippy --features base64 --features crypto --features alloc --features serde --no-default-features -- -D warnings
  cd beacon && cargo clippy -- -D warnings
  cd ts-schema && cargo clippy -- -D warnings
  pnpm run --filter mobile lint
  cd mobile/src-tauri && cargo clippy -- -D warnings
  cd server && cargo clippy --all-targets --all-features -- -D warnings
  cd maintenance-tool && cargo clippy --all-targets --all-features -- -D warnings

fix:
  cd animations && uvx ruff check --fix
  cd gesture_space && uvx ruff check --fix
  cd shared && cargo fix --allow-dirty
  cd shared && cargo fix --allow-dirty --features heapless --no-default-features
  cd shared && cargo fix --allow-dirty --features alloc --no-default-features
  cd shared && cargo fix --allow-dirty --features crypto --features heapless --features serde --no-default-features
  cd shared && cargo fix --allow-dirty --features base64 --features crypto --features alloc --features serde --no-default-features
  cd beacon && cargo fix --allow-dirty
  cd ts-schema && cargo fix --allow-dirty
  pnpm run --filter mobile lint:fix
  cd mobile/src-tauri && cargo fix --allow-dirty
  cd server && cargo fix --allow-dirty --all-targets --all-features
  cd maintenance-tool && cargo fix --allow-dirty --all-targets --all-features

test:
  echo "No tests for beacons yet..."
  cd shared && cargo test
  cd shared && cargo test --features heapless --no-default-features
  cd shared && cargo test --features alloc --no-default-features
  cd shared && cargo test --features crypto --features heapless --features serde --no-default-features
  cd shared && cargo test --features base64 --features crypto --features alloc --features serde --no-default-features
  cd mobile && just test
  cd server && cargo test
  cd maintenance-tool && cargo test

fmt-check:
  taplo format --diff
  cd animations && uvx ruff check --diff
  cd gesture_space && uvx ruff check --diff
  cd shared && cargo fmt -- --check
  cd beacon && cargo fmt -- --check
  cd mobile && just fmt-check
  cd server && cargo fmt -- --check
  cd maintenance-tool && cargo fmt -- --check

clean:
  cargo clean
  rm -rf mobile/.turbo
  rm -rf mobile/dist

clean-deps:
  just clean
  uvx cache clear
  rm -rf node_modules

check:
  cd animations && uvx ty check
  # FIXME: Enable gesture_space type checking
  # cd gesture_space && uvx ty check
  typos
  cargo deny check bans
  cargo deny check licenses
  cargo deny check sources
  pnpm ls-lint
  cd shared && cargo check
  cd shared && cargo check --features heapless --no-default-features
  cd shared && cargo check --features alloc --no-default-features
  cd shared && cargo check --features heapless --features serde --features crypto --no-default-features
  cd shared && cargo check --features base64 --features alloc --features serde --features crypto --no-default-features
  cd beacon && cargo check --release
  cd mobile/src-tauri && cargo check
  pnpm run --filter mobile check
  cd server && cargo check
  cd maintenance-tool && cargo check


ci:
  just init
  just roll
  just clean
  just test

roll:
  just fmt-check
  just lint
  just check
