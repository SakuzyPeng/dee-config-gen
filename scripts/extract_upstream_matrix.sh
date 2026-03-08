#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UPSTREAM_DIR="${UPSTREAM_DIR:-$ROOT_DIR/upstream}"
OUT_FILE="${OUT_FILE:-$ROOT_DIR/docs/upstream_parameter_observations.md}"

DEEW_DIR="$UPSTREAM_DIR/deew"
DEEZY_DIR="$UPSTREAM_DIR/DeeZy"

if [[ ! -d "$DEEW_DIR" || ! -d "$DEEZY_DIR" ]]; then
  echo "Missing upstream repos under $UPSTREAM_DIR. Run scripts/sync_upstream_refs.sh first." >&2
  exit 1
fi

mkdir -p "$(dirname "$OUT_FILE")"

strip_root() {
  sed "s#${ROOT_DIR}/##g"
}

{
  echo "# Upstream Parameter Observations"
  echo
  echo "Generated: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  echo
  echo "## deew (MIT)"
  echo
  echo "### Bluray/bitrate hints"
  echo '```text'
  rg -n "ddp_71_bluray|ddp_71_combined|force-bluray|encoder_mode|custom_dialnorm" "$DEEW_DIR/deew/bitrates.py" "$DEEW_DIR/deew/__main__.py" | strip_root || true
  echo '```'
  echo
  echo "## DeeZy (MIT)"
  echo
  echo "### Atmos bluray mode + extended bitrates"
  echo '```text'
  rg -n "AtmosMode|choices=\(|encoding_backend|encoder_mode|bluray" \
    "$DEEZY_DIR/deezy/enums/atmos.py" \
    "$DEEZY_DIR/deezy/audio_encoders/dee/json/dee_json_generator.py" \
    "$DEEZY_DIR/example_json_flows/atmos-ec3-bluray.json" | strip_root || true
  echo '```'
  echo
  echo "## Notes"
  echo
  echo "- This report is a metadata-only observation snapshot."
  echo "- Do not copy upstream implementation logic verbatim into this project."
  echo "- Runtime validation source of truth remains src/registry.rs + docs/parameter_matrix.atmos_ec3_v1.yaml."
} > "$OUT_FILE"

echo "Wrote $OUT_FILE"
