# Source with: cctx --shell-init fish | source
function cctx --wraps cctx --description 'Switch Claude accounts or manage settings'
    if test (count $argv) -eq 2; and test "$argv[1]" = --account
        set -l config_dir (command cctx --account "$argv[2]" --shell-path)
        or return $status
        if test "$argv[2]" = default
            set -e CLAUDE_CONFIG_DIR
        else
            set -gx CLAUDE_CONFIG_DIR "$config_dir"
        end
        printf 'Selected account "%s". Run claude when ready.\n' "$argv[2]"
    else
        command cctx $argv
    end
end
