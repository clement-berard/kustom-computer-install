# Common entrypoint, shared by every shell (zsh, bash...). Nothing shell-specific here.
# The shell entrypoint exports KC_DIR (repo root); fallback for plain bash sourcing.
SCRIPT_DIR="${KC_DIR:-$(dirname "$(realpath "${BASH_SOURCE[0]:-$0}")")}"

export LANG=en_US.UTF-8

source "$SCRIPT_DIR/cmds/path.sh"

GPG_TTY=$(tty)
export GPG_TTY

hme() {
    if [ -n "$1" ]; then
        glow "$SCRIPT_DIR/cmd-index.md" | grep "$1"
    else
        glow "$SCRIPT_DIR/cmd-index.md"
    fi
}

myl() {
    local prompt="Answer immediately in the same language as my prompt, without any explanation. My Request: $@"
    #llm -m mlx-community/gemma-3-4b-it-8bit "$prompt" 2>/dev/null
    llm -m mlx-community/Phi-3-mini-4k-instruct-4bit "$prompt" 2>/dev/null
}

for f in "$SCRIPT_DIR"/cmds/aliases/*.sh; do source "$f"; done
source "$SCRIPT_DIR/cmds/git.sh"
source "$SCRIPT_DIR/cmds/fs.sh"
source "$SCRIPT_DIR/cmds/check-npm.sh"

# Secrets and machine-specific paths live outside the repo.
[ -f "$HOME/.config/shell/secrets.sh" ] && source "$HOME/.config/shell/secrets.sh"
[ -f "$HOME/.config/shell/local.sh" ] && source "$HOME/.config/shell/local.sh"
