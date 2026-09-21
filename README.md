# cctx

Manage Claude Code accounts and settings contexts. Each named account has its own
Claude configuration directory; Claude Code handles login, credentials and refresh.
Existing settings-context commands continue to work.
Selecting an account never starts a Claude coding session. Run `claude` yourself.

## Everyday commands

| Task | Command |
| --- | --- |
| List account profiles | `cctx --accounts` |
| Select the second account in this terminal | `cctx --account secondary` |
| Return to the original account | `cctx --account default` |
| Check the selected login | `claude auth status --text` |
| Start Claude Code yourself | `claude` |

`cctx` uses flags, not subcommands: the account-list command is
`cctx --accounts`, not `cctx list --account`. Running `cctx` without arguments
lists saved **settings contexts**, not accounts. Account selection requires the
shell integration below.

## Install

```sh
git clone https://github.com/nwiizo/cctx.git
cd cctx
cargo install --path . --locked
```

Requires Rust 1.81 or later to build. Account commands require `claude` on PATH.
The account workflow has been developed against Claude Code 2.1.278. The version
published on crates.io may lag behind this checkout.

To replace an existing installation with this checkout:

```sh
cargo install --path . --locked --force
cctx --version
```

## Shell setup

For Fish, add this to `~/.config/fish/config.fish`:

```fish
if status is-interactive; and command -sq cctx
    cctx --shell-init fish | source
end
```

Open a new terminal to load it. To enable it in an already open Fish session,
run `cctx --shell-init fish | source` once.

For Bash, add this to `~/.bashrc`:

```bash
eval "$(cctx --shell-init bash)"
```

For Zsh, add `eval "$(cctx --shell-init zsh)"` to `~/.zshrc`.

## Accounts

The existing Claude login is available as `default`; there is no need to create
or log in to that profile again. Create another profile and log in once:

```sh
# Keep your existing Claude login as default
cctx --account default --status

# Set up a second login once
cctx --add-account secondary
cctx --account secondary --login
cctx --account secondary --status

# Select a login for this terminal, then launch Claude separately
cctx --account secondary
claude
# After exiting Claude, switch back
cctx --account default
claude

# Claude commands work normally with the selected account
claude --continue
claude --resume SESSION_ID
claude --model sonnet -p 'Explain this project'

# Inspect profiles or sign out of just one
cctx --accounts
cctx --account secondary --account-path
cctx --account secondary --logout
```

Names are local labels, not email addresses. `--accounts` lists directories, not
verified login identities; `--status` asks Claude Code for the actual account.
For example, after creating `secondary`, `cctx --accounts` lists `default` at
`~/.claude` and `secondary` at `~/.cctx/accounts/secondary` (as absolute paths).
Use `cctx --account secondary --status` to inspect that profile without changing
the current shell's selection. The same applies to `--login`, `--logout` and
settings operations: only `cctx --account NAME` on its own selects an account.

Selection affects only the current shell and future child processes. Other terminals
and already running Claude sessions keep their account. A new terminal starts with
its inherited environment; no account selection is persisted globally. The binary
alone cannot modify its parent shell, so account switching requires the shell function.

`default` restores the original Claude configuration by **unsetting**
`CLAUDE_CONFIG_DIR`. Setting it explicitly to `~/.claude` is not equivalent: Claude
Code 2.1.278 then looks for a different global state file and Keychain namespace.
Named profiles live in `~/.cctx/accounts/NAME`; `CCTX_HOME` overrides `~/.cctx`. Profile paths are
canonicalized before selection because macOS Keychain entries depend on
the configuration directory. Do not move or rename a logged-in profile directory.
New profiles are empty: credentials, history, settings and plugins are never copied.
`claude --continue` and `claude --resume` therefore use the selected account's
history; switching accounts does not transfer an existing conversation.
On Unix, newly created account directories have mode `0700`.

When inherited API keys, OAuth tokens, Anthropic profile selectors or cloud-provider
selectors could override the account, cctx stops and names the variables without
printing their values. Settings files, managed policies and gateways are still
interpreted by Claude Code; check `--status` before using sensitive projects.
Project settings remain associated with the project and are not isolated by account.

For convenience, add shell aliases yourself:

```sh
alias cc-work='cctx --account default'
alias cc-secondary='cctx --account secondary'
```

To retire a profile, use `--logout` first. cctx deliberately has no recursive
profile-delete command: conversation history and settings stay available for backup.

## Settings contexts

A context is a saved `settings.json`, independent of your login identity.
The original context commands are retained: `cctx work` activates settings named
`work`, while `cctx --account work` selects an account profile named `work`.

