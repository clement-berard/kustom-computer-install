#!/bin/bash
# Links the home folder to this repo (zsh, Ghostty). Idempotent.
# An existing regular file is kept next to the link as <name>.pre-kc.

set -e
KC_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

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
