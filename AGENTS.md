# Repository Guidelines for Agents

This repository contains experiments for running a language model on Nintendo 64 hardware. The project is split into two directories:

- `n64llm/` – Rust implementation with weight streaming and inference engine.
- `n64_ai_project/` – Minimal C examples.

## Development Notes

- Prefer Rust code in `n64llm/n64-rust` for new features.
- When adding code, keep everything `no_std` and target `mips-nintendo64-none`.
- Model weights are linked at ROM address `0x10000000` via `model_weights.rs`.

## Testing

Building requires a custom toolchain. If unavailable, document build steps rather than compiling.

## TODO Tracking

See `TODO.md` for open tasks and completed milestones.
