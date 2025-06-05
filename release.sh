#!/bin/bash

# Generic Rust Project Release Script
# Automates version bumping, building, testing, and releasing for Rust projects
#
# This script auto-detects project configuration from:
# - Cargo.toml (crate name)
# - Git remote origin (GitHub repository)
#
# You can override defaults by setting environment variables:
# - CRATE_NAME: Override crate name
# - GITHUB_REPO: Override GitHub repository (format: owner/repo)
# - CARGO_TOML: Override Cargo.toml path
#
# Usage examples:
#   ./release.sh patch           # Release patch version
#   ./release.sh status          # Check current release status
#   ./release.sh list            # List recent releases
#   CRATE_NAME=myproject ./release.sh minor  # Override crate name

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration - Override these variables or set them in environment
CARGO_TOML="${CARGO_TOML:-Cargo.toml}"
GITHUB_REPO="${GITHUB_REPO:-$(git config --get remote.origin.url 2>/dev/null | sed -E 's|.*[:/]([^/]+/[^/]+)\.git.*|\1|' || echo "")}"
CRATES_IO_API="https://crates.io/api/v1/crates"

# Auto-detect crate name from Cargo.toml if not set
CRATE_NAME="${CRATE_NAME:-$(grep '^name = ' "$CARGO_TOML" 2>/dev/null | sed 's/name = "\(.*\)"/\1/' || echo "")}"

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in a git repository
check_git_repo() {
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        log_error "Not in a git repository"
        exit 1
    fi
}

# Check if working directory is clean
check_clean_working_dir() {
    if [[ -n $(git status --porcelain) ]]; then
        log_error "Working directory is not clean. Please commit or stash your changes."
        git status --short
        exit 1
    fi
}

# Get current version from Cargo.toml
get_current_version() {
    grep '^version = ' "$CARGO_TOML" | sed 's/version = "\(.*\)"/\1/'
}

# Bump version
bump_version() {
    local current_version="$1"
    local bump_type="$2"
    
    IFS='.' read -ra VERSION_PARTS <<< "$current_version"
    local major="${VERSION_PARTS[0]}"
    local minor="${VERSION_PARTS[1]}"
    local patch="${VERSION_PARTS[2]}"
    
    case "$bump_type" in
        "major")
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        "minor")
            minor=$((minor + 1))
            patch=0
            ;;
        "patch")
            patch=$((patch + 1))
            ;;
        *)
            log_error "Invalid bump type: $bump_type (must be major, minor, or patch)"
            exit 1
            ;;
    esac
    
    echo "${major}.${minor}.${patch}"
}

# Update version in Cargo.toml
update_cargo_version() {
    local new_version="$1"
    local temp_file=$(mktemp)
    
    sed "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML" > "$temp_file"
    mv "$temp_file" "$CARGO_TOML"
    
    log_success "Updated version to $new_version in $CARGO_TOML"
}

# Run tests and checks
run_checks() {
    log_info "Running cargo check..."
    cargo check
    
    log_info "Running cargo clippy..."
    cargo clippy -- -D warnings
    
    log_info "Running cargo fmt check..."
    cargo fmt --check
    
    log_info "Running cargo test..."
    cargo test
    
    log_info "Running cargo build --release..."
    cargo build --release
    
    log_success "All checks passed!"
}

# Create git commit and tag
create_git_release() {
    local version="$1"
    local tag="v$version"
    
    log_info "Creating git commit for version $version..."
    git add "$CARGO_TOML"
    git commit -m "chore: release version $version

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"
    
    log_info "Creating git tag $tag..."
    git tag "$tag"
    
    log_success "Created commit and tag for version $version"
}

# Push to GitHub
push_to_github() {
    local version="$1"
    local tag="v$version"
    
    log_info "Pushing to GitHub..."
    git push origin main
    git push origin "$tag"
    
    log_success "Pushed to GitHub"
}

