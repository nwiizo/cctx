use anyhow::{bail, Context, Result};
use colored::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::merge::MergeManager;
use crate::state::State;

#[derive(Debug, Clone)]
pub enum SettingsLevel {
    User,    // ~/.claude/settings.json (default)
    Project, // ./.claude/settings.json (explicit)
    Local,   // ./.claude/settings.local.json (explicit)
}

pub struct ContextManager {
    pub contexts_dir: PathBuf,
    pub claude_settings_path: PathBuf,
    pub state_path: PathBuf,
    pub settings_level: SettingsLevel,
}

impl ContextManager {
    pub fn new() -> Result<Self> {
        Self::new_with_level(SettingsLevel::User)
    }

    pub fn new_with_level(level: SettingsLevel) -> Result<Self> {
        let home_dir = dirs::home_dir().context("Failed to get home directory")?;
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let (claude_settings_path, contexts_dir, state_path) = match level {
            SettingsLevel::User => {
                let claude_dir = home_dir.join(".claude");
                let contexts_dir = claude_dir.join("settings");
                let claude_settings_path = claude_dir.join("settings.json");
                let state_path = contexts_dir.join(".cctx-state.json");
                (claude_settings_path, contexts_dir, state_path)
            }
            SettingsLevel::Project => {
                let claude_dir = current_dir.join(".claude");
                let contexts_dir = claude_dir.join("settings");
                let claude_settings_path = claude_dir.join("settings.json");
                let state_path = contexts_dir.join(".cctx-state.json");
                (claude_settings_path, contexts_dir, state_path)
            }
            SettingsLevel::Local => {
                let claude_dir = current_dir.join(".claude");
                let contexts_dir = claude_dir.join("settings");
                let claude_settings_path = claude_dir.join("settings.local.json");
                let state_path = contexts_dir.join(".cctx-state.local.json");
                (claude_settings_path, contexts_dir, state_path)
            }
        };

        // Create directories if they don't exist
        fs::create_dir_all(&contexts_dir)?;

        Ok(Self {
            contexts_dir,
            claude_settings_path,
            state_path,
            settings_level: level,
        })
    }

