#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HEADER_PATH="${ROOT_DIR}/include/dee_config_gen_ffi.h"
CONFIG_PATH="${ROOT_DIR}/cbindgen.toml"
SOURCE_PATH="${ROOT_DIR}/src/ffi.rs"

if ! command -v cbindgen >/dev/null 2>&1; then
  echo "error: cbindgen is required but not found in PATH." >&2
  echo "hint: cargo install cbindgen --version 0.29.2" >&2
  exit 1
fi

tmp_header="$(mktemp)"
tmp_header_norm="$(mktemp)"
repo_header_norm="$(mktemp)"
trap 'rm -f "${tmp_header}" "${tmp_header_norm}" "${repo_header_norm}"' EXIT

cbindgen "${SOURCE_PATH}" --config "${CONFIG_PATH}" --output "${tmp_header}"

# Normalize CRLF/LF differences so Windows checkouts do not cause false positives.
tr -d '\r' <"${HEADER_PATH}" >"${repo_header_norm}"
tr -d '\r' <"${tmp_header}" >"${tmp_header_norm}"

if ! diff -u "${repo_header_norm}" "${tmp_header_norm}"; then
  cat <<'EOF' >&2
error: include/dee_config_gen_ffi.h is out of date.
regenerate with:
  cbindgen src/ffi.rs --config cbindgen.toml --output include/dee_config_gen_ffi.h
EOF
  exit 1
fi

echo "ffi header check passed."
