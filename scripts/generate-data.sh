#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GENERATOR_VERSION="1.2.0"

cd "$ROOT"

cargo install standardizer-generate-data \
  --version "=$GENERATOR_VERSION" \
  --locked

CARGO_BIN="${CARGO_HOME:-$HOME/.cargo}/bin"
GENERATOR="$CARGO_BIN/standardizer-generate-data"

if [ -f "$GENERATOR.exe" ]; then
  GENERATOR="$GENERATOR.exe"
fi

exec "$GENERATOR" generate data data
