# cctx - Claude Context Switcher

A fast way to switch between Claude Code contexts (`~/.claude/settings.json`).

## Installation

```bash
cargo install cctx
```

Or build from source:

```bash
git clone https://github.com/nwiizo/cctx.git
cd cctx
cargo build --release
sudo cp target/release/cctx /usr/local/bin/
```

### Shell Completions

Generate and install shell completions for better usability:

```bash
# Bash
cctx --completions bash > ~/.local/share/bash-completion/completions/cctx

# Zsh (add to your ~/.zshrc)
cctx --completions zsh > /usr/local/share/zsh/site-functions/_cctx

# Fish
cctx --completions fish > ~/.config/fish/completions/cctx.fish

# PowerShell (add to your profile)
cctx --completions powershell > cctx.ps1
```

## Quick Setup

After installation, you can quickly set up your contexts using Claude Code:

```bash
# Create your current settings as a 'private' context (full permissions)
cctx -n private

# Create a restricted 'work' context for safer collaboration
claude --model opus <<'EOF'
Create a work.json context file at ~/.claude/settings/work.json with the following restricted permissions:
- Basic development tools: git, npm, yarn, cargo, python
- File operations: ls, cat, grep, find, mkdir, touch, cp, mv, rm
- Read/Edit/Write only in ~/work/** and current directory (./**)
- Deny: docker, kubectl, terraform, ssh, WebFetch, WebSearch, sudo commands
- Environment: shorter timeouts, /bin/bash shell
- Copy the structure from my current ~/.claude/settings.json but with these restrictions
EOF

# Switch to work context for safer operations
cctx work

# Switch back to private when you need full access
cctx private
```

## Usage

```bash
# List all contexts (current context is highlighted)
cctx

# Show only current context (quiet mode)
cctx -q

# Switch to a context
cctx my-context

# Switch to previous context
cctx -

# Create a new context from current settings
cctx -n my-new-context

# Delete a context
cctx -d unwanted-context

# Rename a context
cctx -r old-name

# Show current context name
cctx -c

# Edit context with $EDITOR
cctx -e [context-name]

# Show context content
cctx -s [context-name]

# Unset current context (remove ~/.claude/settings.json)
cctx -u

# Generate shell completions
cctx --completions bash|zsh|fish|powershell
```

## Interactive Mode

If you have `fzf` installed, `cctx` will use it for interactive selection. Otherwise, it uses a built-in fuzzy finder.

```bash
# Interactive context selection
cctx
```

## File Structure

Contexts are stored as individual JSON files in `~/.claude/settings/`:

```
~/.claude/
├── settings.json           # Current active context (managed by cctx)
└── settings/
    ├── work.json          # Work context
    ├── personal.json      # Personal context
    ├── project-alpha.json # Project-specific context
    └── .cctx-state.json   # cctx state (current/previous context)
```

You can also manually create or edit context files in `~/.claude/settings/` and cctx will recognize them.

## Common Workflows

### Setting up contexts with Claude

Use Claude Code to help create specialized contexts:

```bash
# Create a secure context for production work
claude --model opus <<'EOF'
Help me create a production.json context file with these requirements:
- Read-only access to most files
- No docker/kubectl access
- No system file editing
- Limited bash commands for safety
- Based on my current ~/.claude/settings.json
EOF

# Create a testing context with specific tool access
claude --model opus <<'EOF'
Create a testing.json context that allows:
- All testing frameworks (jest, vitest, pytest, cargo test)
- CI/CD tools but no deployment commands
- File editing in test directories only
- No production system access
EOF
```

### Daily workflow

```bash
# Morning: switch to work context for safer operations
cctx work

# Need full access for personal project
cctx personal

# Quick switch back to work
cctx -

# Check what context you're in
cctx -c
```

## Examples

### Quick context switching

```bash
# Working on different projects
cctx work
# ... do work stuff ...
cctx personal
# ... work on personal project ...
cctx -  # back to work
```

### Creating project-specific contexts

```bash
# Set up your Claude settings for a project
# ... configure permissions, env vars, etc ...

# Save as a context
cctx -n project-alpha

# Later, switch back to it
cctx project-alpha
```

### Managing contexts

```bash
# See all contexts
cctx

# Edit work context settings
cctx -e work

# Check what's in a context
cctx -s production

# Clean up old contexts
cctx -d old-project
```

## Import/Export

```bash
# Export a context
cctx --export production > prod-settings.json

# Import a context
cctx --import staging < staging-settings.json
```

## Tips

- Set `EDITOR` environment variable to use your preferred editor with `-e`
- Context names can be anything except `-`, `.`, `..`, or contain `/`
- You can manually copy JSON files to `~/.claude/settings/` to add contexts
- The state file (`.cctx-state.json`) tracks current and previous contexts
- Use Claude Code to generate specialized contexts based on your needs
- Always test new contexts in a safe environment before using in production

## Security Best Practices

1. **Use restricted contexts for collaborative work** - Switch to `work` context when sharing screens or working with others
2. **Separate personal and professional contexts** - Keep different permission sets for different types of work
3. **Regular context audits** - Use `cctx -s <context>` to review permissions periodically
4. **Project-specific contexts** - Create minimal permission contexts for specific projects

## Integration with Claude Code

cctx is designed to work seamlessly with Claude Code. You can:

- Use Claude to generate context configurations
- Ask Claude to help optimize your permission settings
- Get Claude to create project-specific contexts based on your needs
- Use Claude to audit and improve your context security