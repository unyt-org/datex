#!/usr/bin/env bash
set -euo pipefail
cargo clippy -p datex-core --fix --features full,allow_unsigned_blocks
cargo clippy --workspace --exclude datex-core --fix
cargo fmt --all
git commit -a -m "fmt"