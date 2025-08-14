import argparse, json, os, struct, sys

ALIGN = 64

def pad_to(f, align=ALIGN):
    pos = f.tell()
    need = (-pos) % align
    if need:
        f.write(b"\x00" * need)
    return need

def collect_entries(args):
    entries = []
    if args.spec:
        spec = json.load(open(args.spec, "r"))
        for item in spec["layers"]:
            entries.append((item["name"], item["path"]))
    else:
        for pair in args.pairs:
            if "=" not in pair:
                raise SystemExit(f"Bad pair: {pair}")
            name, path = pair.split("=", 1)
            entries.append((name, path))
    return entries

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--spec", help="JSON spec with {'layers':[{'name','path'},...]}")
    ap.add_argument("pairs", nargs="*", help="name=path entries if no --spec")
    ap.add_argument("--out-bin", required=True)
    ap.add_argument("--out-man", required=True)
    args = ap.parse_args()

    entries = collect_entries(args)
    os.makedirs(os.path.dirname(args.out_bin), exist_ok=True)

    manifest_recs = []
    with open(args.out_bin, "wb") as bout:
        for name, path in entries:
            pad_to(bout, ALIGN)
            off = bout.tell()
            with open(path, "rb") as fin:
                data = fin.read()
            bout.write(data)
            manifest_recs.append((name, off, len(data)))

    with open(args.out_man, "wb") as mout:
        mout.write(b"N64W")
        mout.write(struct.pack("<H", 1))
        mout.write(struct.pack("<H", ALIGN))
        mout.write(struct.pack("<I", len(entries)))
        for name, off, size in manifest_recs:
            nb = name.encode("utf-8")
            mout.write(struct.pack("<H", len(nb)))
            mout.write(nb)
            mout.write(struct.pack("<I", off))
            mout.write(struct.pack("<I", size))

if __name__ == "__main__":
    main()
