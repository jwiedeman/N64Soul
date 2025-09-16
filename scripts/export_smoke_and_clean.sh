#!/usr/bin/env bash
set -euo pipefail

ASSETS="n64llm/n64-rust/assets"
BIN="$ASSETS/weights.bin"
MAN="$ASSETS/weights.manifest.bin"

# 0) Export real or debug
MODE="${1:-debug}"   # "debug" or "real"
if [[ "$MODE" == "real" ]]; then
  # Example: keep last 8 blocks of gpt2-medium as fp16 (tweak as needed)
  python tools/export_gpt2_n64.py --model gpt2-medium --dtype fp16 --keep-layers 8 --out-dir "$ASSETS"
else
  python tools/make_debug_weights.py --out-bin "$BIN" --out-man "$MAN"
fi

# 1) Validate
python tools/validate_weights.py --bin "$BIN" --man "$MAN"

# 2) Optional: build + emu smoke
if [[ "${RUN_SMOKE:-1}" == "1" ]]; then
  scripts/emu_smoke.sh || true
fi

# 3) ALWAYS CLEAN (avoid binaries in PRs)
rm -f "$BIN" "$MAN"
echo "[CLEAN] removed $BIN and $MAN"
