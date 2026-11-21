# All PATH entries. Portable: works in bash and zsh.

# pnpm
export PNPM_HOME="$HOME/Library/pnpm"
export PATH="$PNPM_HOME/bin:$PATH"

# Go
export GOBIN="$HOME/go/bin"
export PATH="$PATH:$GOBIN"

# Bun
export BUN_INSTALL="$HOME/.bun"
export PATH="$BUN_INSTALL/bin:$PATH"

# pipx and local tools
export PATH="$PATH:$HOME/.local/bin"
