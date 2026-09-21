# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

cctx is a kubectx-style CLI (Rust 2021, MSRV 1.81, single binary crate with `src/main.rs` as the only entry point) that switches Claude Code `settings.json` files ("contexts") and selects isolated account profiles through shell integration. Account selection must not start a coding session. The public repository is no longer archived. Keep changes small and scoped to what is asked.

## Commands

```bash
cargo build --release
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings   # CI form; `just check` runs `cargo clippy -- -D warnings`
cargo test                                                  # unit tests in src/merge.rs + integration tests in tests/cli.rs
cargo test --test cli                                       # integration tests only
cargo test --test cli accounts_isolate                      # one integration test by name prefix
cargo test merge::tests::partial_permissions                # one unit test
cargo run -- --completions fish                             # run the binary from source
just check                                                  # fmt --check, clippy, test, release build (same gate quick-release.sh uses)
cargo +1.81.0 check                                         # what the CI msrv job runs
```

Integration tests spawn the compiled binary through `CARGO_BIN_EXE_cctx`. The `Sandbox` helper in `tests/cli.rs` points `HOME`, `USERPROFILE`, `CLAUDE_CONFIG_DIR`, and `CCTX_HOME` at a temp dir and sets `CCTX_INTERACTIVE=0` and `NO_COLOR=1`. Ordinary integration tests must go through it so they never touch the real `~/.claude` or `~/.cctx`. User-authorized real-account E2E is opt-in and documented in `docs/e2e.md`.

## Release

`./quick-release.sh patch|minor|major` (or `just release-patch` etc.) is the only release path. It refuses to run unless the tree is clean, the branch is `main`, and `main` matches `origin/main`. It then runs the checks above, bumps `Cargo.toml`, commits, tags `vX.Y.Z`, and pushes. The tag triggers `.github/workflows/release.yml` (binaries for Linux glibc and musl, Windows, macOS x86_64 and aarch64) and `publish.yml` (crates.io, needs the `CARGO_REGISTRY_TOKEN` repository secret). `ci.yml` runs fmt, clippy, tests, and a release build on Ubuntu, macOS, and Windows, plus `cargo audit` and an MSRV `cargo check`.

## Architecture

### Dispatch (`src/cli.rs`, `src/main.rs`)

`Cli` is one flat clap struct, not subcommands. Every mode flag is a member of the `operation` `ArgGroup`, so clap rejects combinations such as `-n -d`; `--in-project` and `--local` conflict; the second positional (`new_name`) is only valid with `-r`. `main.rs` handles completions and shell initialization, account management, config-dir resolution, account path/authentication operations, then settings-context operations. Account path/authentication operations never build a `ContextManager`. A new mode must be added to the group and placed in this chain.

### Two storage roots

- **Config dir** (`account::default_config_dir`): `CLAUDE_CONFIG_DIR` or `~/.claude`. Holds the live `settings.json` and a `settings/` directory with one `<name>.json` per context plus hidden state and merge-history files.
- **Account root** (`account::Accounts`): `CCTX_HOME` or `~/.cctx`, with `accounts/<name>/`. Each named account directory is an independent Claude config dir. The name `default` is reserved for the original `~/.claude` configuration with CLAUDE_CONFIG_DIR unset. `--account X` scopes context operations to `accounts/X` via `ContextManager::with_config_dir`; selection without another operation changes the current shell environment.

### Settings levels (`src/context.rs`)

| Level | Target file | Contexts dir | State file |
|---|---|---|---|
| User (default) | `<config>/settings.json` | `<config>/settings/` | `.cctx-state.json` |
| Project (`--in-project`) | `./.claude/settings.json` | `./.claude/settings/` | `.cctx-state.json` |
| Local (`--local`) | `./.claude/settings.local.json` | `./.claude/settings/` | `.cctx-state.local.json` |

Project and Local share the same context files but keep separate state and targets. `list_contexts` skips dot-files, which is how state and merge history stay out of listings. This repository's own `.claude/settings/` is tracked and holds sample contexts, so `cctx --in-project` from the repo root lists them.

`State` (`src/state.rs`) tracks only `current` and `previous`. Delete and rename must keep both fields consistent, rename also moves the context's merge-history file, and `switch_to_previous` fails when `previous` is unset.

### Invariants in `src/storage.rs`

- `validate_name` is the single naming rule for contexts, accounts, and merge-history keys: no empty name, `-`, leading dot, trailing dot or space, control characters, or `/\:*?"<>|`. Call it before building any path from user input.
- `read_settings` / `validate_settings` require a JSON object. A switch validates the context file before writing, so a broken context can never replace the active settings.
- `atomic_write` (tempfile in the same directory, then persist) is how settings, contexts, state, and merge history are written. Use it for any file Claude Code may read while cctx runs.

### Accounts and credentials (`src/account.rs`)

cctx never reads, copies, or writes Claude credentials. `--add-account` creates an empty `0700` directory and fails if anything already exists at that path, including a symlink. `--login`, `--logout` and `--status` delegate only to `claude auth`. There is no coding-session launcher. Shell integration (`--shell-init fish/bash/zsh`) intercepts exactly `cctx --account NAME`, resolves the profile through `--shell-path`, then changes the shell environment. `default` must unset CLAUDE_CONFIG_DIR, not set it to ~/.claude: explicit configuration uses a different Claude global-state file and Keychain namespace. `resolve` rejects symlinks and canonicalizes named profiles. Inherited API keys, OAuth tokens, Anthropic profile or cloud-provider selectors cause selection/authentication to fail without printing their values.

### Merge (`src/merge.rs`, merge methods in `src/context.rs`)

`--merge-from SRC [TARGET]` and `--unmerge SRC [TARGET]`, optionally `--merge-full`. `ContextManager::merge_source` resolves `SRC` as: `user` = `<config>/settings.json`; anything ending in `.json` = literal path; otherwise a context name. `TARGET` defaults to `current`, which edits the live settings file directly rather than a saved context, with history recorded under the current context's name.

`MergeManager` appends a `MergeHistory` entry per merge to `.<context>-merge-history.json` in the contexts dir so `--unmerge` can remove exactly what one source added. The `merged_items` strings are a persisted format: permission-only merges record `allow:Rule`, full merges record `permissions.allow:Rule`, `env:NAME`, or a bare top-level key. `unmerge` accepts both prefixes and must keep doing so for existing history files. Permission-only unmerge leaves full-merge entries in the history; full unmerge removes both.

### Interactive and completions

`src/interactive.rs` extends `ContextManager` with the interactive flows. It uses `fzf` when it is on `PATH` and `TERM` is set, else `dialoguer::FuzzySelect`. `src/completions.rs` generates clap completions with the user-level context names injected as possible values for the positional argument (needs clap's `string` feature), so the script is a snapshot taken at generation time.

## UX rules

The default level is always User; Project and Local are reached only through explicit flags, and settings-level auto-detection was removed on purpose. Print the hints about project/local contexts only when such files exist. Output stays terse and kubectx-like: green for success and the current context, red for deletion and errors, no file paths in ordinary listings. Every feature must make switching faster or simpler.
