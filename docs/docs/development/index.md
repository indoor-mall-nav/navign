# Development Guides

This section provides development guides, best practices, and planning documentation for contributors working on the Navign project.

## Overview

The Navign project is a complex polyglot monorepo with components in Rust, TypeScript, Python, Go, and Swift. This section helps developers understand the development workflow, coding standards, and planned improvements.

## Getting Started

### Initial Setup

```bash
# Clone repository
git clone <repository-url>
cd navign

# Run initialization (installs all tools and dependencies)
just init
```

This will:
- Install cargo-binstall, cargo-deny, cargo-shear, typos-cli
- Enable corepack and install pnpm packages
- Sync Python dependencies
- Run cargo check

### Development Workflow

```bash
# Format code
just fmt

# Run linters
just lint

# Run tests
just test

# Component-specific CI
just ci-server
just ci-firmware
just ci-mobile
just ci-robot-upper
```

---

## Development Resources

### Critical TODOs
**[Critical TODOs](./critical-todos.md)** - High-priority tasks and known issues

This document tracks critical tasks that need attention, including:
- Security improvements
- Performance optimizations
- Feature completions
- Technical debt

---

### Refactoring Plan
**[Refactoring Plan](./refactoring-plan.md)** - Long-term architectural improvements

This document outlines planned refactorings and architectural changes:
- Code organization improvements
- Performance optimizations
- Dependency updates
- Migration strategies

---

## Coding Standards

### Rust Code Style

- **Edition:** 2024
- **Formatter:** `rustfmt` (default settings)
- **Linter:** `clippy` with `-D warnings`
- **Naming:**
  - `snake_case` for functions, variables, modules
  - `PascalCase` for types, traits, enums
  - `SCREAMING_SNAKE_CASE` for constants

**Error Handling:**
- Use `anyhow::Result` for applications
- Use custom error types (`thiserror`) for libraries
- Always propagate errors with `?`, never `unwrap()` in production

**Async:**
- Prefer `async`/`await` over manual futures
- Use Tokio for server, esp-rtos for firmware

### TypeScript Code Style

- **Formatter:** Prettier
- **Linter:** Oxlint with `--type-aware` mode
- **Type Safety:**
  - Enable strict mode in `tsconfig.json`
  - No `any` types
  - Prefer interfaces over types for objects

**Vue:**
- Use `<script setup lang="ts">` composition API
- Define props with `defineProps<T>()`
- Use Pinia for state management

### Python Code Style

- **Formatter:** Ruff
- **Linter:** Ruff check
- **Type Hints:** Use type annotations for all public functions
- **Package Manager:** uv (not pip)

### Go Code Style

- **Formatter:** gofmt
- **Linter:** golangci-lint
- **Conventions:** Follow standard Go project layout

---

## Git Workflow

### Commit Messages

Follow conventional commits:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style (formatting, no logic change)
- `refactor`: Code restructuring
- `perf`: Performance improvement
- `test`: Add/modify tests
- `chore`: Maintenance tasks

**Scopes:**
- `server`, `beacon`, `mobile`, `shared`, `admin`, `robot`, `docs`

**Examples:**
```
feat(beacon): add servo motor unlock support

Implements UnlockMethod::Servo for beacon access control.
Includes PWM control and angle calibration.

Closes #123
```

### Branch Strategy

- `main` - Production-ready code
- `develop` - Integration branch (if used)
- `feature/*` - New features
- `fix/*` - Bug fixes
- `docs/*` - Documentation updates

---

## Common Development Tasks

### Adding a New API Endpoint

1. Define schema in `server/src/schema/`
2. Implement Service trait if using CRUD
3. Add route in `server/src/main.rs`
4. Update `shared/` if needed by mobile
5. Generate TypeScript types: `just gen-ts-schema`

### Adding a BLE Message Type

1. Define in `shared/src/ble/message.rs`
2. Implement Packetize/Depacketize
3. Update beacon handler in `firmware/src/bin/main.rs`
4. Update mobile Tauri command in `mobile/src-tauri/src/lib.rs`

