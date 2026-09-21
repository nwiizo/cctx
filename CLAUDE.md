# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

cctx is a kubectx-style CLI (Rust edition 2024, MSRV 1.85, single binary crate with `src/main.rs` as the only entry point) that switches Claude Code `settings.json` files ("contexts") and selects isolated account profiles through shell integration. Account selection must not start a coding session. It is published on crates.io as `cctx` and as prebuilt binaries on GitHub Releases; 0.2.0 is the first release with accounts. Keep changes small and scoped to what is asked.

## Commands

```bash
just check                                                  # fmt --check, clippy, tests, release build, all --locked; identical to the CI gate
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --locked                           # all tests live in tests/cli.rs (no unit tests)
cargo test --test cli accounts_isolate                      # one integration test by name prefix
cargo test --test claude_e2e -- --ignored                   # opt-in E2E against two real logins; see docs/e2e.md
cargo build --release --locked
cargo audit                                                 # needs cargo-audit; CI runs it too
cargo run -- --completions fish                             # run the binary from source
cargo +1.85.0 check --locked                                # what the CI msrv job runs
```

Integration tests spawn the compiled binary through `CARGO_BIN_EXE_cctx`. The `Sandbox` helper in `tests/cli.rs` points `HOME`, `USERPROFILE`, `CLAUDE_CONFIG_DIR`, and `CCTX_HOME` at a temp dir, sets `NO_COLOR=1`, and strips the auth-override variables. Ordinary integration tests must go through it so they never touch the real `~/.claude` or `~/.cctx`. The shell test runs the real `bash`, `zsh`, and `fish` when they are installed and skips missing ones. User-authorized real-account E2E is opt-in and documented in `docs/e2e.md`.

## Release

`./quick-release.sh patch|minor|major` is the only release path (the justfile has no release recipe). It refuses to run unless the tree is clean, the branch is `main`, and `main` matches `origin/main`. It then runs the same `--locked` fmt/clippy/test/build gate as CI, bumps `Cargo.toml`, commits `chore(release): bump version to X.Y.Z`, tags `vX.Y.Z`, and pushes. When `Cargo.toml` already holds an unreleased version, tag that commit directly instead of bumping again.

The tag triggers `.github/workflows/release.yml` (binaries `cctx-linux-x86_64`, `cctx-linux-x86_64-musl`, `cctx-windows-x86_64.exe`, `cctx-macos-x86_64`, `cctx-macos-aarch64` plus the GitHub release) and `publish.yml` (crates.io). The publish job fails fast when the `CARGO_REGISTRY_TOKEN` repository secret is missing; after fixing the secret, re-run the failed job rather than re-tagging. The crate ships only `src/`, `shell/`, README and LICENSE through `include` in `Cargo.toml`, and `cargo publish --dry-run --locked` reproduces everything except the upload.

`ci.yml` runs on pushes to `main`, on pull requests, and on manual dispatch: fmt, clippy, tests, and a release build on Ubuntu, macOS, and Windows, plus `cargo audit` and `cargo check` on Rust 1.85. A feature branch gets CI through a pull request, not on push. Every cargo invocation in CI uses `--locked`, so commit `Cargo.lock` changes together with `Cargo.toml` changes.

## Documentation

The README is the user manual and mirrors the binary: its Install section names the release artifacts above and the `cargo install cctx --locked` / `cargo install --git ... --locked` commands, Shell setup shows the `--shell-init` snippets, Everyday commands and the Accounts / Settings contexts blocks list the flags, Completion shows output paths, and Development lists the same commands as this file and `just check`. When a flag, artifact name, or check command changes, update README, this file, and the release-notes body in `release.yml` in the same commit, and make sure `cargo run -- --help` agrees with what the README shows. `docs/e2e.md` describes the opt-in real-account test only.

## Architecture

### Dispatch (`src/cli.rs`, `src/main.rs`)

