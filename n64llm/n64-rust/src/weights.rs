#![allow(dead_code)]

/// Linker-provided ROM symbols (absolute cart addresses).
extern "C" {
    static __model_weights_rom_start: u8;
    static __model_weights_rom_end: u8;
    static __model_weights_rom_size: u8; // value-only, but we'll compute size below
}

/// Return absolute ROM bus address of the weights section start.
#[inline(always)]
pub fn weights_rom_base() -> u64 {
    unsafe { &__model_weights_rom_start as *const u8 as u64 }
}

/// Return the byte size of the weights section.
#[inline(always)]
pub fn weights_rom_size() -> u64 {
    unsafe {
        let start = &__model_weights_rom_start as *const u8 as u64;
        let end = &__model_weights_rom_end as *const u8 as u64;
        end - start
    }
}

/// Convenience: convert a weights-relative offset to a cart-space absolute offset
/// suitable for PI reads (0-based within the cart region expected by pi_dma_read()).
#[inline(always)]
pub fn weights_rel_to_cart_off(rel: u64) -> u64 {
    // Our PI layer expects offsets relative to the cart base (0x1000_0000).
    // If your pi_dma_read() instead wants a full KSEG1 address, adjust there.
    let cart_base = crate::platform::pi::CART_BASE;
    let abs_addr = weights_rom_base() + rel;
    abs_addr - cart_base
}

/// The actual blob embedded in the `.model_weights` section.
/// We keep this tiny placeholder in tree; provide a real `assets/weights.bin`
/// locally to replace it during builds.
#[link_section = ".model_weights"]
#[used] // ensure the linker keeps it
pub static MODEL_WEIGHTS: [u8; 64] = [0; 64];
