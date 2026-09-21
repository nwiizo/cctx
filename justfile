# justfile for cctx project management
# Install just: cargo install just

# Show available commands
default:
    @just --list

# Run all checks (format, clippy, test)
check:
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features --locked -- -D warnings
    cargo test --all-targets --locked
    cargo build --release --locked

# Fix formatting and clippy issues
fix:
    cargo fmt
    cargo clippy --fix --allow-dirty

# Run tests
test:
    cargo test --all-targets --locked

# Build release version
build:
    cargo build --release --locked

# Clean build artifacts
clean:
    cargo clean

# Release patch version (e.g., 0.1.0 -> 0.1.1)
release-patch: check
    @echo "Releasing patch version..."
    ./quick-release.sh patch

# Release minor version (e.g., 0.1.0 -> 0.2.0)
release-minor: check
    @echo "Releasing minor version..."
    ./quick-release.sh minor

# Release major version (e.g., 0.1.0 -> 1.0.0)
release-major: check
    @echo "Releasing major version..."
    ./quick-release.sh major

# Install cctx locally
install:
    cargo install --path . --locked

# Publish to crates.io only (no version bump)
publish:
    cargo publish

# Generate shell completions
completions shell:
    cargo run -- --completions {{shell}}

# Setup development environment
setup:
    rustup component add rustfmt clippy
    cargo install cargo-audit cargo-outdated
    @echo "Development environment setup complete!"

# Check for security vulnerabilities
audit:
    cargo audit

# Check for outdated dependencies
outdated:
    cargo outdated

# Update dependencies
update:
    cargo update