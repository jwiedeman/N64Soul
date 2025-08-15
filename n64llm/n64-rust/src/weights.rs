#![allow(dead_code)]
#[link_section = ".model_weights"]
#[used]
pub static MODEL_WEIGHTS: [u8; { include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),
    "/assets/weights.bin")).len() }] =
    *include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/weights.bin"));

extern "C" {
    static __weights_rom_start: u8;
    static __weights_rom_end: u8;
}
#[inline(always)]
pub fn weights_rom_base() -> u32 {
    crate::n64_sys::CART_ROM_BASE as u32 + unsafe { &__weights_rom_start as *const _ as u32 }
}
#[inline(always)]
pub fn weights_rom_size() -> u64 {
    (unsafe { (&__weights_rom_end as *const _ as usize) - (&__weights_rom_start as *const _ as usize) }) as u64
}
#[inline(always)]
pub fn weights_rel_to_cart_off(rel: u64) -> u64 {
    let abs = weights_rom_base() as u64 + rel;
    abs - crate::n64_sys::CART_ROM_BASE as u64
}
