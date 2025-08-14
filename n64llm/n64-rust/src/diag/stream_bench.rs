use crate::weights_manifest::ManifestView;
use crate::weights::{weights_rom_size, weights_rel_to_cart_off};
use crate::io::rom_reader::RomReader;
use crate::stream::streamer::stream_entry;
use crate::display;
use crate::config::BENCH_MAX_BYTES_PER_ENTRY;
use crate::util::adler32;

pub fn run<R: RomReader>(rr: &mut R, man_bytes: &'static [u8]) {
    display::print_line("=== STREAM BENCH ===");
    let total_weights = weights_rom_size();

    let view = match ManifestView::new(man_bytes) {
        Ok(v) => v,
        Err(_) => { display::print_line("Manifest parse ERR"); return; }
    };

    let mut idx = 0u32;
    let _ = view.for_each(|e| {
        let mut to_read = e.size as u64;
        if BENCH_MAX_BYTES_PER_ENTRY > 0 {
            to_read = core::cmp::min(to_read, BENCH_MAX_BYTES_PER_ENTRY as u64);
        }
        let end = (e.offset as u64).saturating_add(to_read);
        let in_bounds = end <= total_weights;

        // Adler32 over streamed data (as stand-in for compute)
        let mut s1: u32 = 1; let mut s2: u32 = 0;
        let cart_off = weights_rel_to_cart_off(e.offset as u64);

        let stats = if in_bounds {
            stream_entry(rr, cart_off, to_read, |chunk| {
                (s1, s2) = adler32::adler32(s1, s2, chunk);
            })
        } else { None };

        match stats {
            None => display::print_line(&format!("[{idx:02}] {}  FAIL", e.name)),
            Some(st) => {
                let kbps = if st.dma_us > 0 {
                    // KiB/s = bytes / us * 1e6 / 1024
                    ((st.bytes as u128) * 1_000_000u128 / (st.dma_us as u128) / 1024u128) as u64
                } else { 0 };
                let dma_pct = if (st.dma_us + st.compute_us) > 0 {
                    (st.dma_us * 100) / (st.dma_us + st.compute_us)
                } else { 0 };
                display::print_line(&format!(
                    "[{idx:02}] {:16} rd={} KiB  bursts={}  DMA={}ms  CMP={}ms  BW={} KiB/s  DMA%={}  A32={:08X}:{:08X}",
                    e.name,
                    st.bytes / 1024,
                    st.bursts,
                    st.dma_us / 1000,
                    st.compute_us / 1000,
                    kbps,
                    dma_pct,
                    s1, s2
                ));
            }
        }
        idx += 1;
        true
    });
}

