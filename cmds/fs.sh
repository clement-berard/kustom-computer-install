kc() {
    local SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-${(%):-%x}}")" && pwd)"
    local KC_ROOT="$(cd "$SCRIPT_DIR" && git rev-parse --show-toplevel 2>/dev/null)"

    if [[ -z "$KC_ROOT" ]]; then
        echo "Error: Script not in a git repository" >&2
        return 1
    fi

    "$KC_ROOT/go_bin/main_darwin_arm64" "$@"
}
