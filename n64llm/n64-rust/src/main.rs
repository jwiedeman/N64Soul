#![no_std]
#![no_main]

extern crate alloc;
mod config;
mod ipl3;
mod model_weights;
mod n64_math;
mod n64_sys;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::panic::PanicInfo;

mod diag;
mod display;
mod inference_engine;
mod io;
mod memory_manager;
mod tokenizer;
mod manifest;
mod model;

#[no_mangle]
pub extern "C" fn main() -> ! {
    // Initialize display and console.
    display::init();
    display::clear();
    display::print_line("N64 GPT - Flash-Streamed AI Model");
    display::print_line("Initializing...");

    display::print_line("Probing ROM...");
    diag::rom_probe::rom_probe(|off, buf| {
        unsafe {
            n64_sys::pi_read(
                buf.as_mut_ptr(),
                (n64_sys::CART_ROM_BASE as u32).wrapping_add(off as u32),
                buf.len() as u32,
            );
        }
        true
    });

    let manifest = manifest::load();
    display::print_line(&format!("Manifest layers: {}", manifest.layers.len()));

    display::print_line("Running ROM checksum...");
    let mut rr = io::rom_reader::FlatRomReader;
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
