# Kustom Computer Install

My computer config: shell, terminal, tools.

## Install

```bash
git clone <repo> && cd keload-computer
./install.sh
```

`install.sh` sets up a machine, in four steps:

1. Homebrew and packages (fnm, with Node 24 as default)
2. oh-my-zsh and the autosuggestions plugin (your `~/.zshrc` is kept)
3. Symlinks from `~` to this repo: `~/.zshrc`, `~/.zshenv`, `~/.zprofile`, the Ghostty config and its shader
4. The `kc-cli` binary

An existing file is kept next to its link as `<name>.pre-kc`. Safe to run again.

Settings from the old config are not migrated: copy them over from the `.pre-kc` files.

## Migrating from `n` to fnm

`install.sh` installs fnm and Node 24 next to `n`; it never removes `n`. fnm comes first in `PATH`, so both coexist. To clean up `n` on an older machine:

1. Note the global packages to keep: `ls /usr/local/lib/node_modules`.
2. Remove the node installed by `n`: `n uninstall` (it can hang; then delete by hand).
3. Delete what is left in `/usr/local`: `bin/{node,npm,npx,corepack,pnpm,pnpx}`, the links of global packages, `include/node`, `share/doc/node`, `share/man/man1/node.1`, `share/systemtap/tapset/node.stp`, and the packages in `lib/node_modules`. Files owned by root need `sudo rm -rf`.
4. `brew uninstall n`.
5. `corepack enable` (pnpm, yarn), then reinstall the global packages with `npm i -g`.
6. Open a new terminal and check with `kc tool-versions`: node, npm and corepack must come from `fnm_multishells`.

Node versions are only switched by `.node-version` / `.nvmrc` (not by `engines` in `package.json`). JetBrains terminals add their own Node to `PATH`; `zsh/zshrc` puts fnm back first.

## Structure

| Path | Role |
|---|---|
| `.bash_custom.sh` | Common entrypoint for every shell (PATH, `hme`, `myl`, loads `cmds/`) |
| `cmds/` | Aliases, functions, PATH, env: portable across bash and zsh |
| `zsh/` | zsh-specific only (oh-my-zsh, plugins, completions) |
| `ghostty/` | Ghostty config and shader |
| `cmd-index.md` | Command cheat sheet (`hme`) |
| `install.sh` | Machine setup: tools, oh-my-zsh, symlinks, `kc-cli` |
| `rust/` | `kc` CLI (`kc fs rm-nm`, `kc tool-versions`), binary downloaded to `bin/` |

Switching shell: add an entrypoint that sets `KC_DIR` (repo root) and sources `.bash_custom.sh`, like `zsh/zshrc`.

## Secrets and personal paths

Never in the repo. They live in `~/.config/shell/` and are loaded if present:

- `secrets.sh`: tokens (`export GITHUB_TOKEN=...`). Template: `secrets.example.sh`. Create it by hand on each machine.
- `local.sh`: aliases and functions with machine-specific paths.