`Cli` is one flat clap struct, not subcommands. Every mode flag is a member of the `operation` `ArgGroup`, so clap rejects combinations such as `-n -d`; the second positional (`new_name`) is only valid with `-r`, and `-r`, `-n`, `-d` fail with an explicit error when a name is missing (there are no interactive prompts). `main.rs` handles completions and shell initialization, account management, config-dir resolution, account path/authentication operations, then settings-context operations. Account path/authentication operations never build a `ContextManager`. A new mode must be added to the group and placed in this chain.

### Two storage roots

- **Config dir** (`account::default_config_dir`): `CLAUDE_CONFIG_DIR` or `~/.claude`. Holds the live `settings.json` and a `settings/` directory with one `<name>.json` per context plus the hidden `.cctx-state.json`.
- **Account root** (`account::Accounts`): `CCTX_HOME` or `~/.cctx`, with `accounts/<name>/`. Each named account directory is an independent Claude config dir. The name `default` is reserved for the original `~/.claude` configuration with CLAUDE_CONFIG_DIR unset. `--account X` scopes context operations to `accounts/X` via `ContextManager::with_config_dir`; selection without another operation changes the current shell environment.

### Contexts (`src/context.rs`)

`ContextManager::with_config_dir` takes one config dir and derives everything from it: contexts in `<config>/settings/<name>.json`, the switch target `<config>/settings.json`, and state in `<config>/settings/.cctx-state.json`. `list_contexts` skips dot-files, which is how the state file stays out of listings. Project-level `./.claude/settings.json` and `settings.local.json` are Claude Code's own layering and are deliberately not managed by cctx (the `--in-project`/`--local` levels and the merge/unmerge feature were removed in 0.2.0; do not reintroduce them).

`State` (`src/state.rs`) tracks only `current` and `previous`. Delete and rename must keep both fields consistent, and `switch_to_previous` fails when `previous` is unset.

### Invariants in `src/storage.rs`

- `validate_name` is the single naming rule for contexts and accounts: no empty name, `-`, leading dot, trailing dot or space, control characters, or `/\:*?"<>|`. Call it before building any path from user input.
- `read_settings` / `validate_settings` require a JSON object. A switch validates the context file before writing, so a broken context can never replace the active settings.
- `atomic_write` (tempfile in the same directory, then persist) is how settings, contexts, and state are written. Use it for any file Claude Code may read while cctx runs.

### Accounts and credentials (`src/account.rs`)

cctx never reads, copies, or writes Claude credentials. `--add-account` creates an empty `0700` directory and fails if anything already exists at that path, including a symlink. `--login`, `--logout` and `--status` delegate only to `claude auth`. There is no coding-session launcher. Shell integration (`--shell-init fish/bash/zsh`) intercepts exactly `cctx --account NAME`, resolves the profile through `--shell-path`, then changes the shell environment. `default` must unset CLAUDE_CONFIG_DIR, not set it to ~/.claude: explicit configuration uses a different Claude global-state file and Keychain namespace. `resolve` rejects symlinks and canonicalizes named profiles. Inherited API keys, OAuth tokens, Anthropic profile or cloud-provider selectors cause selection/authentication to fail without printing their values.

### Shell integration and completions

`shell/cctx.sh` and `shell/cctx.fish` are embedded with `include_str!` by `src/shell.rs` and printed by `--shell-init`. The shell function intercepts exactly `cctx --account NAME`, asks the binary for the validated path with the hidden `--shell-path`, and exports or unsets `CLAUDE_CONFIG_DIR`; every other invocation is passed to the binary unchanged. The README's Fish snippet guards the call with `status is-interactive; and command -sq cctx`; keep any change to the scripts compatible with that and with `eval "$(cctx --shell-init bash)"`. `src/completions.rs` generates clap completions with the default config dir's context names injected as possible values for the positional argument (needs clap's `string` feature), so the script is a snapshot taken at generation time.

## UX rules

There is exactly one kind of context (the selected config dir's `settings/`) and no interactive mode, so every command is scriptable and fails with a message instead of prompting. Output stays terse and kubectx-like: green for success and the current context, red for deletion and errors, no file paths in ordinary listings. Every feature must make switching faster or simpler.
