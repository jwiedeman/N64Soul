# PROJECT NEURON - Makefile
# N64 Neural Network Visualization System
# Uses libdragon Docker container for building

PROJECT_NAME = neuron
BUILD_DIR = build

# Source files
SOURCES = src/main.c \
          src/neural_net.c \
          src/training.c \
          src/pong.c \
          src/render.c \
          src/ui.c \
          src/save.c

# Object files
OBJECTS = $(SOURCES:src/%.c=$(BUILD_DIR)/%.o)

# Docker command for libdragon
DOCKER_IMAGE = ghcr.io/dragonminded/libdragon:latest
DOCKER_RUN = docker run --rm -v "$(CURDIR):/work" -w /work $(DOCKER_IMAGE)

# Compiler flags
CFLAGS = -std=gnu99 -O2 -Wall -ffast-math -I./include
LDFLAGS = -ldragon -lc -lm -ldragonsys

.PHONY: all clean docker-build run

# Default: build using Docker
all: docker-build

# Build inside Docker container
docker-build:
	@echo "Building PROJECT NEURON with libdragon Docker..."
	$(DOCKER_RUN) make -f Makefile.n64 all

# Clean
clean:
	rm -rf $(BUILD_DIR) $(PROJECT_NAME).z64 filesystem/

# Pull latest libdragon image
docker-pull:
	docker pull $(DOCKER_IMAGE)

# Run in emulator (if ares is installed locally)
run: $(PROJECT_NAME).z64
	@echo "ROM ready: $(PROJECT_NAME).z64"
	@echo "Run with your preferred N64 emulator"
