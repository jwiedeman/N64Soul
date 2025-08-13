RUST_PROJECT = ../n64llm/n64-rust
ROM = n64_gpt.z64

all: $(ROM)

$(ROM):
# Build the Rust N64 binary (requires custom toolchain)
cargo build --manifest-path $(RUST_PROJECT)/Cargo.toml --release --target mips-nintendo64-none
# Package the ROM using n64tool (part of Libdragon)
n64tool -l 8M -t "N64 GPT" -h $(RUST_PROJECT)/target/mips-nintendo64-none/release/ipl3.bin -o $(ROM) $(RUST_PROJECT)/target/mips-nintendo64-none/release/n64_gpt

clean:
cargo clean --manifest-path $(RUST_PROJECT)/Cargo.toml
rm -f $(ROM)
