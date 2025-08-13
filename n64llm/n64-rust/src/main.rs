#![no_std]
#![no_main]

extern crate alloc;
mod model_weights;
mod n64_math;
mod n64_sys;

use alloc::string::String;
use alloc::vec::Vec;
use core::panic::PanicInfo;
use alloc::format;

mod inference_engine;
mod tokenizer;
mod display;
mod memory_manager;
mod keyboard;

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

    // Chat history limited to a few KB
    let mut history: Vec<String> = Vec::new();
    let mut input_buffer = String::new();
    let mut keyboard = keyboard::OnScreenKeyboard::new();

    loop {
        display::clear();

        for line in &history {
            display::print_line(line);
        }

        display::print_line(&format!("[You] {}", input_buffer));
        keyboard.draw();

        let buttons = unsafe { n64_sys::read_controller(n64_sys::CONTROLLER_1).buttons };
        if keyboard.handle_input(buttons, &mut input_buffer) {
            let user_text = input_buffer.clone();
            push_history(&mut history, format!("[You] {}", user_text));

            display::print_line("Processing...");
            let output_text = {
                let mut tokenizer = tokenizer::Tokenizer::new(&mut memory);
                tokenizer.load_basic_vocab();
                let input_tokens = tokenizer.encode(&user_text);

                let output_tokens = match {
                    let mut engine = inference_engine::ModelState::new(&mut memory);
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
                tokenizer.load_basic_vocab();
                tokenizer.decode(&output_tokens)
            };
            memory.log_usage("post_infer");
            push_history(&mut history, format!("[AI] {}", output_text));
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

fn push_history(history: &mut Vec<String>, line: String) {
    history.push(line);
    while history.iter().map(|s| s.len()).sum::<usize>() > 4096 {
        history.remove(0);
    }
}
