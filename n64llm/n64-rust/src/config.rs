#![allow(dead_code)]

pub const BURST_BYTES: usize = 32 * 1024; // Try 16K/32K/64K later
pub const ROM_ALIGN: usize = 64;          // Exporter enforces; reader asserts
pub const BENCH_MAX_BYTES_PER_ENTRY: u32 = 4 * 1024 * 1024; // cap per entry for quick bench
pub const PROBE_OFFSETS: &[u64] = &[
    16  * 1024 * 1024,   // 16 MiB (sanity)
    128 * 1024 * 1024,   // 128 MiB
    256 * 1024 * 1024,   // 256 MiB
    512 * 1024 * 1024,   // 512 MiB
    768 * 1024 * 1024,   // 768 MiB
    960 * 1024 * 1024,   // 960 MiB
    1023 * 1024 * 1024,  // near 1 GiB end
];

pub const PROBE_SAMPLE_BYTES: usize = 64;     // small, quick sanity read
pub const ENABLE_DOUBLE_BUFFER: bool = true;
pub const UI_BURSTS_PER_REFRESH: usize = 4;
// Maximum ROM bytes to treat as readable (e.g., firmware cap)
pub const ROM_LIMIT_BYTES: u64 = 1024 * 1024 * 1024;
