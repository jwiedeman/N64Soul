#![no_std]

/// This static variable embeds the entire model weights binary into the ROM image.
/// The `#[link_section = ".model_weights"]` attribute tells the linker to place
/// this data into the `.model_weights` section, which should be mapped to the desired
/// ROM address in your linker script (e.g. 0x10000000).
#[link_section = ".model_weights"]
pub static MODEL_WEIGHTS: &[u8] = include_bytes!("n64_model_weights_reduced.bin");