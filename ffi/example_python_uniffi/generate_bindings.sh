#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
OUT_DIR="${SCRIPT_DIR}/generated"
UDL_PATH="${REPO_ROOT}/ffi/uniffi_bridge/src/dcg_uniffi.udl"

if ! command -v uniffi-bindgen >/dev/null 2>&1; then
  echo "missing uniffi-bindgen; install with:" >&2
  echo "  cargo install --locked uniffi --version 0.31.0 --features cli" >&2
  exit 1
fi

cd "${REPO_ROOT}"
cargo build -p dee-config-gen-uniffi-bridge --release

case "$(uname -s)" in
  Darwin)
    LIB_PATH="${REPO_ROOT}/target/release/libdcg_uniffi_bridge.dylib"
    BINDING_LIB_NAME="libuniffi.dylib"
    ;;
  Linux)
    LIB_PATH="${REPO_ROOT}/target/release/libdcg_uniffi_bridge.so"
    BINDING_LIB_NAME="libuniffi.so"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    LIB_PATH="${REPO_ROOT}/target/release/dcg_uniffi_bridge.dll"
    BINDING_LIB_NAME="uniffi.dll"
    ;;
  *)
    echo "unsupported platform for demo binding generation: $(uname -s)" >&2
    exit 1
    ;;
esac

if [[ ! -f "${LIB_PATH}" ]]; then
  echo "missing bridge library: ${LIB_PATH}" >&2
  exit 1
fi

rm -rf "${OUT_DIR}"
mkdir -p "${OUT_DIR}"

if uniffi-bindgen generate "${UDL_PATH}" --library "${LIB_PATH}" --language python --out-dir "${OUT_DIR}" >/dev/null 2>&1; then
  :
else
  uniffi-bindgen generate "${UDL_PATH}" --language python --out-dir "${OUT_DIR}"
fi

cp "${LIB_PATH}" "${OUT_DIR}/${BINDING_LIB_NAME}"
echo "generated python bindings under: ${OUT_DIR}"
