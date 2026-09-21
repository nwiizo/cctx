use anyhow::Result;
use clap::{CommandFactory, builder::PossibleValuesParser};
use clap_complete::{Shell, generate};
use std::io;

use crate::cli::Cli;
use crate::context::ContextManager;

pub(crate) fn print_completions(shell: Shell) -> Result<()> {
    let contexts = ContextManager::new()?.list_contexts()?;
    let mut command = Cli::command();
    if !contexts.is_empty() {
        command = command.mut_arg("context", |arg| {
            arg.value_parser(PossibleValuesParser::new(contexts))
        });
    }
    generate(shell, &mut command, "cctx", &mut io::stdout());
    Ok(())
}
