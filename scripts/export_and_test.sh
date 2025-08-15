#!/usr/bin/env bash
set -euo pipefail

# 0) Host tests (no assets)
( cd n64llm/n64-rust && cargo test --lib --features host --verbose )
( cd n64llm/n64-rust && cargo test --test host_sanity --features host --verbose )

# 1) Tiny debug weights (ephemeral)
python tools/make_debug_weights.py \
  --out-bin n64llm/n64-rust/assets/weights.bin \
  --out-man n64llm/n64-rust/assets/weights.manifest.bin

python tools/validate_weights.py \
  --bin n64llm/n64-rust/assets/weights.bin \
  --man n64llm/n64-rust/assets/weights.manifest.bin --crc

# 2) Build ROM (nightly) with build-std only here
( cd n64llm/n64-rust && cargo +nightly -Z build-std=core,alloc n64 build --profile release )

# 3) Optional emu smoke (never moves binaries; asks where to place)
./scripts/emu_smoke.sh || true

# 4) SCRUB every binary weight artifact (CI hard rule)
rm -f n64llm/n64-rust/assets/weights.bin n64llm/n64-rust/assets/weights.manifest.bin

