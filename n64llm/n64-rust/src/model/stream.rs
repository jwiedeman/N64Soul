use crate::platform::cart::RomSource;
use crate::stream::prefetch::Prefetcher;
use crate::manifest::Manifest;

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
    let pre = unsafe { Prefetcher::new(rom, layer.offset as u64, layer.len as u64, &mut A, &mut B) };
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
