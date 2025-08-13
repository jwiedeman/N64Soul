#![no_std]

#[link_section = ".model_weights"]
#[no_mangle]
#[used]
pub static MODEL_WEIGHTS: [u8; include_bytes!("../../assets/weights.bin").len()] =
    *include_bytes!("../../assets/weights.bin");

extern "C" {
    static __model_weights_rom_start: u8;
    static __model_weights_rom_end: u8;
}

#[inline(always)]
pub fn weights_rom_base() -> usize {
    unsafe { &__model_weights_rom_start as *const u8 as usize }
}

#[inline(always)]
pub fn weights_rom_size() -> usize {
    unsafe {
        (&__model_weights_rom_end as *const u8 as usize)
            - (&__model_weights_rom_start as *const u8 as usize)
    }
}
