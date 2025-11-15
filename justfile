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
  cd animations && uvx ruff format
  cd gesture_space && uvx ruff format
  pnpm run --filter mobile format
  cargo fmt
  gofmt -w .

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
  cd firmware && cargo clippy -- -D warnings
  cd ts-schema && cargo clippy -- -D warnings
  pnpm run --filter mobile lint
  cd mobile/src-tauri && cargo clippy -- -D warnings
  cd server && cargo clippy --all-targets --all-features -- -D warnings
  cd maintenance-tool && cargo clippy --all-targets --all-features -- -D warnings

test:
  echo "No tests for firmware yet..."
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
  cd firmware && cargo fmt -- --check
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

ci-firmware:
  cd firmware && cargo check --release
  cd firmware && cargo fmt -- --check
  cd firmware && cargo clippy --release -- -D warnings
  # cd firmware && cargo test --release
  echo "No tests for firmware yet..."

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

ci-plot:
  cd admin/plot && uv sync --extra dev
  cd admin/plot && uvx ruff format --check
  cd admin/plot && uvx ruff check
  cd admin/plot && uv run pytest

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

proto: proto-tower proto-plot

clean-proto:
  rm -f admin/tower/proto/*.pb.go
  rm -f admin/plot/proto/plot_pb2.py admin/plot/proto/plot_pb2_grpc.py admin/plot/proto/plot_pb2.pyi
