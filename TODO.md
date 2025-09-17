# TODO

- [x] Confirm ROM layout and address mapping in `n64.ld` for `.model_weights`.
- [x] Validate `weights.bin` against `weights.manifest.bin` using `tools/validate_weights.py`.
- [x] Implement efficient layer streaming with checkpoints in `memory_manager.rs`.
- [x] Replace placeholder attention and FFN code with real transformer operations.
- [x] Complete tokenizer and controller input logic in `tokenizer.rs` and `display.rs`.
- [x] Document how to run the project in an emulator and on hardware.
- [x] Add memory usage diagnostics using `available_memory()` at key runtime points.
- [x] Exercise `Error` variants in `inference_engine` to test error handling.
- [x] Ensure `tools/validate_weights.py` is run whenever model weights change.
- [x] Add controller input logging for debugging purposes.
- [x] Enable host-side unit tests for tokenizer and math routines.
- [x] Document the full toolchain setup in the README.

# Completed

- Initial weight streaming system in `inference_engine.rs`.
- Bump allocator with checkpoint/restore in `memory_manager.rs`.
- ROM embedding for model weights via `model_weights.rs`.
