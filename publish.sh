#!/usr/bin/env bash
set -euo pipefail

TOKEN="$(gopass show -o crates)"

# Clean stale package caches
rm -rf target/package/

CRATES=(
  anvilkit-core
  anvilkit-input
  anvilkit-ecs
  anvilkit-assets
  anvilkit-render
  anvilkit-audio
  anvilkit-camera
  anvilkit
)

for crate in "${CRATES[@]}"; do
  echo "=== Publishing $crate ==="
  if cargo publish -p "$crate" --allow-dirty --token "$TOKEN" 2>&1; then
    echo "--- $crate published, waiting 30s for index update ---"
    sleep 30
  else
    echo "--- $crate failed or already exists, continuing ---"
  fi
done

echo "All crates published!"
