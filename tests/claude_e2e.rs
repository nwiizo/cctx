//! Opt-in E2E against installed cctx, Fish and two real Claude logins.
//! Never run this in CI: it requires user login and consumes a few model tokens.
use std::process::Command;

#[test]
#[ignore = "requires installed cctx, fish, claude, and authenticated default/secondary accounts"]
fn real_accounts_switch_without_launching_and_both_answer() {
    let directory = tempfile::tempdir().unwrap();
    let status = Command::new("fish")
        .args([
            "--no-config",
            "-c",
            r#"
            cctx --shell-init fish | source
            cctx --account default >/dev/null; or exit
            claude auth status; or exit
            cctx --account secondary >/dev/null; or exit
            claude auth status; or exit
            cctx --account default >/dev/null; or exit
            claude auth status
        "#,
        ])
        .current_dir(directory.path())
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "Authentication check failed: {}",
        String::from_utf8_lossy(&status.stderr)
    );
    let identities: Vec<serde_json::Value> = serde_json::Deserializer::from_slice(&status.stdout)
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(identities.len(), 3);
    for identity in &identities {
        assert_eq!(identity["loggedIn"], true);
    }
    assert!(identities[0]["email"].is_string());
    assert!(identities[1]["email"].is_string());
    assert_ne!(
        identities[0]["email"], identities[1]["email"],
        "Use two different accounts"
    );
    assert_eq!(identities[0]["email"], identities[2]["email"]);

    for account in ["default", "secondary"] {
        let response = Command::new("fish")
            .args(["--no-config", "-c", r#"
                cctx --shell-init fish | source
                cctx --account "$argv[1]" >/dev/null; or exit
                claude -p 'Reply with exactly CCTX_E2E_OK' --model haiku --safe-mode --tools '' --strict-mcp-config --no-session-persistence --max-budget-usd 0.10
            "#, "--", account])
            .current_dir(directory.path())
            .output().unwrap();
        assert!(
            response.status.success(),
            "{account}: {}",
            String::from_utf8_lossy(&response.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&response.stdout).trim(),
            "CCTX_E2E_OK",
            "{account}"
        );
        eprintln!("{account}: identity and real Claude response verified");
    }
}
