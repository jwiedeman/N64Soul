//! Double-buffered ROM prefetch for streaming large layers.
//! Safe API; uses async PI DMA under the hood.

#![allow(dead_code)]

#[cfg(target_arch = "mips")]
use crate::platform::pi::{pi_dma_start, pi_dma_wait_idle};
use crate::platform::cart::RomSource;
use crate::weights::weights_rel_to_cart_off;

pub struct Prefetcher<'a, R: RomSource> {
    rom: R,
    cart_off: u64,     // absolute cart offset to the start of the tensor
    len: u64,          // total bytes remaining
    buf_a: &'a mut [u8],
    buf_b: &'a mut [u8],
    cur: usize,        // 0 => A has data, 1 => B has data
    filled: [usize;2], // valid bytes in each buffer
}

impl<'a, R: RomSource> Prefetcher<'a, R> {
    pub fn new(mut rom: R, weights_rel_off: u64, total_len: u64,
               buf_a: &'a mut [u8], buf_b: &'a mut [u8]) -> Self {
        let cart_off = if cfg!(all(target_arch = "mips", not(test))) {
            weights_rel_to_cart_off(weights_rel_off)
        } else {
            weights_rel_off
        };
        // Prime A synchronously so callers can read immediately.
        let first = core::cmp::min(total_len as usize, buf_a.len());
        rom.read_abs(cart_off, &mut buf_a[..first]).unwrap();
        Self { rom, cart_off: cart_off + first as u64, len: total_len - first as u64,
               buf_a, buf_b, cur: 0, filled: [first, 0] }
    }

    /// Begin fetching the next chunk into the "other" buffer (non-blocking kick).
    pub fn prefetch_next(&mut self) {
        if self.len == 0 { return; }
        let tgt = if self.cur == 0 { 1 } else { 0 };
        let buf = if tgt == 0 { &mut self.buf_a } else { &mut self.buf_b };
        let want = core::cmp::min(self.len as usize, buf.len());
        #[cfg(all(target_arch = "mips", not(test)))]
        unsafe {
            let dst = buf.as_mut_ptr();
            let cart_addr = (crate::n64_sys::CART_ROM_BASE as u64 + self.cart_off) as u32;
            pi_dma_start(dst, cart_addr, want as u32);
        }
        #[cfg(any(test, not(target_arch = "mips")))]
        {
            self.rom.read_abs(self.cart_off, &mut buf[..want]).unwrap();
        }
    }

    /// Wait for any in-flight DMA and swap buffers; return filled slice.
    pub fn next_block(&mut self) -> Option<&[u8]> {
        if self.len == 0 && self.filled[self.cur] == 0 { return None; }
        // If a prefetch was kicked, finalize it and update accounting.
        #[cfg(all(target_arch = "mips", not(test)))]
        pi_dma_wait_idle();
        if self.filled[self.cur] == 0 {
            // We were waiting on the other buffer; mark it filled now.
            let tgt = if self.cur == 0 { 1 } else { 0 };
            let got = core::cmp::min(
                self.len as usize,
                if tgt == 0 { self.buf_a.len() } else { self.buf_b.len() }
            );
            self.filled[tgt] = got;
            self.cart_off += got as u64;
            self.len -= got as u64;
            self.cur = tgt;
        }
        // Prepare the other buffer for the next call before borrowing the slice.
        let tgt = if self.cur == 0 { 1 } else { 0 };
        if self.len > 0 {
            self.filled[tgt] = 0; // mark empty while DMA runs
            self.prefetch_next();
        }
        let slice = if self.cur == 0 {
            &self.buf_a[..self.filled[0]]
        } else {
            &self.buf_b[..self.filled[1]]
        };
        // Mark current buffer as consumed so the next call swaps.
        self.filled[self.cur] = 0;
        Some(slice)
    }

pub fn remaining(&self) -> u64 { self.len + self.filled[self.cur] as u64 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::host_cart::VecRom;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn prefetch_reads_all_bytes_in_order() {
        let data: Vec<u8> = (0..200_000).map(|i| i as u8).collect();
        let mut a = vec![0u8; 8192];
        let mut b = vec![0u8; 8192];
        let mut pf = Prefetcher::new(VecRom(data.clone()), 0, data.len() as u64, &mut a, &mut b);
        let mut acc = Vec::new();
        while let Some(chunk) = pf.next_block() { acc.extend_from_slice(chunk); }
        assert_eq!(acc.len(), data.len());
        assert_eq!(&acc[..], &data[..]);
    }
}
