#!/bin/bash

# Check if zip is installed.
if ! command -v zip >/dev/null; then
  echo "Error: zip is required to build Dune." 1>&2
  echo "=>> You can install zip via \"brew install zip\" on MacOS or \"apt-get install zip -y\" on Linux." 1>&2
  exit 1
fi

# Set the output directory to cwd if no argument is provided.
output=${1:-$(pwd)}

# Build binary for aarch64-apple-darwin.
cargo build --release --target=aarch64-apple-darwin

# Create release zip bundle.
mv ./target/aarch64-apple-darwin/release/dune .

zip -r dune-aarch64-apple-darwin.zip dune README.md LICENSE.md

rm dune

# Move zip to specified output destination.
if [[ $output != $PWD ]]; then
  mv dune-aarch64-apple-darwin.zip $output
fi
