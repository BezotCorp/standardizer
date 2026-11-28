#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PREPARER_VERSION="1.0.0"

cd "$ROOT"

cargo install standardizer-prepare-test-repository \
  --version "=$PREPARER_VERSION" \
  --locked

CARGO_BIN="${CARGO_HOME:-$HOME/.cargo}/bin"
PREPARER="$CARGO_BIN/standardizer-prepare-test-repository"

if [ -f "$PREPARER.exe" ]; then
  PREPARER="$PREPARER.exe"
fi

if [ ! -x "$PREPARER" ]; then
  printf 'Test preparer is not executable: %s\n' "$PREPARER" >&2
  exit 1
fi

if [ "${1:-}" = "--prepare" ]; then
  exit 0
fi

export STANDARDIZER_TEST_PREPARER="$PREPARER"

exec cargo test --locked --all-features "$@"
