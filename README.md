# cctx

[![Crates.io](https://img.shields.io/crates/v/cctx)](https://crates.io/crates/cctx)
[![CI](https://github.com/nwiizo/cctx/actions/workflows/ci.yml/badge.svg)](https://github.com/nwiizo/cctx/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Manage Claude Code accounts and settings contexts from the command line, kubectx
style. Each named account has its own Claude configuration directory; Claude Code
handles login, credentials and refresh, and cctx never reads or copies them. Saved
settings contexts switch the `settings.json` inside any account. Selecting an
account never starts a Claude coding session. Run `claude` yourself.

## Install

Pick one:

```sh
# From crates.io (needs Rust 1.85 or later)
cargo install cctx --locked

# Prebuilt binary from GitHub Releases (example: macOS on Apple Silicon)
curl -fsSLo cctx https://github.com/nwiizo/cctx/releases/latest/download/cctx-macos-aarch64
chmod +x cctx && sudo mv cctx /usr/local/bin/cctx

# From source
cargo install --git https://github.com/nwiizo/cctx --locked
```

Release assets: `cctx-linux-x86_64`, `cctx-linux-x86_64-musl` (static),
`cctx-windows-x86_64.exe`, `cctx-macos-x86_64` and `cctx-macos-aarch64`. Run the same
`cargo install` command again to upgrade.

Account commands need `claude` on PATH. The account workflow was developed against
Claude Code 2.1.278.

## Shell setup

Selecting an account changes `CLAUDE_CONFIG_DIR` in your current shell, which a binary
cannot do by itself, so load the shell function once in your shell configuration.
Every other command works without it.

Fish (`~/.config/fish/config.fish`):

```fish
if status is-interactive; and command -sq cctx
    cctx --shell-init fish | source
end
```

Bash (`~/.bashrc`) or Zsh (`~/.zshrc`):

```sh
eval "$(cctx --shell-init bash)"   # zsh: eval "$(cctx --shell-init zsh)"
```

Open a new terminal, or run the same line once in the current one.

## Everyday commands

| Task | Command |
| --- | --- |
| Create a second profile and log in once | `cctx --add-account secondary && cctx --account secondary --login` |
| List account profiles | `cctx --accounts` |
| Select the second account in this terminal | `cctx --account secondary` |
| Return to the original account | `cctx --account default` |
| Check the selected login | `claude auth status --text` |
| Start Claude Code yourself | `claude` |
| Save the current settings as a context, then activate it | `cctx -n work && cctx work` |

`cctx` uses flags, not subcommands: the account-list command is `cctx --accounts`,
not `cctx list --account`. Running `cctx` without arguments lists saved
**settings contexts**, not accounts.

## Accounts

The existing Claude login is available as `default`; there is no need to create or
log in to that profile again. Create another profile and log in once:

```sh
# Confirm the existing login
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
verified login identities; `--status` asks Claude Code for the actual account. Only
`cctx --account NAME` on its own selects an account; combined with `--login`,
`--logout`, `--status` or a settings operation it scopes that operation to the profile
without changing the current shell's selection.

Selection affects only the current shell and its future child processes. Other
terminals and already running Claude sessions keep their account, and no selection
is persisted globally. Shell aliases such as `alias cc-secondary='cctx --account secondary'`
are a convenient shortcut.

`default` restores the original configuration by **unsetting** `CLAUDE_CONFIG_DIR`.
Setting it explicitly to `~/.claude` is not equivalent: Claude Code 2.1.278 then uses a
different global state file and Keychain namespace. Named profiles live in
`~/.cctx/accounts/NAME`; `CCTX_HOME` overrides `~/.cctx`. Profile paths are
canonicalized before selection because macOS Keychain entries depend on the
configuration directory, so do not move or rename a logged-in profile directory.
New profiles are empty: credentials, history, settings and plugins are never copied,
and `claude --continue` or `claude --resume` see only the selected account's history.
On Unix, new account directories have mode `0700`.

When inherited API keys, OAuth tokens, Anthropic profile selectors or cloud-provider
selectors could override the account, cctx stops and names the variables without
printing their values. Settings files, managed policies and gateways are still
interpreted by Claude Code; check `--status` before using sensitive projects.
Project settings stay with the project and are not isolated by account.

To retire a profile, run `--logout` first. cctx deliberately has no profile-delete
command: conversation history and settings stay available for backup.

## Settings contexts

A context is a saved `settings.json`, independent of your login identity. `cctx work`
activates the settings named `work`, while `cctx --account work` selects an account
profile named `work`.

```sh
cctx -n personal                 # Save current settings (or {} if absent)
cctx -n work
cctx work                        # Activate saved settings
cctx -                           # Return to previous context
cctx                             # List contexts
cctx -c                          # Print current name
cctx -r work restricted          # Rename
cctx -e restricted               # Edit with EDITOR, VISUAL, or vi
cctx -s restricted               # Show JSON
cctx -d personal                 # Delete an inactive context
cctx -u                          # Remove active settings; keep saved contexts

# Apply the same operations inside one account
cctx --account secondary -n restricted
cctx --account secondary restricted
```

Contexts live in `settings/` inside the selected Claude directory. Existing
`~/.claude/settings/*.json` and `.cctx-state.json` files keep their formats.
Project-level `.claude/settings.json` files belong to the project and are not
managed by cctx.

Names must be visible filenames without path separators or platform-reserved
characters; hidden names and `-` are reserved. Activating or importing invalid JSON
or a non-object JSON value fails before anything is replaced. Files are written
through temporary files and atomic replacement, but a settings file plus its state
file are **not** a single transaction, so avoid concurrent settings edits within the
same profile. Separate accounts can run concurrently.

`-q` prints only the current context name.

## Import and export

```sh
cctx --export restricted > restricted.json
cctx --import staging < restricted.json
cctx --account secondary --export restricted | cctx --account work --import restricted
```

Import reads one JSON object from stdin and refuses to overwrite an existing
context. Edit the result with `-e` when it needs adjusting.

## Completion

```sh
cctx --completions fish > ~/.config/fish/completions/cctx.fish
cctx --completions zsh > ~/.zfunc/_cctx                      # any directory on $fpath
cctx --completions bash > ~/.local/share/bash-completion/completions/cctx
```

PowerShell and Elvish are also supported. Options come from the CLI definition, and
context-name candidates reflect the configuration directory at generation time, so
regenerate after changing saved contexts.

## Claude Code compatibility

The integration uses these official interfaces:

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
just check      # fmt, clippy, tests and a release build with --locked, same as CI
cargo audit     # dependency advisories (cargo install cargo-audit)
```

Without `just`:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --locked
cargo build --release --locked
```

CLI tests isolate HOME, USERPROFILE, CCTX_HOME and CLAUDE_CONFIG_DIR in temporary
directories, use a stand-in `claude` executable to check directory isolation and
exit codes, and verify that shell selection switches, rolls back on failure, restores
`default` and never starts Claude. An opt-in test against two real logins is
described in [docs/e2e.md](docs/e2e.md); it consumes usage and is excluded from
ordinary runs.

Releases: `./quick-release.sh patch|minor|major` on a clean, up-to-date `main` bumps
the version, tags `vX.Y.Z` and pushes. The tag builds the binaries above, creates
the GitHub release and publishes to crates.io.

MIT license. Inspired by kubectx.
