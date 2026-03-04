#!/usr/bin/env bash
set -euo pipefail
cargo clippy -p datex-core --fix --features decompiler,compiler,allow_unsigned_blocks
cargo clippy --exclude datex-core --fix
cargo fmt --all
