use anyhow::{bail, Result};
use clap_complete::Shell;

pub fn print_init(shell: Shell) -> Result<()> {
    let script = match shell {
        Shell::Fish => include_str!("../shell/cctx.fish"),
        Shell::Bash | Shell::Zsh => include_str!("../shell/cctx.sh"),
        _ => bail!("Shell integration supports fish, bash and zsh"),
    };
    print!("{script}");
    Ok(())
}
