use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::storage::atomic_write;

/// Current and previous context names, persisted next to the contexts.
#[derive(Serialize, Deserialize, Default)]
pub(crate) struct State {
    pub(crate) current: Option<String>,
    pub(crate) previous: Option<String>,
}

impl State {
    pub(crate) fn load(path: &Path) -> Result<Self> {
        match fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content)
                .with_context(|| format!("Invalid state file {}", path.display())),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error).with_context(|| format!("Cannot read {}", path.display())),
        }
    }

    pub(crate) fn save(&self, path: &Path) -> Result<()> {
        atomic_write(path, serde_json::to_string_pretty(self)?)
    }

    pub(crate) fn set_current(&mut self, context: String) {
        if self
            .current
            .as_ref()
            .is_some_and(|current| *current != context)
        {
            self.previous = self.current.take();
        }
        self.current = Some(context);
    }

    pub(crate) fn unset_current(&mut self) -> Option<String> {
        let current = self.current.take();
        if current.is_some() {
            self.previous = current.clone();
        }
        current
    }

    /// Follow a renamed context; returns whether anything changed.
    pub(crate) fn rename(&mut self, old_name: &str, new_name: &str) -> bool {
        let mut changed = false;
        for slot in [&mut self.current, &mut self.previous] {
            if slot.as_deref() == Some(old_name) {
                *slot = Some(new_name.to_owned());
                changed = true;
            }
        }
        changed
    }
}
