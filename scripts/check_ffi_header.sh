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
trap 'rm -f "${tmp_header}"' EXIT

cbindgen "${SOURCE_PATH}" --config "${CONFIG_PATH}" --output "${tmp_header}"

if ! diff -u "${HEADER_PATH}" "${tmp_header}"; then
  cat <<'EOF' >&2
error: include/dee_config_gen_ffi.h is out of date.
regenerate with:
  cbindgen src/ffi.rs --config cbindgen.toml --output include/dee_config_gen_ffi.h
EOF
  exit 1
fi

echo "ffi header check passed."
