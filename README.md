# N64Soul

This repository experiments with running a language model on Nintendo 64 hardware.

## Project layout

- **n64llm/** – Rust implementation with weight streaming and inference engine.
- **n64_ai_project/** – Minimal C examples.

The Rust code is the focus for new features. All crates are built with
`no_std` and target `mips-nintendo64-none`.

## Dev Quickstart (one command)
Run tests, build a ROM with temporary debug weights, optionally offer an emulator
smoke test, and scrub the weight blobs:

```bash
./scripts/export_and_test.sh
```

If all prerequisites are installed this leaves a `n64_gpt.z64` ROM in
`n64llm/n64-rust/`; otherwise the script exits with a descriptive error.

## Continuous Integration

CI runs host-only unit tests, validates the generated weight manifest, builds and checksums a ROM image, and verifies no binary artifacts leak into the repository.

## Environment setup

If you are preparing a fresh system you will need both the Rust nightly toolchain
with source components and the libdragon MIPS toolchain. The exact commands are
copied below for convenience. See [docs/setup.md](docs/setup.md) for more detail.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install nightly --component rust-src

# Install a patched cargo-n64; upstream 0.2.0 relies on the removed
# `Error::backtrace` API and fails to build on current compilers.
bash tools/install_cargo_n64.sh

cargo install nust64

git clone https://github.com/dragonminded/libdragon.git
cd libdragon
make toolchain
export PATH="$PWD/mips64-elf/bin:$PATH"
```

After installing these tools you can build and run the examples as described
below. Use `python tools/check_python_deps.py` to confirm the Python
dependencies for the export pipeline are present before running any of the
scripts.

> **Note:** `tools/install_cargo_n64.sh` first attempts a stock `cargo +nightly
> install cargo-n64`. If that fails it clones the upstream repository, applies a
> small shim that disables the `backtrace` call, and installs the patched
> version. Re-running the script is idempotent.

## Building the Rust project

A custom toolchain is required. Build with nightly and package the resulting ELF into a ROM:

Before building, make sure `n64llm/n64-rust/assets/` contains a manifest and a
weight blob. For a quick smoke test you can generate synthetic data with:

```bash
python tools/make_debug_weights.py \
  --out-bin n64llm/n64-rust/assets/weights.bin \
  --out-man n64llm/n64-rust/assets/weights.manifest.bin
```

Once you export a real model, validate it with `python tools/validate_weights.py --crc`
so the manifest entries are aligned and the CRC32 checksums match.

Nintendo 64 ROMs also require the CIC-6102 boot code. Because that blob is
copyrighted we cannot ship it; you must provide your own dump via
`--ipl3 /path/to/cic6102.bin` or extract it from a known-good ROM with
`--ipl3-from-rom /path/to/rom.z64`.

With weights in place and an IPL3 path available, build the ROM with:

```bash
cd n64llm/n64-rust
cargo +nightly -Z build-std=core,alloc n64 build \
  --ipl3 /path/to/cic6102.bin \
  -- --release --features embed_assets
```

The `--` separator passes subsequent flags directly to Cargo so you can request
the release profile and enable the `embed_assets` feature. Without a valid CIC
file the subcommand terminates with `invalid argument to option --ipl3`.

On success `cargo-n64` produces `target/n64/release/n64_gpt.z64`. The linker and
configuration reserve roughly 1&nbsp;GiB of cart ROM space; the actual usable size
depends on your flashcart or emulator.

## Building the C project

The C examples rely on the `mips64-elf` toolchain from [libdragon](https://libdragon.dev/). Once installed, run:

```bash
cd n64_ai_project
make
```

A ROM file named `n64_ai_project.z64` will be created.

## Running on an emulator

Both projects output standard N64 ROM images (`.z64`). Run them using an
emulator such as **Mupen64Plus** or **Ares**:

```bash
mupen64plus n64llm/n64-rust/n64_gpt.z64
```

(Replace the path with the built ROM.)
The helper script `scripts/emu_smoke.sh` can perform a headless emulator smoke
test; it will never move binaries for you and instead asks where to place them.
See [docs/emulator.md](docs/emulator.md) for recommended emulator settings and
controller mappings when exercising the on-screen keyboard UI.

## Running on real hardware

Flash the generated ROM onto a flashcart (e.g. an EverDrive&nbsp;64) and
run it on a Nintendo&nbsp;64 console. Ensure your flashcart supports the
ROM size produced by the project.

## Exporting model weights

Model weights live in `n64llm/n64-rust/assets/weights.bin` with layout described by
`weights.manifest.bin`. The helper script `tools/export_model.py` can combine
individual layer files into these artifacts:

```bash
python tools/export_model.py tok_embeddings=emb.bin attn_q_0=attn0.bin
```

Each `name=path` pair becomes an aligned segment in the resulting blob and is
recorded in the manifest.

## Validating weight offsets

Before running on real hardware you can verify that the bundled weight blob
matches its manifest. Run the helper script from the repository root:

```bash
python tools/validate_weights.py --bin n64llm/n64-rust/assets/weights.bin \
  --man n64llm/n64-rust/assets/weights.manifest.bin
```

The script checks that `n64llm/n64-rust/assets/weights.bin` exists, that each layer in
`weights.manifest.bin` is 64-byte aligned, and that the sizes sum to the file
length.

For a one-shot export, validation, build, optional smoke test, and cleanup, run:

```bash
./scripts/export_and_test.sh
```

### Git hook for automatic validation

To ensure this validation runs whenever model weights are updated, a sample
pre-commit hook is provided in `.githooks/pre-commit`. Enable it with:

```bash
git config core.hooksPath .githooks
```

With the hook active, any commit that modifies
`n64llm/n64-rust/assets/weights.bin` will automatically invoke
`tools/validate_weights.py` and abort the commit if validation fails.
