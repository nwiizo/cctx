# Real Claude Code E2E

The ordinary test suite uses isolated temporary configuration directories. The
opt-in E2E uses the installed `cctx`, Fish and real Claude Code credentials.
It makes one small Haiku request per account and consumes subscription/API usage.

## Prerequisites

1. Install this checkout: `cargo install --path . --locked --force`.
2. Keep your original Claude login as `default` (with CLAUDE_CONFIG_DIR unset).
3. Create the second profile: `cctx --add-account secondary`.
4. Authenticate it: `cctx --account secondary --login`. Enter browser codes only
   in your own terminal, never in test logs or chat.

## Run

```sh
cargo test --test claude_e2e -- --ignored --nocapture
```

This runs actual Fish integration and verifies:

- `default → secondary → default` selects two distinct authenticated identities.
- Returning to `default` restores the original identity.
- Plain `claude -p` answers `CCTX_E2E_OK` under each account.

The model requests run in a temporary working directory, with safe mode, no tools,
strict MCP configuration, no session persistence and a USD 0.10 budget cap each.
The test does not print account emails or credentials, log out, change saved
settings, or persist a global account choice. Errors and model output can still
appear in a failing test's diagnostics. Authentication state may be refreshed by
Claude Code itself.

## Verified locally

On 2026-09-21 with macOS, Fish and Claude Code 2.1.278:

- Both real Team accounts had different email addresses and organizations.
- The opt-in E2E passed (both accounts returned the expected response).
- An interactive Fish with the actual dotfiles loaded also switched
  `secondary → default`, and plain `claude auth status --text` showed the correct
  identity each time.
- Selecting an account itself did not start a coding session (covered by the
  separate shell integration regression test).

This verifies the local macOS workflow. Windows runtime behavior and Linux
credential storage have not been exercised locally; Bash/Zsh/Fish integration
tests run on available shells. Completion generation also supports PowerShell
and Elvish, but account-switch shell integration currently supports Bash/Zsh/Fish.
