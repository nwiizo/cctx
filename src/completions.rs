use anyhow::Result;
use clap::CommandFactory;
use clap_complete::Shell;
use clap_complete::{generate, Generator};
use std::io;

use crate::cli::Cli;
use crate::context::ContextManager;

pub fn print_completions<G: Generator>(gen: G, cmd: &mut clap::Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

pub fn print_enhanced_completions(shell: Shell) -> Result<()> {
    let manager = ContextManager::new()?;
    let contexts = manager.list_contexts()?;
    let context_list = contexts.join(" ");

    match shell {
        Shell::Bash => {
            println!(
                r#"_cctx() {{
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
fi"#,
                context_list, context_list
            );
        }
        Shell::Fish => {
            println!("complete -c cctx -l completions -d 'Generate shell completions' -r -f -a \"bash\t''
elvish\t''
fish\t''
powershell\t''
zsh\t''\"");
            for opt in ["-d", "-e", "-s", "--delete", "--edit", "--show", "--export"] {
                println!(
                    "complete -c cctx {} -d 'Context name' -r -f -a \"{}\"",
                    opt, context_list
                );
            }
            println!(
                "complete -c cctx -s d -l delete -d 'Delete context mode'
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
complete -c cctx -s V -l version -d 'Print version'"
            );
            if !contexts.is_empty() {
                println!("complete -c cctx -f -a \"{}\"", context_list);
            }
        }
        Shell::Zsh => {
            let context_completions = if contexts.is_empty() {
                String::new()
            } else {
                format!(
                    "local contexts=({})\n    _describe 'contexts' contexts",
                    contexts
                        .iter()
                        .map(|c| format!("'{}'", c))
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            };

            println!(
                r#"#compdef cctx

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
fi"#,
                context_completions
            );
        }
        _ => {
            // Fallback to basic completion for other shells
            let mut cmd = Cli::command();
            print_completions(shell, &mut cmd);
        }
    }
    Ok(())
}