### Adding a New Component

1. Create directory structure
2. Add to workspace in `Cargo.toml` or `pnpm-workspace.yaml`
3. Add CI task in `justfile`
4. Add documentation in `docs/docs/components/`
5. Update navigation in `docs/docs/.vitepress/config.ts`

---

## Feature Flags

The `shared/` library uses feature flags for no_std compatibility:

**For firmware (embedded):**
```toml
navign-shared = {
  path = "../shared",
  default-features = false,
  features = ["heapless", "serde", "crypto"]
}
```

**For server:**
```toml
navign-shared = {
  path = "../shared",
  features = ["std", "serde", "mongodb", "crypto"]
}
```

**CRITICAL:** Never enable both `heapless` and `alloc` features simultaneously.

---

## Architecture Decisions

### Technology Choices

**Why Rust?**
- Memory safety without garbage collection
- Excellent async support (Tokio)
- no_std support for embedded systems
- Strong type system prevents bugs at compile time

**Why Tauri?**
- Native performance
- Small bundle size compared to Electron
- Rust backend for cryptography and BLE
- Cross-platform support (mobile + desktop)

**Why Zenoh?**
- Efficient pub/sub messaging for robots
- Zero-copy message passing
- Network-transparent (works across machines)
- Protocol Buffers integration

**Why MongoDB + PostgreSQL?**
- MongoDB: Flexible schema, fast prototyping
- PostgreSQL: ACID compliance, relational queries
- Dual-database: Gradual migration strategy

---

## Development Environment

### Required Tools

**Rust:**
- Rust 1.83+ (nightly for some features)
- cargo-binstall, cargo-deny, cargo-shear
- espup (for firmware development)

**Node.js:**
- Node 23.11.0+
- pnpm 10.15.0+
- Corepack enabled

**Python:**
- Python 3.13+
- uv package manager

**Go:**
- Go 1.24+

**Other:**
- MongoDB 8.0+
- PostgreSQL 16+ (optional)
- Just command runner
- Typos spell checker

### IDE Setup

**Recommended:**
- Visual Studio Code with extensions:
  - rust-analyzer
  - Volar (Vue)
  - Prettier
  - ESLint
  - Python
  - Go

**Configuration:**
- Enable format on save
- Enable clippy lints
- Configure type checking for TypeScript

---

## Performance Considerations

### Firmware

- **Size Optimization:** `opt-level = "s"` required for ESP32-C3
- **Code Units:** `codegen-units = 1` for smaller binaries
- **Stack Usage:** Limited to 8KB on ESP32-C3
- **Heap Usage:** Minimize allocations, use `heapless` where possible

### Server

- **Pathfinding:** Uses bump allocation for zero-cost routing
- **Database:** Index all frequently queried fields
- **Connection Pooling:** Configure appropriate pool sizes
- **Async:** Use `#[tokio::main]` for async runtime

### Mobile

- **Bundle Size:** Use dynamic imports for large dependencies
- **Map Tiles:** Cache tiles in SQLite
- **BLE Scanning:** Throttle scan frequency to save battery
- **State Management:** Minimize reactive state, use computed values

---

## Security Considerations

1. **Cryptography:**
   - P-256 ECDSA for signatures
   - Nonce-based challenge-response
   - Secure key storage (efuse, Stronghold)

2. **Authentication:**
   - JWT tokens with 24h expiration
   - OAuth2 for third-party login
   - bcrypt for password hashing (cost factor 12)

3. **Rate Limiting:**
   - 5 unlock attempts per 5 minutes
   - API rate limiting (planned)

4. **Input Validation:**
   - Validate all user inputs
   - Sanitize database queries
   - Check bounds for coordinates

---

## See Also

- [CLAUDE.md](../../CLAUDE.md) - Comprehensive development guide
- [Components](../components/) - Component documentation
- [Pipelines](../pipelines/) - End-to-end data flows
- [Testing](../testing/) - Testing strategies
