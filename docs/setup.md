# Development Environment Setup

This guide lists the commands needed to install the toolchains used by the
N64Soul project. These instructions assume a Unix-like host with `bash` and
`git` installed.

## Rust toolchain

Install Rust using [`rustup`](https://rustup.rs/) if it is not already available:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then add the Nintendo 64 target and install the helper subcommand:

```bash
rustup toolchain install nightly --component rust-src
rustup target add mips-nintendo64-none --toolchain nightly
bash tools/install_cargo_n64.sh
cargo install nust64
```

The `tools/install_cargo_n64.sh` script first attempts a stock
`cargo +nightly install cargo-n64`. If that fails it clones upstream,
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
