SCRIPT_DIR=$(dirname "$(realpath "$0")")

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

source "$SCRIPT_DIR/cmds/aliases.sh"
source "$SCRIPT_DIR/cmds/git.sh"
source "$SCRIPT_DIR/cmds/fs.sh"
