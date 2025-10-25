fmt:
  pnpm run format
  cd src-tauri && cargo fmt

lint:
  pnpm run lint
  cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings

fix:
  pnpm run fix
  cd src-tauri && cargo clippy --all-targets --all-features --fix -- -D warnings

test:
  pnpm run test
  cd src-tauri && cargo test

fmt-check:
  pnpm run format:check
  cd src-tauri && cargo fmt -- --check