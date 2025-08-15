use crate::{display, io::rom_reader::RomReader, weights_manifest::ManifestView};
use crate::weights::weights_rel_to_cart_off;
use alloc::format;

pub fn manifest_check<R: RomReader>(rr: &mut R, man_bytes: &'static [u8], weights_size: u64) {
    display::print_line("=== MANIFEST CHECK ===");
    match ManifestView::new(man_bytes) {
        Err(_) => {
            display::print_line("Manifest: ERR (parse)");
            return;
        }
        Ok(view) => {
            let mut ok_all = true;
            let mut buf = [0u8; 32];
            let mut idx: u32 = 0;
            let _ = view.for_each(|e| {
                let end = (e.offset as u64) + (e.size as u64);
                let in_bounds = end <= weights_size && (e.offset as u64) % (view.align() as u64) == 0;
                let to_read = core::cmp::min(32u32, e.size) as usize;
                let cart_off = weights_rel_to_cart_off(e.offset as u64);
                let ok_read = if to_read > 0 {
                    rr.read(cart_off, &mut buf[..to_read])
                } else {
                    true
                };
                display::print_line(&format!(
                    "[{idx:02}] {} off={} sz={}  {} {}",
                    e.name,
                    e.offset,
                    e.size,
                    if in_bounds { "BND" } else { "OOB" },
                    if ok_read { "RD" } else { "ERR" }
                ));
                idx += 1;
                ok_all &= in_bounds && ok_read;
                true
            });
            if ok_all {
                display::print_line("Manifest check: OK");
            } else {
                display::print_line("Manifest check: FAIL");
            }
        }
    }
}
