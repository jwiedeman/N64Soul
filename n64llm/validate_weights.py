#!/usr/bin/env python3
"""Validate model weight layout.

This script verifies that `n64_model_weights_reduced.bin`
contains enough data for the offsets and sizes defined in
`inference_engine.rs`.
"""

import os
import sys

LAYER_OFFSETS = [
    0x00000000,
    0x00100000,
    0x00200000,
    0x00300000,
    0x00400000,
    0x00500000,
    0x00600000,
    0x00700000,
    0x00800000,
    0x00900000,
    0x00A00000,
    0x00B00000,
    0x00C00000,
    0x00D00000,
]

# All layers are 1MB in size in this reduced model.
LAYER_SIZES = [1024 * 1024 for _ in range(len(LAYER_OFFSETS))]

SCRIPT_DIR = os.path.dirname(os.path.realpath(__file__))
WEIGHTS_PATH = os.path.join(SCRIPT_DIR, "n64-rust", "src", "n64_model_weights_reduced.bin")


def main() -> int:
    if not os.path.exists(WEIGHTS_PATH):
        print(f"weights file not found: {WEIGHTS_PATH}")
        return 1

    file_size = os.path.getsize(WEIGHTS_PATH)
    print(f"Weight file size: {file_size} bytes")

    for idx, (offset, length) in enumerate(zip(LAYER_OFFSETS, LAYER_SIZES)):
        end = offset + length
        if end > file_size:
            print(
                f"Layer {idx} exceeds file size: offset {offset:#08x} end {end:#08x}"
            )
            return 1
        if idx + 1 < len(LAYER_OFFSETS) and LAYER_OFFSETS[idx + 1] < end:
            print(
                f"Layer {idx} overlaps next layer: {LAYER_OFFSETS[idx + 1]:#08x} < {end:#08x}"
            )
            return 1

    expected_size = LAYER_OFFSETS[-1] + LAYER_SIZES[-1]
    if file_size < expected_size:
        print(
            f"File smaller than expected end {expected_size:#08x}: {file_size:#08x}"
        )
        return 1

    print("All offsets and sizes are valid.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
