#![allow(dead_code)]
#[cfg(feature = "embed_assets")]
#[link_section = ".model_weights"]
#[used]
pub static MODEL_WEIGHTS: [u8; { include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),
    "/assets/weights.bin")).len() }] =
    *include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/weights.bin"));

// Use the embedded static directly instead of linker symbols
#[cfg(feature = "embed_assets")]
#[inline(always)]
pub fn weights_rom_base() -> u32 {
    // Get the address of MODEL_WEIGHTS as a ROM address
    // The static is in ROM so we just need its address relative to cart base
    MODEL_WEIGHTS.as_ptr() as u32
}
#[cfg(feature = "embed_assets")]
#[inline(always)]
pub fn weights_rom_size() -> u64 {
    MODEL_WEIGHTS.len() as u64
}
#[cfg(feature = "embed_assets")]
#[inline(always)]
pub fn weights_rel_to_cart_off(rel: u64) -> u64 {
    let abs = weights_rom_base() as u64 + rel;
    abs - crate::n64_sys::CART_ROM_BASE as u64
}
#[cfg(not(feature = "embed_assets"))]
#[inline(always)]
pub fn weights_rom_base() -> u32 { 0 }
#[cfg(not(feature = "embed_assets"))]
#[inline(always)]
pub fn weights_rom_size() -> u64 { 0 }
#[cfg(not(feature = "embed_assets"))]
#[inline(always)]
pub fn weights_rel_to_cart_off(rel: u64) -> u64 { rel }
