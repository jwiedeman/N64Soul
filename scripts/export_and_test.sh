#!/usr/bin/env bash
set -euo pipefail

# 0) Host tests (no assets)
( cd n64llm/n64-rust && cargo test --lib --features host --verbose )
( cd n64llm/n64-rust && cargo test --test host_sanity --features host --verbose )

# 1) Export GPT weights
if [ "${SKIP_EXPORT:-0}" != "1" ]; then
  python tools/export_gpt2_n64.py \
    --model "${MODEL_ID:-gpt2}" \
    --dtype fp32 \
    --out-dir n64llm/n64-rust/assets
fi

python tools/validate_weights.py \
  --bin n64llm/n64-rust/assets/weights.bin \
  --man n64llm/n64-rust/assets/weights.manifest.bin --crc

# 2) Build ROM (nightly) with build-std only here
 ( cd n64llm/n64-rust && cargo +nightly -Z build-std=core,alloc n64 build --profile release --features embed_assets )

# 3) Optional emu smoke (never moves binaries; asks where to place)
./scripts/emu_smoke.sh || true

# Assets are preserved for ROM flashing.
