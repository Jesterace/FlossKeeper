#!/usr/bin/env bash
set -e

echo "Building FlossKeeper release..."
cargo build --release

echo
echo "Done."
echo "Release binary:"
echo "  target/release/flosskeeper"
echo
echo "Run it with:"
echo "  ./target/release/flosskeeper"
