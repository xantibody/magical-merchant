mod core
mod dioxus 'dioxus-app'

[private]
default:
  @just --list

# Run formatter on workspace
fmt:
  cargo fmt --all

# Run formatter check
fmt-check:
  cargo fmt --all --check

# Run clippy on workspace
lint:
  cargo clippy --workspace -- -D warnings

# Run tests on workspace
test:
  cargo test --workspace

# Run all checks (lint + fmt-check + test)
check: lint fmt-check test

# Full verify
verify: fmt check

# Run Dioxus desktop dev
dev: dioxus::dev
