# TODO

- [x] Confirm ROM layout and address mapping in `n64.ld` for `.model_weights`.
- [x] Validate `n64_model_weights_reduced.bin` offsets against `LAYER_OFFSETS` and `LAYER_SIZES` in `inference_engine.rs`.
- [x] Implement efficient layer streaming with checkpoints in `memory_manager.rs`.
- [x] Replace placeholder attention and FFN code with real transformer operations.
- [x] Complete tokenizer and controller input logic in `tokenizer.rs` and `display.rs`.
- [x] Document how to run the project in an emulator and on hardware.

# Completed

- Initial weight streaming system in `inference_engine.rs`.
- Bump allocator with checkpoint/restore in `memory_manager.rs`.
- ROM embedding for model weights via `model_weights.rs`.
