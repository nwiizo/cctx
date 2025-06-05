# CLAUDE.md - cctx Project Documentation

## Project Overview

`cctx` (Claude Context) is a command-line tool for managing multiple Claude Code `settings.json` configurations.

## Architecture

### Core Concept
- **Context**: A saved Claude Code configuration stored as a JSON file
- **Current Context**: The active configuration (`~/.claude/settings.json`)
- **Context Storage**: All contexts are stored in `~/.claude/settings/` as individual JSON files
- **State Management**: Current and previous context tracked in `~/.claude/settings/.cctx-state.json`

### File Structure
```
~/.claude/
├── settings.json           # Current active context (managed by cctx)
└── settings/
    ├── work.json          # Work context
    ├── personal.json      # Personal context
    ├── project-alpha.json # Project-specific context
    └── .cctx-state.json   # Hidden state file (tracks current/previous)
```

### Key Design Decisions
1. **File-based contexts**: Each context is a separate JSON file, making manual management possible
2. **Simple naming**: Filename (without .json) = context name
3. **Atomic operations**: Context switching is done by copying files
4. **Hidden state file**: Prefixed with `.` to hide from context listings

## Command Reference

### Basic Commands
- `cctx` - List contexts or interactive selection
- `cctx <name>` - Switch to context
- `cctx -` - Switch to previous context

### Management Commands
- `cctx -n <name>` - Create new context from current settings
- `cctx -d <name>` - Delete context
- `cctx -r <old> <new>` - Rename context
- `cctx -c` - Show current context name
- `cctx -e [name]` - Edit context with $EDITOR
- `cctx -s [name]` - Show context content
- `cctx -u` - Unset current context

### Import/Export
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

### Interactive Features
1. **fzf integration**: Auto-detect and use if available
2. **Built-in fuzzy finder**: Fallback when fzf not available
3. **Color coding**: Current context highlighted in green

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

## Release Checklist

Before releasing a new version:

1. Update version in `Cargo.toml`
2. Run full test suite
3. Test on macOS, Linux, and Windows (WSL)
4. Update README if needed
5. Check for any deprecation warnings
6. Ensure all dependencies are up to date

## Notes for AI Assistants

When working on this codebase:

1. **Always run `cargo clippy` and fix warnings** before suggesting code
2. **Test your changes** - don't assume code works
3. **Preserve existing behavior** unless explicitly asked to change it
4. **Follow Rust idioms** and best practices
5. **Keep the kubectx-inspired UX** - simple, fast, intuitive
6. **Document any new features** in both code and README
7. **Consider edge cases** - empty states, missing files, permissions

Remember: This tool is about speed and simplicity. Every feature should make context switching faster or easier, not more complex.
