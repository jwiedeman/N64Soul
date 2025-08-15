# N64Soul

This repository experiments with running a language model on Nintendo 64 hardware.

## Project layout

- **n64llm/** – Rust implementation with weight streaming and inference engine.
- **n64_ai_project/** – Minimal C examples.

The Rust code is the focus for new features. All crates are built with
`no_std` and target `mips-nintendo64-none`.

## Environment setup

If you are preparing a fresh system you will need both the Rust `cargo-n64`
toolchain and the libdragon MIPS toolchain. The exact commands are copied
below for convenience. See [docs/setup.md](docs/setup.md) for more detail.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add mips-nintendo64-none
cargo install cargo-n64

git clone https://github.com/dragonminded/libdragon.git
cd libdragon
make toolchain
export PATH="$PWD/mips64-elf/bin:$PATH"
```

After installing these tools you can build and run the examples as described
below.

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

### Git hook for automatic validation

To ensure this validation runs whenever model weights are updated, a sample
pre-commit hook is provided in `.githooks/pre-commit`. Enable it with:

```bash
git config core.hooksPath .githooks
```

With the hook active, any commit that modifies
`n64llm/n64-rust/assets/weights.bin` will automatically invoke
`tools/validate_weights.py` and abort the commit if validation fails.
