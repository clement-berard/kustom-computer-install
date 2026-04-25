#!/usr/bin/env bash

set -e

BINARY_NAME="hello-rust"

NAMES=("macos-arm64" "linux-x64" "windows-x64")
TARGETS=("aarch64-apple-darwin" "x86_64-unknown-linux-musl" "x86_64-pc-windows-gnu")

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
