use clap::Parser;
use clap_complete::Shell;

#[derive(Parser)]
#[command(name = "cctx")]
#[command(about = "Claude Code context switcher", version)]
#[command(author, long_about = None)]
#[command(group(clap::ArgGroup::new("merge_operation").args(["merge_from", "unmerge"])))]
#[command(group(clap::ArgGroup::new("operation").args([
    "delete", "current", "rename", "new", "edit", "show", "export", "import",
    "unset", "completions", "merge_from", "unmerge", "merge_history",
    "add_account", "accounts", "account_path", "shell_path", "shell_init", "login", "status", "logout"
])))]
pub struct Cli {
    /// Context name to switch to, or '-' to switch to previous context
    pub context: Option<String>,

    /// New name when using --rename OLD NEW
    #[arg(requires = "rename")]
    pub new_name: Option<String>,

    /// Select an account, or scope a settings/authentication operation to it
    #[arg(long, conflicts_with_all = ["in_project", "local", "add_account", "accounts", "completions", "shell_init"])]
    pub account: Option<String>,

    /// Create an empty account profile; does not copy credentials or settings
    #[arg(long, conflicts_with_all = ["context", "in_project", "local"])]
    pub add_account: Option<String>,

    /// List account names and their configuration directories
    #[arg(long, conflicts_with_all = ["context", "in_project", "local"])]
    pub accounts: bool,

    /// Print the selected account's configuration directory
    #[arg(long, requires = "account", conflicts_with = "context")]
    pub account_path: bool,

    /// Log in using Claude Code in the selected account
    #[arg(long, requires = "account", conflicts_with = "context")]
    pub login: bool,

    /// Show the selected account's authentication status using Claude Code
    #[arg(long, requires = "account", conflicts_with = "context")]
    pub status: bool,

    /// Log out of the selected account using Claude Code
    #[arg(long, requires = "account", conflicts_with = "context")]
    pub logout: bool,

    /// Print shell integration for fish, bash or zsh
    #[arg(long, conflicts_with_all = ["context", "in_project", "local"])]
    pub shell_init: Option<Shell>,

    /// Internal: validated configuration path for shell selection
    #[arg(long, hide = true, requires = "account", conflicts_with = "context")]
    pub shell_path: bool,

    /// Delete context mode
    #[arg(short = 'd', long = "delete")]
    pub delete: bool,

    /// Current context mode
    #[arg(short = 'c', long = "current")]
    pub current: bool,

    /// Rename context mode
    #[arg(short = 'r', long = "rename")]
    pub rename: bool,

    /// Create new context from current settings
    #[arg(short = 'n', long = "new")]
    pub new: bool,

    /// Edit context with $EDITOR
    #[arg(short = 'e', long = "edit")]
    pub edit: bool,

    /// Show context content
    #[arg(short = 's', long = "show")]
    pub show: bool,

    /// Export context to stdout
    #[arg(long = "export")]
    pub export: bool,

    /// Import context from stdin
    #[arg(long = "import")]
    pub import: bool,

    /// Unset current context (removes settings file)
    #[arg(short = 'u', long = "unset")]
    pub unset: bool,

    /// Generate shell completions
    #[arg(long = "completions")]
    pub completions: Option<Shell>,

    /// Show only current context (no highlighting when listing)
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Manage project-level contexts (./.claude/settings.json)
    #[arg(long = "in-project", conflicts_with = "local")]
    pub in_project: bool,

    /// Manage local project contexts (./.claude/settings.local.json)
    #[arg(long = "local")]
    pub local: bool,

    /// Merge permissions from another context or settings file
    #[arg(long = "merge-from")]
    pub merge_from: Option<String>,

    /// Remove previously merged permissions from a specific source
    #[arg(long = "unmerge")]
    pub unmerge: Option<String>,

    /// Show merge history for the current context
    #[arg(long = "merge-history")]
    pub merge_history: bool,

    /// Merge all settings (not just permissions) from source
    #[arg(long = "merge-full", requires = "merge_operation")]
    pub merge_full: bool,
}
