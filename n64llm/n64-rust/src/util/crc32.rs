// Simple bitwise CRC32 (poly 0xEDB88320); fast enough vs PI/compute.
pub fn crc32_update(mut crc: u32, buf: &[u8]) -> u32 {
    // streaming-friendly: pass crc from previous call; init with !0
    for &b in buf {
        let mut x = (crc ^ (b as u32)) & 0xFF;
        for _ in 0..8 {
            let lsb = x & 1;
            x >>= 1;
            if lsb != 0 { x ^= 0xEDB88320; }
        }
        crc = (crc >> 8) ^ x;
    }
    crc
}
pub fn crc32_finish(crc: u32) -> u32 { !crc }

#[cfg(test)]
mod t_crc32 {
    use super::*;

    #[test]
    fn crc32_empty() { assert_eq!(crc32_finish(crc32_update(!0, &[])), 0); }
    #[test]
    fn crc32_abc() {
        let c = crc32_finish(crc32_update(!0, b"123456789"));
        assert_eq!(c, 0xCBF43926);
    }
}
