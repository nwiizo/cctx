mod cli;
mod completions;
mod context;
mod interactive;
mod state;

use anyhow::Result;
use clap::Parser;

use cli::Cli;
use completions::print_enhanced_completions;
use context::ContextManager;
use context::SettingsLevel;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle completions first
    if let Some(shell) = cli.completions {
        return print_enhanced_completions(shell);
    }

    // Determine settings level: default to User, explicit flags override
    let settings_level = if cli.local {
        SettingsLevel::Local
    } else if cli.in_project {
        SettingsLevel::Project
    } else {
        // Default: always use user level for predictable behavior
        SettingsLevel::User
    };

    let manager = ContextManager::new_with_level(settings_level)?;

    // Handle special modes first
    if cli.current {
        if let Some(current) = manager.get_current_context()? {
            println!("{}", current);
        }
        return Ok(());
    }

    if cli.unset {
        return manager.unset_context();
    }

    if cli.delete {
        if let Some(context) = cli.context {
            return manager.delete_context(&context);
        } else {
            return manager.interactive_delete();
        }
    }

    if cli.rename {
        if let Some(old_name) = cli.context {
            let new_name: String = dialoguer::Input::new()
                .with_prompt("New name")
                .interact_text()?;
            return manager.rename_context(&old_name, &new_name);
        } else {
            return manager.interactive_rename();
        }
    }

    if cli.new {
        if let Some(name) = cli.context {
            return manager.create_context(&name);
        } else {
            return manager.interactive_create_context();
        }
    }

    if cli.edit {
        let context = if let Some(ctx) = cli.context {
            ctx
        } else if let Some(current) = manager.get_current_context()? {
            current
        } else {
            return Err(anyhow::anyhow!("error: no current context set"));
        };
        return manager.edit_context(&context);
    }

    if cli.show {
        let context = if let Some(ctx) = cli.context {
            ctx
        } else if let Some(current) = manager.get_current_context()? {
            current
        } else {
            return Err(anyhow::anyhow!("error: no current context set"));
        };
        return manager.show_context(&context);
    }

    if cli.export {
        let context = if let Some(ctx) = cli.context {
            ctx
        } else if let Some(current) = manager.get_current_context()? {
            current
        } else {
            return Err(anyhow::anyhow!("error: no current context set"));
        };
        return manager.export_context(&context);
    }

    if cli.import {
        if let Some(name) = cli.context {
            return manager.import_context(&name);
        } else {
            return Err(anyhow::anyhow!("error: context name required for import"));
        }
    }

    // Normal operation
    match cli.context {
        Some(ref name) if name == "-" => {
            // Switch to previous context
            manager.switch_to_previous()
        }
        Some(name) => {
            // Switch to named context
            manager.switch_context(&name)
        }
        None => {
            // No argument - show list or interactive select
            if std::env::var("CCTX_INTERACTIVE").unwrap_or_default() == "1" {
                // Interactive mode
                manager.interactive_select()
            } else {
                // List contexts
                manager.list_contexts_with_current(cli.quiet)
            }
        }
    }
}
