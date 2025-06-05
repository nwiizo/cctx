# 🔄 CLAUDE.md - cctx Project Documentation

## 📋 Project Overview

**cctx** (Claude Context) is a fast, secure, and intuitive command-line tool for managing multiple Claude Code `settings.json` configurations. Built with Rust for maximum performance and reliability.

## 🏗️ Architecture

### 🎯 Core Concept
- **🔧 Context**: A saved Claude Code configuration stored as a JSON file
- **⚡ Current Context**: The active configuration (`~/.claude/settings.json`)
- **📁 Context Storage**: All contexts are stored in `~/.claude/settings/` as individual JSON files
- **📊 State Management**: Current and previous context tracked in `~/.claude/settings/.cctx-state.json`

### 📁 File Structure
```
📁 ~/.claude/
├── ⚙️ settings.json           # Current active context (managed by cctx)
└── 📁 settings/
    ├── 💼 work.json          # Work context
    ├── 🏠 personal.json      # Personal context
    ├── 🚀 project-alpha.json # Project-specific context
    └── 🔒 .cctx-state.json   # Hidden state file (tracks current/previous)
```

### 🎯 Key Design Decisions
1. **File-based contexts**: Each context is a separate JSON file, making manual management possible
2. **Simple naming**: Filename (without .json) = context name
3. **Atomic operations**: Context switching is done by copying files
4. **Hidden state file**: Prefixed with `.` to hide from context listings
5. **Predictable UX**: Default behavior always uses user-level contexts for consistency
6. **Progressive disclosure**: Helpful hints show when project/local contexts are available

## 🎯 Command Reference

### 🚀 Basic Commands
- `cctx` - List contexts (defaults to user-level, shows helpful hints)
- `cctx <name>` - Switch to context
- `cctx -` - Switch to previous context

### 🏗️ Settings Level Management
- `cctx` - Default: user-level contexts (`~/.claude/settings.json`)
- `cctx --in-project` - Project-level contexts (`./.claude/settings.json`)
- `cctx --local` - Local project contexts (`./.claude/settings.local.json`)

### 🛠️ Management Commands
- `cctx -n <name>` - Create new context from current settings
- `cctx -d <name>` - Delete context
- `cctx -r <old> <new>` - Rename context
- `cctx -c` - Show current context name
- `cctx -e [name]` - Edit context with $EDITOR
- `cctx -s [name]` - Show context content
- `cctx -u` - Unset current context

### 📥📤 Import/Export
- `cctx --export <name>` - Export to stdout
- `cctx --import <name>` - Import from stdin

## Implementation Details

### Language & Dependencies
- **Language**: Rust (edition 2021)
- **Key Dependencies**:
  - `clap` - Command-line argument parsing
  - `serde`/`serde_json` - JSON serialization
  - `dialoguer` - Interactive prompts
  - `colored` - Terminal colors
  - `anyhow` - Error handling
  - `dirs` - Platform-specific directories

### Error Handling
- Use `anyhow::Result` for all functions that can fail
- Provide clear error messages with context
- Validate context names (no `/`, `.`, `..`, or empty)
- Check for active context before deletion

### 🎨 Interactive Features
1. **fzf integration**: Auto-detect and use if available
2. **Built-in fuzzy finder**: Fallback when fzf not available
3. **Color coding**: Current context highlighted in green
4. **Helpful hints**: Shows available project/local contexts when at user level
5. **Visual indicators**: Emojis for different context levels (👤 User, 📁 Project, 💻 Local)

## Release Management

### Automated Release System

The project includes a comprehensive release automation system with multiple tools:

#### 1. **release.sh** - Primary Release Script ⭐

A generic, feature-rich release script that can be used for any Rust project:

```bash
# Release new versions
./release.sh patch          # 0.1.0 -> 0.1.1
./release.sh minor          # 0.1.0 -> 0.2.0  
./release.sh major          # 0.1.0 -> 1.0.0

# Check release status
./release.sh status         # Check current version
./release.sh status 0.1.0   # Check specific version
./release.sh list           # List recent releases

# Options
./release.sh --dry-run patch      # Preview changes
./release.sh --skip-crates minor  # Skip crates.io publishing
```

**Features:**
- ✅ Auto-detects crate name and GitHub repo from project
- ✅ Comprehensive pre-flight checks (format, clippy, tests)
- ✅ Version bumping with validation
- ✅ Git tagging and GitHub release creation
- ✅ Crates.io publishing with confirmation
- ✅ CI/CD status monitoring
- ✅ Release status checking across all platforms
- ✅ Dry-run mode for safe testing
- ✅ Configurable via environment variables

