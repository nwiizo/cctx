use anyhow::{Context, Result, bail};
use clap::{Parser, CommandFactory};
use clap_complete::{generate, Generator, Shell};
use colored::*;
use dialoguer::{FuzzySelect, Input, Confirm};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use which::which;

#[derive(Parser)]
#[command(name = "cctx")]
#[command(about = "Claude Code context switcher", version)]
#[command(author, long_about = None)]
struct Cli {
    /// Context name to switch to, or '-' to switch to previous context
    context: Option<String>,
    
    /// Delete context mode
    #[arg(short = 'd', long = "delete")]
    delete: bool,
    
    /// Current context mode
    #[arg(short = 'c', long = "current")]
    current: bool,
    
    /// Rename context mode
    #[arg(short = 'r', long = "rename")]
    rename: bool,
    
    /// Create new context from current settings
    #[arg(short = 'n', long = "new")]
    new: bool,
    
    /// Edit context with $EDITOR
    #[arg(short = 'e', long = "edit")]
    edit: bool,
    
    /// Show context content
    #[arg(short = 's', long = "show")]
    show: bool,
    
    /// Export context to stdout
    #[arg(long = "export")]
    export: bool,
    
    /// Import context from stdin
    #[arg(long = "import")]
    import: bool,
    
    /// Unset current context (removes ~/.claude/settings.json)
    #[arg(short = 'u', long = "unset")]
    unset: bool,
    
    /// Generate shell completions
    #[arg(long = "completions")]
    completions: Option<Shell>,
    
    /// Show only current context (no highlighting when listing)
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,
}

#[derive(Serialize, Deserialize, Default)]
struct State {
    current: Option<String>,
    previous: Option<String>,
}

struct ContextManager {
    contexts_dir: PathBuf,
    claude_settings_path: PathBuf,
    state_path: PathBuf,
}

impl ContextManager {
    fn new() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .context("Failed to get home directory")?;
        
        let claude_dir = home_dir.join(".claude");
        let contexts_dir = claude_dir.join("settings");
        let claude_settings_path = claude_dir.join("settings.json");
        let state_path = contexts_dir.join(".cctx-state.json");
        
        // Create directories if they don't exist
        fs::create_dir_all(&contexts_dir)?;
        
