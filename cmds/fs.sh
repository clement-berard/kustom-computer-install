kc-old() {
    if [ "$1" = "rmd" ]; then
        gum spin --title "Remove recursively 'node_modules' in $(pwd)" -- \
            find . -name "node_modules" -type d -prune -exec rm -rf {} +
    else
        echo "Usage: kc rmd"
    fi
}

kc() {
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    "$SCRIPT_DIR/go_bin/main_darwin_arm64" "$@"
}
