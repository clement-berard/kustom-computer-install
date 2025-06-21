git-list-orphan-branches() {
    echo "🔍 Searching for orphaned local branches..."
    echo

    # Update remote references to get the latest info
    echo "📡 Updating remote references..."
    git fetch --prune --quiet 2>/dev/null || {
        echo "❌ Error: Not in a git repository or remote unavailable"
        return 1
    }

    # Get all local branches
    local_branches=$(git branch --format='%(refname:short)')

    # Get all remote branches (removing the remote prefix)
    remote_branches=$(git branch -r --format='%(refname:short)' | grep -v 'HEAD' | sed 's|^origin/||')

    orphan_branches=()
    current_branch=$(git branch --show-current)

    echo "🔎 Analyzing local branches..."
    echo

    while IFS= read -r branch; do
        # Skip empty lines
        [ -z "$branch" ] && continue

        # Check if the local branch has a corresponding remote branch
        if ! echo "$remote_branches" | grep -Fx "$branch" >/dev/null; then
            # Check if branch has been merged into main/master
            is_merged=false

            # Check against main
            if git show-ref --verify --quiet refs/heads/main; then
                if git merge-base --is-ancestor "$branch" main 2>/dev/null; then
                    is_merged=true
                fi
            fi

            # Check against master if main doesn't exist or branch wasn't merged to main
            if [ "$is_merged" = false ] && git show-ref --verify --quiet refs/heads/master; then
                if git merge-base --is-ancestor "$branch" master 2>/dev/null; then
                    is_merged=true
                fi
            fi

            if [ "$is_merged" = true ]; then
                echo "🟡 $branch (merged but not on remote)"
            else
                echo "🔴 $branch (not merged and not on remote)"
            fi
            orphan_branches+=("$branch")
        fi
    done <<< "$local_branches"

    echo

    if [ ${#orphan_branches[@]} -eq 0 ]; then
        echo "✅ No orphaned branches found!"
    else
        echo "📊 Summary: ${#orphan_branches[@]} orphaned branch(es) found"
        echo
        echo "💡 To delete merged branches:"
        echo "   git branch -d <branch_name>"
        echo
        echo "⚠️  To delete unmerged branches (CAUTION):"
        echo "   git branch -D <branch_name>"
        echo
        echo "🔧 Quick cleanup of merged orphaned branches:"
        echo "   cleanup_merged_orphans"
    fi
}

git-interactive-cleanup() {
    git fetch --prune --quiet
    for branch in $(git for-each-ref --format='%(refname:short)' refs/heads/ | grep -v "$(git branch --show-current)"); do
        if ! git branch -r | grep -q "origin/$branch"; then
            printf "Delete '$branch'? [y/N]: "
            read -r answer
            [[ "$answer" =~ ^[Yy]$ ]] && git branch -D "$branch"
        fi
    done
}
