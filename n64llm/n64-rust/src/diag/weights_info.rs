use crate::weights::{weights_rel_to_cart_off, weights_rom_base, weights_rom_size};
use crate::{display, io::rom_reader::RomReader};
use alloc::format;

pub fn show_weights_info<R: RomReader>(rr: &mut R) {
    let base = weights_rom_base();
    let size = weights_rom_size();
    let mut buf = [0u8; 64];

    display::print_line("=== WEIGHTS INFO ===");

    // Read first 64 bytes
    {
        let ok = rr.read(weights_rel_to_cart_off(0), &mut buf);
        display::print_line(&format!("base=0x{base:08X} size={} bytes", size));
        display::print_line(&format!(
            "first[0..8]: {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
            buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7]
        ));
        if !ok {
            display::print_line("FIRST READ: ERR");
            return;
        }
    }

    // Read last 64 bytes (guard zero-sized case)
    if size >= 64 {
        let off = size - 64;
        let ok = rr.read(weights_rel_to_cart_off(off), &mut buf);
        display::print_line(&format!(
            "last@+0x{off:08X}..: {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
            buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7]
        ));
        if !ok {
            display::print_line("LAST READ: ERR");
        }
    }
}
