#!/bin/bash

# go assets links
# https://github.com/clement-berard/kustom-computer-install/releases/latest/download/main_darwin_arm64

# Install Homebrew if it's not already installed
if ! command -v brew; then
    echo "Homebrew is not installed. Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    if [ $? -ne 0 ]; then
        echo "Error installing Homebrew" >&2
        return 1
    fi
else
    echo "Homebrew is already installed."
fi

# Update Homebrew
echo "Updating Homebrew..."
brew update
if [ $? -ne 0 ]; then
    echo "Error updating Homebrew" >&2
    return 1
fi

programs=("zsh" "btop" "bat" "n" "glow" "ncdu" "fd" "gum")

# Loop to install each program with Homebrew
for program in "${programs[@]}"; do
    echo "Installing $program..."
    brew install "$program"
done

sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"
git clone https://github.com/zsh-users/zsh-autosuggestions ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/zsh-autosuggestions

# in zshrc add
# plugins=(git zsh-autosuggestions)

echo "All programs have been installed."

# Download kc-cli binary
echo "Downloading main_darwin_arm64 binary..."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GO_BIN_DIR="$SCRIPT_DIR/go_bin"

# Create go_bin directory if it doesn't exist
mkdir -p "$GO_BIN_DIR"

# Download the binary
curl -L https://github.com/clement-berard/kustom-computer-install/releases/latest/download/main_darwin_arm64 -o "$GO_BIN_DIR/main_darwin_arm64"

if [ $? -ne 0 ]; then
    echo "Error downloading main_darwin_arm64" >&2
    return 1
fi

# Make it executable
chmod +x "$GO_BIN_DIR/main_darwin_arm64"

echo "main_darwin_arm64 downloaded successfully to $GO_BIN_DIR/main_darwin_arm64"
echo "To use it, run: $GO_BIN_DIR/main_darwin_arm64"
