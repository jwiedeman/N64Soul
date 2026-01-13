# PROJECT NEURON - Makefile
# N64 Neural Network Visualization System

# Configuration
ROM_NAME = neuron
BUILD_DIR = build

# Source files
SRCS = src/main.c \
       src/neural_net.c \
       src/training.c \
       src/pong.c \
       src/render.c \
       src/ui.c \
       src/save.c

# libdragon setup
include $(N64_INST)/include/n64.mk

# Include paths
N64_CFLAGS += -I./include

# Optimization flags
N64_CFLAGS += -O2 -ffast-math

# ROM configuration
N64_ROM_TITLE = "PROJECT NEURON"
N64_ROM_SAVETYPE = eeprom4k  # For settings
N64_ROM_REGIONFREE = true

all: $(ROM_NAME).z64

# Filesystem for assets
ASSETS = $(wildcard assets/fonts/*.sprite) \
         $(wildcard assets/sprites/*.sprite) \
         $(wildcard assets/checkpoints/*.bin)

filesystem/%.sprite: assets/%.png
	@mkdir -p $(dir $@)
	$(N64_MKSPRITE) -f RGBA16 $< $@

$(BUILD_DIR)/$(ROM_NAME).dfs: $(ASSETS)
	@mkdir -p $(BUILD_DIR)
	$(N64_MKDFS) $@ filesystem/

$(BUILD_DIR)/$(ROM_NAME).elf: $(SRCS:%.c=$(BUILD_DIR)/%.o)
	@mkdir -p $(BUILD_DIR)
	$(N64_LD) -o $@ $^ $(N64_LDFLAGS)

$(ROM_NAME).z64: $(BUILD_DIR)/$(ROM_NAME).elf $(BUILD_DIR)/$(ROM_NAME).dfs
	$(N64_ROM) $@ $< -d $(BUILD_DIR)/$(ROM_NAME).dfs

$(BUILD_DIR)/%.o: %.c
	@mkdir -p $(dir $@)
	$(N64_CC) $(N64_CFLAGS) -c -o $@ $<

clean:
	rm -rf $(BUILD_DIR) filesystem/ $(ROM_NAME).z64

# Development helpers
.PHONY: all clean run

run: $(ROM_NAME).z64
	@echo "Run in emulator: ares $(ROM_NAME).z64"
