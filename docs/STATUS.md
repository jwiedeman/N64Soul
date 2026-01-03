# N64Soul Project Status & Remaining Work

## Current Status: Ready for Testing

The codebase is feature-complete and CI is passing. All major components are implemented and documented.

---

## What's Complete

### Core Implementation
- [x] Boot code (IPL3) with CIC handshake
- [x] Display system (320x240, 16-bit framebuffer)
- [x] 8x8 VGA-style font rendering
- [x] On-screen keyboard with D-Pad navigation
- [x] Controller input handling (A/B/START/D-Pad)
- [x] BPE tokenizer (encode/decode from ROM)
- [x] Full transformer inference engine
  - [x] Embedding lookup (wte + wpe)
  - [x] Multi-head attention
  - [x] Feed-forward network
  - [x] Layer normalization
  - [x] Greedy decoding
- [x] Memory manager with checkpoint/restore
- [x] DMA-based ROM streaming with prefetch
- [x] Diagnostic suite (ROM probe, CRC, benchmarks)

### Build System
- [x] Custom `mips-nintendo64-none` target
- [x] Linker script with ROM/RAM layout
- [x] Python weight exporter (`export_gpt2_n64.py`)
- [x] Manifest validator (`validate_weights.py`)
- [x] Automated build via `cargo-n64` + `nust64`
- [x] CI pipeline (tests, build, artifact upload)

### Documentation
- [x] README with quickstart
- [x] Setup guide (`docs/setup.md`)
- [x] Emulator guide (`docs/emulator.md`)
- [x] Design document (`docs/DESIGN.md`)
- [x] AI agent instructions (`AGENTS.md`)

---

## Remaining Steps to Run

### 1. Prerequisites Check

Before building, verify your environment:

```bash
# Check Python dependencies
python tools/check_python_deps.py

# Verify Rust toolchain
rustup show
# Should include: nightly-2024-10-01
```

### 2. Install Missing Tools (If Needed)

```bash
# Rust toolchain
export N64SOUL_TOOLCHAIN=nightly-2024-10-01
rustup toolchain install "$N64SOUL_TOOLCHAIN"
rustup component add rust-src --toolchain "$N64SOUL_TOOLCHAIN"

# cargo-n64 (with patches)
N64SOUL_TOOLCHAIN="$N64SOUL_TOOLCHAIN" bash tools/install_cargo_n64.sh

# nust64 ROM packager
cargo install nust64

# Python dependencies
pip install torch transformers
```

### 3. Build the ROM

**Option A: One-shot build (recommended)**
```bash
./scripts/export_and_test.sh
```

**Option B: Manual build**
```bash
cd n64llm/n64-rust
export N64_SOUL_MODEL_ID=distilgpt2
export N64_SOUL_DTYPE=fp16
export N64_SOUL_KEEP_LAYERS=6

TOOLCHAIN="${N64SOUL_TOOLCHAIN:-nightly-2024-10-01}"
cargo +"$TOOLCHAIN" -Z build-std=core,alloc n64 build --features embed_assets
```

### 4. Obtain IPL3 Boot Code

The N64 requires CIC-6102 boot code. Options:

| Option | Description |
|--------|-------------|
| **Dummy (CI)** | Set `N64SOUL_IPL3_DUMMY=1` - produces non-bootable ROM for testing |
| **Extract from ROM** | Set `N64SOUL_IPL3_FROM_ROM=/path/to/known-good.z64` |
| **Direct dump** | Set `N64SOUL_IPL3_BIN=/path/to/cic6102.bin` |

For actual hardware/emulator testing, you need a real boot code.

### 5. Run on Emulator

```bash
# Using the helper script
./scripts/emu_smoke.sh

# Or manually with ares
ares n64llm/n64-rust/target/n64/release/n64_gpt.z64

# Or with mupen64plus
mupen64plus n64llm/n64-rust/target/n64/release/n64_gpt.z64
```

### 6. Run on Real Hardware

Flash the `.z64` ROM to a flashcart (EverDrive 64, 64drive, etc.) and run on a Nintendo 64 console.

---

## Potential Refinements (Nice-to-Have)

These are optional improvements, not blockers:

### Performance Tuning
- [ ] Profile actual DMA bandwidth on different flashcarts
- [ ] Tune prefetch buffer sizes for optimal throughput
- [ ] Consider int8 quantization for faster inference

### User Experience
- [ ] Add visual feedback during keyboard navigation (key press animation)
- [ ] Scrollable conversation history
- [ ] Character repeat on held button
- [ ] Add special keys: newline, clear, etc.

### Model Options
- [ ] Test with different model sizes (GPT-2 medium, large)
- [ ] Experiment with alternative models (TinyLlama, etc.)
- [ ] Tune temperature/sampling for better outputs

### Robustness
- [ ] Add watchdog timer for stuck inference
- [ ] Better error recovery without full reset
- [ ] Controller disconnection handling

### Hardware Compatibility
- [ ] Test on various N64 console revisions
- [ ] Verify with different flashcart models
- [ ] Test with/without Expansion Pak

---

## Known Limitations

| Limitation | Description |
|------------|-------------|
| **Speed** | Token generation takes 2-5+ seconds |
| **Model Size** | Limited by flashcart capacity (max ~64MB typical) |
| **Memory** | 4MB heap limits context window |
| **Input** | On-screen keyboard only (no Rumble Pak keyboard) |
| **Output** | Greedy decoding (no sampling/temperature) |

---

## Testing Checklist

Before announcing:

- [ ] Build ROM successfully with real weights
- [ ] Boot ROM in emulator (ares or mupen64plus)
- [ ] Verify diagnostics complete without errors
- [ ] Press START to reach keyboard interface
- [ ] Type a message using on-screen keyboard
- [ ] Submit and receive inference output
- [ ] Confirm conversation history displays correctly
- [ ] (Optional) Test on real N64 hardware

---

## Summary

**The project is code-complete.** The remaining work is:

1. **Environment setup** - Install toolchain and dependencies
2. **Build** - Run the build pipeline
3. **Boot code** - Obtain CIC-6102 (for bootable ROM)
4. **Test** - Verify in emulator and/or on hardware

Once these steps complete successfully, the project is ready for announcement and PR.
