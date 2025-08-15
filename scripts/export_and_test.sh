#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.."; pwd)"
ASSETS="$ROOT/n64llm/n64-rust/assets"
BIN="$ASSETS/weights.bin"
MAN="$ASSETS/weights.manifest.bin"

# Never move user binaries. Only write ephemeral debug blobs here.
cleanup() {
  rm -f "$BIN" "$MAN" || true
  # also delete any ROM made here
  find "$ROOT/target" -maxdepth 3 -type f \( -name "*.z64" -o -name "*.n64" \) -delete || true
}
trap cleanup EXIT

mkdir -p "$ASSETS"

# 1) Export (v2 manifest by default). Either --spec or name=path pairs are accepted.
python3 "$ROOT/tools/export_model.py" \
  --out-bin "$BIN" --out-man "$MAN" --man-version 2 "$@"

# 2) Validate + CRC check
python3 "$ROOT/tools/validate_weights.py" --bin "$BIN" --man "$MAN" --crc

# 3) Build N64 ROM (no moving binaries)
rustup target add mips-nintendo64-none >/dev/null 2>&1 || true
cargo install cargo-n64 >/dev/null 2>&1 || true
( cd "$ROOT/n64llm/n64-rust" && cargo n64 build --release )

# 4) Try running headless smoke if the helper is present (optional)
if [[ -x "$ROOT/scripts/emu_smoke.sh" ]]; then
  "$ROOT/scripts/emu_smoke.sh" || true
fi

echo "All good. Ephemeral blobs will now be deleted."
