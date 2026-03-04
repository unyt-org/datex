#!/usr/bin/env bash
set -euo pipefail

cargo test-nostd
cargo test-std