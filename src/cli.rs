use clap::Parser;
use clap_complete::Shell;

#[derive(Parser)]
#[command(name = "cctx")]
#[command(about = "Claude Code context switcher", version)]
#[command(author, long_about = None)]
pub struct Cli {
    /// Context name to switch to, or '-' to switch to previous context
    pub context: Option<String>,

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

    /// Settings level: user (~/.claude), project (./.claude), local (./.claude/settings.local.json)
    #[arg(long = "level", value_enum)]
    pub level: Option<SettingsLevel>,

    /// Force user-level settings (~/.claude/settings.json)
    #[arg(long = "user", conflicts_with = "level")]
    pub user: bool,

    /// Force project-level settings (./.claude/settings.json)
    #[arg(long = "project", conflicts_with = "level")]
    pub project: bool,

    /// Force local project settings (./.claude/settings.local.json)
    #[arg(long = "local", conflicts_with = "level")]
    pub local: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum SettingsLevel {
    User,
    Project,
    Local,
}
