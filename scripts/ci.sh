#!/bin/sh
# Everything CI checks (`.woodpecker/ci.yaml` calls it): format, lints as errors, tests.
set -eu
cd "$(dirname "$0")/.."
rustup component add rustfmt clippy >/dev/null 2>&1
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
# Known vulnerabilities, licenses, sources (deny.toml).
command -v cargo-deny >/dev/null || { echo 'cargo-deny missing: cargo install cargo-deny --locked' >&2; exit 1; }
cargo deny check
cargo test --locked
