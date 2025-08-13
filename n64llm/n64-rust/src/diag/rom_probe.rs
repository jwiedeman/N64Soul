use crate::config::PROBE_OFFSETS;

pub fn rom_probe(read_from_rom: impl Fn(u64, &mut [u8]) -> bool) {
    let mut buf = [0u8; 64];
    for &off in PROBE_OFFSETS {
        let ok = read_from_rom(off, &mut buf);
        // Draw to screen/log: print offset and first 8 bytes hex
        // If any read fails, print a big red “X” so it’s obvious.
        crate::display::log_probe(off, ok, &buf[..8]);
    }
}
