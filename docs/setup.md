# Development Environment Setup

This guide lists the commands needed to install the toolchains used by the
N64Soul project. These instructions assume a Unix-like host with `bash` and
`git` installed.

## Rust toolchain

Install Rust using [`rustup`](https://rustup.rs/) if it is not already available:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then add the Nintendo 64 target and install the helper subcommand. The
`mips-nintendo64-none` target disappeared from recent nightlies, so the build
expects the pinned toolchain `nightly-2022-06-21`. Export the toolchain once so
the helper scripts and docs remain in sync:

```bash
export N64SOUL_TOOLCHAIN=nightly-2022-06-21

rustup toolchain install "$N64SOUL_TOOLCHAIN"
rustup component add rust-src --toolchain "$N64SOUL_TOOLCHAIN"

# Install cargo-n64 with the pinned toolchain.
N64SOUL_TOOLCHAIN="$N64SOUL_TOOLCHAIN" bash tools/install_cargo_n64.sh

# Optional utilities can use stable.
cargo install nust64
```

Rustup no longer ships a prebuilt `mips-nintendo64-none` standard library for any
host platform, so attempting `rustup target add mips-nintendo64-none` now fails
with `toolchain 'nightly-2022-06-21-…' does not support target`. That is
expected—`cargo-n64` bundles the target specification and the build uses
`-Zbuild-std=core,alloc` to compile the required crates from `rust-src`, so no
additional `rustup target` installation is necessary.

The `tools/install_cargo_n64.sh` script first attempts a stock
`cargo +"$N64SOUL_TOOLCHAIN" install cargo-n64`. If that fails it clones upstream,
patches the offending dependency, and reinstalls the tool in-place.
Re-running the script is idempotent.

## Python dependencies

The export pipeline imports PyTorch and Hugging Face Transformers. Use the helper
script to confirm the modules are available before running any of the weight
pipelines:

```bash
python tools/check_python_deps.py
```

The script lists any missing modules together with the pip command needed to
install them.

## Emulator

For local testing you can install an N64 emulator such as **Mupen64Plus**. On
Debian-based systems:

```bash
sudo apt-get install mupen64plus
```

You can then run generated ROMs with `mupen64plus <path-to-rom>`.
