#![no_std]
#![no_main]

extern crate alloc;
mod model_weights;
mod n64_math;
mod n64_sys;

use alloc::string::String;
use core::panic::PanicInfo;
use alloc::format;

mod inference_engine;
mod tokenizer;
mod display;
mod memory_manager;

#[no_mangle]
pub extern "C" fn main() -> ! {
    // Initialize display and console.
    display::init();
    display::clear();
    display::print_line("N64 GPT - Flash-Streamed AI Model");
    display::print_line("Initializing...");

    // Initialize memory management system.
    let mut memory = unsafe { memory_manager::init() };
    display::print_line("Memory manager initialized");
    memory.log_usage("init");

    // Main interactive loop.
    display::print_line("\nEnter text with the controller:");
    let mut input_buffer = String::new();

    loop {
        if let Some(input) = display::read_input() {
            input_buffer.push_str(&input);
            display::print_line(&format!("Input: {}", input_buffer));

            // Check for newline, which signals the end of the input.
            if input == "\n" {
                display::print_line("Processing (this will take a while)...");

                // Use a scoped block to create a tokenizer and engine so that their mutable borrows
                // of `memory` are dropped after processing.
                let output_text = {
                    // Create a tokenizer to encode the input.
                    let mut tokenizer = tokenizer::Tokenizer::new(&mut memory);
                    let input_tokens = tokenizer.encode(&input_buffer);

                    // Create an inference engine instance to run inference.
                    let output_tokens = match {
                        let mut engine = inference_engine::ModelState::new(&mut memory);
                        engine.run_inference(&input_tokens)
                    } {
                        Ok(tokens) => tokens,
                        Err(e) => {
                            display::print_line(&format!("Error: {:?}", e));
                            // On error, clear the input and continue.
                            input_buffer.clear();
                            display::print_line("\nEnter next input:");
                            continue;
                        }
                    };

                    // Create a new tokenizer instance to decode the output tokens.
                    let mut tokenizer = tokenizer::Tokenizer::new(&mut memory);
                    tokenizer.decode(&output_tokens)
                };
                memory.log_usage("post_infer");

                display::print_line(&format!("Output: {}", output_text));

                // Clear the input buffer for the next input.
                input_buffer.clear();
                display::print_line("\nEnter next input:");
            }
        } else {
            delay(1000);
        }
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
