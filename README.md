# Kustom Computer Install

My computer config: shell, terminal, tools.

## Install

```bash
git clone <repo> && cd keload-computer
./install.sh
```

`install.sh` symlinks `~` to this repo: `~/.zshrc`, `~/.zshenv`, `~/.zprofile`, the Ghostty config and its shader. An existing file is kept next to the link as `<name>.pre-kc`. Safe to run again.

Settings from the old config are not migrated: copy them over from the `.pre-kc` files.

## Structure

| Path | Role |
|---|---|
| `.bash_custom.sh` | Common entrypoint for every shell (PATH, `hme`, `myl`, loads `cmds/`) |
| `cmds/` | Aliases, functions, PATH, env: portable across bash and zsh |
| `zsh/` | zsh-specific only (oh-my-zsh, plugins, completions) |
| `ghostty/` | Ghostty config and shader |
| `cmd-index.md` | Command cheat sheet (`hme`) |
| `init.sh` | Installs tools (Homebrew, oh-my-zsh, packages) |
| `go/`, `rust/` | Custom CLIs |

Switching shell: add an entrypoint that sets `KC_DIR` (repo root) and sources `.bash_custom.sh`, like `zsh/zshrc`.

## Secrets and personal paths

Never in the repo. They live in `~/.config/shell/` and are loaded if present:

- `secrets.sh`: tokens (`export GITHUB_TOKEN=...`). Template: `secrets.example.sh`. Create it by hand on each machine.
- `local.sh`: aliases and functions with machine-specific paths.
