use anyhow::{Result, bail};
use colored::Colorize;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

use crate::state::State;
use crate::storage::{atomic_write, read_settings, validate_name, validate_settings};

/// Saved `settings.json` files inside one Claude configuration directory.
pub(crate) struct ContextManager {
    contexts_dir: PathBuf,
    settings_path: PathBuf,
    state_path: PathBuf,
}

impl ContextManager {
    pub(crate) fn new() -> Result<Self> {
        Self::with_config_dir(crate::account::default_config_dir()?)
    }

    /// Contexts live in `settings/` next to the `settings.json` they replace.
    pub(crate) fn with_config_dir(config_dir: PathBuf) -> Result<Self> {
        let contexts_dir = config_dir.join("settings");
        fs::create_dir_all(&contexts_dir)?;
        Ok(Self {
            settings_path: config_dir.join("settings.json"),
            state_path: contexts_dir.join(".cctx-state.json"),
            contexts_dir,
        })
    }

    fn context_path(&self, name: &str) -> Result<PathBuf> {
        validate_name(name)?;
        Ok(self.contexts_dir.join(format!("{name}.json")))
    }

    /// Path of a saved context that must already exist.
    fn existing_context(&self, name: &str) -> Result<PathBuf> {
        let path = self.context_path(name)?;
        if !path.is_file() {
            bail!("no context exists with the name {name:?}");
        }
        Ok(path)
    }

    /// Path for a context that must not exist yet.
    fn new_context(&self, name: &str) -> Result<PathBuf> {
        let path = self.context_path(name)?;
        if path.exists() {
            bail!("context {name:?} already exists");
        }
        Ok(path)
    }

    fn load_state(&self) -> Result<State> {
        State::load(&self.state_path)
    }

    fn save_state(&self, state: &State) -> Result<()> {
        state.save(&self.state_path)
    }

    /// Sorted context names; hidden files such as the state file are skipped.
    pub(crate) fn list_contexts(&self) -> Result<Vec<String>> {
        let mut contexts: Vec<String> = fs::read_dir(&self.contexts_dir)?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "json")
            })
            .filter_map(|path| path.file_stem()?.to_str().map(str::to_owned))
            .filter(|name| !name.starts_with('.'))
            .collect();
        contexts.sort();
        Ok(contexts)
    }

    pub(crate) fn current_context(&self) -> Result<Option<String>> {
        Ok(self.load_state()?.current)
    }

    pub(crate) fn switch_context(&self, name: &str) -> Result<()> {
        // Validate before touching the active settings so a broken context never replaces them.
        let content = read_settings(&self.existing_context(name)?)?;
        let mut state = self.load_state()?;
        state.set_current(name.to_owned());
        atomic_write(&self.settings_path, content)?;
        self.save_state(&state)?;
        println!("Switched to context \"{}\"", name.green().bold());
        Ok(())
    }

    pub(crate) fn switch_to_previous(&self) -> Result<()> {
        match self.load_state()?.previous {
            Some(previous) => self.switch_context(&previous),
            None => bail!("no previous context"),
        }
    }

    pub(crate) fn create_context(&self, name: &str) -> Result<()> {
        let path = self.new_context(name)?;
        if self.settings_path.is_file() {
            atomic_write(&path, read_settings(&self.settings_path)?)?;
            println!(
                "Context \"{}\" created from current settings",
                name.green().bold()
            );
        } else {
            atomic_write(&path, "{}")?;
            println!("Context \"{}\" created (empty)", name.green().bold());
        }
        Ok(())
    }

    pub(crate) fn delete_context(&self, name: &str) -> Result<()> {
        let mut state = self.load_state()?;
        if state.current.as_deref() == Some(name) {
            bail!("cannot delete the active context {name:?}");
        }
        fs::remove_file(self.existing_context(name)?)?;
        if state.previous.as_deref() == Some(name) {
            state.previous = None;
            self.save_state(&state)?;
        }
        println!("Context \"{}\" deleted", name.red());
        Ok(())
    }

    pub(crate) fn rename_context(&self, old_name: &str, new_name: &str) -> Result<()> {
        let old_path = self.existing_context(old_name)?;
        let new_path = self.new_context(new_name)?;
        let mut state = self.load_state()?;
        fs::rename(old_path, new_path)?;
        if state.rename(old_name, new_name) {
            self.save_state(&state)?;
        }
        println!(
            "Context \"{}\" renamed to \"{}\"",
            old_name,
            new_name.green().bold()
        );
        Ok(())
    }

    pub(crate) fn show_context(&self, name: &str) -> Result<()> {
        let content = fs::read_to_string(self.existing_context(name)?)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        println!("{}", serde_json::to_string_pretty(&json)?);
        Ok(())
    }

    pub(crate) fn edit_context(&self, name: &str) -> Result<()> {
        let path = self.existing_context(name)?;
        let editor = std::env::var_os("EDITOR")
            .or_else(|| std::env::var_os("VISUAL"))
            .unwrap_or_else(|| "vi".into());
        let status = Command::new(editor).arg(path).status()?;
        if !status.success() {
            bail!("editor exited with non-zero status");
        }
        Ok(())
    }

    pub(crate) fn export_context(&self, name: &str) -> Result<()> {
        print!("{}", fs::read_to_string(self.existing_context(name)?)?);
        Ok(())
    }

    pub(crate) fn import_context(&self, name: &str) -> Result<()> {
        let path = self.new_context(name)?;
        let mut content = String::new();
        std::io::stdin().read_to_string(&mut content)?;
        validate_settings(&content)?;
        atomic_write(&path, content)?;
        println!("Context \"{}\" imported", name.green().bold());
        Ok(())
    }

    pub(crate) fn unset_context(&self) -> Result<()> {
        let mut state = self.load_state()?;
        if self.settings_path.exists() {
            fs::remove_file(&self.settings_path)?;
        }
        if state.unset_current().is_some() {
            self.save_state(&state)?;
        }
        println!("Unset current context");
        Ok(())
    }

    pub(crate) fn list_contexts_with_current(&self, quiet: bool) -> Result<()> {
        let current = self.current_context()?;
        if quiet {
            if let Some(current) = current {
                println!("{current}");
            }
            return Ok(());
        }
        let contexts = self.list_contexts()?;
        if contexts.is_empty() {
            println!("No contexts found. Create one with: cctx -n <name>");
            return Ok(());
        }
        for context in contexts {
            if current.as_deref() == Some(context.as_str()) {
                println!("{} {}", context.green().bold(), "(current)".dimmed());
            } else {
                println!("{context}");
            }
        }
        Ok(())
    }
}
