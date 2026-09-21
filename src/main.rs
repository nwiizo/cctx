mod account;
mod cli;
mod completions;
mod context;
mod shell;
mod state;
mod storage;

use anyhow::{Result, bail};
use clap::Parser;

use cli::Cli;
use context::ContextManager;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(shell) = cli.completions {
        return completions::print_completions(shell);
    }
    if let Some(shell) = cli.shell_init {
        return shell::print_init(shell);
    }

    if cli.accounts {
        return account::Accounts::new()?.list();
    }
    if let Some(name) = &cli.add_account {
        return account::Accounts::new()?.add(name);
    }
    let config_dir = match &cli.account {
        Some(name) => account::Accounts::new()?.resolve(name)?,
        None => account::default_config_dir()?,
    };
    if cli.account_path {
        println!("{}", config_dir.display());
        return Ok(());
    }
    if cli.shell_path {
        account::check_auth_overrides()?;
        if cli.account.as_deref() != Some("default") {
            println!("{}", config_dir.display());
        }
        return Ok(());
    }
    if cli.login || cli.status || cli.logout {
        let action = if cli.login {
            "login"
        } else if cli.status {
            "status"
        } else {
            "logout"
        };
        let auth_dir = (cli.account.as_deref() != Some("default")).then_some(config_dir);
        let status = account::authenticate(auth_dir, action)?;
        std::process::exit(status.code().unwrap_or(1));
    }

    let manager = ContextManager::with_config_dir(config_dir)?;

    if cli.current {
        if let Some(current) = manager.current_context()? {
            println!("{current}");
        }
        return Ok(());
    }
    if cli.unset {
        return manager.unset_context();
    }
    if cli.delete {
        return match cli.context {
            Some(name) => manager.delete_context(&name),
            None => bail!("context name required for delete"),
        };
    }
    if cli.rename {
        return match (cli.context, cli.new_name) {
            (Some(old_name), Some(new_name)) => manager.rename_context(&old_name, &new_name),
            _ => bail!("--rename requires OLD and NEW context names"),
        };
    }
    if cli.new {
        return match cli.context {
            Some(name) => manager.create_context(&name),
            None => bail!("context name required for new"),
        };
    }
    if cli.import {
        return match cli.context {
            Some(name) => manager.import_context(&name),
            None => bail!("context name required for import"),
        };
    }
    if cli.edit || cli.show || cli.export {
        let name = match cli.context.or(manager.current_context()?) {
            Some(name) => name,
            None => bail!("no current context set"),
        };
        return if cli.edit {
            manager.edit_context(&name)
        } else if cli.show {
            manager.show_context(&name)
        } else {
            manager.export_context(&name)
        };
    }

    match cli.context.as_deref() {
        Some("-") => manager.switch_to_previous(),
        Some(name) => manager.switch_context(name),
        None => {
            if cli.account.is_some() && !cli.quiet {
                bail!(
                    "Account selection needs shell integration. Fish: cctx --shell-init fish | source; Bash/Zsh: eval \"$(cctx --shell-init bash)\". Then run cctx --account NAME again."
                );
            }
            manager.list_contexts_with_current(cli.quiet)
        }
    }
}
