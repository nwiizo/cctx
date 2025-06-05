#!/bin/bash

# Alternative: Using cargo-release for simpler automation
# Install: cargo install cargo-release

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Check if cargo-release is installed
if ! command -v cargo-release &> /dev/null; then
    log_info "Installing cargo-release..."
    cargo install cargo-release
fi

# Show usage
if [[ $# -eq 0 ]]; then
    echo "Usage: $0 <patch|minor|major>"
    echo ""
    echo "Examples:"
    echo "  $0 patch   # 0.1.0 -> 0.1.1"
    echo "  $0 minor   # 0.1.0 -> 0.2.0"
    echo "  $0 major   # 0.1.0 -> 1.0.0"
    exit 1
fi

LEVEL="$1"

log_info "Starting release with cargo-release..."

# Run cargo-release with the specified level
# This will:
# 1. Run tests
# 2. Update version in Cargo.toml
# 3. Create git commit and tag
# 4. Push to GitHub
# 5. Publish to crates.io
cargo release "$LEVEL" --execute

log_success "Release completed with cargo-release! 🎉"