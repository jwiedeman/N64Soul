# N64Soul Design Document

## Overview

N64Soul is an experimental project that runs a GPT-2 language model on Nintendo 64 hardware. The system streams transformer weights directly from the cartridge ROM, performs inference on-device, and provides an interactive on-screen keyboard UI for user input.

---

## System Architecture

```
+----------------------------------------------------------+
|                    Nintendo 64 Hardware                   |
+----------------------------------------------------------+
|                                                          |
|  +----------------+     +---------------------------+    |
|  |   RDRAM (8MB)  |     |   Cartridge ROM (up to    |    |
|  |  - Heap: 4MB   |<--->|   1GB)                    |    |
|  |  - Stack: 64KB |     |   - Code + Data           |    |
|  |  - Framebuffer |     |   - Weights (~100MB+)     |    |
|  +----------------+     +---------------------------+    |
|          |                         |                     |
|          v                         v                     |
|  +----------------+     +---------------------------+    |
|  |  Video Output  |     |  PI DMA (Peripheral       |    |
|  |  (320x240      |     |  Interface)               |    |
|  |   16-bit)      |     |  - 32KB burst transfers   |    |
|  +----------------+     |  - Double-buffered        |    |
|                         +---------------------------+    |
|  +----------------+                                      |
|  |  Controller    |                                      |
|  |  (PIF/SI)      |                                      |
|  +----------------+                                      |
+----------------------------------------------------------+
```

### Key Components

| Component | File | Purpose |
|-----------|------|---------|
| **Boot** | `ipl3.rs` | N64 boot entry, CIC handshake, memory init |
| **Display** | `display.rs` | 16-bit framebuffer, 8x8 font, on-screen keyboard |
| **Input** | `n64_sys.rs` | Controller reading via PIF/SI |
| **Tokenizer** | `tokenizer.rs` | BPE encode/decode from ROM |
| **Inference** | `inference_engine.rs` | Transformer layers, attention, FFN |
| **Memory** | `memory_manager.rs` | Bump allocator with checkpoint/restore |
| **ROM I/O** | `platform/pi.rs` | DMA-based weight streaming |
| **Diagnostics** | `diag/*.rs` | Boot-time ROM probing, CRC, benchmarks |

---

## Memory Layout

### RDRAM (8MB with Expansion Pak)

```
Address         Size        Purpose
-----------------------------------------------
0x80000000      1KB         OS/Boot reserved
0x80000400      1MB         Boot code (.text)
0x80100400      4MB         Heap (bump allocator)
0x80500400      64KB        Stack (grows down)
0x80510400      ~2.5MB      BSS + Data
0xA0400000      ~150KB      Framebuffer (320x240x2)
```

### ROM Layout

```
Offset          Content
-----------------------------------------------
0x00000000      N64 Header (64 bytes)
0x00000040      IPL3 boot code (4032 bytes)
0x00001000      Application code (.text, .rodata)
0x00??????      Manifest (64-byte aligned, N64W v2)
0x00??????      Weights binary (64-byte aligned)
...             (continues to end of ROM)
```

---

## Screen Flow

### Boot Sequence

```
+------------------------------------------+
|  N64 GPT - Flash-Streamed AI Model       |
|  Initializing...                         |
|                                          |
|  Probing ROM...                          |
|  0x10000000  OK   XX XX XX XX XX XX XX XX|
|  0x11000000  OK   XX XX XX XX XX XX XX XX|
|  0x12000000  OK   XX XX XX XX XX XX XX XX|
|  ...                                     |
|                                          |
|  Weights: 123456789 bytes, 156 entries   |
|                                          |
|  Manifest check: OK                      |
|  Stream CRC: OK (1.2s, 98.4 MB/s)        |
|  Bench: 12.3 MB/s DMA, 45.6ms/layer      |
|  Decode test: OK                         |
|                                          |
|  Press START to continue...              |
+------------------------------------------+
```

### Main Interface

```
+------------------------------------------+
|  Manifest layers: 12                     |
|  Model dims: d_model=768 vocab=50257     |
|  Memory manager initialized              |
|                                          |
|  Use the on-screen keyboard.             |
|  Start to submit.                        |
|                                          |
|  Input: HELLO_                           |
|                                          |
|   A   B   C   D   E   F   G   H   I   J  |
|   K   L   M   N  [O]  P   Q   R   S   T  |
|   U   V   W   X   Y   Z   0   1   2   3  |
|   4   5   6   7   8   9   .   ,   ?      |
|                                          |
+------------------------------------------+
```
*Note: `[O]` shows the currently selected character*

