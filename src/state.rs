use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    pub current: Option<String>,
    pub previous: Option<String>,
}

impl State {
    pub fn load(state_path: &PathBuf) -> Result<Self> {
        if state_path.exists() {
            let content = fs::read_to_string(state_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(State::default())
        }
    }

    pub fn save(&self, state_path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(state_path, content)?;
        Ok(())
    }

    pub fn set_current(&mut self, context: String) {
        if let Some(current) = &self.current {
            if current != &context {
                self.previous = Some(current.clone());
            }
        }
        self.current = Some(context);
    }

    pub fn unset_current(&mut self) -> Option<String> {
        let current = self.current.take();
        if let Some(prev) = current.as_ref() {
            self.previous = Some(prev.clone());
        }
        current
    }
}
