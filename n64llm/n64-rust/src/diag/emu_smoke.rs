#![allow(dead_code)]

use crate::platform::cart::RomSource;
use crate::manifest;
use crate::stream::prefetch::Prefetcher;
use crate::display::{show_progress, print_line};
use crate::util::crc32::{crc32_update, crc32_finish};
use alloc::format;

pub fn run<R: RomSource + Copy>(rom: R) {
    let man = manifest::load();

    let mut total_bytes: u64 = 0;
    for l in &man.layers { total_bytes += l.size as u64; }
    print_line("EMU SMOKE: streaming all layers");
    show_progress(0, total_bytes as usize);

    let mut crc: u32 = !0;
    let mut done: u64 = 0;

    for (i, l) in man.layers.iter().enumerate() {
        // 2×32KiB static buffers already set in streamer config.
        static mut A: [u8; crate::config::STREAM_BLOCK_BYTES] = [0; crate::config::STREAM_BLOCK_BYTES];
        static mut B: [u8; crate::config::STREAM_BLOCK_BYTES] = [0; crate::config::STREAM_BLOCK_BYTES];
        let mut pf = unsafe { Prefetcher::new(rom, l.offset as u64, l.size as u64, &mut A, &mut B) };
        while let Some(chunk) = pf.next_block() {
            crc = crc32_update(crc, chunk);
            done += chunk.len() as u64;
            show_progress(done as usize, total_bytes as usize);
        }
        print_line(&format!("Layer {} OK", i));
    }
    let crc = crc32_finish(crc);
    print_line(&format!("EMU SMOKE done, CRC32 = 0x{:08X}", crc));
}
