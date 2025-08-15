#!/usr/bin/env bash
set -euo pipefail

# 0) Sanity: confirm assets exist where the code expects them.
ASSETS="n64llm/n64-rust/assets"
[[ -f "$ASSETS/weights.bin" ]] || { echo "Joshua: please move your weights to $ASSETS/weights.bin"; exit 1; }
[[ -f "$ASSETS/weights.manifest.bin" ]] || { echo "Joshua: please move your manifest to $ASSETS/weights.manifest.bin"; exit 1; }

# 1) Build (no moving binaries).
PACK_ROM=1 cargo build --release

# 2) Find the produced ROM and run emulator if available.
ROM="$(ls -1 target/**/release/*.z64 2>/dev/null | head -n1 || true)"
if [[ -z "${ROM}" ]]; then
  echo "No .z64 found. Joshua: please tell me where your packed ROM is written."
  exit 1
fi
echo "ROM: ${ROM}"

# Try ares if on PATH; otherwise just print path.
if command -v ares >/dev/null 2>&1; then
  ares "${ROM}"
else
  echo "ares not found. Joshua: launch your emulator manually with ${ROM}"
fi
