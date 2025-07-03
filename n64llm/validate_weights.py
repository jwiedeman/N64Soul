#!/usr/bin/env python3
"""Validate model weight layout.

This script verifies that `n64_model_weights_reduced.bin`
contains enough data for the offsets and sizes defined in
`inference_engine.rs`.
"""

import os
import re
import sys

SCRIPT_DIR = os.path.dirname(os.path.realpath(__file__))
INF_ENGINE = os.path.join(SCRIPT_DIR, "n64-rust", "src", "inference_engine.rs")
WEIGHTS_PATH = os.path.join(SCRIPT_DIR, "n64-rust", "src", "n64_model_weights_reduced.bin")


def _parse_array(source: str, name: str) -> list[int]:
    """Extract a Rust array from inference_engine.rs and return a list of ints."""
    pattern = rf"const\s+{name}\s*:\s*\[[^\]]+\]\s*=\s*\[(.*?)\];"
    match = re.search(pattern, source, re.DOTALL)
    if not match:
        raise RuntimeError(f"{name} not found in inference_engine.rs")
    body = match.group(1)
    values = []
    for item in body.split(','):
        item = item.split('//')[0].strip()
        if item:
            try:
                values.append(int(item, 0))
            except ValueError:
                # Handle expressions like "1024 * 1024"
                values.append(int(eval(item, {"__builtins__": None}, {})))
    return values


def load_constants() -> tuple[list[int], list[int]]:
    with open(INF_ENGINE, "r", encoding="utf-8") as f:
        src = f.read()
    offsets = _parse_array(src, "LAYER_OFFSETS")
    sizes = _parse_array(src, "LAYER_SIZES")
    return offsets, sizes


def main() -> int:
    if not os.path.exists(WEIGHTS_PATH):
        print(f"weights file not found: {WEIGHTS_PATH}")
        return 1

    offsets, sizes = load_constants()

    file_size = os.path.getsize(WEIGHTS_PATH)
    print(f"Weight file size: {file_size} bytes")

    for idx, (offset, length) in enumerate(zip(offsets, sizes)):
        end = offset + length
        if end > file_size:
            print(
                f"Layer {idx} exceeds file size: offset {offset:#08x} end {end:#08x}"
            )
            return 1
        if idx + 1 < len(offsets) and offsets[idx + 1] < end:
            print(
                f"Layer {idx} overlaps next layer: {offsets[idx + 1]:#08x} < {end:#08x}"
            )
            return 1

    expected_size = offsets[-1] + sizes[-1]
    if file_size < expected_size:
        print(
            f"File smaller than expected end {expected_size:#08x}: {file_size:#08x}"
        )
        return 1

    print("All offsets and sizes are valid.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