    /// Check if project-level contexts are available in current directory
    pub fn has_project_contexts() -> bool {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let project_contexts_dir = current_dir.join(".claude").join("settings");

        if let Ok(entries) = fs::read_dir(&project_contexts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                        if !filename.starts_with('.') {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Check if local contexts are available in current directory  
    pub fn has_local_contexts() -> bool {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        current_dir
            .join(".claude")
            .join("settings.local.json")
            .exists()
    }

    pub fn context_path(&self, name: &str) -> PathBuf {
        self.contexts_dir.join(format!("{name}.json"))
    }

    fn load_state(&self) -> Result<State> {
        State::load(&self.state_path)
    }

    fn save_state(&self, state: &State) -> Result<()> {
        state.save(&self.state_path)
    }

    pub fn list_contexts(&self) -> Result<Vec<String>> {
        let mut contexts = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.contexts_dir) {
            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                // Skip hidden files and non-JSON files
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if filename.starts_with('.') {
                        continue;
                    }
                }

                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        contexts.push(name.to_string());
                    }
                }
            }
        }

        contexts.sort();
        Ok(contexts)
    }

    pub fn get_current_context(&self) -> Result<Option<String>> {
        let state = self.load_state()?;
        Ok(state.current)
    }

    pub fn switch_context(&self, name: &str) -> Result<()> {
        let contexts = self.list_contexts()?;
        if !contexts.contains(&name.to_string()) {
            bail!("error: no context exists with the name \"{}\"", name);
        }

        let mut state = self.load_state()?;
        state.set_current(name.to_string());

        // Copy context settings to Claude settings
        let context_path = self.context_path(name);
        let content = fs::read_to_string(&context_path)?;

        // Create .claude directory if it doesn't exist
        if let Some(parent) = self.claude_settings_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.claude_settings_path, content)?;
        self.save_state(&state)?;

        println!("Switched to context \"{}\"", name.green().bold());
        Ok(())
    }

    pub fn switch_to_previous(&self) -> Result<()> {
        let state = self.load_state()?;

        if let Some(previous) = state.previous {
            self.switch_context(&previous)?;
        } else {
            bail!("error: no previous context");
        }

        Ok(())
    }

    pub fn create_context(&self, name: &str) -> Result<()> {
        if name.is_empty() || name == "-" || name == "." || name == ".." || name.contains('/') {
            bail!("error: invalid context name \"{}\"", name);
        }

        let contexts = self.list_contexts()?;
        if contexts.contains(&name.to_string()) {
            bail!("error: context \"{}\" already exists", name);
        }

        let context_path = self.context_path(name);

        if self.claude_settings_path.exists() {
            // Copy current Claude settings
            fs::copy(&self.claude_settings_path, &context_path)?;
            println!(
                "Context \"{}\" created from current settings",
                name.green().bold()
            );
        } else {
            // Create empty settings
            let empty_settings = serde_json::json!({});
            fs::write(
                &context_path,
                serde_json::to_string_pretty(&empty_settings)?,
            )?;
            println!("Context \"{}\" created (empty)", name.green().bold());
        }

        Ok(())
    }

    pub fn delete_context(&self, name: &str) -> Result<()> {
        let state = self.load_state()?;

        if state.current.as_ref() == Some(&name.to_string()) {
            bail!("error: cannot delete the active context \"{}\"", name);
        }

        let context_path = self.context_path(name);
        if !context_path.exists() {
            bail!("error: no context exists with the name \"{}\"", name);
        }

        fs::remove_file(context_path)?;

        // Update state if this was the previous context
        if state.previous.as_ref() == Some(&name.to_string()) {
            let mut new_state = state;
            new_state.previous = None;
            self.save_state(&new_state)?;
        }

        println!("Context \"{}\" deleted", name.red());
        Ok(())
    }

    pub fn rename_context(&self, old_name: &str, new_name: &str) -> Result<()> {
        if new_name.is_empty()
            || new_name == "-"
            || new_name == "."
            || new_name == ".."
            || new_name.contains('/')
        {
            bail!("error: invalid context name \"{}\"", new_name);
        }

        let contexts = self.list_contexts()?;
        if !contexts.contains(&old_name.to_string()) {
            bail!("error: no context exists with the name \"{}\"", old_name);
        }

        if contexts.contains(&new_name.to_string()) {
            bail!("error: context \"{}\" already exists", new_name);
        }

        let old_path = self.context_path(old_name);
        let new_path = self.context_path(new_name);
        fs::rename(old_path, new_path)?;

        // Update state if needed
        let mut state = self.load_state()?;
        let mut updated = false;

        if state.current.as_ref() == Some(&old_name.to_string()) {
            state.current = Some(new_name.to_string());
            updated = true;
        }

        if state.previous.as_ref() == Some(&old_name.to_string()) {
            state.previous = Some(new_name.to_string());
            updated = true;
        }

        if updated {
            self.save_state(&state)?;
        }

        println!(
            "Context \"{}\" renamed to \"{}\"",
            old_name,
            new_name.green().bold()
        );
        Ok(())
    }

    pub fn show_context(&self, name: &str) -> Result<()> {
        let context_path = self.context_path(name);
        if !context_path.exists() {
            bail!("error: no context exists with the name \"{}\"", name);
        }

        let content = fs::read_to_string(context_path)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        let pretty = serde_json::to_string_pretty(&json)?;

        println!("{pretty}");
        Ok(())
    }

    pub fn edit_context(&self, name: &str) -> Result<()> {
        let context_path = self.context_path(name);
        if !context_path.exists() {
            bail!("error: no context exists with the name \"{}\"", name);
        }

        let editor = std::env::var("EDITOR")
            .or_else(|_| std::env::var("VISUAL"))
            .unwrap_or_else(|_| "vi".to_string());

        let status = Command::new(&editor).arg(&context_path).status()?;

        if !status.success() {
            bail!("error: editor exited with non-zero status");
        }

        Ok(())
    }

    pub fn export_context(&self, name: &str) -> Result<()> {
        let context_path = self.context_path(name);
        if !context_path.exists() {
            bail!("error: no context exists with the name \"{}\"", name);
        }

        let content = fs::read_to_string(context_path)?;
        print!("{content}");
        Ok(())
    }

    pub fn import_context(&self, name: &str) -> Result<()> {
        if name.is_empty() || name == "-" || name == "." || name == ".." || name.contains('/') {
            bail!("error: invalid context name \"{}\"", name);
        }

        let contexts = self.list_contexts()?;
        if contexts.contains(&name.to_string()) {
            bail!("error: context \"{}\" already exists", name);
        }

        use std::io::Read;
        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer)?;

        // Validate JSON
        let _: serde_json::Value =
            serde_json::from_str(&buffer).context("error: invalid JSON input")?;

        let context_path = self.context_path(name);
        fs::write(&context_path, buffer)?;

        println!("Context \"{}\" imported", name.green().bold());
        Ok(())
    }

    pub fn unset_context(&self) -> Result<()> {
        if self.claude_settings_path.exists() {
            fs::remove_file(&self.claude_settings_path)?;
        }

        let mut state = self.load_state()?;
        if let Some(_current) = state.unset_current() {
            self.save_state(&state)?;
        }

        println!("Unset current context");
        Ok(())
    }

    pub fn list_contexts_with_current(&self, quiet: bool) -> Result<()> {
        let contexts = self.list_contexts()?;
        let current = self.get_current_context()?;

        if quiet {
            // Quiet mode - only show current context
            if let Some(current_ctx) = current {
                println!("{current_ctx}");
            }
            return Ok(());
        }

        // Show helpful information for user-level contexts
        if matches!(self.settings_level, SettingsLevel::User) {
            // Show available project contexts as suggestion
            if Self::has_project_contexts() {
                println!(
                    "{} Project contexts available: run 'cctx --in-project' to manage",
                    "💡".yellow()
                );
            }
            if Self::has_local_contexts() {
                println!(
                    "{} Local contexts available: run 'cctx --local' to manage",
                    "💡".yellow()
                );
            }
        }

        // Show current settings level (condensed)
        let level_emoji = match self.settings_level {
            SettingsLevel::User => "👤",
            SettingsLevel::Project => "📁",
            SettingsLevel::Local => "💻",
        };

        if contexts.is_empty() {
            println!(
                "{} {} contexts: No contexts found. Create one with: cctx -n <name>",
                level_emoji,
                format!("{:?}", self.settings_level).cyan()
            );
            return Ok(());
        }

        println!(
            "{} {} contexts:",
            level_emoji,
            format!("{:?}", self.settings_level).cyan().bold()
        );

        // List contexts with current highlighted
        for ctx in contexts {
            if Some(&ctx) == current.as_ref() {
                println!("  {} {}", ctx.green().bold(), "(current)".dimmed());
            } else {
                println!("  {ctx}");
            }
        }

        Ok(())
    }

    /// Merge permissions from another context or settings file
    pub fn merge_from(&self, target_context: &str, source: &str) -> Result<()> {
        // Load target context
        let target_path = if target_context == "current" {
            if !self.claude_settings_path.exists() {
                bail!("error: no current context is set");
            }
            self.claude_settings_path.clone()
        } else {
            let path = self.context_path(target_context);
            if !path.exists() {
                bail!(
                    "error: no context exists with the name \"{}\"",
                    target_context
                );
            }
            path
        };

        // Load source settings
        let source_content = if source == "user" {
            // Merge from user-level settings.json
            let home_dir = dirs::home_dir().context("Failed to get home directory")?;
            let user_settings = home_dir.join(".claude").join("settings.json");
            if !user_settings.exists() {
                bail!("error: user settings file not found at {:?}", user_settings);
            }
            fs::read_to_string(&user_settings)?
        } else if source.ends_with(".json") {
            // Merge from a file path
            let source_path = PathBuf::from(source);
            if !source_path.exists() {
                bail!("error: source file not found at {:?}", source_path);
            }
            fs::read_to_string(&source_path)?
        } else {
            // Merge from another context
            let source_path = self.context_path(source);
            if !source_path.exists() {
                bail!("error: no context exists with the name \"{}\"", source);
            }
            fs::read_to_string(&source_path)?
        };

        // Parse JSON
        let mut target_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&target_path)?)?;
        let source_json: serde_json::Value = serde_json::from_str(&source_content)?;

        // Perform merge
        let merge_manager = MergeManager::new(self.contexts_dir.clone());
        let history_entry =
            merge_manager.merge_permissions(&mut target_json, &source_json, source)?;

        // Save updated target
        fs::write(&target_path, serde_json::to_string_pretty(&target_json)?)?;

        // Update history
        let context_name = if target_context == "current" {
            self.get_current_context()?
                .unwrap_or_else(|| "current".to_string())
        } else {
            target_context.to_string()
        };

        let mut history = merge_manager.load_history(&context_name)?;
        history.push(history_entry.clone());
        merge_manager.save_history(&context_name, &history)?;

        println!(
            "✅ Merged {} permissions from '{}' into '{}'",
            history_entry.merged_items.len(),
            source.green(),
            target_context.green().bold()
        );

        if !history_entry.merged_items.is_empty() {
            println!("\n📋 Merged items:");
            for (i, item) in history_entry.merged_items.iter().enumerate() {
                if i < 5 {
                    println!("  • {}", item);
                } else if i == 5 {
                    println!("  ... and {} more", history_entry.merged_items.len() - 5);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Remove previously merged permissions
    pub fn unmerge_from(&self, target_context: &str, source: &str) -> Result<()> {
        // Load target context
        let target_path = if target_context == "current" {
            if !self.claude_settings_path.exists() {
                bail!("error: no current context is set");
            }
            self.claude_settings_path.clone()
        } else {
            let path = self.context_path(target_context);
            if !path.exists() {
                bail!(
                    "error: no context exists with the name \"{}\"",
                    target_context
                );
            }
            path
        };

        // Load and parse target JSON
        let mut target_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&target_path)?)?;

        // Get context name for history
        let context_name = if target_context == "current" {
            self.get_current_context()?
                .unwrap_or_else(|| "current".to_string())
        } else {
            target_context.to_string()
        };

        // Perform unmerge
        let merge_manager = MergeManager::new(self.contexts_dir.clone());
        merge_manager.unmerge_permissions(&mut target_json, &context_name, source)?;

        // Save updated target
        fs::write(&target_path, serde_json::to_string_pretty(&target_json)?)?;

        println!(
            "✅ Removed all permissions previously merged from '{}' in '{}'",
            source.red(),
            target_context.green().bold()
        );

        Ok(())
    }

    /// Merge all settings from another context or settings file (full merge)
    pub fn merge_from_full(&self, target_context: &str, source: &str) -> Result<()> {
        // Load target context
        let target_path = if target_context == "current" {
            if !self.claude_settings_path.exists() {
                bail!("error: no current context is set");
            }
            self.claude_settings_path.clone()
        } else {
            let path = self.context_path(target_context);
            if !path.exists() {
                bail!(
                    "error: no context exists with the name \"{}\"",
                    target_context
                );
            }
            path
        };

        // Load source settings
        let source_content = if source == "user" {
            // Merge from user-level settings.json
            let home_dir = dirs::home_dir().context("Failed to get home directory")?;
            let user_settings = home_dir.join(".claude").join("settings.json");
            if !user_settings.exists() {
                bail!("error: user settings file not found at {:?}", user_settings);
            }
            fs::read_to_string(&user_settings)?
        } else if source.ends_with(".json") {
            // Merge from a file path
            let source_path = PathBuf::from(source);
            if !source_path.exists() {
                bail!("error: source file not found at {:?}", source_path);
            }
            fs::read_to_string(&source_path)?
        } else {
            // Merge from another context
            let source_path = self.context_path(source);
            if !source_path.exists() {
                bail!("error: no context exists with the name \"{}\"", source);
            }
            fs::read_to_string(&source_path)?
        };

        // Parse JSON
        let mut target_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&target_path)?)?;
        let source_json: serde_json::Value = serde_json::from_str(&source_content)?;

        // Perform full merge
        let merge_manager = MergeManager::new(self.contexts_dir.clone());
        let history_entry = merge_manager.merge_full(&mut target_json, &source_json, source)?;

        // Save updated target
        fs::write(&target_path, serde_json::to_string_pretty(&target_json)?)?;

        // Update history
        let context_name = if target_context == "current" {
            self.get_current_context()?
                .unwrap_or_else(|| "current".to_string())
        } else {
            target_context.to_string()
        };

        let mut history = merge_manager.load_history(&context_name)?;
        history.push(history_entry.clone());
        merge_manager.save_history(&context_name, &history)?;

        println!(
            "✅ Full merge completed: {} items from '{}' into '{}'",
            history_entry.merged_items.len(),
            source.green(),
            target_context.green().bold()
        );

        if !history_entry.merged_items.is_empty() {
            println!("\n📋 Merged items:");

            // Group items by type for better display
            let mut permissions_items = Vec::new();
            let mut env_items = Vec::new();
            let mut other_items = Vec::new();

            for item in &history_entry.merged_items {
                if item.starts_with("permissions.") {
                    permissions_items.push(item);
                } else if item.starts_with("env:") {
                    env_items.push(item);
                } else {
                    other_items.push(item);
                }
            }

            if !permissions_items.is_empty() {
                println!("  🔒 Permissions: {} items", permissions_items.len());
            }
            if !env_items.is_empty() {
                println!("  🌍 Environment: {} variables", env_items.len());
            }
            if !other_items.is_empty() {
                let items_str: Vec<String> = other_items.iter().map(|s| s.to_string()).collect();
                println!("  ⚙️  Settings: {}", items_str.join(", "));
            }
        }

        Ok(())
    }

    /// Remove all settings that were previously merged from a specific source (full unmerge)
    pub fn unmerge_from_full(&self, target_context: &str, source: &str) -> Result<()> {
        // Load target context
        let target_path = if target_context == "current" {
            if !self.claude_settings_path.exists() {
                bail!("error: no current context is set");
            }
            self.claude_settings_path.clone()
        } else {
            let path = self.context_path(target_context);
            if !path.exists() {
                bail!(
                    "error: no context exists with the name \"{}\"",
                    target_context
                );
            }
            path
        };

        // Load and parse target JSON
        let mut target_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&target_path)?)?;

        // Get context name for history
        let context_name = if target_context == "current" {
            self.get_current_context()?
                .unwrap_or_else(|| "current".to_string())
        } else {
            target_context.to_string()
        };

        // Perform full unmerge
        let merge_manager = MergeManager::new(self.contexts_dir.clone());
        merge_manager.unmerge_full(&mut target_json, &context_name, source)?;

        // Save updated target
        fs::write(&target_path, serde_json::to_string_pretty(&target_json)?)?;

        println!(
            "✅ Removed all settings previously merged from '{}' in '{}'",
            source.red(),
            target_context.green().bold()
        );

        Ok(())
    }

    /// Display merge history for a context
    pub fn show_merge_history(&self, context_name: Option<&str>) -> Result<()> {
        let name = if let Some(n) = context_name {
            n.to_string()
        } else {
            self.get_current_context()?
                .ok_or_else(|| anyhow::anyhow!("error: no current context set"))?
        };

        let merge_manager = MergeManager::new(self.contexts_dir.clone());
        merge_manager.display_history(&name)?;

        Ok(())
    }
}
