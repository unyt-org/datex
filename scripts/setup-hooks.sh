#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

chmod +x .githooks/pre-commit

git config core.hooksPath .githooks

echo "Git hooks successfully configured!"