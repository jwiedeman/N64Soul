#![allow(dead_code)]

pub const BURST_BYTES: usize = 32 * 1024; // Try 16K, 32K, 64K
pub const ROM_ALIGN: usize = 64;          // Exporter enforces; reader asserts
pub const PROBE_OFFSETS: &[u64] = &[
    16 * 1024 * 1024,
    256 * 1024 * 1024,
    480 * 1024 * 1024,
];
pub const ENABLE_DOUBLE_BUFFER: bool = true;
pub const UI_BURSTS_PER_REFRESH: usize = 4;
// Maximum ROM bytes to treat as readable (e.g., firmware cap)
pub const ROM_LIMIT_BYTES: u64 = 480 * 1024 * 1024;
