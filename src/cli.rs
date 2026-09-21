use clap::Parser;
use clap_complete::Shell;

#[derive(Parser)]
#[command(name = "cctx")]
#[command(about = "Claude Code context switcher", version)]
#[command(author, long_about = None)]
#[command(group(clap::ArgGroup::new("operation").args([
    "delete", "current", "rename", "new", "edit", "show", "export", "import",
    "unset", "completions", "add_account", "accounts", "account_path", "shell_path",
    "shell_init", "login", "status", "logout"
])))]
pub(crate) struct Cli {
    /// Context name to switch to, or '-' to switch to previous context
    pub(crate) context: Option<String>,

    /// New name when using --rename OLD NEW
    #[arg(requires = "rename")]
    pub(crate) new_name: Option<String>,

    /// Select an account, or scope a settings/authentication operation to it
    #[arg(long, conflicts_with_all = ["add_account", "accounts", "completions", "shell_init"])]
    pub(crate) account: Option<String>,

    /// Create an empty account profile; does not copy credentials or settings
    #[arg(long, conflicts_with = "context")]
    pub(crate) add_account: Option<String>,

    /// List account names and their configuration directories
    #[arg(long, conflicts_with = "context")]
    pub(crate) accounts: bool,

    /// Print the selected account's configuration directory
    #[arg(long, requires = "account", conflicts_with = "context")]
    pub(crate) account_path: bool,

    /// Log in using Claude Code in the selected account
    #[arg(long, requires = "account", conflicts_with = "context")]
    pub(crate) login: bool,

    /// Show the selected account's authentication status using Claude Code
    #[arg(long, requires = "account", conflicts_with = "context")]
    pub(crate) status: bool,

    /// Log out of the selected account using Claude Code
    #[arg(long, requires = "account", conflicts_with = "context")]
    pub(crate) logout: bool,

    /// Print shell integration for fish, bash or zsh
    #[arg(long, conflicts_with = "context")]
    pub(crate) shell_init: Option<Shell>,

    /// Internal: validated configuration path for shell selection
    #[arg(long, hide = true, requires = "account", conflicts_with = "context")]
    pub(crate) shell_path: bool,

    /// Delete context mode
    #[arg(short = 'd', long = "delete")]
    pub(crate) delete: bool,

    /// Current context mode
    #[arg(short = 'c', long = "current")]
    pub(crate) current: bool,

    /// Rename context mode
    #[arg(short = 'r', long = "rename")]
    pub(crate) rename: bool,

    /// Create new context from current settings
    #[arg(short = 'n', long = "new")]
    pub(crate) new: bool,

    /// Edit context with $EDITOR
    #[arg(short = 'e', long = "edit")]
    pub(crate) edit: bool,

    /// Show context content
    #[arg(short = 's', long = "show")]
    pub(crate) show: bool,

    /// Export context to stdout
    #[arg(long = "export")]
    pub(crate) export: bool,

    /// Import context from stdin
    #[arg(long = "import")]
    pub(crate) import: bool,

    /// Unset current context (removes settings file)
    #[arg(short = 'u', long = "unset")]
    pub(crate) unset: bool,

    /// Generate shell completions
    #[arg(long = "completions")]
    pub(crate) completions: Option<Shell>,

    /// Show only current context (no highlighting when listing)
    #[arg(short = 'q', long = "quiet")]
    pub(crate) quiet: bool,
}
