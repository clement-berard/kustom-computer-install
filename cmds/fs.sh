kc() {
    if [ "$1" = "rmd" ]; then
        echo "🗑️ in $(pwd)"
        find . -name "node_modules" -type d -prune -exec rm -rf {} +
    else
        echo "Usage: kc rmd"
    fi
}
