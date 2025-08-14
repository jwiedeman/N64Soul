import argparse, os, struct, sys


def read_exact(f, n):
    b = f.read(n)
    if len(b) != n:
        raise EOFError
    return b

def parse_manifest(path):
    with open(path, "rb") as f:
        magic = read_exact(f, 4)
        if magic != b"N64W":
            raise SystemExit("Bad magic")
        ver, align = struct.unpack("<HH", read_exact(f, 4))
        if ver != 1:
            raise SystemExit(f"Bad version: {ver}")
        count, = struct.unpack("<I", read_exact(f, 4))
        entries = []
        for _ in range(count):
            nlen, = struct.unpack("<H", read_exact(f, 2))
            name = read_exact(f, nlen).decode("utf-8", "replace")
            off, size = struct.unpack("<II", read_exact(f, 8))
            entries.append((name, off, size))
        return align, entries

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bin", required=True)
    ap.add_argument("--man", required=True)
    args = ap.parse_args()

    size = os.path.getsize(args.bin)
    align, entries = parse_manifest(args.man)

    used = []
    for name, off, sz in entries:
        if off % align != 0:
            print(f"[FAIL] {name}: offset {off} not aligned to {align}")
            sys.exit(2)
        if off + sz > size:
            print(f"[FAIL] {name}: out of bounds ({off}+{sz}>{size})")
            sys.exit(2)
        used.append((off, off + sz, name))
    used.sort()
    for (s1, e1, n1), (s2, e2, n2) in zip(used, used[1:]):
        if s2 < e1:
            print(f"[FAIL] overlap: {n1} [{s1},{e1}) with {n2} [{s2},{e2})")
            sys.exit(2)

    print(f"[OK] {len(entries)} entries; bin={size} bytes; align={align}")

if __name__ == "__main__":
    main()