# Publish to crates.io
publish_to_crates() {
    log_info "Publishing to crates.io..."
    
    # Dry run first
    log_info "Running dry-run..."
    cargo publish --dry-run
    
    # Ask for confirmation
    echo -n "Publish to crates.io? (y/N): "
    read -r confirm
    if [[ "$confirm" =~ ^[Yy]$ ]]; then
        cargo publish
        log_success "Published to crates.io"
    else
        log_warning "Skipped crates.io publishing"
    fi
}

# Check release status
check_release_status() {
    local version="$1"
    local tag="v$version"
    
    log_info "🔍 Checking release status for version $version..."
    echo
    
    # Check local git status
    log_info "📋 Local Git Status:"
    if git rev-parse --verify "$tag" >/dev/null 2>&1; then
        log_success "✅ Git tag $tag exists locally"
        local tag_commit=$(git rev-list -n 1 "$tag")
        local tag_date=$(git log -1 --format="%ai" "$tag")
        echo "   📅 Created: $tag_date"
        echo "   📝 Commit: ${tag_commit:0:7}"
    else
        log_warning "❌ Git tag $tag not found locally"
    fi
    echo
    
    # Check remote git status
    log_info "🌐 Remote Git Status:"
    if git ls-remote --tags origin | grep -q "refs/tags/$tag"; then
        log_success "✅ Git tag $tag exists on GitHub"
        
        # Check if we can fetch GitHub release info
        if command -v gh &> /dev/null; then
            local release_info=$(gh release view "$tag" --json url,publishedAt,name 2>/dev/null || echo "")
            if [[ -n "$release_info" ]]; then
                local release_url=$(echo "$release_info" | jq -r '.url // "N/A"')
                local published_at=$(echo "$release_info" | jq -r '.publishedAt // "N/A"')
                local release_name=$(echo "$release_info" | jq -r '.name // "N/A"')
                log_success "✅ GitHub Release published"
                echo "   📋 Name: $release_name"
                echo "   📅 Published: $published_at"
                echo "   🔗 URL: $release_url"
            else
                log_warning "⏳ GitHub Release not found (may still be processing)"
            fi
        else
            log_info "   ℹ️  GitHub CLI not available - cannot check release details"
        fi
    else
        log_warning "❌ Git tag $tag not found on GitHub"
    fi
    echo
    
    # Check GitHub Actions status
    log_info "🔄 GitHub Actions Status:"
    if command -v gh &> /dev/null; then
        # Check CI workflow
        local ci_runs=$(gh run list --workflow=ci.yml --limit=3 --json=status,conclusion,createdAt,headSha --jq '.[] | select(.headSha == "'$(git rev-parse HEAD)'")' 2>/dev/null || echo "")
        if [[ -n "$ci_runs" ]]; then
            local ci_status=$(echo "$ci_runs" | jq -r '.status // "unknown"')
            local ci_conclusion=$(echo "$ci_runs" | jq -r '.conclusion // "unknown"')
            local ci_created=$(echo "$ci_runs" | jq -r '.createdAt // "unknown"')
            
            case "$ci_conclusion" in
                "success")
                    log_success "✅ CI workflow completed successfully"
                    ;;
                "failure")
                    log_error "❌ CI workflow failed"
                    ;;
                "cancelled")
                    log_warning "⚠️  CI workflow was cancelled"
                    ;;
                *)
                    if [[ "$ci_status" == "in_progress" ]]; then
                        log_info "⏳ CI workflow is still running"
                    else
                        log_warning "❓ CI workflow status: $ci_status ($ci_conclusion)"
                    fi
                    ;;
            esac
            echo "   📅 Started: $ci_created"
        else
            log_warning "❓ No CI workflow runs found for current commit"
        fi
        
        # Check release workflow
        local release_runs=$(gh run list --workflow=release.yml --limit=3 --json=status,conclusion,createdAt,headSha --jq '.[] | select(.headSha == "'$(git rev-parse "$tag" 2>/dev/null || echo "none")'")' 2>/dev/null || echo "")
        if [[ -n "$release_runs" ]]; then
            local release_status=$(echo "$release_runs" | jq -r '.status // "unknown"')
            local release_conclusion=$(echo "$release_runs" | jq -r '.conclusion // "unknown"')
            local release_created=$(echo "$release_runs" | jq -r '.createdAt // "unknown"')
            
            case "$release_conclusion" in
                "success")
                    log_success "✅ Release workflow completed successfully"
                    ;;
                "failure")
                    log_error "❌ Release workflow failed"
                    ;;
                "cancelled")
                    log_warning "⚠️  Release workflow was cancelled"
                    ;;
                *)
                    if [[ "$release_status" == "in_progress" ]]; then
                        log_info "⏳ Release workflow is still running"
                    else
                        log_warning "❓ Release workflow status: $release_status ($release_conclusion)"
                    fi
                    ;;
            esac
            echo "   📅 Started: $release_created"
        else
            log_warning "❓ No release workflow runs found for tag $tag"
        fi
    else
        log_warning "❓ GitHub CLI not available - cannot check workflow status"
    fi
    echo
    
    # Check crates.io status
    log_info "📦 Crates.io Status:"
    local crates_response
    if crates_response=$(curl -s "$CRATES_IO_API/$CRATE_NAME" 2>/dev/null); then
        local published_version=$(echo "$crates_response" | jq -r '.crate.max_version // "unknown"')
        local published_date=$(echo "$crates_response" | jq -r '.crate.updated_at // "unknown"')
        local download_count=$(echo "$crates_response" | jq -r '.crate.downloads // "unknown"')
        
        if [[ "$published_version" == "$version" ]]; then
            log_success "✅ Version $version is published on crates.io"
        elif [[ "$published_version" == "unknown" ]]; then
            log_warning "❓ Could not determine published version"
        else
            log_warning "⚠️  Latest version on crates.io is $published_version (not $version)"
        fi
        
        echo "   📋 Latest version: $published_version"
        echo "   📅 Last updated: $published_date"
        echo "   📥 Total downloads: $download_count"
        echo "   🔗 URL: https://crates.io/crates/$CRATE_NAME"
    else
        log_warning "❌ Could not fetch crates.io information"
    fi
    echo
    
    # Summary
    log_info "📊 Release Status Summary:"
    local all_good=true
    
    # Check each component
    if git rev-parse --verify "$tag" >/dev/null 2>&1; then
        echo "   ✅ Local git tag"
    else
        echo "   ❌ Local git tag"
        all_good=false
    fi
    
    if git ls-remote --tags origin | grep -q "refs/tags/$tag"; then
        echo "   ✅ Remote git tag"
    else
        echo "   ❌ Remote git tag"
        all_good=false
    fi
    
    if command -v gh &> /dev/null; then
        if gh release view "$tag" >/dev/null 2>&1; then
            echo "   ✅ GitHub Release"
        else
            echo "   ⏳ GitHub Release (may be processing)"
        fi
    else
        echo "   ❓ GitHub Release (cannot check)"
    fi
    
    if [[ -n "$crates_response" ]]; then
        local published_version=$(echo "$crates_response" | jq -r '.crate.max_version // "unknown"')
        if [[ "$published_version" == "$version" ]]; then
            echo "   ✅ Crates.io publication"
        else
            echo "   ⏳ Crates.io publication (may be processing)"
        fi
    else
        echo "   ❓ Crates.io publication (cannot check)"
    fi
    
    echo
    if [[ "$all_good" == "true" ]]; then
        log_success "🎉 Release $version appears to be fully deployed!"
    else
        log_info "⏳ Release $version is in progress or has issues"
    fi
}