**Configuration:**
```bash
# Override defaults if needed
export CRATE_NAME="custom-name"
export GITHUB_REPO="owner/repo"
export CARGO_TOML="path/to/Cargo.toml"
./release.sh patch
```

#### 2. **GitHub Actions Workflows**

**CI Pipeline** (`.github/workflows/ci.yml`):
- Multi-platform testing (Ubuntu, macOS, Windows)
- Rust stable and beta versions
- Format checking (`cargo fmt`)
- Linting (`cargo clippy`)
- Security audit (`cargo audit`)
- MSRV (Minimum Supported Rust Version) testing

**Release Pipeline** (`.github/workflows/release.yml`):
- Triggered by version tags (v*.*.*)
- Cross-platform binary builds
- Automatic GitHub Release creation
- Asset uploads with release notes

#### 3. **Alternative Release Tools**

**cargo-release** (`release-cargo.sh`):
```bash
./release-cargo.sh patch    # Simple one-command release
```

**release-plz** (削除済み):
- GitHub ActionsのデフォルトGITHUB_TOKENではPR作成権限がないため削除
- 代替案: GitHub App tokenまたはPersonal Access Token (PAT)を使用
- 現在は`quick-release.sh`スクリプトで同等の機能を提供

**justfile** (Task runner):
```bash
just release-patch         # Release with pre-checks
just dry-run-minor         # Test release process
just check                 # Run all quality checks
```

### Release Process Workflow

1. **Development Phase:**
   ```bash
   # During development
   just check                    # Run all checks
   cargo clippy --fix           # Fix issues
   cargo fmt                    # Format code
   ```

2. **Pre-Release Validation:**
   ```bash
   # Test release process
   ./release.sh --dry-run patch
   ./release.sh status          # Check current state
   ```

3. **Release Execution:**
   ```bash
   # Actual release
   ./release.sh patch           # Interactive with confirmations
   ```

4. **Post-Release Verification:**
   ```bash
   # Verify deployment
   ./release.sh status 0.1.1    # Check specific version
   ./release.sh list            # View recent releases
   ```

### Quality Gates

All releases must pass:
- ✅ `cargo fmt --check` (code formatting)
- ✅ `cargo clippy -- -D warnings` (linting)
- ✅ `cargo test` (unit tests)
- ✅ `cargo build --release` (release build)
- ✅ `cargo audit` (security vulnerabilities)
- ✅ Working directory clean (no uncommitted changes)

### Versioning Strategy

