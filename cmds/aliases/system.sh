alias top="btop"

# eza
EZA_OPTS=(--long --git --all --group-directories-first)
alias l='eza "${EZA_OPTS[@]}"'
fde() {
    fd "$@" -X eza "${EZA_OPTS[@]}"
}


#alias cat='bat'