### Inference Progress

```
+------------------------------------------+
|  [You] HELLO                             |
|  Working...                              |
|  Working... [########------------] 8/20  |
|                                          |
+------------------------------------------+
```

### Conversation View

```
+------------------------------------------+
|  [You] HELLO                             |
|  [AI] Hello! How can I help you today?   |
|                                          |
|  [You] WHAT IS 2 PLUS 2                  |
|  [AI] 2 plus 2 equals 4.                 |
|                                          |
|  Input: _                                |
|                                          |
|   A   B   C   D   E   F   G   H   I   J  |
|  [K]  L   M   N   O   P   Q   R   S   T  |
|   U   V   W   X   Y   Z   0   1   2   3  |
|   4   5   6   7   8   9   .   ,   ?      |
+------------------------------------------+
```

---

## Controller Input

### Button Mapping

| Button | Function |
|--------|----------|
| **D-Pad Up** | Move keyboard cursor up |
| **D-Pad Down** | Move keyboard cursor down |
| **D-Pad Left** | Move keyboard cursor left |
| **D-Pad Right** | Move keyboard cursor right |
| **A** | Select character (add to input buffer) |
| **B** | Backspace (delete last character) |
| **START** | Submit input for inference |

### On-Screen Keyboard Layout

```
Row 0:  A  B  C  D  E  F  G  H  I  J
Row 1:  K  L  M  N  O  P  Q  R  S  T
Row 2:  U  V  W  X  Y  Z  0  1  2  3
Row 3:  4  5  6  7  8  9  .  ,  ?  [SPACE]
```

### Input Flow

```
                    +----------------+
                    |  D-Pad Move    |
                    |  (navigate)    |
                    +-------+--------+
                            |
                            v
+---------------+   +----------------+   +---------------+
|  A Button     |-->| Selected Char  |-->| Input Buffer  |
|  (select)     |   | Highlighted    |   | Updated       |
+---------------+   +----------------+   +---------------+
                            |
                            v
                    +----------------+
                    |  B Button      |
                    |  (backspace)   |
                    +----------------+
                            |
                            v
                    +----------------+
                    |  START Button  |
                    |  (submit)      |
                    +-------+--------+
                            |
                            v
                    +----------------+
                    |  Tokenize &    |
                    |  Run Inference |
                    +----------------+
```

---

## Inference Pipeline

### Token Flow

```
User Input (String)
        |
        v
+------------------+
| BPE Tokenization |  (tokenizer.rs)
| - Load vocab     |
| - Merge rules    |
| - Encode to IDs  |
+--------+---------+
         |
         v
   [Token IDs]
         |
         v
+------------------+
| Embedding Lookup |  (inference_engine.rs)
| - wte (token)    |
| - wpe (position) |
+--------+---------+
         |
         v
+------------------+
| Transformer      |  (12 layers for GPT-2 small)
| - Layer Norm     |
| - Multi-Head     |
|   Attention      |
| - Feed Forward   |
+--------+---------+
         |
    [Repeat for
     each layer]
         |
         v
+------------------+
| Final Layer Norm |
| + LM Head        |
| (vocab logits)   |
+--------+---------+
         |
         v
+------------------+
| Greedy Decode    |
| (argmax token)   |
+--------+---------+
         |
         v
+------------------+
| BPE Decode       |
| IDs -> String    |
+--------+---------+
         |
         v
   Output Text
```

### Memory Checkpoint System

```
+------------------+     +------------------+
|  Allocate Layer  |     |  Process Layer   |
|  Buffers         |---->|  (Attention +    |
|  (checkpoint)    |     |   FFN)           |
+--------+---------+     +--------+---------+
         ^                        |
         |                        v
         |               +------------------+
         +---------------|  Restore         |
                         |  Checkpoint      |
                         |  (free buffers)  |
                         +------------------+
```

This allows processing large models layer-by-layer without exhausting the 4MB heap.

---

## Weight Streaming

### DMA Transfer Pattern

```
ROM Cartridge                    RDRAM
+------------+                   +------------+
| Layer N    |                   |            |
| Weights    |====DMA (32KB)===>| Buffer A   |
| (64B align)|                   | (active)   |
+------------+                   +------------+
| Layer N+1  |                   |            |
| Weights    |====DMA (32KB)===>| Buffer B   |
| (prefetch) |                   | (prefetch) |
+------------+                   +------------+
```

### Prefetch Strategy

