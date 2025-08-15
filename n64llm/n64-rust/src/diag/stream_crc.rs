use crate::{display, io::rom_reader::RomReader, weights_manifest::ManifestView};
use crate::weights::weights_rel_to_cart_off;
use crate::stream::streamer::stream_entry;
use crate::util::crc32;
use alloc::format;

pub fn run<R: RomReader>(rr: &mut R, man_bytes: &'static [u8]) {
    display::print_line("=== STREAM CRC ===");
    let view = match ManifestView::new(man_bytes) {
        Ok(v) => v,
        Err(_) => { display::print_line("Manifest parse ERR"); return; }
    };
    let mut final_crc: u32 = !0u32;
    let mut idx: u32 = 0;
    let ver = view.version();
    let _ = view.for_each(|e| {
        let cart_off = weights_rel_to_cart_off(e.offset as u64);
        let mut crc: u32 = !0u32;
        let _ = stream_entry(rr, cart_off, e.size as u64, |chunk| {
            crc = crc32::crc32_update(crc, chunk);
            final_crc = crc32::crc32_update(final_crc, chunk);
        });
        if ver >= 2 {
            let got = crc32::crc32_finish(crc);
            match e.crc32 {
                Some(exp) if got == exp => {
                    display::print_line(&format!("[{idx:02}] {:16} CRC={:08X} verified \u{2713}", e.name, got));
                }
                Some(exp) => {
                    display::print_line(&format!("[{idx:02}] {:16} CRC={:08X} expected={:08X} CRC mismatch", e.name, got, exp));
                }
                None => {
                    display::print_line(&format!("[{idx:02}] {:16} CRC={:08X} (no ref)", e.name, got));
                }
            }
        }
        idx += 1;
        true
    });
    let final_crc = crc32::crc32_finish(final_crc);
    display::print_line(&format!("Final CRC32 {:08X}", final_crc));
}
