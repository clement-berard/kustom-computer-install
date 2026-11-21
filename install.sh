#!/bin/bash
# Sets up a machine: tools, oh-my-zsh, symlinks into the home folder, kc-cli binary.
# Every step is safe to run again. An existing file is kept next to its link as <name>.pre-kc.

set -e
KC_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 1. Homebrew and packages
if ! command -v brew >/dev/null; then
  echo "Installing Homebrew..."
  /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
  eval "$(/opt/homebrew/bin/brew shellenv)"
fi

echo "Updating Homebrew..."
brew update

programs=(zsh btop bat eza n glow ncdu fd gum witr gh)
for program in "${programs[@]}"; do
  echo "Installing $program..."
  brew install "$program"
done

# 2. oh-my-zsh and plugins (KEEP_ZSHRC: the installer must not replace ~/.zshrc)
if [ ! -d "$HOME/.oh-my-zsh" ]; then
  echo "Installing oh-my-zsh..."
  RUNZSH=no CHSH=no KEEP_ZSHRC=yes \
    sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"
fi

AUTOSUGGESTIONS="${ZSH_CUSTOM:-$HOME/.oh-my-zsh/custom}/plugins/zsh-autosuggestions"
if [ ! -d "$AUTOSUGGESTIONS" ]; then
  git clone https://github.com/zsh-users/zsh-autosuggestions "$AUTOSUGGESTIONS"
fi

# 3. Symlinks
link() {
  local src="$1" dst="$2"
  mkdir -p "$(dirname "$dst")"
  if [ -L "$dst" ] && [ "$(readlink "$dst")" = "$src" ]; then
    echo "ok      $dst"
    return
  fi
  if [ -e "$dst" ] || [ -L "$dst" ]; then
    mv "$dst" "$dst.pre-kc"
    echo "backup  $dst.pre-kc"
  fi
  ln -s "$src" "$dst"
  echo "linked  $dst -> $src"
}

# zsh
link "$KC_DIR/zsh/zshrc" "$HOME/.zshrc"
link "$KC_DIR/zsh/zshenv" "$HOME/.zshenv"
link "$KC_DIR/zsh/zprofile" "$HOME/.zprofile"

# Ghostty
link "$KC_DIR/ghostty/config.ghostty" "$HOME/Library/Application Support/com.mitchellh.ghostty/config.ghostty"
link "$KC_DIR/ghostty/shaders/shader.glsl" "$HOME/.config/ghostty/shaders/shader.glsl"

# 4. kc-cli binary (Rust, built by the Release workflow)
BIN_DIR="$KC_DIR/bin"
mkdir -p "$BIN_DIR"
echo "Downloading kc-cli..."
curl -fL "https://github.com/clement-berard/kustom-computer-install/releases/latest/download/kc-cli-macos-arm64" \
  -o "$BIN_DIR/kc-cli"
chmod +x "$BIN_DIR/kc-cli"

echo "Done. Create ~/.config/shell/secrets.sh from secrets.example.sh, then open a new terminal."
