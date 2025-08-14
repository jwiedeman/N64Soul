#![no_std]
#![no_main]

extern crate alloc;
mod config;
mod ipl3;
mod n64_math;
mod n64_sys;
mod platform;
mod weights;
mod weights_manifest;
mod weights_manifest_find;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::panic::PanicInfo;

mod diag;
mod display;
mod inference_engine;
mod io;
mod manifest;
mod memory_manager;
mod model;
mod tokenizer;
mod util;
mod infer;
mod stream;

#[no_mangle]
pub extern "C" fn main() -> ! {
    // Initialize display and console.
    display::init();
    display::clear();
    display::print_line("N64 GPT - Flash-Streamed AI Model");
    display::print_line("Initializing...");

    display::print_line("Probing ROM...");
    let mut rr = io::rom_reader::FlatRomReader::new();
    diag::rom_probe::run_probe(&mut rr);
    diag::weights_info::show_weights_info(&mut rr);

    diag::manifest_check::manifest_check(
        &mut rr,
        &weights_manifest::MODEL_MANIFEST,
        weights::weights_rom_size(),
    );

    diag::stream_bench::run(&mut rr, &crate::weights_manifest::MODEL_MANIFEST);
    diag::decode_once::run(&mut rr, &crate::weights_manifest::MODEL_MANIFEST, 42);
    wait_for_start_button();

    let manifest = manifest::load();
    display::print_line(&format!("Manifest layers: {}", manifest.layers.len()));

    display::print_line("Running ROM checksum...");
    match model::stream::checksum_all_layers(&mut rr, &manifest) {
        Some(sum) => display::print_line(&format!("Checksum: 0x{:08X}", sum)),
        None => display::print_line("Checksum failed"),
    }

    // Initialize memory management system.
    let mut memory = unsafe { memory_manager::init() };
    display::print_line("Memory manager initialized");
    memory.log_usage("init");

    // Main interactive loop with on-screen keyboard.
    let mut input_buffer = String::new();
    let mut history: Vec<String> = Vec::new();
    display::print_line("\nUse the on-screen keyboard. Start to submit.");

    loop {
        if display::keyboard_input(&mut input_buffer) {
            history.push(format!("[You] {}", input_buffer));
            display::print_line("Working...");

            let output_text = {
                let mut tokenizer = tokenizer::Tokenizer::new(&mut memory);
                let input_tokens = tokenizer.encode(&input_buffer);

                let output_tokens = match {
                    let mut engine = inference_engine::ModelState::new(&mut memory, &manifest);
                    engine.run_inference(&input_tokens)
                } {
                    Ok(tokens) => tokens,
                    Err(e) => {
                        display::print_line(&format!("Error: {:?}", e));
                        input_buffer.clear();
                        continue;
                    }
                };

                let mut tokenizer = tokenizer::Tokenizer::new(&mut memory);
                tokenizer.decode(&output_tokens)
            };
            memory.log_usage("post_infer");

            history.push(format!("[AI] {}", output_text));
            // Limit history to a few KB to stay within memory limits.
            let mut total: usize = history.iter().map(|s| s.len()).sum();
            while total > 4096 {
                if let Some(first) = history.first() {
                    total -= first.len();
                }
                history.remove(0);
            }

            display::clear();
            for line in &history {
                display::print_line(line);
            }
            input_buffer.clear();
        }

        delay(1000);
    }
}

fn wait_for_start_button() {
    // Controller polling not implemented; placeholder for hardware pause.
}

fn delay(ms: u32) {
    let mut i = 0;
    while i < ms * 1000 {
        i += 1;
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    display::print_line("PANIC: System error occurred");
    loop {}
}
