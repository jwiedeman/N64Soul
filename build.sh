#!/bin/bash
# PROJECT NEURON Build Script
# Builds the N64 ROM using libdragon Docker container

set -e

DOCKER_IMAGE="ghcr.io/dragonminded/libdragon:latest"
PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "======================================"
echo "  PROJECT NEURON - Build System"
echo "======================================"
echo ""

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "ERROR: Docker is not installed."
    echo "Please install Docker from https://docker.com"
    exit 1
fi

# Check if image exists, pull if not
if ! docker image inspect "$DOCKER_IMAGE" &> /dev/null; then
    echo "Pulling libdragon Docker image (this may take a few minutes)..."
    docker pull "$DOCKER_IMAGE"
fi

echo "Building ROM..."
echo ""

# Run build inside container
docker run --rm \
    -v "$PROJECT_DIR:/work" \
    -w /work \
    "$DOCKER_IMAGE" \
    make -f Makefile.n64 clean all

echo ""
echo "======================================"
if [ -f "neuron.z64" ]; then
    echo "  BUILD SUCCESSFUL!"
    echo "  ROM: neuron.z64"
    echo ""
    echo "  Test with: ares neuron.z64"
    echo "         or: simple64 neuron.z64"
else
    echo "  BUILD FAILED"
    exit 1
fi
echo "======================================"
