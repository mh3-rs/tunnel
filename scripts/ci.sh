#!/bin/sh
# Everything CI checks (`.woodpecker/ci.yaml` calls it): format, lints as errors, tests.
set -eu
cd "$(dirname "$0")/.."
rustup component add rustfmt clippy >/dev/null 2>&1
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
