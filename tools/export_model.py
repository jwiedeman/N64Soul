#!/usr/bin/env python3
"""Export model weights to a contiguous blob with manifest.

Usage:
    python export_model.py name1=path1 name2=path2 ...
Each argument specifies a layer name and the path to its raw weights.
The script concatenates the files into `weights.bin` with 64-byte alignment
and writes `weights.manifest.json` describing the layout.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

ALIGN = 64


def pad_size(size: int, align: int = ALIGN) -> int:
    return (size + align - 1) // align * align


def write_aligned(f, data: bytes) -> tuple[int, int]:
    offset = f.tell()
    pad = (-offset) % ALIGN
    if pad:
        f.write(b"\0" * pad)
        offset += pad
    padded = pad_size(len(data))
    f.write(data)
    if padded > len(data):
        f.write(b"\0" * (padded - len(data)))
    return offset, padded


def main() -> None:
    parser = argparse.ArgumentParser(description="Create weights.bin and manifest")
    parser.add_argument("segments", nargs="+", help="Layer segments in name=path form")
    parser.add_argument(
        "--outdir",
        default=Path(__file__).resolve().parent.parent / "n64llm" / "assets",
        type=Path,
        help="Output directory for weights.bin and manifest",
    )
    args = parser.parse_args()

    outdir: Path = args.outdir
    outdir.mkdir(parents=True, exist_ok=True)
    weights_path = outdir / "weights.bin"
    manifest_path = outdir / "weights.manifest.json"

    layers = []
    with weights_path.open("wb") as wf:
        for spec in args.segments:
            if "=" not in spec:
                raise SystemExit(f"Invalid segment spec '{spec}', expected name=path")
            name, path = spec.split("=", 1)
            data = Path(path).read_bytes()
            offset, size = write_aligned(wf, data)
            layers.append({"name": name, "offset": offset, "size": size})
    manifest = {"version": 1, "align": ALIGN, "layers": layers}
    manifest_path.write_text(json.dumps(manifest, indent=2))
    print(f"wrote {weights_path} ({weights_path.stat().st_size} bytes)")
    print(f"wrote {manifest_path}")


if __name__ == "__main__":
    main()
