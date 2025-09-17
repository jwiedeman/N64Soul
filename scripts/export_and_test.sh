#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROM_DIR="$ROOT_DIR/n64llm/n64-rust"
ASSETS_DIR="$ROM_DIR/assets"

cd "$ROOT_DIR"

# Surface host tests before touching any heavyweight export logic.
( cd "$ROM_DIR" && cargo test --lib --features host --verbose )
( cd "$ROM_DIR" && cargo test --test host_sanity --features host --verbose )

# Resolve a Python interpreter up front so we can run the export helpers on
# systems where the executable is named `python3` (as on modern macOS).
if command -v python >/dev/null 2>&1; then
  PYTHON_BIN=python
elif command -v python3 >/dev/null 2>&1; then
  PYTHON_BIN=python3
else
  echo "error: Python interpreter not found. Install python3 and ensure it is on your PATH." >&2
  exit 1
fi

# Verify Python dependencies needed for export if requested.
if [ "${SKIP_PY_DEPS:-0}" != "1" ]; then
  "$PYTHON_BIN" tools/check_python_deps.py
fi

# Propagate build configuration to the exporter run by build.rs.
export N64_SOUL_MODEL_ID="${MODEL_ID:-gpt2}"
export N64_SOUL_DTYPE="${MODEL_DTYPE:-fp16}"
if [ -n "${KEEP_LAYERS:-}" ]; then
  export N64_SOUL_KEEP_LAYERS="$KEEP_LAYERS"
else
  unset N64_SOUL_KEEP_LAYERS 2>/dev/null || true
fi
if [ -n "${TUNE_CONFIG:-}" ]; then
  export N64_SOUL_TUNE_CONFIG="$TUNE_CONFIG"
else
  unset N64_SOUL_TUNE_CONFIG 2>/dev/null || true
fi

# Build the ROM. The build script exports and validates fresh weights.
(
  cd "$ROM_DIR" && \
  cargo +nightly -Z build-std=core,alloc n64 build --release --features embed_assets
)

# Confirm the assets exist for downstream tooling.
"$PYTHON_BIN" tools/validate_weights.py \
  --bin "$ASSETS_DIR/weights.bin" \
  --man "$ASSETS_DIR/weights.manifest.bin" --crc

# Optional emulator smoke test.
"$ROOT_DIR"/scripts/emu_smoke.sh || true

# Assets are preserved for ROM flashing.
