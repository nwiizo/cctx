use std::fs;
use std::process::{Command, Output};
use tempfile::TempDir;

struct Sandbox(TempDir);

impl Sandbox {
    fn new() -> Self {
        Self(tempfile::tempdir().unwrap())
    }

    fn command(&self) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_cctx"));
        cmd.current_dir(self.0.path())
            .env("HOME", self.0.path())
            .env("USERPROFILE", self.0.path())
            .env("CCTX_HOME", self.0.path().join("profiles"))
            .env("CLAUDE_CONFIG_DIR", self.0.path().join("claude"))
            .env("CCTX_INTERACTIVE", "0")
            .env("NO_COLOR", "1");
        for name in [
            "ANTHROPIC_API_KEY",
            "ANTHROPIC_AUTH_TOKEN",
            "CLAUDE_CODE_OAUTH_TOKEN",
            "ANTHROPIC_PROFILE",
            "CLAUDE_CODE_USE_BEDROCK",
            "CLAUDE_CODE_USE_VERTEX",
            "CLAUDE_CODE_USE_FOUNDRY",
        ] {
            cmd.env_remove(name);
        }
        cmd
    }

    fn ok(&self, args: &[&str]) -> Output {
        let output = self.command().args(args).output().unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }
}

#[cfg(unix)]
#[test]
fn authentication_uses_isolated_directory_and_preserves_exit_status() {
    use std::os::unix::fs::PermissionsExt;
    let s = Sandbox::new();
    s.ok(&["--add-account", "work"]);
    let bin = s.0.path().join("bin");
    fs::create_dir(&bin).unwrap();
    let executable = bin.join("claude");
    fs::write(
        &executable,
        "#!/bin/sh\nprintf '%s\\n' \"$CLAUDE_CONFIG_DIR\" \"$PWD\" \"$@\"\nexit 23\n",
    )
    .unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
    let output = s
        .command()
        .env("PATH", &bin)
        .args(["--account", "work", "--status"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(23));
    let text = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<_> = text.lines().collect();
    assert_eq!(
        std::path::Path::new(lines[0]),
        fs::canonicalize(s.0.path().join("profiles/accounts/work")).unwrap()
    );
    assert_eq!(
        std::path::Path::new(lines[1]),
        fs::canonicalize(s.0.path()).unwrap()
    );
    assert_eq!(&lines[2..], &["auth", "status", "--text"]);
    let default = s
        .command()
        .env("PATH", &bin)
        .args(["--account", "default", "--status"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8(default.stdout).unwrap().lines().next(),
        Some("")
    );
    for (flag, expected) in [
        ("--login", vec!["auth", "login"]),
        ("--status", vec!["auth", "status", "--text"]),
        ("--logout", vec!["auth", "logout"]),
    ] {
        let output = s
            .command()
            .env("PATH", &bin)
            .args(["--account", "work", flag])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(23));
        assert_eq!(
            String::from_utf8(output.stdout)
                .unwrap()
                .lines()
                .skip(2)
                .collect::<Vec<_>>(),
            expected
        );
    }
    let output = s
        .command()
        .env("PATH", &bin)
        .env("ANTHROPIC_API_KEY", "test-secret-do-not-print")
        .args(["--account", "work", "--shell-path"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(error.contains("ANTHROPIC_API_KEY"));
    assert!(!error.contains("test-secret-do-not-print"));
}

#[test]
fn merge_history_survives_rename_and_full_unmerge() {
    let s = Sandbox::new();
    s.ok(&["-n", "target"]);
    fs::write(
        s.0.path().join("source.json"),
        r#"{"model":"sonnet","permissions":{"deny":["Bash"]}}"#,
    )
    .unwrap();
    s.ok(&["--merge-from", "source.json", "--merge-full", "target"]);
    s.ok(&["-r", "target", "renamed"]);
    s.ok(&["--unmerge", "source.json", "--merge-full", "renamed"]);
    let result: serde_json::Value =
        serde_json::from_slice(&s.ok(&["-s", "renamed"]).stdout).unwrap();
    assert!(result.get("model").is_none());
    assert_eq!(result["permissions"]["deny"], serde_json::json!([]));
}

#[test]
fn completions_include_account_commands_for_every_shell() {
    let s = Sandbox::new();
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        let output = s.ok(&["--completions", shell]);
        let completion = String::from_utf8(output.stdout).unwrap();
        let option = if shell == "fish" {
            "-l account"
        } else {
            "--account"
        };
        assert!(completion.contains(option), "{shell}");
    }
}

#[test]
fn accounts_isolate_settings_and_preserve_existing_login() {
    let s = Sandbox::new();
    let existing = s.0.path().join("claude");
    fs::create_dir_all(&existing).unwrap();
    fs::write(existing.join(".credentials.json"), "do not copy").unwrap();
    fs::write(existing.join("settings.json"), r#"{"model":"sonnet"}"#).unwrap();
    s.ok(&["--add-account", "work"]);
    s.ok(&["--add-account", "personal"]);
    s.ok(&["--account", "work", "-n", "restricted"]);
    s.ok(&["--account", "work", "restricted"]);
    let path = s.ok(&["--account", "work", "--account-path"]);
    let path = std::path::PathBuf::from(String::from_utf8(path.stdout).unwrap().trim());
    assert!(!path.join(".credentials.json").exists());
    assert_eq!(
        fs::read_to_string(path.join("settings.json")).unwrap(),
        "{}"
    );
    assert_eq!(
        String::from_utf8(s.ok(&["--account", "work", "-c"]).stdout)
            .unwrap()
            .trim(),
        "restricted"
    );
    assert!(s.ok(&["--account", "personal", "-c"]).stdout.is_empty());
    assert_eq!(
        fs::read_to_string(existing.join("settings.json")).unwrap(),
        r#"{"model":"sonnet"}"#
    );
    let list = String::from_utf8(s.ok(&["--accounts"]).stdout).unwrap();
    assert!(list.contains("work") && list.contains("personal"));
    assert!(!s
        .command()
        .args(["--add-account", "work"])
        .status()
        .unwrap()
        .success());
    assert!(!s
        .command()
        .args(["--account", "missing", "--run"])
        .status()
        .unwrap()
        .success());
}

#[test]
fn legacy_contexts_respect_config_dir_and_rename_accepts_two_names() {
    let s = Sandbox::new();
    s.ok(&["-n", "one"]);
    s.ok(&["-n", "two"]);
    s.ok(&["one"]);
    s.ok(&["two"]);
    s.ok(&["-"]);
    s.ok(&["-r", "one", "renamed"]);
    assert_eq!(
        String::from_utf8(s.ok(&["-c"]).stdout).unwrap().trim(),
        "renamed"
    );
    assert!(s.0.path().join("claude/settings/renamed.json").exists());
    assert!(!s
        .command()
        .args(["-d", "renamed"])
        .output()
        .unwrap()
        .status
        .success());
}

#[test]
fn invalid_context_cannot_overwrite_active_settings() {
    let s = Sandbox::new();
    s.ok(&["-n", "valid"]);
    s.ok(&["valid"]);
    let base = s.0.path().join("claude");
    fs::write(base.join("settings/broken.json"), "not json").unwrap();
    assert!(!s.command().arg("broken").output().unwrap().status.success());
    assert_eq!(
        fs::read_to_string(base.join("settings.json")).unwrap(),
        "{}"
    );
    assert_eq!(
        String::from_utf8(s.ok(&["-c"]).stdout).unwrap().trim(),
        "valid"
    );
}

#[test]
fn conflicting_modes_and_unsafe_names_are_rejected() {
    let s = Sandbox::new();
    for args in [
        vec!["-n", "-d", "x"],
        vec!["--local", "--in-project"],
        vec!["-n", ".hidden"],
        vec!["-s", "../outside"],
        vec!["--add-account", "../escape"],
    ] {
        assert!(
            !s.command().args(&args).output().unwrap().status.success(),
            "{args:?}"
        );
    }
}

#[cfg(unix)]
#[test]
fn shell_switches_and_restores_default_without_starting_claude() {
    use std::os::unix::fs::PermissionsExt;
    let s = Sandbox::new();
    s.ok(&["--add-account", "work"]);
    let bin = s.0.path().join("bin");
    fs::create_dir(&bin).unwrap();
    std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_cctx"), bin.join("cctx")).unwrap();
    fs::write(bin.join("claude"), "#!/bin/sh\ntouch claude-was-started\n").unwrap();
    fs::set_permissions(bin.join("claude"), fs::Permissions::from_mode(0o700)).unwrap();
    let mut path = vec![bin];
    path.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
    let path = std::env::join_paths(path).unwrap();
    let scripts = [
        (
            "bash",
            r#"eval "$(cctx --shell-init bash)"; cctx --account work || exit; test "$CLAUDE_CONFIG_DIR" = "$EXPECTED" || exit 10; cctx --account missing && exit 11; test "$CLAUDE_CONFIG_DIR" = "$EXPECTED" || exit 12; cctx --account default || exit; test "${CLAUDE_CONFIG_DIR+x}" != x"#,
        ),
        (
            "zsh",
            r#"eval "$(cctx --shell-init zsh)"; cctx --account work || exit; test "$CLAUDE_CONFIG_DIR" = "$EXPECTED" || exit 10; cctx --account missing && exit 11; test "$CLAUDE_CONFIG_DIR" = "$EXPECTED" || exit 12; cctx --account default || exit; test "${CLAUDE_CONFIG_DIR+x}" != x"#,
        ),
        (
            "fish",
            r#"cctx --shell-init fish | source; cctx --account work; or exit; test "$CLAUDE_CONFIG_DIR" = "$EXPECTED"; or exit 10; cctx --account missing; and exit 11; test "$CLAUDE_CONFIG_DIR" = "$EXPECTED"; or exit 12; cctx --account default; or exit; not set -q CLAUDE_CONFIG_DIR"#,
        ),
    ];
    for (shell, script) in scripts {
        let Ok(executable) = which::which(shell) else {
            continue;
        };
        let mut command = Command::new(executable);
        if shell == "fish" {
            command.arg("--no-config");
        }
        if shell == "zsh" {
            command.arg("-f");
        }
        let output = command
            .args(["-c", script])
            .current_dir(s.0.path())
            .envs(
                s.command()
                    .get_envs()
                    .filter_map(|(key, value)| value.map(|v| (key, v))),
            )
            .env("PATH", &path)
            .env(
                "EXPECTED",
                fs::canonicalize(s.0.path().join("profiles/accounts/work")).unwrap(),
            )
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{shell}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert!(!s.0.path().join("claude-was-started").exists());
}
