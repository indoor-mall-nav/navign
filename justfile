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

roll:
  just fmt-check
  just lint
  just check

# Selective CI based on modified files
# If shared/ is modified, run all Rust CI tasks
ci-selective base="main":
  #!/usr/bin/env bash
  set -euo pipefail

  echo "Checking for modified files compared to {{base}}..."

  # Get list of modified files
  modified_files=$(git diff --name-only {{base}}...HEAD || echo "")

  if [ -z "$modified_files" ]; then
    echo "No files modified. Skipping CI."
    exit 0
  fi

  echo "Modified files:"
  echo "$modified_files"
  echo ""

  # Initialize flags for each CI task
  run_shared=false
  run_server=false
  run_beacon=false
  run_mobile=false
  run_tower=false
  run_orchestrator=false
  run_repo=false

  # Check which components are affected
  while IFS= read -r file; do
    case "$file" in
      shared/*)
        run_shared=true
        # If shared is modified, run all Rust CI
        run_server=true
        run_beacon=true
        run_orchestrator=true
        ;;
      server/*)
        run_server=true
        ;;
      beacon/*)
        run_beacon=true
        ;;
      mobile/*)
        run_mobile=true
        ;;
      admin/tower/*)
        run_tower=true
        ;;
      admin/orchestrator/*)
        run_orchestrator=true
        ;;
      justfile|*.toml|*.md|.github/*|deny.toml|.typos.toml|package.json|pnpm-workspace.yaml|pnpm-lock.yaml)
        run_repo=true
        ;;
      maintenance-tool/*)
        # Maintenance tool changes don't require CI
        ;;
      ts-schema/*)
        # TypeScript schema generator - might affect mobile
        run_mobile=true
        ;;
      gesture_space/*|animations/*|vision/*|miniapp/*|docs/*|presentation/*|schematics/*)
        # These components don't have CI tasks yet
        ;;
    esac
  done <<< "$modified_files"

  # Run the appropriate CI tasks
  echo "Running selective CI tasks..."
  echo ""

  if [ "$run_repo" = true ]; then
    echo "=== Running ci-repo ==="
    just ci-repo
  fi

  if [ "$run_shared" = true ]; then
    echo "=== Running ci-shared ==="
    just ci-shared
  fi

  if [ "$run_server" = true ]; then
    echo "=== Running ci-server ==="
    just ci-server
  fi

  if [ "$run_beacon" = true ]; then
    echo "=== Running ci-beacon ==="
    just ci-beacon
  fi

  if [ "$run_mobile" = true ]; then
    echo "=== Running ci-mobile ==="
    just ci-mobile
  fi

  if [ "$run_tower" = true ]; then
    echo "=== Running ci-tower ==="
    just ci-tower
  fi

  if [ "$run_orchestrator" = true ]; then
    echo "=== Running ci-orchestrator ==="
    just ci-orchestrator
  fi

  echo ""
  echo "✓ Selective CI completed successfully!"
