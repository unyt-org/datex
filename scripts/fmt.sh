#!/usr/bin/env bash
set -euo pipefail

# cargo fix
cargo fix \
  -p datex-core \
  --features "full,allow_unsigned_blocks" \
  --allow-dirty \
  --allow-staged

# clippy fix
cargo clippy \
  -p datex-core \
  --features "full,allow_unsigned_blocks" \
  --fix \
  --allow-dirty \
  --allow-staged

cargo clippy \
  --workspace \
  --exclude datex-core \
  --fix \
  --allow-dirty

# cargo fmt
cargo fmt --all

# commit changes
git commit -a -m "fmt"