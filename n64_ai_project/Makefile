RUST_PROJECT = ../n64llm/n64-rust
TARGET = n64_soul.z64
ROM_BIN = $(RUST_PROJECT)/target/mips-nintendo64-none/release/n64-rust

all: $(TARGET)

$(ROM_BIN):
cargo build --release --manifest-path $(RUST_PROJECT)/Cargo.toml --target mips-nintendo64-none || true

$(TARGET): $(ROM_BIN)
n64tool -l $(TARGET) -t "N64SOUL" -h 6102 $(ROM_BIN)

clean:
cargo clean --manifest-path $(RUST_PROJECT)/Cargo.toml || true
rm -f $(TARGET)
