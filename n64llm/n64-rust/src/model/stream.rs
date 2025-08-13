use crate::{
    config,
    io::{dbuf::Dbuf, rom_reader::RomReader},
    manifest::Manifest,
    model_weights::{weights_rom_base, weights_rom_size},
};

/// Streams a model layer from ROM using a [`RomReader`] and double buffering.
/// Calls `on_chunk` for each chunk of data read.
/// Returns `false` on any read error.
pub fn stream_layer<R: RomReader, F>(
    rr: &mut R,
    manifest: &Manifest,
    layer_idx: usize,
    mut on_chunk: F,
) -> bool
where
    F: FnMut(&[u8]),
{
    if layer_idx >= manifest.layers.len() {
        return false;
    }

    let layer = &manifest.layers[layer_idx];
    if layer.offset as usize + layer.size as usize > weights_rom_size() {
        return false;
    }
    let mut off = layer.offset as u64 + weights_rom_base() as u64;
    let mut remain = layer.size as u64;

    const BURST: usize = config::BURST_BYTES;
    let mut dbuf: Dbuf<BURST> = Dbuf::new();
    let (mut cur, mut nxt) = dbuf.pair();

    let first = core::cmp::min(remain as usize, BURST);
    if !rr.read(off, &mut cur[..first]) {
        return false;
    }
    off += first as u64;
    remain -= first as u64;
    let mut cur_len = first;

    while remain > 0 {
        let to_read = core::cmp::min(remain as usize, BURST);
        if !rr.read(off, &mut nxt[..to_read]) {
            return false;
        }
        on_chunk(&cur[..cur_len]);
        core::mem::swap(&mut cur, &mut nxt);
        cur_len = to_read;
        off += to_read as u64;
        remain -= to_read as u64;
    }

    on_chunk(&cur[..cur_len]);
    true
}
