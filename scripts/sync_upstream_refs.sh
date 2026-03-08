#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UPSTREAM_DIR="${UPSTREAM_DIR:-$ROOT_DIR/upstream}"

mkdir -p "$UPSTREAM_DIR"

clone_or_pull() {
  local repo_url="$1"
  local dir_name="$2"
  local target="$UPSTREAM_DIR/$dir_name"

  if [[ -d "$target/.git" ]]; then
    echo "Updating $dir_name..."
    git -C "$target" fetch --tags --prune
    git -C "$target" pull --ff-only
  else
    echo "Cloning $dir_name..."
    git clone "$repo_url" "$target"
  fi
}

clone_or_pull "https://github.com/pcroland/deew.git" "deew"
clone_or_pull "https://github.com/jessielw/DeeZy.git" "DeeZy"

"$ROOT_DIR/scripts/extract_upstream_matrix.sh"

echo "Done. Upstream mirrors are in: $UPSTREAM_DIR"
