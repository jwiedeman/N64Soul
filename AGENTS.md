# Repository Guidelines for Agents

This repository contains experiments for running a language model on Nintendo 64 hardware. The project is split into two directories:

- `n64llm/n64-rust/` – Rust implementation with weight streaming and inference engine.

## Development Notes

- New work should go into `n64llm/n64-rust`.
- Keep everything `no_std` and target `mips-nintendo64-none`.
- Building with the `embed_assets` feature runs the exporter so fresh weights land at ROM address `0x10000000` via `model_weights.rs`.

## Testing

Building requires a custom toolchain. If unavailable, document build steps rather than compiling.

## TODO Tracking

See `TODO.md` for open tasks and completed milestones.
