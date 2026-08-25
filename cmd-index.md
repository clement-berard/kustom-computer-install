# git

- `git rebase --onto <newparent> <oldparent> <currentbranch>`
- `git log -n 1 --pretty=format:"%H" master` _hash last commit_
- `git checkout origin/dev -- path/to/file` _revert one file_
- `git reset --soft HEAD~1` _undo last commit but keep changes_

# gh stack

- `gh stack view` _status de la stack (branches, PR liées)_
- `gh stack checkout <branch>` _va sur une branche précise (nom, n° PR, URL)_
- `gh stack switch` _sélecteur interactif pour naviguer dans la stack_
- `gh stack up [n]` / `gh stack down [n]` _monte / descend de n couches_
- `gh stack top` / `gh stack bottom` / `gh stack trunk` _va en haut / bas / trunk_
- `gh stack rebase --upstack` _rebase en cascade les branches au-dessus de la courante_
- `gh stack push` _push toutes les branches modifiées (force-with-lease)_
- `gh stack sync --prune` _fetch + rebase cascade + push + nettoie branches mergées (à lancer après un merge de PR)_
