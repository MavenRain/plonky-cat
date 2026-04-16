#!/usr/bin/env bash
set -euo pipefail

# Publish plonky-cat crates to crates.io in dependency order.
#
# Usage:
#   ./publish.sh              publish all tiers
#   ./publish.sh --dry-run    dry-run all tiers (no upload)

DRY_RUN=""
if [ "${1:-}" = "--dry-run" ]; then
  DRY_RUN="--dry-run"
  echo "=== DRY RUN MODE ==="
  echo
fi

publish_tier() {
  local tier_name="$1"
  shift
  echo "--- ${tier_name} ---"
  for crate in "$@"; do
    echo "  publishing ${crate}..."
    cargo publish -p "${crate}" ${DRY_RUN}
    if [ -z "${DRY_RUN}" ]; then
      echo "  waiting for crates.io index to update..."
      sleep 30
    fi
  done
  echo
}

publish_tier "Tier 1 (no internal deps)" \
  plonky-cat-field \
  plonky-cat-reduce

publish_tier "Tier 2 (depends on tier 1)" \
  plonky-cat-poly \
  plonky-cat-hash \
  plonky-cat-fft \
  plonky-cat-transcript

publish_tier "Tier 3 (depends on tiers 1-2)" \
  plonky-cat-merkle \
  plonky-cat-code \
  plonky-cat-sumcheck \
  plonky-cat-plonk \
  plonky-cat-prover \
  plonky-cat-verifier

publish_tier "Tier 4 (depends on tiers 1-3)" \
  plonky-cat-fri \
  plonky-cat-tensor-pcs

publish_tier "Tier 5 (depends on tiers 1-4)" \
  plonky-cat-basefold

echo "=== done ==="
