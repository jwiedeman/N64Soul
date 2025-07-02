# Development Environment Setup

This guide lists the commands needed to install the toolchains used by the N64Soul project.
These instructions assume a Unix-like host with `bash` and `git` installed.

## Rust toolchain

Install Rust using [`rustup`](https://rustup.rs/) if it is not already available:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then add the Nintendo 64 target and install `cargo-n64`:

```bash
rustup target add mips-nintendo64-none
cargo install cargo-n64
```

## Libdragon MIPS toolchain (for C examples)

Clone the libdragon repository and build its cross compiler:

```bash
git clone https://github.com/dragonminded/libdragon.git
cd libdragon
make toolchain
```

After the build completes, add the toolchain to your `PATH`:

```bash
export PATH="$PWD/mips64-elf/bin:$PATH"
```

## Emulator

For local testing you can install an N64 emulator such as **Mupen64Plus**. On Debian-based systems:

```bash
sudo apt-get install mupen64plus
```

You can then run generated ROMs with `mupen64plus <path-to-rom>`.
