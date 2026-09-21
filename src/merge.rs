use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::storage::{atomic_write, validate_name};

/// Retain compatibility with existing history files and permission prefixes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeHistory {
    pub source: String,
    pub timestamp: String,
    pub merged_items: Vec<String>,
    pub full_merge: bool,
}

pub struct MergeManager {
    settings_dir: PathBuf,
}

impl MergeManager {
    pub fn new(settings_dir: PathBuf) -> Self {
        Self { settings_dir }
    }

    pub fn history_path(&self, name: &str) -> Result<PathBuf> {
        validate_name(name)?;
        Ok(self
            .settings_dir
            .join(format!(".{name}-merge-history.json")))
    }

    pub fn load_history(&self, name: &str) -> Result<Vec<MergeHistory>> {
        let path = self.history_path(name)?;
        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content)
                .with_context(|| format!("Invalid merge history: {}", path.display())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(error) => Err(error).with_context(|| format!("Cannot read {}", path.display())),
        }
    }

    pub fn save_history(&self, name: &str, history: &[MergeHistory]) -> Result<()> {
        atomic_write(
            &self.history_path(name)?,
            serde_json::to_vec_pretty(history)?,
        )
    }

    pub fn merge_permissions(
        &self,
        target: &mut Value,
        source: &Value,
        name: &str,
    ) -> Result<MergeHistory> {
        self.merge(target, source, name, false)
    }

    pub fn merge_full(
        &self,
        target: &mut Value,
        source: &Value,
        name: &str,
    ) -> Result<MergeHistory> {
        self.merge(target, source, name, true)
    }

    fn merge(
        &self,
        target: &mut Value,
        source: &Value,
        name: &str,
        full: bool,
    ) -> Result<MergeHistory> {
        let source = source
            .as_object()
            .context("Source settings must be an object")?;
        let target = target
            .as_object_mut()
            .context("Target settings must be an object")?;
        let mut items = Vec::new();
        if let Some(permissions) = source.get("permissions") {
            let permissions = permissions
                .as_object()
                .context("Source permissions must be an object")?;
            let target_permissions = target
                .entry("permissions")
                .or_insert_with(|| json!({}))
                .as_object_mut()
                .context("Target permissions must be an object")?;
            for key in ["allow", "deny", "ask"] {
                if let Some(rules) = permissions.get(key) {
                    let rules = rules
                        .as_array()
                        .context("Source permission rules must be arrays")?;
                    let target_rules = target_permissions
                        .entry(key)
                        .or_insert_with(|| json!([]))
                        .as_array_mut()
                        .context("Target permission rules must be arrays")?;
                    if target_rules.iter().any(|rule| !rule.is_string()) {
                        bail!("Target permission rules must be strings");
                    }
                    for rule in rules {
                        let text = rule
                            .as_str()
                            .context("Source permission rules must be strings")?;
                        if !target_rules.contains(rule) {
                            target_rules.push(rule.clone());
                            let prefix = if full { "permissions." } else { "" };
                            items.push(format!("{prefix}{key}:{text}"));
                        }
                    }
                }
            }
        }
        if full {
            for (key, value) in source {
                match key.as_str() {
                    "permissions" => {}
                    "env" => {
                        let source_env =
                            value.as_object().context("Source env must be an object")?;
                        let target_env = target
                            .entry("env")
                            .or_insert_with(|| json!({}))
                            .as_object_mut()
                            .context("Target env must be an object")?;
                        for (name, value) in source_env {
                            if !target_env.contains_key(name) {
                                target_env.insert(name.clone(), value.clone());
                                items.push(format!("env:{name}"));
                            }
                        }
                    }
                    _ if !target.contains_key(key) => {
                        target.insert(key.clone(), value.clone());
                        items.push(key.clone());
                    }
                    _ => {}
                }
            }
        }
        Ok(MergeHistory {
            source: name.to_string(),
            timestamp: chrono::Local::now().to_rfc3339(),
            merged_items: items,
            full_merge: full,
        })
    }

    pub fn unmerge_permissions(
        &self,
        target: &mut Value,
        context: &str,
        source: &str,
    ) -> Result<()> {
        self.unmerge(target, context, source, false)
    }

    pub fn unmerge_full(&self, target: &mut Value, context: &str, source: &str) -> Result<()> {
        self.unmerge(target, context, source, true)
    }

    fn unmerge(&self, target: &mut Value, context: &str, source: &str, full: bool) -> Result<()> {
        let history = self.load_history(context)?;
        let items: HashSet<_> = history
            .iter()
            .filter(|entry| entry.source == source && (full || !entry.full_merge))
            .flat_map(|entry| entry.merged_items.iter().cloned())
            .collect();
        let target = target
            .as_object_mut()
            .context("Target settings must be an object")?;
        if let Some(permissions) = target.get_mut("permissions").and_then(Value::as_object_mut) {
            for key in ["allow", "deny", "ask"] {
                if let Some(rules) = permissions.get_mut(key).and_then(Value::as_array_mut) {
                    rules.retain(|rule| {
                        rule.as_str().map_or(true, |text| {
                            !items.contains(&format!("{key}:{text}"))
                                && !items.contains(&format!("permissions.{key}:{text}"))
                        })
                    });
                }
            }
        }
        if full {
            for item in &items {
                if let Some(name) = item.strip_prefix("env:") {
                    if let Some(env) = target.get_mut("env").and_then(Value::as_object_mut) {
                        env.remove(name);
                    }
                } else if !item.starts_with("permissions.")
                    && !item.starts_with("allow:")
                    && !item.starts_with("deny:")
                    && !item.starts_with("ask:")
                {
                    target.remove(item);
                }
            }
        }
        let remaining: Vec<_> = history
            .into_iter()
            .filter(|entry| !(entry.source == source && (full || !entry.full_merge)))
            .collect();
        self.save_history(context, &remaining)
    }

    pub fn display_history(&self, name: &str) -> Result<()> {
        let history = self.load_history(name)?;
        if history.is_empty() {
            println!("No merge history for context '{name}'");
        } else {
            println!("Merge history for context '{name}':");
            for entry in history {
                println!(
                    "  {}  {}  {} items{}",
                    entry.timestamp,
                    entry.source,
                    entry.merged_items.len(),
                    if entry.full_merge {
                        " (full merge)"
                    } else {
                        ""
                    }
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_permissions_merge_without_losing_order_or_existing_rules() {
        let dir = tempfile::tempdir().unwrap();
        let manager = MergeManager::new(dir.path().into());
        let mut target = json!({"permissions": {"allow": ["Read", "Edit"]}});
        let source = json!({"permissions": {"allow": ["Read", "Bash"], "deny": ["WebFetch"], "ask": ["Write"]}});
        let entry = manager
            .merge_permissions(&mut target, &source, "work")
            .unwrap();
        assert_eq!(
            target,
            json!({"permissions": {"allow": ["Read", "Edit", "Bash"], "deny": ["WebFetch"], "ask": ["Write"]}})
        );
        manager.save_history("target", &[entry]).unwrap();
        manager
            .unmerge_permissions(&mut target, "target", "work")
            .unwrap();
        assert_eq!(target["permissions"]["allow"], json!(["Read", "Edit"]));
        assert_eq!(target["permissions"]["deny"], json!([]));
        assert_eq!(target["permissions"]["ask"], json!([]));
    }

    #[test]
    fn permission_unmerge_retains_full_merge_history() {
        let dir = tempfile::tempdir().unwrap();
        let manager = MergeManager::new(dir.path().into());
        let mut target = json!({});
        let entry = manager
            .merge_full(&mut target, &json!({"model": "sonnet"}), "work")
            .unwrap();
        manager.save_history("target", &[entry]).unwrap();
        manager
            .unmerge_permissions(&mut target, "target", "work")
            .unwrap();
        manager.unmerge_full(&mut target, "target", "work").unwrap();
        assert!(target.get("model").is_none());
    }

    #[test]
    fn legacy_full_history_removes_only_added_values() {
        let dir = tempfile::tempdir().unwrap();
        let manager = MergeManager::new(dir.path().into());
        fs::write(manager.history_path("target").unwrap(), r#"[{"source":"work","timestamp":"2025-01-01","merged_items":["permissions.allow:Edit","env:EXAMPLE","model"],"full_merge":true}]"#).unwrap();
        let mut target = json!({"permissions":{"allow":["Read","Edit"]},"env":{"EXAMPLE":"1","KEEP":"2"},"model":"sonnet"});
        manager.unmerge_full(&mut target, "target", "work").unwrap();
        assert_eq!(
            target,
            json!({"permissions":{"allow":["Read"]},"env":{"KEEP":"2"}})
        );
    }
}
