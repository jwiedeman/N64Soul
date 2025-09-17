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
      if command -v python >/dev/null 2>&1; then
        PYTHON_BIN=python
      elif command -v python3 >/dev/null 2>&1; then
        PYTHON_BIN=python3
      else
        echo "error: Python interpreter not found; required for dummy IPL3 generation." >&2
        exit 1
      fi

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
error: cargo-n64 now requires the CIC-6102 bootcode to rebuild the ROM.
Set N64SOUL_IPL3_BIN to a cic6102 dump, N64SOUL_IPL3_FROM_ROM to a ROM image,
or export N64SOUL_IPL3_DUMMY=1 to rebuild with a non-bootable placeholder.
EOF3
      exit 1
    fi
  fi

  (
    cd "$ROM_DIR" && \
    N64_SOUL_SKIP_EXPORT=1 cargo +"$TOOLCHAIN" -Z build-std=core,alloc n64 build "${IPL3_ARGS[@]}" -- --profile release --features embed_assets
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