        Ok(Self {
            contexts_dir,
            claude_settings_path,
            state_path,
        })
    }
    
    fn context_path(&self, name: &str) -> PathBuf {
        self.contexts_dir.join(format!("{}.json", name))
    }
    
    fn load_state(&self) -> Result<State> {
        if self.state_path.exists() {
            let content = fs::read_to_string(&self.state_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(State::default())
        }
    }
    
    fn save_state(&self, state: &State) -> Result<()> {
        let content = serde_json::to_string_pretty(state)?;
        fs::write(&self.state_path, content)?;
        Ok(())
    }
    
    fn list_contexts(&self) -> Result<Vec<String>> {
        let mut contexts = Vec::new();
        
        if let Ok(entries) = fs::read_dir(&self.contexts_dir) {
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                
                // Skip hidden files and non-JSON files
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if filename.starts_with('.') {
                        continue;
                    }
                }
                
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        contexts.push(name.to_string());
                    }
                }
            }
        }
        
        contexts.sort();
        Ok(contexts)
    }
    
    fn get_current_context(&self) -> Result<Option<String>> {
        let state = self.load_state()?;
        Ok(state.current)
    }
    
    fn switch_context(&self, name: &str) -> Result<()> {
        let contexts = self.list_contexts()?;
        if !contexts.contains(&name.to_string()) {
            bail!("error: no context exists with the name \"{}\"", name);
        }
        
        let mut state = self.load_state()?;
        
        // Save previous context
        if let Some(current) = &state.current {
            if current != name {
                state.previous = Some(current.clone());
            }
        }
        
        // Copy context settings to Claude settings
        let context_path = self.context_path(name);
        let content = fs::read_to_string(&context_path)?;
        
        // Create .claude directory if it doesn't exist
        if let Some(parent) = self.claude_settings_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(&self.claude_settings_path, content)?;
        
        // Update state
        state.current = Some(name.to_string());
        self.save_state(&state)?;
        
        println!("Switched to context \"{}\"", name.green().bold());
        Ok(())
    }
    
    fn switch_to_previous(&self) -> Result<()> {
        let state = self.load_state()?;
        
        if let Some(previous) = state.previous {
            self.switch_context(&previous)?;
        } else {
            bail!("error: no previous context");
        }
        
        Ok(())
    }
    
    fn create_context(&self, name: &str) -> Result<()> {
        if name.is_empty() || name == "-" || name == "." || name == ".." || name.contains('/') {
            bail!("error: invalid context name \"{}\"", name);
        }
        
        let contexts = self.list_contexts()?;
        if contexts.contains(&name.to_string()) {
            bail!("error: context \"{}\" already exists", name);
        }
        
        let context_path = self.context_path(name);
        
        if self.claude_settings_path.exists() {
            // Copy current Claude settings
            fs::copy(&self.claude_settings_path, &context_path)?;
            println!("Context \"{}\" created from current settings", name.green().bold());
        } else {
            // Create empty settings
            let empty_settings = serde_json::json!({});
            fs::write(&context_path, serde_json::to_string_pretty(&empty_settings)?)?;
            println!("Context \"{}\" created (empty)", name.green().bold());
        }
        
        Ok(())
    }
    
    fn delete_context(&self, name: &str) -> Result<()> {
        let state = self.load_state()?;
        
        if state.current.as_ref() == Some(&name.to_string()) {
            bail!("error: cannot delete the active context \"{}\"", name);
        }
        
        let context_path = self.context_path(name);
        if !context_path.exists() {
            bail!("error: no context exists with the name \"{}\"", name);
        }
        
        fs::remove_file(context_path)?;
        
        // Update state if this was the previous context
        if state.previous.as_ref() == Some(&name.to_string()) {
            let mut new_state = state;
            new_state.previous = None;
            self.save_state(&new_state)?;
        }
        
        println!("Context \"{}\" deleted", name.red());
        Ok(())
    }
    
    fn rename_context(&self, old_name: &str, new_name: &str) -> Result<()> {
        if new_name.is_empty() || new_name == "-" || new_name == "." || new_name == ".." || new_name.contains('/') {
            bail!("error: invalid context name \"{}\"", new_name);
        }
        
        let contexts = self.list_contexts()?;
        if !contexts.contains(&old_name.to_string()) {
            bail!("error: no context exists with the name \"{}\"", old_name);
        }
        
        if contexts.contains(&new_name.to_string()) {
            bail!("error: context \"{}\" already exists", new_name);
        }
        
        let old_path = self.context_path(old_name);
        let new_path = self.context_path(new_name);
        fs::rename(old_path, new_path)?;
        
        // Update state if needed
        let mut state = self.load_state()?;
        let mut updated = false;
        
        if state.current.as_ref() == Some(&old_name.to_string()) {
            state.current = Some(new_name.to_string());
            updated = true;
        }
        
        if state.previous.as_ref() == Some(&old_name.to_string()) {
            state.previous = Some(new_name.to_string());
            updated = true;
        }
        
        if updated {
            self.save_state(&state)?;
        }
        
        println!("Context \"{}\" renamed to \"{}\"", old_name, new_name.green().bold());
        Ok(())
    }
    
    fn show_context(&self, name: &str) -> Result<()> {
        let context_path = self.context_path(name);
        if !context_path.exists() {
            bail!("error: no context exists with the name \"{}\"", name);
        }
        
        let content = fs::read_to_string(context_path)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        let pretty = serde_json::to_string_pretty(&json)?;
        
        println!("{}", pretty);
        Ok(())
    }
    
    fn edit_context(&self, name: &str) -> Result<()> {
        let context_path = self.context_path(name);
        if !context_path.exists() {
            bail!("error: no context exists with the name \"{}\"", name);
        }
        
        let editor = std::env::var("EDITOR")
            .or_else(|_| std::env::var("VISUAL"))
            .unwrap_or_else(|_| "vi".to_string());
        
        let status = Command::new(&editor)
            .arg(&context_path)
            .status()?;
        
        if !status.success() {
            bail!("error: editor exited with non-zero status");
        }
        
        Ok(())
    }
    
    fn export_context(&self, name: &str) -> Result<()> {
        let context_path = self.context_path(name);
        if !context_path.exists() {
            bail!("error: no context exists with the name \"{}\"", name);
        }
        
        let content = fs::read_to_string(context_path)?;
        print!("{}", content);
        Ok(())
    }
    
    fn import_context(&self, name: &str) -> Result<()> {
        if name.is_empty() || name == "-" || name == "." || name == ".." || name.contains('/') {
            bail!("error: invalid context name \"{}\"", name);
        }
        
        let contexts = self.list_contexts()?;
        if contexts.contains(&name.to_string()) {
            bail!("error: context \"{}\" already exists", name);
        }
        
        use std::io::Read;
        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer)?;
        
        // Validate JSON
        let _: serde_json::Value = serde_json::from_str(&buffer)
            .context("error: invalid JSON input")?;
        
        let context_path = self.context_path(name);
        fs::write(&context_path, buffer)?;
        
        println!("Context \"{}\" imported", name.green().bold());
        Ok(())
    }
    
    fn unset_context(&self) -> Result<()> {
        if self.claude_settings_path.exists() {
            fs::remove_file(&self.claude_settings_path)?;
        }
        
        let mut state = self.load_state()?;
        if let Some(current) = state.current.take() {
            state.previous = Some(current);
            self.save_state(&state)?;
        }
        
        println!("Unset current context");
        Ok(())
    }
    
    fn interactive_select(&self) -> Result<()> {
        let contexts = self.list_contexts()?;
        if contexts.is_empty() {
            println!("No contexts found. Create one with: cctx -n <name>");
            return Ok(());
        }
        
        let current = self.get_current_context()?;
        
        // Use fzf if available, otherwise use built-in fuzzy selector
        if which("fzf").is_ok() && std::env::var("TERM").is_ok() {
            // Create temporary file with contexts
            use std::io::Write;
            let mut cmd = Command::new("fzf");
            cmd.arg("--ansi");
            cmd.arg("--no-multi");
            
            if let Some(ref current_ctx) = current {
                cmd.arg("--header").arg(format!("Current: {}", current_ctx));
            }
            
            let mut child = cmd
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()?;
            
            if let Some(mut stdin) = child.stdin.take() {
                for ctx in &contexts {
                    if Some(ctx) == current.as_ref() {
                        writeln!(stdin, "{} {}", ctx.green().bold(), "(current)".dimmed())?;
                    } else {
                        writeln!(stdin, "{}", ctx)?;
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
        } else {
            // Use built-in fuzzy selector
            let items: Vec<String> = contexts.iter().map(|ctx| {
                if Some(ctx) == current.as_ref() {
                    format!("{} (current)", ctx)
                } else {
                    ctx.clone()
                }
            }).collect();
            
            let selection = FuzzySelect::new()
                .with_prompt("Select context")
                .items(&items)
                .interact()?;
            
            let selected = &contexts[selection];
            if Some(selected) != current.as_ref() {
                self.switch_context(selected)?;
            }
        }
        
        Ok(())
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut clap::Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

fn print_enhanced_completions(shell: Shell) -> Result<()> {
    let manager = ContextManager::new()?;
    let contexts = manager.list_contexts()?;
    let context_list = contexts.join(" ");
    
    match shell {
        Shell::Bash => {
            println!(r#"_cctx() {{
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${{BASH_VERSINFO[0]}}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${{COMP_WORDS[COMP_CWORD]}}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${{COMP_WORDS[@]:0:COMP_CWORD}}"
    do
        case "${{cmd}},${{i}}" in
            ",$1")
                cmd="cctx"
                ;;
            *)
                ;;
        esac
    done

    case "${{cmd}}" in
        cctx)
            opts="-d -c -r -n -e -s -u -q -h -V --delete --current --rename --new --edit --show --export --import --unset --completions --quiet --help --version"
            if [[ ${{cur}} == -* || ${{COMP_CWORD}} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${{opts}}" -- "${{cur}}") )
                return 0
            fi
            case "${{prev}}" in
                --completions)
                    COMPREPLY=($(compgen -W "bash elvish fish powershell zsh" -- "${{cur}}"))
                    return 0
                    ;;
                -d|--delete|-e|--edit|-s|--show|--export)
                    COMPREPLY=($(compgen -W "{}" -- "${{cur}}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=($(compgen -W "{} ${{opts}}" -- "${{cur}}"))
                    return 0
                    ;;
            esac
            ;;
    esac
}}

if [[ "${{BASH_VERSINFO[0]}}" -eq 4 && "${{BASH_VERSINFO[1]}}" -ge 4 || "${{BASH_VERSINFO[0]}}" -gt 4 ]]; then
    complete -F _cctx -o nosort -o bashdefault -o default cctx
else
    complete -F _cctx -o bashdefault -o default cctx
fi"#, context_list, context_list);
        },
        Shell::Fish => {
            println!("complete -c cctx -l completions -d 'Generate shell completions' -r -f -a \"bash\t''
elvish\t''
fish\t''
powershell\t''
zsh\t''\"");
            for opt in ["-d", "-e", "-s", "--delete", "--edit", "--show", "--export"] {
                println!("complete -c cctx {} -d 'Context name' -r -f -a \"{}\"", opt, context_list);
            }
            println!("complete -c cctx -s d -l delete -d 'Delete context mode'
complete -c cctx -s c -l current -d 'Current context mode'
complete -c cctx -s r -l rename -d 'Rename context mode'
complete -c cctx -s n -l new -d 'Create new context from current settings'
complete -c cctx -s e -l edit -d 'Edit context with $EDITOR'
complete -c cctx -s s -l show -d 'Show context content'
complete -c cctx -l export -d 'Export context to stdout'
complete -c cctx -l import -d 'Import context from stdin'
complete -c cctx -s u -l unset -d 'Unset current context (removes ~/.claude/settings.json)'
complete -c cctx -s q -l quiet -d 'Show only current context (no highlighting when listing)'
complete -c cctx -s h -l help -d 'Print help'
complete -c cctx -s V -l version -d 'Print version'");
            if !contexts.is_empty() {
                println!("complete -c cctx -f -a \"{}\"", context_list);
            }
        },
        Shell::Zsh => {
            let context_completions = if contexts.is_empty() {
                String::new()
            } else {
                format!("local contexts=({})\n    _describe 'contexts' contexts", contexts.iter().map(|c| format!("'{}'", c)).collect::<Vec<_>>().join(" "))
            };
            
            println!(r#"#compdef cctx

autoload -U is-at-least

_cctx() {{
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${{_arguments_options[@]}}" : \
'--completions=[Generate shell completions]:COMPLETIONS:(bash elvish fish powershell zsh)' \
'-d[Delete context mode]:context:_cctx_contexts' \
'--delete[Delete context mode]:context:_cctx_contexts' \
'-c[Current context mode]' \
'--current[Current context mode]' \
'-r[Rename context mode]' \
'--rename[Rename context mode]' \
'-n[Create new context from current settings]' \
'--new[Create new context from current settings]' \
'-e[Edit context with \$EDITOR]:context:_cctx_contexts' \
'--edit[Edit context with \$EDITOR]:context:_cctx_contexts' \
'-s[Show context content]:context:_cctx_contexts' \
'--show[Show context content]:context:_cctx_contexts' \
'--export[Export context to stdout]:context:_cctx_contexts' \
'--import[Import context from stdin]' \
'-u[Unset current context (removes ~/.claude/settings.json)]' \
'--unset[Unset current context (removes ~/.claude/settings.json)]' \
'-q[Show only current context (no highlighting when listing)]' \
'--quiet[Show only current context (no highlighting when listing)]' \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
'::context:_cctx_contexts' \
&& ret=0
}}

_cctx_contexts() {{
    {}
}}

(( $+functions[_cctx_commands] )) ||
_cctx_commands() {{
    local commands; commands=()
    _describe -t commands 'cctx commands' commands "$@"
}}

if [ "$funcstack[1]" = "_cctx" ]; then
    _cctx "$@"
else
    compdef _cctx cctx
fi"#, context_completions);
        },
        _ => {
            // Fallback to basic completion for other shells
            let mut cmd = Cli::command();
            print_completions(shell, &mut cmd);
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Handle completions first
    if let Some(shell) = cli.completions {
        return print_enhanced_completions(shell);
    }
    
    let manager = ContextManager::new()?;
    
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
            // Interactive delete
            let contexts = manager.list_contexts()?;
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
                .with_prompt(format!("Delete context \"{}\"?", selected))
                .default(false)
                .interact()?;
            
            if confirm {
                return manager.delete_context(selected);
            }
            return Ok(());
        }
    }
    
    if cli.rename {
        if let Some(old_name) = cli.context {
            let new_name: String = Input::new()
                .with_prompt("New name")
                .interact_text()?;
            return manager.rename_context(&old_name, &new_name);
        } else {
            // Interactive rename
            let contexts = manager.list_contexts()?;
            if contexts.is_empty() {
                println!("No contexts found");
                return Ok(());
            }
            
            let selection = FuzzySelect::new()
                .with_prompt("Select context to rename")
                .items(&contexts)
                .interact()?;
            
            let old_name = &contexts[selection];
            let new_name: String = Input::new()
                .with_prompt("New name")
                .interact_text()?;
            
            return manager.rename_context(old_name, &new_name);
        }
    }
    
    if cli.new {
        if let Some(name) = cli.context {
            return manager.create_context(&name);
        } else {
            let name: String = Input::new()
                .with_prompt("Context name")
                .interact_text()?;
            return manager.create_context(&name);
        }
    }
    
    if cli.edit {
        let context = if let Some(ctx) = cli.context {
            ctx
        } else if let Some(current) = manager.get_current_context()? {
            current
        } else {
            bail!("error: no current context set");
        };
        return manager.edit_context(&context);
    }
    
    if cli.show {
        let context = if let Some(ctx) = cli.context {
            ctx
        } else if let Some(current) = manager.get_current_context()? {
            current
        } else {
            bail!("error: no current context set");
        };
        return manager.show_context(&context);
    }
    
    if cli.export {
        let context = if let Some(ctx) = cli.context {
            ctx
        } else if let Some(current) = manager.get_current_context()? {
            current
        } else {
            bail!("error: no current context set");
        };
        return manager.export_context(&context);
    }
    
    if cli.import {
        if let Some(name) = cli.context {
            return manager.import_context(&name);
        } else {
            bail!("error: context name required for import");
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
            let contexts = manager.list_contexts()?;
            let current = manager.get_current_context()?;
            
            if std::env::var("CCTX_INTERACTIVE").unwrap_or_default() == "1" {
                // Interactive mode
                manager.interactive_select()
            } else if cli.quiet {
                // Quiet mode - only show current context
                if let Some(current_ctx) = current {
                    println!("{}", current_ctx);
                }
                Ok(())
            } else {
                // Just list contexts with current highlighted
                if contexts.is_empty() {
                    println!("No contexts found. Create one with: cctx -n <name>");
                    return Ok(());
                }
                
                for ctx in contexts {
                    if Some(&ctx) == current.as_ref() {
                        println!("{} {}", ctx.green().bold(), "(current)".dimmed());
                    } else {
                        println!("{}", ctx);
                    }
                }
                Ok(())
            }
        }
    }
}