use crate::io::rom_reader::RomReader;
use crate::io::dbuf::Dbuf;
use crate::platform::time::Stopwatch;
use crate::config::{BURST_BYTES, ROM_ALIGN};

pub struct StreamStats {
    pub bytes: u64,
    pub bursts: u32,
    pub dma_us: u64,
    pub compute_us: u64,
}

pub fn stream_entry<R, F>(
    rr: &mut R,
    entry_offset: u64,  // weights-relative
    entry_size: u64,
    mut on_chunk: F,     // compute callback: fn(&[u8])
) -> Option<StreamStats>
where
    R: RomReader,
    F: FnMut(&[u8]),
{
    if BURST_BYTES == 0 { return None; }

    let mut stats = StreamStats { bytes: 0, bursts: 0, dma_us: 0, compute_us: 0 };
    let mut remain = entry_size;
    let mut off    = entry_offset;

    let mut dbuf: Dbuf<{ BURST_BYTES }> = Dbuf::new();
    let mut dma_sw = Stopwatch::new();
    let mut cmp_sw = Stopwatch::new();

    // Helper: read exactly `len` bytes into buf[0..len]
    let mut read_burst = |rom_off: u64, buf: &mut [u8], len: usize| -> bool {
        // Enforce minimum alignment at the start; head/tail alignment handled in pi layer if needed.
        debug_assert!(rom_off % (ROM_ALIGN as u64) == 0 || len < ROM_ALIGN);
        dma_sw.start();
        let ok = rr.read(rom_off, &mut buf[..len]);
        dma_sw.stop_add();
        ok
    };

    // Prime first buffer
    let mut first = core::cmp::min(BURST_BYTES as u64, remain) as usize;
    if first == 0 { return Some(stats); }
    if !read_burst(off, dbuf.cur_mut(), first) { return None; }
    off += first as u64;
    remain -= first as u64;

    while remain > 0 {
        let next_len = core::cmp::min(BURST_BYTES as u64, remain) as usize;
        // Start next DMA into nxt
        if !read_burst(off, dbuf.nxt_mut(), next_len) { return None; }

        // Compute on current
        cmp_sw.start();
        on_chunk(&dbuf.cur_mut()[..first]);
        cmp_sw.stop_add();

        // Bookkeeping & swap
        stats.bytes += first as u64;
        stats.bursts += 1;
        dbuf.swap();
        first = next_len;
        off += next_len as u64;
        remain -= next_len as u64;
    }

    // Drain last
    cmp_sw.start();
    on_chunk(&dbuf.cur_mut()[..first]);
    cmp_sw.stop_add();
    stats.bytes += first as u64;
    stats.bursts += 1;

    stats.dma_us = dma_sw.micros();
    stats.compute_us = cmp_sw.micros();
    Some(stats)
}

