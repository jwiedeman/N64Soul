# N64 GPT Build System
# Requires: cargo, nust64 (cargo install nust64), nightly Rust

ROM_NAME = n64_gpt
IPL3 = ipl3_dummy.bin
TARGET = mips-nintendo64-none
PROFILE = release

ELF = target/$(TARGET)/$(PROFILE)/$(ROM_NAME)
ROM = $(ROM_NAME).n64

# Default: build the ROM
all: $(ROM)

# Build ELF with cargo
$(ELF): FORCE
	cargo +nightly -Z build-std=core,alloc build \
		--target $(TARGET) \
		--$(PROFILE) \
		--features embed_assets

# Convert ELF to N64 ROM using nust64
$(ROM): $(ELF)
	nust64 --ipl3 $(IPL3) $(ELF) $(ROM)
	@echo "Built: $(ROM) ($$(du -h $(ROM) | cut -f1))"

# Build with fewer layers for smaller ROM (default: 1 layer)
small:
	N64_SOUL_KEEP_LAYERS=1 $(MAKE) all

# Clean build artifacts
clean:
	cargo clean
	rm -f $(ROM)

# Run in emulator (requires ares, cen64, or similar in PATH)
run: $(ROM)
	@if command -v ares >/dev/null 2>&1; then \
		ares $(ROM); \
	elif command -v cen64 >/dev/null 2>&1; then \
		cen64 $(ROM); \
	else \
		echo "No N64 emulator found. Install ares or cen64."; \
	fi

# Check if tools are installed
check:
	@echo "Checking build tools..."
	@command -v cargo >/dev/null && echo "  cargo: OK" || echo "  cargo: MISSING"
	@command -v nust64 >/dev/null && echo "  nust64: OK" || echo "  nust64: MISSING (cargo install nust64)"
	@rustup run nightly rustc --version >/dev/null 2>&1 && echo "  nightly: OK" || echo "  nightly: MISSING (rustup install nightly)"
	@rustup component list --toolchain nightly 2>/dev/null | grep -q "rust-src (installed)" && echo "  rust-src: OK" || echo "  rust-src: MISSING (rustup +nightly component add rust-src)"

.PHONY: all clean run check small FORCE
FORCE:
