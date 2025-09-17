#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROM_DIR="$ROOT_DIR/n64llm/n64-rust"
ASSETS="$ROM_DIR/assets"
ROM_GLOB="$ROM_DIR/target"/n64/release/*.z64
TOOLCHAIN="${N64SOUL_TOOLCHAIN:-nightly-2022-06-21}"

cd "$ROOT_DIR"

[[ -f "$ASSETS/weights.bin" ]] || { echo "weights.bin missing under $ASSETS"; exit 1; }
[[ -f "$ASSETS/weights.manifest.bin" ]] || { echo "weights.manifest.bin missing under $ASSETS"; exit 1; }

ROM_PATH=$(ls -1 $ROM_GLOB 2>/dev/null | head -n1 || true)
if [[ -z "$ROM_PATH" ]]; then
  echo "No ROM found; rebuilding with existing assets."
  (
    cd "$ROM_DIR" && \
    N64_SOUL_SKIP_EXPORT=1 cargo +"$TOOLCHAIN" -Z build-std=core,alloc n64 build --profile release --features embed_assets
  )
  ROM_PATH=$(ls -1 $ROM_GLOB 2>/dev/null | head -n1 || true)
fi

if [[ -z "$ROM_PATH" ]]; then
  echo "Unable to locate the built ROM under $ROM_DIR/target."
  exit 1
fi

echo "ROM: $ROM_PATH"

if command -v ares >/dev/null 2>&1; then
  ares "$ROM_PATH"
elif command -v mupen64plus >/dev/null 2>&1; then
  mupen64plus "$ROM_PATH"
else
  echo "No emulator found in PATH; launch your preferred emulator manually."
fi
