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
  taplo format
  cd animations && uvx ruff format
  cd gesture_space && uvx ruff format
  pnpm run --filter mobile format
  pnpm run --filter ts-schema format
  cargo fmt

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
  cd shared && cargo clippy --features mongodb --features serde --features crypto
  cd shared && cargo clippy --features sql --features serde --features crypto
  cd beacon && cargo clippy -- -D warnings
  cd ts-schema && cargo clippy -- -D warnings
  pnpm run --filter mobile lint
  cd mobile/src-tauri && cargo clippy -- -D warnings
  cd server && cargo clippy --all-targets --all-features -- -D warnings
  cd maintenance-tool && cargo clippy --all-targets --all-features -- -D warnings

test:
  echo "No tests for beacons yet..."
  cd shared && cargo test
  cd shared && cargo test --features heapless --no-default-features
  cd shared && cargo test --features alloc --no-default-features
  cd shared && cargo test --features crypto --features heapless --features serde --no-default-features
  cd shared && cargo test --features base64 --features crypto --features alloc --features serde --no-default-features
  cd shared && cargo test --features mongodb --features serde --features crypto
  cd shared && cargo test --features sql --features serde --features crypto
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

ci-server:
  cd server && cargo check
  cd server && cargo fmt -- --check
  cd server && cargo clippy -- -D warnings
  cd server && cargo test

ci-beacon:
  cd beacon && cargo check --release
  cd beacon && cargo fmt -- --check
  cd beacon && cargo clippy --release -- -D warnings
  # cd beacon && cargo test --release
  echo "No tests for beacons yet..."

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
  cd shared && cargo check --features heapless --features serde --features crypto --no-default-features
  cd shared && cargo check --features base64 --features alloc --features serde --features crypto --no-default-features
  cd shared && cargo check --features mongodb --features serde --features crypto
  cd shared && cargo check --features sql --features serde --features crypto
  cd shared && cargo fmt -- --check
  cd shared && cargo clippy -- -D warnings
  cd shared && cargo clippy --features heapless --no-default-features
  cd shared && cargo clippy --features alloc --no-default-features
  cd shared && cargo clippy --features heapless --features serde --features crypto --no-default-features
  cd shared && cargo clippy --features base64 --features alloc --features serde --features crypto --no-default-features
  cd shared && cargo clippy --features mongodb --features serde --features crypto
  cd shared && cargo clippy --features sql --features serde --features crypto
  cd shared && cargo test

ci-repo:
  taplo format --diff
  typos
  cargo deny check bans
  cargo deny check licenses
  cargo deny check sources

roll:
  just fmt-check
  just lint
  just check
