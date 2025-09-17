# N64Soul

This repository experiments with running a language model on Nintendo 64 hardware.

## Project layout

The repository is intentionally trimmed down to a single ROM crate plus the
canonical exporter script. Everything else exists to support that workflow.

- **n64llm/n64-rust/** – Rust implementation with weight streaming, tokenizer,
  and ROM packaging.
- **tools/export_gpt2_n64.py** – exporter that distills Hugging Face models into
  the `weights.bin`/`weights.manifest.bin` pair consumed by the build.
- **scripts/export_and_test.sh** – one-shot helper that runs host tests, exports
  fresh weights, builds the ROM, and optionally launches an emulator smoke
  check.

All development now lives in the Rust crate, and every weight export flows
through the Python helper. The crate builds with `no_std` for the
`mips-nintendo64-none` target and embeds assets at link time.

Older host-side prototypes have been removed from the tree; use the exporter and
crate above as the single source of truth when working on the project.

## Dev Quickstart (one command)
Run host tests, export real weights, build a ROM, optionally offer an emulator
smoke test, and leave the assets in place for flashing:

```bash
./scripts/export_and_test.sh
```

If all prerequisites are installed this leaves a `n64_gpt.z64` ROM in
`n64llm/n64-rust/target/n64/release/`; otherwise the script exits with a
descriptive error.

## Continuous Integration

CI runs host-only unit tests, validates the generated weight manifest, builds and
checksums a ROM image, and verifies no binary artifacts leak into the
repository.

## Environment setup

Install the Rust toolchain and helper utilities. See
[docs/setup.md](docs/setup.md) for more detail. The project is pinned to the
`nightly-2022-06-21` toolchain because newer nightlies no longer ship the
`mips-nintendo64-none` target:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
export N64SOUL_TOOLCHAIN=nightly-2022-06-21

rustup toolchain install "$N64SOUL_TOOLCHAIN"
rustup component add rust-src --toolchain "$N64SOUL_TOOLCHAIN"

# Install a patched cargo-n64; upstream 0.2.0 relies on the removed
# `Error::backtrace` API and fails to build on current compilers.
N64SOUL_TOOLCHAIN="$N64SOUL_TOOLCHAIN" bash tools/install_cargo_n64.sh

cargo install nust64
```

Rustup no longer publishes a `mips-nintendo64-none` standard library, so running
`rustup target add` now reports `toolchain 'nightly-2022-06-21-…' does not
support target`. That failure is expected—`cargo-n64` provides the target
specification and the build uses `-Zbuild-std=core,alloc` to compile `core` and
`alloc` from the `rust-src` component.

After installing these tools you can build and run the project as described
below. Use `python tools/check_python_deps.py` to confirm the Python
dependencies for the export pipeline are present before running any of the
scripts.

> **Note:** `tools/install_cargo_n64.sh` first attempts a stock `cargo +"$N64SOUL_TOOLCHAIN" install cargo-n64`. If that fails it clones the upstream repository, applies a
> small shim that disables the `backtrace` call, and installs the patched
> version. Re-running the script is idempotent.

## Building the Rust project

Building with the `embed_assets` feature runs the Python exporter automatically.
The build script invokes `tools/export_gpt2_n64.py`, writes fresh weights to the
crate's `assets/` directory, validates the manifest, and finally packages the
ROM.

```bash
cd n64llm/n64-rust
# optional overrides for the exporter
export N64_SOUL_MODEL_ID=distilgpt2
export N64_SOUL_DTYPE=fp16
export N64_SOUL_KEEP_LAYERS=8

TOOLCHAIN="${N64SOUL_TOOLCHAIN:-nightly-2022-06-21}"
cargo +"$TOOLCHAIN" -Z build-std=core,alloc n64 build --features embed_assets
```

Unset `N64_SOUL_KEEP_LAYERS` (or skip exporting entirely) to use the full model.
The following environment variables control the exporter:

- `N64_SOUL_MODEL_ID` (default `gpt2`) – Hugging Face model id or local path.
- `N64_SOUL_DTYPE` (default `fp16`) – `fp16` or `fp32` output weights.
- `N64_SOUL_KEEP_LAYERS` – keep only the last N transformer blocks.
- `N64_SOUL_TUNE_CONFIG` – optional JSON file archived with the export.
- `N64_SOUL_SKIP_EXPORT` – set to `1` to reuse the assets already on disk.

The build script always invokes `tools/export_gpt2_n64.py`. Adjust that script
directly if you need to change export behavior; keeping a single exporter avoids
stale artifacts from earlier experiments.

`cargo-n64` automatically forwards `--release` to the inner `cargo build` call,
so the wrapper will produce an optimized ROM without additional flags.

Nintendo 64 ROMs also require the CIC-6102 boot code. Because that blob is
copyrighted we cannot ship it; you must provide your own dump via
`--ipl3 /path/to/cic6102.bin` or extract it from a known-good ROM with
`--ipl3-from-rom /path/to/rom.z64` when running `cargo-n64`.
Helper scripts such as `scripts/export_and_test.sh` and `scripts/emu_smoke.sh`
forward the boot code automatically when the related environment variables are
set before invocation. When none of them are provided the scripts now generate a
zeroed placeholder so the ROM packages successfully; set
`N64SOUL_IPL3_DUMMY=0` to require a real bootcode dump.

- `N64SOUL_IPL3_BIN` – absolute or relative path to a CIC-6102 dump.
- `N64SOUL_IPL3_FROM_ROM` – path to a ROM image; `cargo-n64` extracts the
  boot code automatically.
- `N64SOUL_IPL3_DUMMY=1` – explicitly generate the placeholder (non-bootable,
  useful for CI smoke tests).

On success `cargo-n64` produces `target/n64/release/n64_gpt.z64`. The linker and
configuration reserve roughly 1&nbsp;GiB of cart ROM space; the actual usable size
depends on your flashcart or emulator.

## Running on an emulator

The project outputs a standard N64 ROM image (`.z64`). Run it using an emulator
such as **Mupen64Plus** or **Ares**:

```bash
mupen64plus n64llm/n64-rust/target/n64/release/n64_gpt.z64
```

(Replace the path with the built ROM.) The helper script `scripts/emu_smoke.sh`
performs a quick smoke test: it verifies that exported weights exist, rebuilds
with the current assets if needed, and attempts to launch `ares` or
`mupen64plus` if they are available on your `PATH`. See
[docs/emulator.md](docs/emulator.md) for recommended emulator settings and
controller mappings when exercising the on-screen keyboard UI.

## Running on real hardware

Flash the generated ROM onto a flashcart (e.g. an EverDrive&nbsp;64) and run it on a
Nintendo&nbsp;64 console. Ensure your flashcart supports the ROM size produced by
the project.

## Exporting model weights manually

The automatic exporter covers the usual workflow, but the helper script remains
available if you need manual control:

```bash
python tools/export_gpt2_n64.py --model distilgpt2 --dtype fp16 --out-dir n64llm/n64-rust/assets
```

Model weights live in `n64llm/n64-rust/assets/weights.bin` with layout described
by `weights.manifest.bin`.

## Validating weight offsets

`build.rs` already runs `tools/validate_weights.py --crc` after each export, but
you can re-run it manually:

```bash
python tools/validate_weights.py --bin n64llm/n64-rust/assets/weights.bin \
  --man n64llm/n64-rust/assets/weights.manifest.bin --crc
```

The script checks that `n64llm/n64-rust/assets/weights.bin` exists, that each
layer in `weights.manifest.bin` is 64-byte aligned, and that the sizes sum to the
file length.

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
