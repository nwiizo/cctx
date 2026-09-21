# Source with: eval "$(cctx --shell-init bash)" (also supports zsh)
cctx() {
    if [ "$#" -eq 2 ] && [ "$1" = --account ]; then
        local cctx_config_dir
        cctx_config_dir="$(command cctx --account "$2" --shell-path)" || return $?
        if [ "$2" = default ]; then
            unset CLAUDE_CONFIG_DIR
        else
            export CLAUDE_CONFIG_DIR="$cctx_config_dir"
        fi
        printf 'Selected account "%s". Run claude when ready.\n' "$2"
    else
        command cctx "$@"
    fi
}
