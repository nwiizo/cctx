use anyhow::Result;
use colored::*;
use dialoguer::{Confirm, FuzzySelect, Input};
use std::io::Write;
use std::process::Command;
use which::which;

use crate::context::ContextManager;

impl ContextManager {
    pub fn interactive_select(&self) -> Result<()> {
        let contexts = self.list_contexts()?;
        if contexts.is_empty() {
            println!("No contexts found. Create one with: cctx -n <name>");
            return Ok(());
        }

        let current = self.get_current_context()?;

        // Use fzf if available, otherwise use built-in fuzzy selector
        if which("fzf").is_ok() && std::env::var("TERM").is_ok() {
            self.interactive_select_with_fzf(&contexts, &current)
        } else {
            self.interactive_select_builtin(&contexts, &current)
        }
    }

    fn interactive_select_with_fzf(
        &self,
        contexts: &[String],
        current: &Option<String>,
    ) -> Result<()> {
        let mut cmd = Command::new("fzf");
        cmd.arg("--ansi");
        cmd.arg("--no-multi");

        if let Some(ref current_ctx) = current {
            cmd.arg("--header").arg(format!("Current: {current_ctx}"));
        }

        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            for ctx in contexts {
                if Some(ctx) == current.as_ref() {
                    writeln!(stdin, "{} {}", ctx.green().bold(), "(current)".dimmed())?;
                } else {
                    writeln!(stdin, "{ctx}")?;
                }
            }
        }

        let output = child.wait_with_output()?;

        if output.status.success() {
            let selected = String::from_utf8_lossy(&output.stdout);
            let selected = selected.split_whitespace().next();

            if let Some(name) = selected {
                self.switch_context(name)?;
            }
        }

        Ok(())
    }

    fn interactive_select_builtin(
        &self,
        contexts: &[String],
        current: &Option<String>,
    ) -> Result<()> {
        let items: Vec<String> = contexts
            .iter()
            .map(|ctx| {
                if Some(ctx) == current.as_ref() {
                    format!("{ctx} (current)")
                } else {
                    ctx.clone()
                }
            })
            .collect();

        let selection = FuzzySelect::new()
            .with_prompt("Select context")
            .items(&items)
            .interact()?;

        let selected = &contexts[selection];
        if Some(selected) != current.as_ref() {
            self.switch_context(selected)?;
        }

        Ok(())
    }

    pub fn interactive_delete(&self) -> Result<()> {
        let contexts = self.list_contexts()?;
        if contexts.is_empty() {
            println!("No contexts found");
            return Ok(());
        }

        let selection = FuzzySelect::new()
            .with_prompt("Select context to delete")
            .items(&contexts)
            .interact()?;

        let selected = &contexts[selection];
        let confirm = Confirm::new()
            .with_prompt(format!("Delete context \"{selected}\"?"))
            .default(false)
            .interact()?;

        if confirm {
            self.delete_context(selected)?;
        }

        Ok(())
    }

    pub fn interactive_rename(&self) -> Result<()> {
        let contexts = self.list_contexts()?;
        if contexts.is_empty() {
            println!("No contexts found");
            return Ok(());
        }

        let selection = FuzzySelect::new()
            .with_prompt("Select context to rename")
            .items(&contexts)
            .interact()?;

        let old_name = &contexts[selection];
        let new_name: String = Input::new().with_prompt("New name").interact_text()?;

        self.rename_context(old_name, &new_name)
    }

    pub fn interactive_create_context(&self) -> Result<()> {
        let name: String = Input::new().with_prompt("Context name").interact_text()?;
        self.create_context(&name)
    }
}
