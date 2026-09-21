# Task runner for cctx. Install just: cargo install just
# Releases go through ./quick-release.sh, which runs the same checks as CI.

# Show available commands
default:
    @just --list

# Run the CI gate (format, clippy, tests, release build)
check:
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features --locked -- -D warnings
    cargo test --all-targets --locked
    cargo build --release --locked

# Fix formatting and clippy issues
fix:
    cargo fmt --all
    cargo clippy --all-targets --all-features --fix --allow-dirty

# Run tests
test:
    cargo test --all-targets --locked

# Build release version
build:
    cargo build --release --locked

# Install cctx from this checkout
install:
    cargo install --path . --locked

# Generate shell completions
completions shell:
    cargo run -- --completions {{shell}}

# Check for security vulnerabilities (cargo install cargo-audit)
audit:
    cargo audit
