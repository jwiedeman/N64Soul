# Codex Environment Setup

The project targets the `mips-nintendo64-none` architecture and requires a nightly Rust toolchain to build. The `cargo-n64` subcommand is also needed. The recommended install commands are below.

```bash
# 1. Install a pinned nightly toolchain and rust-src
rustup toolchain install nightly-2022-06-21
rustup component add rust-src --toolchain nightly-2022-06-21

# 2. Install cargo-n64 using the same nightly
cargo +nightly-2022-06-21 install --git https://github.com/rust-console/cargo-n64.git --locked

# 3. Add the target for the Nintendo 64
rustup target add mips-nintendo64-none --toolchain nightly-2022-06-21
```

After these tools are installed, build the Rust project with:

```bash
cd n64llm/n64-rust
cargo +nightly-2022-06-21 n64 build --profile release --features embed_assets
```

Enabling the `embed_assets` feature ensures the ROM includes the exported weights and manifest files.

This produces a bootable ROM under `target/n64/release/` which can be run in an emulator such as `mupen64plus`.
