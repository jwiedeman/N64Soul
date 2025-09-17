#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROM_DIR="$ROOT_DIR/n64llm/n64-rust"
ASSETS_DIR="$ROM_DIR/assets"
TOOLCHAIN="${N64SOUL_TOOLCHAIN:-nightly-2024-10-01}"

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

# Determine the IPL3 argument expected by cargo-n64.
IPL3_ARGS=()
AUTO_DUMMY=0
if [[ -n "${N64SOUL_IPL3_BIN:-}" && -n "${N64SOUL_IPL3_FROM_ROM:-}" ]]; then
  echo "error: set only one of N64SOUL_IPL3_BIN or N64SOUL_IPL3_FROM_ROM" >&2
  exit 1
elif [[ -n "${N64SOUL_IPL3_BIN:-}" ]]; then
  if [[ ! -f "${N64SOUL_IPL3_BIN}" ]]; then
    echo "error: N64SOUL_IPL3_BIN does not exist: ${N64SOUL_IPL3_BIN}" >&2
    exit 1
  fi
  IPL3_ARGS=(--ipl3 "${N64SOUL_IPL3_BIN}")
elif [[ -n "${N64SOUL_IPL3_FROM_ROM:-}" ]]; then
  if [[ ! -f "${N64SOUL_IPL3_FROM_ROM}" ]]; then
    echo "error: N64SOUL_IPL3_FROM_ROM does not exist: ${N64SOUL_IPL3_FROM_ROM}" >&2
    exit 1
  fi
  IPL3_ARGS=(--ipl3-from-rom "${N64SOUL_IPL3_FROM_ROM}")
else
  USE_DUMMY=0
  if [[ -z "${N64SOUL_IPL3_DUMMY+x}" ]]; then
    USE_DUMMY=1
    AUTO_DUMMY=1
  elif [[ "${N64SOUL_IPL3_DUMMY:-0}" == "1" ]]; then
    USE_DUMMY=1
  fi

  if [[ "$USE_DUMMY" == "1" ]]; then
    DUMMY_IPL3="$ROM_DIR/ipl3_dummy.bin"
    if [[ ! -f "$DUMMY_IPL3" ]]; then
      DUMMY_IPL3_PATH="$DUMMY_IPL3" "$PYTHON_BIN" - <<'PY'
import os
path = os.environ["DUMMY_IPL3_PATH"]
os.makedirs(os.path.dirname(path), exist_ok=True)
with open(path, "wb") as f:
    f.write(b"\x00" * 4032)
PY
    fi
    if [[ "$AUTO_DUMMY" == "1" ]]; then
      cat <<'EOF2' >&2
warning: no CIC-6102 bootcode provided; using a zeroed placeholder ROM header.
Set N64SOUL_IPL3_BIN or N64SOUL_IPL3_FROM_ROM for a bootable image.
EOF2
    fi
    IPL3_ARGS=(--ipl3 "$DUMMY_IPL3")
  else
    cat <<'EOF3' >&2
error: cargo-n64 requires the CIC-6102 bootcode.
Set N64SOUL_IPL3_BIN to a cic6102 dump, N64SOUL_IPL3_FROM_ROM to a ROM image,
or export N64SOUL_IPL3_DUMMY=1 to build with a non-bootable placeholder.
EOF3
    exit 1
  fi
fi

# Build the ROM. The build script exports and validates fresh weights.
(
  cd "$ROM_DIR" && \
  cargo +"$TOOLCHAIN" -Z build-std=core,alloc n64 build "${IPL3_ARGS[@]}" -- --features embed_assets
)

# Confirm the assets exist for downstream tooling.
"$PYTHON_BIN" tools/validate_weights.py \
  --bin "$ASSETS_DIR/weights.bin" \
  --man "$ASSETS_DIR/weights.manifest.bin" --crc

# Optional emulator smoke test.
"$ROOT_DIR"/scripts/emu_smoke.sh || true

# Assets are preserved for ROM flashing.