```sh
cctx -n personal                 # Save current settings (or {} if absent)
cctx -n work
cctx work                       # Activate saved settings
cctx -                          # Return to previous context
cctx                            # List contexts
cctx -c                         # Print current name
cctx -r work restricted          # Rename; prompts if new name is omitted
cctx -e restricted               # Edit with EDITOR, VISUAL, or vi
cctx -s restricted               # Show JSON
cctx -d personal                 # Delete an inactive context
cctx -u                         # Remove active settings; keep saved contexts

# Apply the same operations inside one account
cctx --account secondary -n restricted
cctx --account secondary restricted

# Explicit project scopes (cannot be combined with --account)
cctx --in-project -n staging     # ./.claude/settings.json
cctx --local -n development      # ./.claude/settings.local.json
```

User contexts live in `settings/` inside the selected Claude directory. Existing
`~/.claude/settings/*.json`, `.cctx-state.json` and merge-history files retain their
formats. Project and local scopes retain the existing shared `./.claude/settings/`
context library and separate state files.

Names must be visible filenames without path separators or platform-reserved
characters. Hidden names and `-` are reserved. Activating or importing invalid JSON
or a non-object JSON value fails before replacing settings. Individual file writes
use temporary files and atomic replacement; a settings file plus its state/history
file are **not** a single transaction. Avoid concurrent settings edits within the
same profile. Separate accounts can run concurrently.

Use `CCTX_INTERACTIVE=1 cctx` for interactive selection (`fzf` when available,
otherwise the built-in fuzzy selector). With no environment override, cctx lists
contexts. `-q` prints only the current context name.

## Import, export and merge

```sh
cctx --export restricted > restricted.json
cctx --import staging < restricted.json
cctx --merge-from restricted staging
cctx --merge-from ./extra.json --merge-full staging
cctx --merge-history staging
cctx --unmerge ./extra.json --merge-full staging
```

The source is `user` (the selected account's active settings), a context name, or a
path ending in `.json`. Omitting the target operates on the active settings file;
an explicit target changes the saved context. Activate that context to use it.

Permission merges add unique `allow`, `deny` and `ask` rules while preserving their
order. Full merges also add absent environment variables and top-level settings;
existing values win. Other fields within `permissions` are not merged.
History records additions, not complete snapshots. Unmerge removes recorded
additions; it does not reconstruct later manual edits. Permission-only unmerge
leaves full-merge entries available for a later `--unmerge --merge-full`.

## Completion

```sh
cctx --completions zsh > _cctx
cctx --completions bash > cctx.bash
cctx --completions fish > cctx.fish
```

PowerShell and Elvish are also supported. Options are generated from the CLI
definition, and context candidates reflect the configuration directory at generation
time. Regenerate after changing saved contexts.

## Claude Code compatibility

The current integration uses these official interfaces:

| Claude Code capability | cctx usage |
| --- | --- |
| Directory-scoped credentials and state | `--account NAME` changes `CLAUDE_CONFIG_DIR` in the current shell |
| Login, logout, identity | `--login`, `--logout`, `--status` delegate to `claude auth` |
| Resume a conversation | Select the account, then `claude --continue` or `claude --resume ID` |
| MCP servers and plugins | Select the account, then `claude mcp ...` or `claude plugin ...` |
| Worktrees, model, effort, safe mode | Use Claude's own flags when starting it |

No credential extraction, undocumented quota polling, automatic account rotation,
or cross-account transcript copying is used.

Sources: [environment variables](https://code.claude.com/docs/en/env-vars),
[authentication](https://code.claude.com/docs/en/authentication),
[CLI reference](https://code.claude.com/docs/en/cli-reference),
[2.1.278 release](https://github.com/anthropics/claude-code/releases/tag/v2.1.278).

## Development

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --all-targets
cargo build --release --locked
cargo audit
```

CLI tests isolate HOME, USERPROFILE, CCTX_HOME and CLAUDE_CONFIG_DIR in temporary
directories. Unix authentication tests use a controlled executable to check
directory isolation and child exit codes without accessing a real login. Shell tests
check account selection, failed-selection rollback, default restoration and that
selection never starts Claude.
See [docs/e2e.md](docs/e2e.md) for real Claude Code verification.

The macOS/Fish workflow was verified with two real accounts on Claude Code
2.1.278: `default → secondary → default` restored the expected identities, and
both accounts returned a response through plain `claude`. To repeat this check
after authenticating both accounts and installing this checkout:

```sh
cargo test --test claude_e2e -- --ignored --nocapture
```

This opt-in test makes a small model request under each account and consumes
usage. It is excluded from ordinary test runs.

MIT license. Inspired by kubectx.
