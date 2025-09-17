use crate::platform::cart::RomSource;
use crate::stream::prefetch::Prefetcher;
use crate::manifest::Manifest;
use core::{ptr::addr_of_mut, slice};

pub struct LayerDesc {
    pub offset: u32,
    pub len: u32,
}

pub enum StreamError {
    Io,
}

pub fn stream_layer<R: RomSource>(
    rom: R,
    layer: &LayerDesc,
    mut consume: impl FnMut(&[u8]) -> (),
    mut on_progress: impl FnMut(u64, u64) -> (),
) -> Result<(), StreamError> {
    // Static 2×32 KiB (tweak in config if you like).
    static mut A: [u8; crate::config::STREAM_BLOCK_BYTES] = [0; crate::config::STREAM_BLOCK_BYTES];
    static mut B: [u8; crate::config::STREAM_BLOCK_BYTES] = [0; crate::config::STREAM_BLOCK_BYTES];
    let pre = {
        #[allow(static_mut_refs)]
        unsafe {
            let buf_a = slice::from_raw_parts_mut(addr_of_mut!(A) as *mut u8, A.len());
            let buf_b = slice::from_raw_parts_mut(addr_of_mut!(B) as *mut u8, B.len());
            Prefetcher::new(rom, layer.offset as u64, layer.len as u64, buf_a, buf_b)
        }
    };
    let mut pf = pre;
    let total = layer.len as u64;
    let mut bursts = 0usize;
    while let Some(chunk) = pf.next_block() {
        consume(chunk);
        bursts += 1;
        if bursts >= crate::config::UI_BURSTS_PER_REFRESH {
            let done = total - pf.remaining();
            on_progress(done, total);
            bursts = 0;
        }
    }
    on_progress(total, total);
    Ok(())
}

pub fn checksum_all_layers<R: RomSource + Copy>(
    rom: R,
    manifest: &Manifest,
) -> Option<u32> {
    let mut crc: u32 = !0u32;
    for layer in &manifest.layers {
        let desc = LayerDesc { offset: layer.offset, len: layer.size };
        if stream_layer(rom, &desc, |chunk| {
            crc = crate::util::crc32::crc32_update(crc, chunk);
        }, |_, _| ()).is_err() {
            return None;
        }
    }
    Some(crate::util::crc32::crc32_finish(crc))
}

#[cfg(test)]
mod t_stream {
    use super::*;
    use crate::platform::host_cart::VecRom;
    // use crate::stream::prefetch::Prefetcher; // not needed in this host-only test
    use alloc::vec::Vec;

    #[test]
    fn stream_reports_progress_and_consumes_all() {
        let data: Vec<u8> = (0..100_000).map(|i| (i as u8)).collect();
        let desc = LayerDesc { offset: 0, len: data.len() as u32 };
        let mut got = Vec::<u8>::new();
        let mut last = (0u64, desc.len as u64);
        let rom = VecRom(data.clone());
        let ok = stream_layer(rom, &desc, |chunk| got.extend_from_slice(chunk), |d,t| { last = (d,t); }).is_ok();
        assert!(ok);
        assert_eq!(got, data);
        assert_eq!(last, (desc.len as u64, desc.len as u64));
    }
}
