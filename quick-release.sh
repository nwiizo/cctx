#!/bin/bash
# Quick Release Script for cctx
# Usage: ./quick-release.sh [patch|minor|major]

set -euo pipefail

RELEASE_TYPE="${1:-patch}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Validate release type
if [[ ! "$RELEASE_TYPE" =~ ^(patch|minor|major)$ ]]; then
    error "Invalid release type. Use: patch, minor, or major"
fi

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    error "Not in a git repository"
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    error "You have uncommitted changes. Please commit or stash them first."
fi

# Ensure we're on main branch
current_branch=$(git rev-parse --abbrev-ref HEAD)
if [[ "$current_branch" != "main" ]]; then
    error "You must be on the main branch to release. Current branch: $current_branch"
fi

# Ensure we're up to date with remote
git fetch origin main
if ! git diff --quiet HEAD origin/main; then
    error "Your local main branch is not up to date with origin/main"
fi

log "Starting $RELEASE_TYPE release..."

# Get current version from Cargo.toml
current_version=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
log "Current version: $current_version"

# Calculate new version
IFS='.' read -r major minor patch <<< "$current_version"
case "$RELEASE_TYPE" in
    patch)
        new_version="$major.$minor.$((patch + 1))"
        ;;
    minor)
        new_version="$major.$((minor + 1)).0"
        ;;
    major)
        new_version="$((major + 1)).0.0"
        ;;
esac

log "New version will be: $new_version"

# Ask for confirmation
echo
read -p "Do you want to continue with $RELEASE_TYPE release v$new_version? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    log "Release cancelled"
    exit 0
fi

log "Running quality checks..."

# Run all quality checks
cargo fmt --all -- --check || error "Code formatting check failed"
cargo clippy -- -D warnings || error "Clippy check failed"
cargo test || error "Tests failed"
cargo build --release || error "Release build failed"

success "All quality checks passed!"

log "Updating version to $new_version..."

# Update version in Cargo.toml
sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
rm Cargo.toml.bak

# Update Cargo.lock
cargo build --quiet

log "Committing version bump..."

# Commit the version change
git add Cargo.toml Cargo.lock
git commit -m "Bump version to $new_version

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"

log "Creating and pushing git tag..."

# Create and push tag
git tag "v$new_version"
git push origin main
git push origin "v$new_version"

success "Release v$new_version initiated!"
log "GitHub Actions will now:"
log "  1. Run CI checks"
log "  2. Build release binaries"
log "  3. Create GitHub release"
log "  4. Publish to crates.io"
log ""
log "Monitor progress at: https://github.com/nwiizo/cctx/actions"
log "Release will be available at: https://github.com/nwiizo/cctx/releases/tag/v$new_version"