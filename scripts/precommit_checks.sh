#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

echo "[1/4] cargo fmt --all --check"
cargo fmt --all --check

echo "[2/4] cargo test -q"
cargo test -q

echo "[3/4] cargo clippy --all-targets --all-features -- -D warnings"
cargo clippy --all-targets --all-features -- -D warnings

echo "[4/4] cargo clippy selected pedantic"
cargo clippy --all-targets --all-features -- \
  -D warnings \
  -W clippy::manual_let_else \
  -W clippy::match_wildcard_for_single_variants \
  -W clippy::redundant_else \
  -W clippy::similar_names \
  -W clippy::uninlined_format_args
