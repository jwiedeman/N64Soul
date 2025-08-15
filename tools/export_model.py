import argparse, json, os, struct, zlib

ALIGN = 64

def pad_to(f, align=ALIGN):
    need = (-f.tell()) % align
    if need: f.write(b"\x00"*need)

def collect_entries(args):
    if args.spec:
        spec = json.load(open(args.spec))
        return [(it["name"], it["path"]) for it in spec["layers"]]
    return [pair.split("=",1) for pair in args.pairs]

ap = argparse.ArgumentParser()
ap.add_argument("--spec")
ap.add_argument("pairs", nargs="*")
ap.add_argument("--out-bin", required=True)
ap.add_argument("--out-man", required=True)
ap.add_argument("--man-version", type=int, choices=[1,2], default=2)
args = ap.parse_args()

entries = collect_entries(args)
os.makedirs(os.path.dirname(args.out_bin), exist_ok=True)

records = []
with open(args.out_bin, "wb") as bout:
    for name, path in entries:
        pad_to(bout, ALIGN)
        off = bout.tell()
        with open(path, "rb") as fin:
            data = fin.read()
        bout.write(data)
        crc = (zlib.crc32(data) & 0xffffffff) if args.man_version >= 2 else None
        records.append((name, off, len(data), crc))

with open(args.out_man, "wb") as mout:
    mout.write(b"N64W")
    mout.write(struct.pack("<H", args.man_version))
    mout.write(struct.pack("<H", ALIGN))
    mout.write(struct.pack("<I", len(records)))
    for name, off, size, crc in records:
        nb = name.encode("utf-8")
        mout.write(struct.pack("<H", len(nb))); mout.write(nb)
        mout.write(struct.pack("<I", off));    mout.write(struct.pack("<I", size))
        if args.man_version >= 2:
            mout.write(struct.pack("<I", crc))

