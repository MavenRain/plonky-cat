#!/usr/bin/env bash
set -euo pipefail

# Publish plonky-cat crates to crates.io in dependency order.
# Skips already-published crates.  Retries on rate limit (429).
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

publish_crate() {
  local crate="$1"
  local output
  local exit_code

  output=$(cargo publish -p "${crate}" ${DRY_RUN} 2>&1) && exit_code=0 || exit_code=$?

  if [ ${exit_code} -eq 0 ]; then
    echo "  ${crate}: published"
    if [ -z "${DRY_RUN}" ]; then
      echo "  waiting 30s for index..."
      sleep 30
    fi
  elif echo "${output}" | grep -q "already exists"; then
    echo "  ${crate}: already published, skipping"
  elif echo "${output}" | grep -q "429"; then
    local retry_after
    retry_after=$(echo "${output}" | grep -oE "after [A-Za-z]+, [0-9]+ [A-Za-z]+ [0-9]+ [0-9:]+")
    echo "  ${crate}: rate limited (${retry_after:-unknown})"
    echo "  waiting 120s before retry..."
    sleep 120
    publish_crate "${crate}"
  else
    echo "  ${crate}: FAILED"
    echo "${output}"
    exit 1
  fi
}

publish_tier() {
  local tier_name="$1"
  shift
  echo "--- ${tier_name} ---"
  for crate in "$@"; do
    publish_crate "${crate}"
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