# Check status of current or specified version
check_current_status() {
    local version="${1:-}"
    
    if [[ -z "$version" ]]; then
        version=$(get_current_version)
        log_info "Checking status for current version: $version"
    else
        log_info "Checking status for specified version: $version"
    fi
    
    check_release_status "$version"
}

# List recent releases
list_recent_releases() {
    log_info "📋 Recent Releases:"
    echo
    
    # Get recent git tags
    log_info "🏷️  Recent Git Tags:"
    if git tag -l --sort=-version:refname | head -5 | grep -q .; then
        git tag -l --sort=-version:refname | head -5 | while read -r tag; do
            if [[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
                local tag_date=$(git log -1 --format="%ai" "$tag" 2>/dev/null || echo "unknown")
                local tag_commit=$(git rev-list -n 1 "$tag" 2>/dev/null || echo "unknown")
                echo "   📌 $tag (${tag_commit:0:7}) - $tag_date"
            fi
        done
    else
        log_warning "   No version tags found"
    fi
    echo
    
    # Get recent GitHub releases
    if command -v gh &> /dev/null; then
        log_info "🚀 Recent GitHub Releases:"
        local releases=$(gh release list --limit 5 --json tagName,publishedAt,name 2>/dev/null || echo "[]")
        if [[ "$releases" != "[]" ]] && [[ -n "$releases" ]]; then
            echo "$releases" | jq -r '.[] | "   🚀 \(.tagName) - \(.name) (\(.publishedAt))"'
        else
            log_warning "   No GitHub releases found"
        fi
        echo
    fi
    
    # Get crates.io version history
    log_info "📦 Crates.io Version History:"
    local crates_response
    if crates_response=$(curl -s "$CRATES_IO_API/$CRATE_NAME/versions" 2>/dev/null); then
        local versions=$(echo "$crates_response" | jq -r '.versions[0:5][] | "   📦 \(.num) - \(.created_at)"' 2>/dev/null || echo "")
        if [[ -n "$versions" ]]; then
            echo "$versions"
        else
            log_warning "   No versions found on crates.io"
        fi
    else
        log_warning "   Could not fetch crates.io version information"
    fi
}

# Wait for GitHub Actions to complete
wait_for_ci() {
    local version="$1"
    local tag="v$version"
    
    if command -v gh &> /dev/null; then
        log_info "Waiting for GitHub Actions to complete..."
        
        # Wait for a few seconds for the workflow to start
        sleep 10
        
        # Check workflow status
        local max_attempts=30
        local attempt=0
        
        while [[ $attempt -lt $max_attempts ]]; do
            local status=$(gh run list --workflow=release.yml --limit=1 --json=status --jq='.[0].status' 2>/dev/null || echo "unknown")
            
            case "$status" in
                "completed")
                    local conclusion=$(gh run list --workflow=release.yml --limit=1 --json=conclusion --jq='.[0].conclusion' 2>/dev/null || echo "unknown")
                    if [[ "$conclusion" == "success" ]]; then
                        log_success "GitHub Actions completed successfully"
                        return 0
                    else
                        log_error "GitHub Actions failed with conclusion: $conclusion"
                        return 1
                    fi
                    ;;
                "in_progress"|"queued")
                    log_info "GitHub Actions still running... (attempt $((attempt + 1))/$max_attempts)"
                    sleep 30
                    ;;
                *)
                    log_warning "Unknown GitHub Actions status: $status"
                    ;;
            esac
            
            attempt=$((attempt + 1))
        done
        
        log_warning "Timeout waiting for GitHub Actions to complete"
        return 1
    else
        log_warning "GitHub CLI not installed, skipping CI wait"
        return 0
    fi
}

