# Codex Environment Setup

The project targets the `mips-nintendo64-none` architecture and requires a nightly Rust toolchain to build. The `cargo-n64` subcommand is also needed. The recommended install commands are below.

```bash
# 1. Install a pinned nightly toolchain and rust-src
rustup toolchain install nightly-2022-06-21
rustup component add rust-src --toolchain nightly-2022-06-21

# 2. Install cargo-n64 using the same nightly
cargo +nightly-2022-06-21 install --git https://github.com/rust-console/cargo-n64.git --locked
```

`rustup` no longer provides a `mips-nintendo64-none` standard library, so
attempting to add that target prints `toolchain 'nightly-2022-06-21-…' does not
support target`. The build instead relies on the `rust-src` component together
with `-Zbuild-std=core,alloc`, and `cargo-n64` bundles the target specification
needed to invoke `rustc`.

After these tools are installed, build the Rust project with:

```bash
cd n64llm/n64-rust
cargo +nightly-2022-06-21 -Z build-std=core,alloc n64 build -- --features embed_assets
```

Enabling the `embed_assets` feature ensures the ROM includes the exported weights and manifest files.

This produces a bootable ROM under `target/n64/release/` which can be run in an emulator such as `mupen64plus`.
Building with `--features embed_assets` triggers the Python exporter defined in
`build.rs`. Override the default settings with environment variables such as
`N64_SOUL_MODEL_ID`, `N64_SOUL_DTYPE`, or `N64_SOUL_KEEP_LAYERS` before invoking
`cargo-n64`. Set `N64_SOUL_SKIP_EXPORT=1` to reuse existing assets.

`cargo-n64` automatically appends `--release` to the underlying `cargo build`
command, so the resulting ROM uses optimized code paths by default.

