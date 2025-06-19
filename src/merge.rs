use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Represents the merge history for tracking what was merged from where
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeHistory {
    pub source: String,
    pub timestamp: String,
    pub merged_items: Vec<String>,
    pub full_merge: bool,
}

/// Manages merge operations for Claude Code settings
pub struct MergeManager {
    settings_dir: PathBuf,
}

impl MergeManager {
    /// Create a new MergeManager
    pub fn new(settings_dir: PathBuf) -> Self {
        Self { settings_dir }
    }

    /// Get the path to the merge history file for a specific context
    fn get_history_path(&self, context_name: &str) -> PathBuf {
        self.settings_dir
            .join(format!(".{}-merge-history.json", context_name))
    }

    /// Load merge history for a context
    pub fn load_history(&self, context_name: &str) -> Result<Vec<MergeHistory>> {
        let history_path = self.get_history_path(context_name);
        if !history_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&history_path)
            .with_context(|| format!("Failed to read merge history from {:?}", history_path))?;

        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse merge history from {:?}", history_path))
    }

    /// Save merge history for a context
    pub fn save_history(&self, context_name: &str, history: &[MergeHistory]) -> Result<()> {
        let history_path = self.get_history_path(context_name);
        let content =
            serde_json::to_string_pretty(history).context("Failed to serialize merge history")?;

        fs::write(&history_path, content)
            .with_context(|| format!("Failed to write merge history to {:?}", history_path))
    }

    /// Merge permissions from source into target
    pub fn merge_permissions(
        &self,
        target: &mut Value,
        source: &Value,
        source_name: &str,
    ) -> Result<MergeHistory> {
        let mut merged_items = Vec::new();

        // Ensure target has permissions object
        if target.get("permissions").is_none() {
            target["permissions"] = serde_json::json!({
                "allow": [],
                "deny": []
            });
        }

        // Merge allow permissions
        if let Some(source_allow) = source
            .get("permissions")
            .and_then(|p| p.get("allow"))
            .and_then(|a| a.as_array())
        {
            let target_allow = target["permissions"]["allow"]
                .as_array_mut()
                .ok_or_else(|| anyhow::anyhow!("Target permissions.allow is not an array"))?;

            // Convert to HashSet for deduplication
            let mut allow_set: HashSet<String> = target_allow
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();

            for item in source_allow {
                if let Some(s) = item.as_str() {
                    if allow_set.insert(s.to_string()) {
                        merged_items.push(format!("allow:{}", s));
                    }
                }
            }

            // Convert back to array
            *target_allow = allow_set
                .into_iter()
                .map(serde_json::Value::String)
                .collect();
        }

        // Merge deny permissions
        if let Some(source_deny) = source
            .get("permissions")
            .and_then(|p| p.get("deny"))
            .and_then(|a| a.as_array())
        {
            let target_deny = target["permissions"]["deny"]
                .as_array_mut()
                .ok_or_else(|| anyhow::anyhow!("Target permissions.deny is not an array"))?;

            // Convert to HashSet for deduplication
            let mut deny_set: HashSet<String> = target_deny
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();

            for item in source_deny {
                if let Some(s) = item.as_str() {
                    if deny_set.insert(s.to_string()) {
                        merged_items.push(format!("deny:{}", s));
                    }
                }
            }

            // Convert back to array
            *target_deny = deny_set
                .into_iter()
                .map(serde_json::Value::String)
                .collect();
        }

        // Create history entry
        let history = MergeHistory {
            source: source_name.to_string(),
            timestamp: chrono::Local::now().to_rfc3339(),
            merged_items,
            full_merge: false,
        };

        Ok(history)
    }

    /// Remove permissions that were previously merged from a specific source
    pub fn unmerge_permissions(
        &self,
        target: &mut Value,
        context_name: &str,
        source_name: &str,
    ) -> Result<()> {
        let history = self.load_history(context_name)?;

        // Find all items merged from this source
        let items_to_remove: HashSet<String> = history
            .iter()
            .filter(|h| h.source == source_name)
            .flat_map(|h| h.merged_items.iter().cloned())
            .collect();

        // Remove from allow list
        if let Some(allow) = target
            .get_mut("permissions")
            .and_then(|p| p.get_mut("allow"))
            .and_then(|a| a.as_array_mut())
        {
            allow.retain(|v| {
                if let Some(s) = v.as_str() {
                    !items_to_remove.contains(&format!("allow:{}", s))
                } else {
                    true
                }
            });
        }

        // Remove from deny list
        if let Some(deny) = target
            .get_mut("permissions")
            .and_then(|p| p.get_mut("deny"))
            .and_then(|a| a.as_array_mut())
        {
            deny.retain(|v| {
                if let Some(s) = v.as_str() {
                    !items_to_remove.contains(&format!("deny:{}", s))
                } else {
                    true
                }
            });
        }

        // Update history to remove entries from this source
        let updated_history: Vec<MergeHistory> = history
            .into_iter()
            .filter(|h| h.source != source_name)
            .collect();

        self.save_history(context_name, &updated_history)?;

        Ok(())
    }

    /// Merge all settings from source into target (full merge)
    pub fn merge_full(
        &self,
        target: &mut Value,
        source: &Value,
        source_name: &str,
    ) -> Result<MergeHistory> {
        let mut merged_items = Vec::new();

        // Deep merge all fields from source to target
        if let Some(source_obj) = source.as_object() {
            if let Some(target_obj) = target.as_object_mut() {
                for (key, value) in source_obj {
                    match key.as_str() {
                        "permissions" => {
                            // Handle permissions specially to merge arrays
                            if !target_obj.contains_key("permissions") {
                                target_obj.insert(
                                    "permissions".to_string(),
                                    serde_json::json!({
                                        "allow": [],
                                        "deny": []
                                    }),
                                );
                            }

                            if let Some(source_perms) = value.as_object() {
                                // Merge allow permissions
                                if let Some(source_allow) =
                                    source_perms.get("allow").and_then(|a| a.as_array())
                                {
                                    let target_allow = target_obj["permissions"]["allow"]
                                        .as_array_mut()
                                        .ok_or_else(|| {
                                            anyhow::anyhow!(
                                                "Target permissions.allow is not an array"
                                            )
                                        })?;

                                    let mut allow_set: HashSet<String> = target_allow
                                        .iter()
                                        .filter_map(|v| v.as_str().map(String::from))
                                        .collect();

                                    for item in source_allow {
                                        if let Some(s) = item.as_str() {
                                            if allow_set.insert(s.to_string()) {
                                                merged_items
                                                    .push(format!("permissions.allow:{}", s));
                                            }
                                        }
                                    }

                                    *target_allow = allow_set
                                        .into_iter()
                                        .map(serde_json::Value::String)
                                        .collect();
                                }

                                // Merge deny permissions
                                if let Some(source_deny) =
                                    source_perms.get("deny").and_then(|a| a.as_array())
                                {
                                    let target_deny = target_obj["permissions"]["deny"]
                                        .as_array_mut()
                                        .ok_or_else(|| {
                                            anyhow::anyhow!(
                                                "Target permissions.deny is not an array"
                                            )
                                        })?;

                                    let mut deny_set: HashSet<String> = target_deny
                                        .iter()
                                        .filter_map(|v| v.as_str().map(String::from))
                                        .collect();

                                    for item in source_deny {
                                        if let Some(s) = item.as_str() {
                                            if deny_set.insert(s.to_string()) {
                                                merged_items
                                                    .push(format!("permissions.deny:{}", s));
                                            }
                                        }
                                    }

                                    *target_deny = deny_set
                                        .into_iter()
                                        .map(serde_json::Value::String)
                                        .collect();
                                }
                            }
                        }
                        "env" => {
                            // Merge environment variables
                            if let Some(source_env) = value.as_object() {
                                if !target_obj.contains_key("env") {
                                    target_obj.insert("env".to_string(), serde_json::json!({}));
                                }

                                if let Some(target_env) =
                                    target_obj.get_mut("env").and_then(|e| e.as_object_mut())
                                {
                                    for (env_key, env_value) in source_env {
                                        if !target_env.contains_key(env_key) {
                                            target_env.insert(env_key.clone(), env_value.clone());
                                            merged_items.push(format!("env:{}", env_key));
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            // For other fields, overwrite if not present
                            if !target_obj.contains_key(key) {
                                target_obj.insert(key.clone(), value.clone());
                                merged_items.push(key.clone());
                            }
                        }
                    }
                }
            }
        }

        // Create history entry
        let history = MergeHistory {
            source: source_name.to_string(),
            timestamp: chrono::Local::now().to_rfc3339(),
            merged_items,
            full_merge: true,
        };

        Ok(history)
    }

    /// Remove all settings that were previously merged from a specific source (full unmerge)
    pub fn unmerge_full(
        &self,
        target: &mut Value,
        context_name: &str,
        source_name: &str,
    ) -> Result<()> {
        let history = self.load_history(context_name)?;

        // Find all full merge entries from this source
        let full_merge_items: HashSet<String> = history
            .iter()
            .filter(|h| h.source == source_name && h.full_merge)
            .flat_map(|h| h.merged_items.iter().cloned())
            .collect();

        if let Some(target_obj) = target.as_object_mut() {
            // Remove top-level keys that were merged
            target_obj.retain(|key, _| !full_merge_items.contains(key));

            // Handle special cases for nested structures
            for item in &full_merge_items {
                if item.starts_with("env:") {
                    if let Some(env_key) = item.strip_prefix("env:") {
                        if let Some(env_obj) =
                            target_obj.get_mut("env").and_then(|e| e.as_object_mut())
                        {
                            env_obj.remove(env_key);
                        }
                    }
                } else if item.starts_with("permissions.allow:")
                    || item.starts_with("permissions.deny:")
                {
                    // Handle permissions removal
                    let (perm_type, perm_value) = if item.starts_with("permissions.allow:") {
                        ("allow", item.strip_prefix("permissions.allow:").unwrap())
                    } else {
                        ("deny", item.strip_prefix("permissions.deny:").unwrap())
                    };

                    if let Some(perms) = target_obj
                        .get_mut("permissions")
                        .and_then(|p| p.get_mut(perm_type))
                        .and_then(|a| a.as_array_mut())
                    {
                        perms.retain(|v| v.as_str() != Some(perm_value));
                    }
                }
            }
        }

        // Also handle regular permission unmerge
        self.unmerge_permissions(target, context_name, source_name)?;

        Ok(())
    }

    /// Display merge history for a context
    pub fn display_history(&self, context_name: &str) -> Result<()> {
        let history = self.load_history(context_name)?;

        if history.is_empty() {
            println!("No merge history for context '{}'", context_name);
            return Ok(());
        }

        println!("📋 Merge history for context '{}':", context_name);
        println!();

        for entry in &history {
            println!("  📅 {}", entry.timestamp);
            println!("  📁 Source: {}", entry.source);
            println!(
                "  📝 Merged {} items{}",
                entry.merged_items.len(),
                if entry.full_merge {
                    " (full merge)"
                } else {
                    ""
                }
            );
            println!();
        }

        Ok(())
    }
}

/// Add chrono dependency for timestamps
pub use chrono;
