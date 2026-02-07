# Contributing to next.rs

Thank you for your interest in contributing to next.rs! This document provides guidelines and instructions for contributing.

## Getting Started

### Prerequisites

- Rust 1.75.0 or later
- wasm-pack (for WASM builds)

### Setup

```bash
git clone https://github.com/user/next.rs.git
cd next.rs
cargo build --workspace
cargo test --workspace
```

## Development Workflow

### Code Style

We use standard Rust formatting and linting:

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --workspace --all-targets --all-features
```

All code must pass `cargo fmt` and `cargo clippy` with zero warnings.

### Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p react-rs-core

# Run tests with output
cargo test --workspace -- --nocapture
```

### Building

```bash
# Build all crates
cargo build --workspace

# Build WASM target
cargo build -p react-rs-wasm --target wasm32-unknown-unknown

# Build documentation
cargo doc --workspace --no-deps --open
```

## Pull Request Process

1. **Fork** the repository
2. **Create a branch** from `main`:
   ```bash
   git checkout -b feature/my-feature
   ```
3. **Make your changes** following our code style
4. **Add tests** for new functionality
5. **Run CI locally**:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets --all-features
   cargo test --workspace
   ```
6. **Commit** with a clear message following conventional commits:
   - `feat: add new feature`
   - `fix: resolve bug`
   - `docs: update documentation`
   - `refactor: improve code structure`
   - `test: add tests`
   - `chore: maintenance tasks`
7. **Push** and create a Pull Request

## PR Requirements

- [ ] All CI checks pass
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings
- [ ] Tests pass and new tests added if applicable
- [ ] Documentation updated if API changes

## Project Structure

```
next.rs/
├── crates/
│   ├── react-core/        # Signal/Effect/Memo/Context
│   ├── react-elements/    # Pure Rust element API
│   ├── react-dom/         # SSR rendering
│   ├── react-wasm/        # WASM runtime + Hydration
│   ├── next-router/       # File-system routing
│   ├── next-server/       # HTTP server (SSR/SSG/ISR)
│   ├── next-cli/          # CLI tool
│   ├── next-rsc/          # React Server Components
│   ├── next-actions/      # Server Actions
│   ├── next-middleware/   # Request middleware
│   └── next-assets/       # Image/Font optimization
├── examples/
│   ├── hello-world/
│   ├── counter/
│   ├── todo-app/
│   └── blog/
└── README.md
```

## Design Principles

1. **Pure Rust API** - No RSX/JSX macros, use method chaining
2. **Type Safety** - Leverage Rust's type system
3. **Performance** - Zero-cost abstractions where possible
4. **Compatibility** - Follow Next.js patterns and conventions

## Questions?

Feel free to open an issue for questions or discussions.
