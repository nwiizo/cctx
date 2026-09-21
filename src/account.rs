use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

use crate::storage::validate_name;

pub fn default_config_dir() -> Result<PathBuf> {
    match std::env::var_os("CLAUDE_CONFIG_DIR") {
        Some(path) if !path.is_empty() => Ok(PathBuf::from(path)),
        _ => Ok(dirs::home_dir()
            .context("Failed to get home directory")?
            .join(".claude")),
    }
}

/// Account directories never depend on the currently selected Claude profile.
pub struct Accounts {
    root: PathBuf,
}

impl Accounts {
    pub fn new() -> Result<Self> {
        let home = match std::env::var_os("CCTX_HOME") {
            Some(path) if !path.is_empty() => PathBuf::from(path),
            _ => dirs::home_dir()
                .context("Failed to get home directory")?
                .join(".cctx"),
        };
        Ok(Self {
            root: home.join("accounts"),
        })
    }

    fn path(&self, name: &str) -> Result<PathBuf> {
        validate_name(name)?;
        if name == "default" {
            bail!("'default' is reserved for the existing Claude configuration");
        }
        Ok(self.root.join(name))
    }

    pub fn add(&self, name: &str) -> Result<()> {
        let path = self.path(name)?;
        fs::create_dir_all(&self.root)?;
        // create_dir deliberately refuses existing profiles, including symlinks.
        fs::create_dir(&path)
            .with_context(|| format!("Cannot create account {name:?}; it may already exist"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
        }
        println!("Created account {name:?}. Log in with: cctx --account {name:?} --login");
        Ok(())
    }

    pub fn resolve(&self, name: &str) -> Result<PathBuf> {
        if name == "default" {
            return Ok(dirs::home_dir()
                .context("Failed to get home directory")?
                .join(".claude"));
        }
        let path = self.path(name)?;
        let metadata = fs::symlink_metadata(&path).with_context(|| {
            format!("Unknown account {name:?}; create it with cctx --add-account {name:?}")
        })?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            bail!(
                "Account directory must be a real directory: {}",
                path.display()
            );
        }
        // A stable absolute path is important for Claude's Keychain namespace.
        fs::canonicalize(path).context("Failed to resolve account directory")
    }

    pub fn list(&self) -> Result<()> {
        println!("default\t{}", self.resolve("default")?.display());
        if !self.root.exists() {
            return Ok(());
        }
        let mut names = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    if validate_name(name).is_ok() && name != "default" {
                        names.push(name.to_string());
                    }
                }
            }
        }
        names.sort();
        for name in names {
            println!("{name}\t{}", self.resolve(&name)?.display());
        }
        Ok(())
    }
}

pub fn check_auth_overrides() -> Result<()> {
    let overrides = [
        "ANTHROPIC_API_KEY",
        "ANTHROPIC_AUTH_TOKEN",
        "CLAUDE_CODE_OAUTH_TOKEN",
        "ANTHROPIC_PROFILE",
        "CLAUDE_CODE_USE_BEDROCK",
        "CLAUDE_CODE_USE_VERTEX",
        "CLAUDE_CODE_USE_FOUNDRY",
    ];
    let present: Vec<_> = overrides
        .into_iter()
        .filter(|name| std::env::var_os(name).is_some_and(|value| !value.is_empty()))
        .collect();
    if !present.is_empty() {
        bail!("Account login may be overridden by {}. Unset these variables before selecting an account.", present.join(", "));
    }
    Ok(())
}

/// Only authentication commands are delegated. Never start a coding session.
pub fn authenticate(config_dir: Option<PathBuf>, action: &str) -> Result<ExitStatus> {
    check_auth_overrides()?;
    let mut command = Command::new("claude");
    match config_dir {
        Some(path) => command.env("CLAUDE_CONFIG_DIR", path),
        None => command.env_remove("CLAUDE_CONFIG_DIR"),
    };
    command.args(["auth", action]);
    if action == "status" {
        command.arg("--text");
    }
    command
        .status()
        .context("Failed to run Claude authentication; ensure 'claude' is on PATH")
}
