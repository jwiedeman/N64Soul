pub fn rom_probe(read_from_rom: impl Fn(u64, &mut [u8]) -> bool) {
    let mut buf = [0u8; 64];
    let probe_offsets = [
        16 * 1024 * 1024u64,          // 16 MiB
        256 * 1024 * 1024u64,         // 256 MiB
        480 * 1024 * 1024u64,         // 480 MiB (below the ~500 MiB limit)
    ];

    for &off in &probe_offsets {
        let ok = read_from_rom(off, &mut buf);
        // Draw to screen/log: print offset and first 8 bytes hex
        // If any read fails, print a big red “X” so it’s obvious.
        crate::display::log_probe(off, ok, &buf[..8]);
    }
}
