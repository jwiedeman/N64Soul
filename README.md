# N64Soul

This repository experiments with running a language model on Nintendo 64 hardware.

## Project layout

- **n64llm/** – Rust implementation with weight streaming and inference engine.
- **n64_ai_project/** – Minimal C examples.

The Rust code is the focus for new features. All crates are built with
`no_std` and target `mips-nintendo64-none`.

## Environment setup

If you are preparing a fresh system, the commands required to install the
Rust and MIPS toolchains are listed in [docs/setup.md](docs/setup.md). Follow
those steps to enable full debugging and testing on your machine.

## Building the Rust project

A custom toolchain is required. Install [`cargo-n64`](https://github.com/rust-console/cargo-n64) and add the `mips-nintendo64-none` target:

```bash
rustup target add mips-nintendo64-none
cargo install cargo-n64
```

Then build the project:

```bash
cd n64llm/n64-rust
cargo n64 build --release
```

This will produce a bootable Nintendo&nbsp;64 ROM in `target/mips-nintendo64-none/release/`.

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
mupen64plus target/mips-nintendo64-none/release/n64_gpt.n64
```

(Replace the path with the built ROM.)

## Running on real hardware

Flash the generated ROM onto a flashcart (e.g. an EverDrive&nbsp;64) and
run it on a Nintendo&nbsp;64 console. Ensure your flashcart supports the
ROM size produced by the project.

## Validating weight offsets

Before running on real hardware you can verify that the bundled weight
binary matches the offsets expected by the Rust code. Run the helper
script from the repository root:

```bash
python3 n64llm/validate_weights.py
```

The script prints an error if any section in `n64_model_weights_reduced.bin`
does not align with the constants in `inference_engine.rs`.
