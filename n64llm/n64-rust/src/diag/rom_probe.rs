use crate::{io::rom_reader::RomReader, config, display};

pub fn run_probe<R: RomReader>(rr: &mut R) {
    display::print_line("=== ROM PROBE ===");
    let mut buf = [0u8; config::PROBE_SAMPLE_BYTES];

    for &off in config::PROBE_OFFSETS {
        // Clear buffer so “unchanged” is obvious in an emulator with null reads.
        buf.fill(0xCC);
        let ok = rr.read(off, &mut buf);
        display::print_probe_result(off, ok, &buf);
    }
}

