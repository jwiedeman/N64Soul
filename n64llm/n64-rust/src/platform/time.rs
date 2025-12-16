//! CP0 Count increments at ~CPU/2. Common N64 value ~46_875_000 Hz (93.75 MHz / 2).
//! Adjust COUNT_HZ if your environment differs.

#![allow(dead_code)]
pub const COUNT_HZ: u64 = 46_875_000;

#[inline(always)]
#[cfg(target_arch = "mips")]
pub fn now_cycles() -> u64 {
    let v: u32;
    unsafe { core::arch::asm!("mfc0 {0}, $9", out(reg) v) };
    v as u64 // 32-bit; fine for short intervals
}
#[inline(always)]
#[cfg(not(target_arch = "mips"))]
pub fn now_cycles() -> u64 { 0 }

#[inline(always)]
pub fn cycles_to_us(cycles: u64) -> u64 {
    (cycles * 1_000_000) / COUNT_HZ
}

pub struct Stopwatch { start: u64, acc: u64 }

impl Stopwatch {
    pub fn new() -> Self { Self { start: 0, acc: 0 } }
    pub fn start(&mut self) { self.start = now_cycles(); }
    pub fn stop_add(&mut self) { self.acc += now_cycles().saturating_sub(self.start); }
    pub fn cycles(&self) -> u64 { self.acc }
    pub fn micros(&self) -> u64 { cycles_to_us(self.acc) }
    pub fn reset(&mut self) { self.acc = 0; }
}

