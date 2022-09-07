#!/bin/sh

set -e

if ! command -v unzip >/dev/null; then
  echo "Error: unzip is required to install Dune." 1>&2
  echo "=>> You can install unzip via \"brew install unzip\" on MacOS or \"apt-get install unzip -y\" on Linux." 1>&2
  exit 1
fi

case $(uname -sm) in
  "Darwin x86_64") target="x86_64-apple-darwin" ;;
  "Darwin arm64") target="aarch64-apple-darwin" ;;
  "Linux aarch64")
    echo "Error: Dune builds for Linux aarch64 are not available." 1>&2
    exit 1
  *) target="x86_64-unknown-linux-gnu" ;;
esac

dune_uri="https://github.com/aalykiot/dune/releases/latest/download/dune-${target}.zip"

dune_root="$DUNE_ROOT:-$HOME/.dune}"
dune_bin="$dune_root/bin"
dune_exe="$dune_bin/dune"

if [ ! -d "$dune_bin" ]; then
  mkdir -p "$dune_bin"
fi

curl --fail --location --progress-bar --output "$dune_exe.zip" "$dune_uri"
unzip -d "$dune_bin" -o "$dune_exe.zip"
chmod +x "$dune_exe"
rm "$dune_exe.zip"

echo "Dune was installed successfully to $dune_exe"

if command -v dune >/dev/null; then
  echo "Run 'dune --help' to get started"
else
  case $SHELL in
  /bin/zsh) shell_profile=".zshrc" ;;
  *) shell_profile=".bashrc" ;;
  esac
  echo "Manually add the directory to your \$HOME/$shell_profile (or similar)"
  echo "  export DUNE_ROOT=\"$dune_root\""
  echo "  export PATH=\"\$DUNE_ROOT/bin:\$PATH\""
  echo "Run '$dune_exe --help' to get started"
fi
