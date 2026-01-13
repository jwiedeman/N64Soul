# PROJECT NEURON - Makefile
# N64 Neural Network Visualization System
# Built with libdragon

# Project name
PROJECT_NAME = neuron

# Source files
SOURCES = src/main.c \
          src/neural_net.c \
          src/training.c \
          src/pong.c \
          src/render.c \
          src/ui.c \
          src/save.c

# Include libdragon build system
include $(N64_INST)/include/n64.mk

# Additional include paths
CFLAGS += -I./include

# Optimization
CFLAGS += -O2 -ffast-math

# Build the ROM
all: $(PROJECT_NAME).z64

# Source to object mapping
OBJS = $(SOURCES:%.c=build/%.o)

# Compile source files
build/%.o: %.c
	@mkdir -p $(dir $@)
	$(CC) -c $(CFLAGS) -o $@ $<

# Link
build/$(PROJECT_NAME).elf: $(OBJS)
	$(LD) -o $@ $^ $(LDFLAGS) -ldragon -lc -lm -ldragonsys

# Build filesystem (if we have assets)
ASSETS_LIST = $(wildcard assets/fonts/*.png) \
              $(wildcard assets/sprites/*.png)

ifneq ($(ASSETS_LIST),)
filesystem/font.sprite: assets/fonts/font.png
	@mkdir -p $(dir $@)
	$(N64_MKSPRITE) -f RGBA16 -o $@ $<

build/$(PROJECT_NAME).dfs: $(ASSETS_LIST:assets/%.png=filesystem/%.sprite)
	@mkdir -p build
	$(N64_MKDFS) $@ filesystem/

$(PROJECT_NAME).z64: build/$(PROJECT_NAME).elf build/$(PROJECT_NAME).dfs
	$(N64_TOOL) $(N64_FLAGS) -o $@ --title "PROJECT NEURON" \
		--toc build/$(PROJECT_NAME).elf \
		--dfs build/$(PROJECT_NAME).dfs
else
$(PROJECT_NAME).z64: build/$(PROJECT_NAME).elf
	$(N64_TOOL) $(N64_FLAGS) -o $@ --title "PROJECT NEURON" \
		build/$(PROJECT_NAME).elf
endif

# Clean build artifacts
clean:
	rm -rf build/ filesystem/ $(PROJECT_NAME).z64

# Run in emulator (requires ares or similar in PATH)
run: $(PROJECT_NAME).z64
	@echo "ROM built: $(PROJECT_NAME).z64"
	@echo "Run with: ares $(PROJECT_NAME).z64"

.PHONY: all clean run
