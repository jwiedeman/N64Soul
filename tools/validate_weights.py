#!/usr/bin/env python3
"""Validate a weights.bin file against its manifest."""

import argparse
import json
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate weights manifest")
    default_dir = Path(__file__).resolve().parent.parent / "n64llm" / "assets"
    parser.add_argument("--weights", type=Path, default=default_dir / "weights.bin")
    parser.add_argument(
        "--manifest", type=Path, default=default_dir / "weights.manifest.json"
    )
    args = parser.parse_args()

    if not args.weights.exists():
        print(f"missing weights file: {args.weights}")
        return 1
    if not args.manifest.exists():
        print(f"missing manifest: {args.manifest}")
        return 1

    data = json.loads(args.manifest.read_text())
    align = data.get("align", 64)
    layers = data.get("layers", [])
    file_size = args.weights.stat().st_size

    last_end = 0
    total = 0
    for layer in layers:
        name = layer["name"]
        off = layer["offset"]
        size = layer["size"]
        if off % align != 0:
            print(f"layer {name} offset {off} not aligned to {align}")
            return 1
        if off < last_end:
            print(f"layer {name} overlaps previous segment")
            return 1
        end = off + size
        if end > file_size:
            print(f"layer {name} exceeds file size")
            return 1
        last_end = end
        total += size

    if total != file_size:
        print(f"sum of layer sizes {total} != file size {file_size}")
        return 1

    print("weights and manifest validated")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
