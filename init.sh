#!/bin/bash

# Install Homebrew if not already installed

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

programs=("zsh" "btop" "bat" "n" "glow" "ncdu" "fd")

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
