# 🔄 cctx - Claude Context Switcher

> ⚡ **Fast and intuitive** way to switch between Claude Code contexts (`~/.claude/settings.json`)

[![Crates.io](https://img.shields.io/crates/v/cctx)](https://crates.io/crates/cctx)
[![CI](https://github.com/nwiizo/cctx/workflows/CI/badge.svg)](https://github.com/nwiizo/cctx/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.81%2B-orange.svg)](https://www.rust-lang.org/)

**cctx** (Claude Context) is a kubectx-inspired command-line tool for managing multiple Claude Code configurations. Switch between different permission sets, environments, and settings with a single command.

## ✨ Features

- 🔀 **Instant context switching** - Switch between configurations in milliseconds
- 🎯 **Predictable UX** - Default behavior always uses user-level contexts (no surprises!)
- 🛡️ **Security-first** - Separate permissions for work, personal, and project contexts
- 🎨 **Beautiful CLI** - Colorized output with helpful hints and visual indicators
- 🚀 **Shell completions** - Tab completion for all major shells
- 📦 **Zero dependencies** - Single binary, works everywhere
- 🔄 **Previous context** - Quick switch back with `cctx -`
- 📁 **File-based** - Simple JSON files you can edit manually
- 🎭 **Kubectx-inspired** - Familiar UX for Kubernetes users
- 💡 **Progressive disclosure** - Shows project/local contexts when available

## 🚀 Quick Start

### 📦 Installation

**From crates.io (recommended):**
```bash
cargo install cctx
```

**From source:**
```bash
git clone https://github.com/nwiizo/cctx.git
cd cctx
cargo install --path .
```

**Pre-built binaries:**
Download from [GitHub Releases](https://github.com/nwiizo/cctx/releases)

### ⚡ 30-Second Setup

```bash
# 1. Create your first context from current settings
cctx -n personal

# 2. Create a restricted work context
cctx -n work

# 3. Switch between contexts
cctx work      # Switch to work
cctx personal  # Switch to personal  
cctx -         # Switch back to previous
```

## 🎯 Usage

### 🔍 Basic Commands

```bash
# List all contexts (current highlighted in green)
cctx

# Switch to a context
cctx work

# Switch to previous context  
cctx -

# Show current context
cctx -c
```

### 🏗️ Settings Level Management

cctx respects [Claude Code's settings hierarchy](https://docs.anthropic.com/en/docs/claude-code/settings) with a simple, predictable approach:

1. **Enterprise policies** (highest priority)
2. **Command-line arguments** 
3. **Local project settings** (`./.claude/settings.local.json`)
4. **Shared project settings** (`./.claude/settings.json`)
5. **User settings** (`~/.claude/settings.json`) (lowest priority)

```bash
# Default: always uses user-level contexts (predictable)
cctx                       # Manages ~/.claude/settings.json

# Explicit flags for project/local contexts
cctx --in-project          # Manages ./.claude/settings.json
cctx --local               # Manages ./.claude/settings.local.json

# All commands work with any level
cctx --in-project work     # Switch to 'work' in project contexts
cctx --local staging       # Switch to 'staging' in local contexts
```

### 🛠️ Context Management

```bash
# Create new context from current settings
cctx -n project-alpha

# Delete a context
cctx -d old-project

# Rename a context
cctx -r old-name new-name

# Edit context with $EDITOR
cctx -e work

# Show context content (JSON)
cctx -s production

# Unset current context
cctx -u
```

### 📥📤 Import/Export

```bash
# Export context to file
cctx --export production > prod-settings.json

# Import context from file
cctx --import staging < staging-settings.json

# Share contexts between machines
cctx --export work | ssh remote-host 'cctx --import work'
```

### 🖥️ Shell Completions

Enable tab completion for faster workflow:

```bash
# Bash
cctx --completions bash > ~/.local/share/bash-completion/completions/cctx

# Zsh  
cctx --completions zsh > /usr/local/share/zsh/site-functions/_cctx

# Fish
cctx --completions fish > ~/.config/fish/completions/cctx.fish

# PowerShell
cctx --completions powershell > cctx.ps1
```

## 🏗️ File Structure

Contexts are stored as individual JSON files at different levels:

**🏠 User Level (`~/.claude/`):**
```
📁 ~/.claude/
├── ⚙️ settings.json           # Active user context
└── 📁 settings/
    ├── 💼 work.json          # Work context  
    ├── 🏠 personal.json      # Personal context
    └── 🔒 .cctx-state.json   # State tracking
```

**📁 Project Level (`./.claude/`):**
```
📁 ./.claude/
├── ⚙️ settings.json           # Shared project context
├── 🔒 settings.local.json     # Local project context (gitignored)
└── 📁 settings/
    ├── 🚀 staging.json       # Staging context
    ├── 🏭 production.json    # Production context
    ├── 🔒 .cctx-state.json   # Project state
    └── 🔒 .cctx-state.local.json # Local state
```

## 🎭 Interactive Mode

When no arguments are provided, cctx enters interactive mode:

- 🔍 **fzf integration** - Uses fzf if available for fuzzy search
- 🎯 **Built-in finder** - Fallback fuzzy finder when fzf not installed
- 🌈 **Color coding** - Current context highlighted in green
- ⌨️ **Keyboard navigation** - Arrow keys and type-ahead search

```bash
# Interactive context selection
cctx
```

## 💼 Common Workflows

### 🏢 Professional Setup

```bash
# Create restricted work context for safer collaboration
cctx -n work
cctx -e work  # Edit to add restrictions:
# - Read/Edit only in ~/work/** and current directory
# - Deny: docker, kubectl, terraform, ssh, WebFetch, WebSearch
# - Basic dev tools: git, npm, cargo, python only
```

### 🚀 Project-Based Contexts

```bash
# Create project-specific contexts
cctx -n client-alpha    # For client work
cctx -n side-project    # For personal projects  
cctx -n experiments     # For trying new things

# Switch based on current work
cctx client-alpha       # Restricted permissions
cctx experiments        # Full permissions for exploration
```

### 🔄 Daily Context Switching

```bash
# Morning: start with work context
cctx work

# Need full access for personal project  
cctx personal

# Quick switch back to work
cctx -

# Check current context anytime
cctx -c
```

### 🛡️ Security-First Approach

```bash
# Default restricted context for screen sharing
cctx work

# Full permissions only when needed
cctx personal

# Project-specific minimal permissions
cctx -n client-project
# Configure: only access to ~/projects/client/** 
```

### 🎯 Settings Level Workflows

**👤 User-Level Development:**
```bash
# Personal development with full permissions (default behavior)
cctx personal

# Work context with restrictions (default behavior)
cctx work
```

**📁 Project-Level Collaboration:**
```bash
# Shared team settings (committed to git)
cctx --in-project staging
cctx --in-project production

# Personal project overrides (gitignored)
cctx --local development
cctx --local debug
```

**🔄 Multi-Level Management:**
```bash
# Check current level (always shows helpful context)
cctx                    # Shows: 👤 User contexts + hints for project/local if available

# Switch levels in same directory
cctx personal           # User level (default)
cctx --in-project staging  # Project level  
cctx --local debug      # Local level
```

## 🔧 Advanced Usage

### 📝 Context Creation with Claude

Use Claude Code to help create specialized contexts:

```bash
# Create production-safe context
claude --model opus <<'EOF'
Create a production.json context file with these restrictions:
- Read-only access to most files
- No docker/kubectl/terraform access  
- No system file editing
- Limited bash commands for safety
- Based on my current ~/.claude/settings.json but secured
EOF
```

### 🎨 Custom Context Templates

```bash
# Create template contexts for different scenarios
cctx -n template-minimal     # Minimal permissions
cctx -n template-dev         # Development tools only
cctx -n template-ops         # Operations/deployment tools
cctx -n template-restricted  # Screen-sharing safe
```

### 🔄 Context Synchronization

```bash
# Sync contexts across machines
rsync -av ~/.claude/settings/ remote:~/.claude/settings/

# Or use git for version control
cd ~/.claude/settings
git init && git add . && git commit -m "Initial contexts"
git remote add origin git@github.com:user/claude-contexts.git
git push -u origin main
```

## 🛡️ Security Best Practices

### 🔒 Permission Isolation

1. **🏢 Work context** - Restricted permissions for professional use
2. **🏠 Personal context** - Full permissions for personal projects
3. **📺 Demo context** - Ultra-restricted for screen sharing/demos
4. **🧪 Testing context** - Isolated environment for experiments

### 🎯 Context Strategy

```bash
# Create permission hierarchy
cctx -n restricted   # No file write, no network, no system access
cctx -n development  # File access to ~/dev/**, basic tools only  
cctx -n full        # All permissions for personal use
cctx -n demo        # Read-only, safe for presentations
```

### 🔍 Regular Audits

```bash
# Review context permissions regularly
cctx -s work        # Check work context permissions
cctx -s personal    # Review personal context
cctx -s production  # Audit production context

# Quick security check
cctx -s restricted | grep -i "allow\|deny"
```

## 🎯 Tips & Tricks

### ⚡ Productivity Boosters

- 🔄 **Use `cctx -` frequently** - Quick toggle between two contexts
- 🎯 **Trust the defaults** - `cctx` (no flags) handles 90% of use cases perfectly
- 💡 **Follow the hints** - When cctx shows hints, they're contextually relevant
- ⌨️ **Set up aliases** - `alias work='cctx work'`, `alias home='cctx personal'`
- 📝 **Document your contexts** - Add comments in JSON for future reference

### 🛠️ Environment Setup

```bash
# Add to your shell profile (~/.bashrc, ~/.zshrc)
export EDITOR=code                    # For cctx -e
alias cx='cctx'                      # Shorter command
alias cxs='cctx -s'                  # Show context content
alias cxc='cctx -c'                  # Show current context

# Git hooks for automatic context switching
# Pre-commit hook to ensure proper context
#!/bin/bash
if [[ $(cctx -c) != "work" ]]; then
  echo "⚠️  Switching to work context for this repo"
  cctx work
fi
```

### 🔧 Integration Examples

```bash
# Tmux integration - show context in status bar
set -g status-right "Context: #(cctx -c) | %H:%M"

# VS Code integration - add to settings.json
"terminal.integrated.env.osx": {
  "CLAUDE_CONTEXT": "$(cctx -c 2>/dev/null || echo 'none')"
}

# Fish shell prompt integration
function fish_prompt
    set_color cyan
    echo -n (cctx -c 2>/dev/null || echo 'no-context')
    set_color normal
    echo -n '> '
end
```

## 🔧 Development & Release Tools

This project includes comprehensive automation tools:

### 🚀 Release Management

**Simple One-Command Release:**
```bash
# Automatic release with all quality checks
./quick-release.sh patch      # 0.1.0 -> 0.1.1
./quick-release.sh minor      # 0.1.0 -> 0.2.0
./quick-release.sh major      # 0.1.0 -> 1.0.0
```

The script automatically:
- ✅ Runs quality checks (fmt, clippy, test, build)
- ✅ Updates version in Cargo.toml
- ✅ Creates git commit and tag
- ✅ Pushes to GitHub
- ✅ Triggers GitHub Actions for binary builds and crates.io publishing

### 🛠️ Development Tasks

```bash
# Using justfile (install: cargo install just)
just check              # Run all quality checks
just release-patch      # Same as ./quick-release.sh patch
just setup              # Setup development environment
just audit              # Security audit
just completions fish   # Generate shell completions
```

## 🤝 Contributing

We welcome contributions! This project includes:

- 🔄 **Automated CI/CD** - GitHub Actions for testing and releases
- 🧪 **Quality gates** - Formatting, linting, and tests required
- 📦 **Multi-platform** - Builds for Linux, macOS, and Windows
- 🚀 **Auto-releases** - Semantic versioning with automated publishing

See [CLAUDE.md](CLAUDE.md) for detailed development guidelines.

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🎯 Design Philosophy (v0.1.1+)

**cctx follows the principle of "Predictable defaults with explicit overrides":**

- 🎯 **Default behavior is always the same** - uses user-level contexts (`~/.claude/settings.json`)
- 💡 **Helpful discovery** - shows hints when project/local contexts are available
- 🚀 **Simple when simple** - 90% of usage needs zero flags
- 🔧 **Explicit when needed** - `--in-project` and `--local` for specific cases

This approach eliminates surprises and cognitive overhead while maintaining full functionality.

## ⚠️ Compatibility Notice

**cctx** is designed to work with [Claude Code](https://github.com/anthropics/claude-code) configuration files. As Claude Code is actively developed by Anthropic, configuration formats and file structures may change over time.

**We are committed to maintaining compatibility:**
- 🔄 **Active monitoring** of Claude Code updates and changes
- 🚀 **Prompt updates** when configuration formats change
- 🛠️ **Backward compatibility** whenever possible
- 📢 **Clear migration guides** for breaking changes

If you encounter compatibility issues after a Claude Code update, please [open an issue](https://github.com/nwiizo/cctx/issues) and we'll address it promptly.

## 🙏 Acknowledgments

- 🎯 Inspired by [kubectx](https://github.com/ahmetb/kubectx) - the amazing Kubernetes context switcher
- 🤖 Built for [Claude Code](https://claude.ai/code) - Anthropic's CLI for Claude
- 🦀 Powered by [Rust](https://www.rust-lang.org/) - fast, safe, and beautiful

---

<div align="center">

**⭐ Star this repo if cctx makes your Claude Code workflow better!**

[🐛 Report Bug](https://github.com/nwiizo/cctx/issues) • [💡 Request Feature](https://github.com/nwiizo/cctx/issues) • [💬 Discussions](https://github.com/nwiizo/cctx/discussions)

</div>