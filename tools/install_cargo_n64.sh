#!/usr/bin/env bash
set -euo pipefail

# Always prefer a pinned Git commit known to compile on our runner.
# If upstream breaks, add another rev below and try again.
REVS=(
  "main"            # try latest main first
  # "v0.3.0"        # uncomment if a future tag is stable for us
)

for rev in "${REVS[@]}"; do
  if cargo +nightly install cargo-n64 \
      --git https://github.com/rust-console/cargo-n64 \
      --branch "$rev" --locked; then
    echo "cargo-n64 installed from $rev"
    exit 0
  fi
done

echo "ERROR: cargo-n64 could not be installed from any pinned revs." >&2
exit 1