- Double-buffered prefetching hides DMA latency
- While processing layer N, prefetch layer N+1
- 32KB burst size optimizes throughput
- 64-byte alignment required for DMA

---

## Diagnostics

### Boot-Time Tests

| Diagnostic | Purpose |
|------------|---------|
| `rom_probe` | Test ROM accessibility at various addresses (16MB-1GB) |
| `weights_info` | Display total weight size and entry count |
| `manifest_check` | Validate manifest integrity and alignment |
| `stream_crc` | Stream entire weights, verify CRC32 per entry |
| `stream_bench` | Measure DMA bandwidth and compute time |
| `decode_once` | Single inference pass to verify math |

### Error Display

```
+------------------------------------------+
|  ERROR: Tokenizer error: RomRead         |
|                                          |
|  or                                      |
|                                          |
|  PANIC: System error occurred            |
|  (infinite loop - requires reset)        |
+------------------------------------------+
```

---

## Build Configuration

### Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `N64_SOUL_MODEL_ID` | `gpt2` | Hugging Face model ID |
| `N64_SOUL_DTYPE` | `fp16` | Weight precision (`fp16`/`fp32`) |
| `N64_SOUL_KEEP_LAYERS` | (all) | Limit transformer depth |
| `N64_SOUL_SKIP_EXPORT` | `0` | Reuse existing weights |
| `N64SOUL_TOOLCHAIN` | `nightly-2024-10-01` | Rust toolchain |
| `N64SOUL_IPL3_BIN` | - | CIC-6102 boot code path |

### Build Command

```bash
cd n64llm/n64-rust
export N64_SOUL_MODEL_ID=distilgpt2
export N64_SOUL_DTYPE=fp16
export N64_SOUL_KEEP_LAYERS=6

TOOLCHAIN="${N64SOUL_TOOLCHAIN:-nightly-2024-10-01}"
cargo +"$TOOLCHAIN" -Z build-std=core,alloc n64 build --features embed_assets
```

### Output

```
n64llm/n64-rust/target/n64/release/n64_gpt.z64
```

---

## Performance Characteristics

| Metric | Typical Value |
|--------|---------------|
| DMA Bandwidth | 8-16 MB/s (varies by flashcart) |
| Layer Processing | ~50-100ms per layer |
| Token Generation | 2-5 seconds per token |
| Memory Usage | ~3MB peak during inference |
| ROM Size | 100-500MB (depends on model) |

---

## File Structure

```
N64Soul/
├── n64llm/n64-rust/
│   ├── src/
│   │   ├── main.rs              # Entry point, main loop
│   │   ├── display.rs           # Framebuffer, keyboard UI
│   │   ├── inference_engine.rs  # Transformer inference
│   │   ├── tokenizer.rs         # BPE tokenization
│   │   ├── memory_manager.rs    # Bump allocator
│   │   ├── n64_sys.rs           # Hardware registers
│   │   ├── ipl3.rs              # Boot code
│   │   ├── weights_manifest.rs  # Manifest parsing
│   │   ├── platform/
│   │   │   └── pi.rs            # Peripheral Interface DMA
│   │   ├── diag/
│   │   │   ├── rom_probe.rs
│   │   │   ├── stream_crc.rs
│   │   │   ├── stream_bench.rs
│   │   │   └── ...
│   │   ├── model/
│   │   │   └── stream.rs        # Layer streaming
│   │   └── stream/
│   │       └── prefetch.rs      # Double-buffer prefetch
│   ├── assets/                  # Generated weights
│   ├── Cargo.toml
│   ├── n64.ld                   # Linker script
│   └── mips-nintendo64-none.json
├── tools/
│   ├── export_gpt2_n64.py       # Weight exporter
│   ├── validate_weights.py      # Manifest validator
│   └── check_python_deps.py     # Dependency checker
├── scripts/
│   ├── export_and_test.sh       # Full build pipeline
│   └── emu_smoke.sh             # Emulator launcher
└── docs/
    ├── setup.md
    ├── emulator.md
    └── DESIGN.md                # This document
```

---

## Status Summary

| Component | Status |
|-----------|--------|
| Boot/IPL3 | Complete |
| Display System | Complete |
| Controller Input | Complete |
| On-Screen Keyboard | Complete |
| Tokenizer | Complete |
| Inference Engine | Complete |
| Memory Management | Complete |
| ROM I/O / DMA | Complete |
| Weight Streaming | Complete |
| Diagnostics | Complete |
| Build System | Complete |
| CI/CD | Complete |
| Documentation | Complete |

**Project Status: Ready for Testing**