Following [Semantic Versioning](https://semver.org/):
- **Patch** (0.1.0 → 0.1.1): Bug fixes, minor improvements
- **Minor** (0.1.0 → 0.2.0): New features, backward compatible
- **Major** (0.1.0 → 1.0.0): Breaking changes

### CI/CD Configuration

**GitHub Actions Secrets:**
- `CARGO_REGISTRY_TOKEN`: Required for crates.io publishing

**Environment Variables:**
- `RUST_VERSION`: "1.75" (MSRV)
- `CARGO_TERM_COLOR`: "always"

## Development Guidelines

### Before Making Changes

1. **Understand the current implementation**:
   ```bash
   cargo check
   cargo clippy
   ```

2. **Run existing tests** (if any):
   ```bash
   cargo test
   ```

3. **Use development tools**:
   ```bash
   just setup                   # Setup dev environment
   just check                   # Run all checks
   ```

### Making Changes

1. **Always run linting** before committing:
   ```bash
   cargo clippy -- -D warnings
   ```

2. **Format code** using Rust standards:
   ```bash
   cargo fmt
   ```

3. **Test thoroughly**:
   - Test basic operations: create, switch, delete contexts
   - Test edge cases: empty names, special characters, missing files
   - Test interactive mode with and without fzf
   - Test on different platforms if possible

4. **Validate JSON handling**:
   - Ensure invalid JSON files are rejected
   - Preserve JSON formatting when possible
   - Handle missing or corrupted state files gracefully

### Testing Checklist

When testing changes, verify:

- [ ] `cctx` lists all contexts correctly
- [ ] `cctx <name>` switches context
- [ ] `cctx -` returns to previous context
- [ ] `cctx -n <name>` creates new context
- [ ] `cctx -d <name>` deletes context (not if current)
- [ ] `cctx -r <old> <new>` renames context
- [ ] Interactive mode works (both fzf and built-in)
- [ ] Error messages are clear and helpful
- [ ] State persistence works across sessions
- [ ] Hidden files are excluded from listings

### Common Pitfalls

1. **File permissions**: Ensure created files have appropriate permissions
2. **Path handling**: Use PathBuf consistently, avoid string manipulation
3. **JSON validation**: Always validate JSON before writing
4. **State consistency**: Update state file atomically

## Future Considerations

### Potential Enhancements
- Context templates/inheritance
- Context validation against Claude Code schema
- Backup/restore functionality
- Context history beyond just previous
- Shell completions

### Compatibility
- Maintain backward compatibility with existing contexts
- Keep command-line interface stable
- Preserve kubectx-like user experience

## Code Quality Standards

1. **Every function should**:
   - Have a clear, single responsibility
   - Return `Result` for fallible operations
   - Include error context with `.context()`

2. **User-facing messages**:
   - Error messages should be helpful and actionable
   - Success messages should be concise
   - Use color coding consistently (green=success, red=error)

3. **File operations**:
   - Always check if directories exist before use
   - Handle missing files gracefully
   - Use atomic operations where possible

## Release Tools Summary

| Tool | Use Case | Command | Automation Level |
|------|----------|---------|------------------|
| **release.sh** | Primary release tool | `./release.sh patch` | Semi-automated with checks |
| release-cargo.sh | Simple releases | `./release-cargo.sh patch` | Fully automated |
| justfile | Development tasks | `just release-patch` | Task-based |
| release-plz | Git-flow releases | Automatic on PR merge | Fully automated |
| Manual | Emergency/custom | `cargo publish` | Manual |

### Recommended Workflow

For most releases, use the primary **release.sh** script:

1. **Development** → `just check` (validate changes)
2. **Pre-release** → `./release.sh --dry-run patch` (test)
3. **Release** → `./release.sh patch` (execute)
4. **Verify** → `./release.sh status` (confirm deployment)

## Release Checklist (Automated)

The release.sh script automatically handles:

- ✅ Version validation and bumping in `Cargo.toml`
- ✅ Code formatting (`cargo fmt --check`)
- ✅ Linting (`cargo clippy -- -D warnings`)
- ✅ Test execution (`cargo test`)
- ✅ Release build (`cargo build --release`)
- ✅ Git tagging and commit creation
- ✅ GitHub push and release creation
- ✅ CI/CD monitoring and status checking
- ✅ Crates.io publishing with confirmation
- ✅ Cross-platform deployment verification

Manual steps (if needed):
- Update README for major changes
- Update documentation for new features

## 🎯 UX Design Philosophy

### 🏆 Simplified User Experience (v0.1.1+)

**Core Principle**: **Predictable defaults with explicit overrides**

#### ✅ What We Did Right
- **Removed complex auto-detection** that was confusing users
- **Default always uses user-level** for predictable behavior
- **Clear explicit flags** (`--in-project`, `--local`) when needed
- **Helpful progressive disclosure** - hints when other contexts available
- **Visual clarity** with emojis and condensed information

#### ❌ What We Avoided
- **Complex flag combinations** (`--user`, `--project`, `--local`, `--level`)
- **Unpredictable auto-detection logic** 
- **Verbose technical output** showing file paths
- **Cognitive overhead** from too many options

#### 🎯 UX Goals Achieved
1. **⚡ Speed**: Default behavior is instant and predictable
2. **🧠 Simplicity**: Two explicit flags instead of four confusing ones
3. **🎯 Discoverability**: Helpful hints guide users to advanced features
4. **🔄 Consistency**: Always behaves the same way (user-level default)

### 📝 Usage Patterns

```bash
# 90% of usage - simple and predictable
cctx                    # List user contexts + helpful hints
cctx work              # Switch to work context

# 10% of usage - explicit when needed  
cctx --in-project staging   # Project-specific contexts
cctx --local debug         # Local development contexts
```

## 📚 Notes for AI Assistants

When working on this codebase:

1. **Always run `cargo clippy` and fix warnings** before suggesting code
2. **Test your changes** - don't assume code works
3. **Preserve existing behavior** unless explicitly asked to change it
4. **Follow Rust idioms** and best practices
5. **Keep the kubectx-inspired UX** - simple, fast, intuitive
6. **Maintain predictable defaults** - user should never be surprised
7. **Document any new features** in both code and README
8. **Consider edge cases** - empty states, missing files, permissions
9. **Progressive disclosure** - show advanced features only when relevant

Remember: This tool is about speed and simplicity. Every feature should make context switching faster or easier, not more complex. **Predictability beats cleverness.**
