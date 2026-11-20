#!/usr/bin/env bash
set -euo pipefail

script_dir=$(
    cd -P "$(dirname "$0")" && pwd -P
)

cd "$script_dir/.."

if ! command -v cargo >/dev/null 2>&1; then
    echo "Erreur : Cargo est introuvable." >&2
    exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
    echo "Erreur : Python 3 est requis par le lanceur." >&2
    exit 1
fi

echo "Compilation du préparateur..."

preparer=$(
    cargo build \
        --locked \
        -p prepare-test-repository \
        --message-format=json |
    python3 -c '
import json
import sys

executables = []

for line in sys.stdin:
    message = json.loads(line)

    if message.get("reason") != "compiler-artifact":
        continue

    target = message.get("target", {})

    if target.get("name") != "prepare-test-repository":
        continue

    if "bin" not in target.get("kind", []):
        continue

    executable = message.get("executable")

    if executable:
        executables.append(executable)

if len(executables) != 1:
    sys.exit("Erreur : binaire préparateur introuvable ou ambigu.")

print(executables[0])
'
)

if [ ! -f "$preparer" ] || [ ! -x "$preparer" ]; then
    echo "Erreur : préparateur non exécutable : $preparer" >&2
    exit 1
fi

echo "Préparateur : $preparer"
echo "Exécution des tests..."

STANDARDIZER_TEST_PREPARER="$preparer" \
    cargo test --locked -p standardizer --tests
