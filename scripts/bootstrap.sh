#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT"

for TOOL in git cargo rustc; do
  if ! command -v "$TOOL" >/dev/null 2>&1; then
    printf 'Required tool not found: %s\n' "$TOOL" >&2
    exit 1
  fi
done

if ! command -v rustfmt >/dev/null 2>&1; then
  if command -v rustup >/dev/null 2>&1; then
    rustup component add rustfmt
  else
    printf 'Required tool not found: rustfmt\n' >&2
    exit 1
  fi
fi

if ! command -v standardizer >/dev/null 2>&1; then
  cargo install standardizer --locked
fi

export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"

if ! command -v standardizer >/dev/null 2>&1; then
  printf 'Standardizer is not available in PATH.\n' >&2
  exit 1
fi

./scripts/generate-data.sh
./.githooks/install

printf 'Development environment ready.\n'