# Show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS] <COMMAND>

Automate the release process for cctx

COMMANDS:
    major|minor|patch   Bump version and release
    status [VERSION]    Check release status for current or specified version
    list                List recent releases and their status

OPTIONS:
    -h, --help          Show this help message
    -n, --dry-run       Perform a dry run (don't actually release)
    -s, --skip-crates   Skip crates.io publishing
    -f, --skip-ci-wait  Skip waiting for CI to complete

EXAMPLES:
    # Release commands
    $0 patch            # Bump patch version (0.1.0 -> 0.1.1)
    $0 minor            # Bump minor version (0.1.0 -> 0.2.0)
    $0 major            # Bump major version (0.1.0 -> 1.0.0)
    $0 --dry-run patch  # Show what would happen without making changes
    
    # Status checking commands
    $0 status           # Check status of current version
    $0 status 0.1.0     # Check status of specific version
    $0 list             # List recent releases

EOF
}

# Parse command line arguments
DRY_RUN=false
SKIP_CRATES=false
SKIP_CI_WAIT=false
BUMP_TYPE=""
COMMAND=""
STATUS_VERSION=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -n|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -s|--skip-crates)
            SKIP_CRATES=true
            shift
            ;;
        -f|--skip-ci-wait)
            SKIP_CI_WAIT=true
            shift
            ;;
        major|minor|patch)
            BUMP_TYPE="$1"
            shift
            ;;
        status)
            COMMAND="status"
            shift
            # Check if version is specified after status
            if [[ $# -gt 0 && ! "$1" =~ ^- ]]; then
                STATUS_VERSION="$1"
                shift
            fi
            ;;
        list)
            COMMAND="list"
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Validate command
if [[ -z "$BUMP_TYPE" && -z "$COMMAND" ]]; then
    log_error "Command is required"
    show_usage
    exit 1
fi

# Validate configuration
validate_config() {
    if [[ -z "$CRATE_NAME" ]]; then
        log_error "Could not detect crate name from $CARGO_TOML"
        log_info "Set CRATE_NAME environment variable or ensure Cargo.toml has 'name = \"...\"'"
        exit 1
    fi
    
    if [[ -z "$GITHUB_REPO" ]]; then
        log_warning "Could not detect GitHub repository from git remote"
        log_info "Set GITHUB_REPO environment variable (format: owner/repo)"
    fi
    
    log_info "Configuration:"
    log_info "  📦 Crate name: $CRATE_NAME"
    log_info "  🏠 GitHub repo: ${GITHUB_REPO:-<not detected>}"
    log_info "  📄 Cargo.toml: $CARGO_TOML"
    echo
}

# Main execution
main() {
    # Handle status and list commands
    if [[ "$COMMAND" == "status" ]]; then
        check_git_repo
        validate_config
        check_current_status "$STATUS_VERSION"
        return 0
    elif [[ "$COMMAND" == "list" ]]; then
        check_git_repo
        validate_config
        list_recent_releases
        return 0
    fi
    
    log_info "Starting release process for $CRATE_NAME..."
    validate_config
    
    # Pre-flight checks
    check_git_repo
    check_clean_working_dir
    
    # Get current version and calculate new version
    local current_version
    current_version=$(get_current_version)
    local new_version
    new_version=$(bump_version "$current_version" "$BUMP_TYPE")
    
    log_info "Current version: $current_version"
    log_info "New version: $new_version"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_warning "DRY RUN MODE - No changes will be made"
        log_info "Would update version from $current_version to $new_version"
        log_info "Would run tests and build"
        log_info "Would create git commit and tag v$new_version"
        log_info "Would push to GitHub"
        if [[ "$SKIP_CRATES" == "false" ]]; then
            log_info "Would publish to crates.io"
        fi
        log_success "Dry run completed successfully"
        exit 0
    fi
    
    # Confirm with user
    echo -n "Release version $new_version? (y/N): "
    read -r confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        log_info "Release cancelled by user"
        exit 0
    fi
    
    # Update version
    update_cargo_version "$new_version"
    
    # Run all checks
    run_checks
    
    # Create git release
    create_git_release "$new_version"
    
    # Push to GitHub
    push_to_github "$new_version"
    
    # Wait for CI if requested
    if [[ "$SKIP_CI_WAIT" == "false" ]]; then
        wait_for_ci "$new_version"
    fi
    
    # Publish to crates.io
    if [[ "$SKIP_CRATES" == "false" ]]; then
        publish_to_crates
    fi
    
    log_success "Release $new_version completed successfully! 🎉"
    log_info "GitHub Release: https://github.com/$GITHUB_REPO/releases/tag/v$new_version"
    log_info "Crates.io: https://crates.io/crates/$CRATE_NAME"
}

# Run main function
main "$@"