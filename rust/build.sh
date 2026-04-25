#!/usr/bin/env bash

set -e

BINARY_NAME="kc-cli"

NAMES=("linux-x64" "windows-x64" "macos-arm64" "macos-x64")
TARGETS=("x86_64-unknown-linux-musl" "x86_64-pc-windows-gnu" "aarch64-apple-darwin" "x86_64-apple-darwin")

echo "🔨 Building..."
for i in "${!NAMES[@]}"; do
  name="${NAMES[$i]}"
  target="${TARGETS[$i]}"
  echo "  → $name ($target)"
  cargo build --release --target "$target"
done

echo ""
echo "📁 Copying to dist/..."
rm -rf dist && mkdir -p dist

for i in "${!NAMES[@]}"; do
  name="${NAMES[$i]}"
  target="${TARGETS[$i]}"
  if [[ "$name" == *"windows"* ]]; then
    src="target/$target/release/$BINARY_NAME.exe"
    dst="dist/$BINARY_NAME-$name.exe"
  else
    src="target/$target/release/$BINARY_NAME"
    dst="dist/$BINARY_NAME-$name"
  fi

  if [ -f "$src" ]; then
    cp "$src" "$dst"
    echo "  ✓ $dst"
  else
    echo "  ✗ $dst (build failed or skipped)"
  fi
done

echo ""
echo "✅ Done!"
