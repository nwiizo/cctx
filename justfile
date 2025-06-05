# justfile for cctx project management
# Install just: cargo install just

# Show available commands
default:
    @just --list

# Run all checks (format, clippy, test)
check:
    cargo fmt --check
    cargo clippy -- -D warnings
    cargo test
    cargo build --release

# Fix formatting and clippy issues
fix:
    cargo fmt
    cargo clippy --fix --allow-dirty

# Run tests
test:
    cargo test

# Build release version
build:
    cargo build --release

# Clean build artifacts
clean:
    cargo clean

# Release patch version (e.g., 0.1.0 -> 0.1.1)
release-patch: check
    @echo "Releasing patch version..."
    ./release.sh patch

# Release minor version (e.g., 0.1.0 -> 0.2.0)
release-minor: check
    @echo "Releasing minor version..."
    ./release.sh minor

# Release major version (e.g., 0.1.0 -> 1.0.0)
release-major: check
    @echo "Releasing major version..."
    ./release.sh major

# Dry run for patch release
dry-run-patch:
    ./release.sh --dry-run patch

# Dry run for minor release
dry-run-minor:
    ./release.sh --dry-run minor

# Dry run for major release
dry-run-major:
    ./release.sh --dry-run major

# Install cctx locally
install:
    cargo install --path .

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