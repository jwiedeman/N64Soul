# Emulator bring-up guide

The ROM spends its first few seconds probing the cartridge mapping, verifying
CRC checksums, and measuring sustained PI bandwidth before dropping into the
controller-driven keyboard loop. The guidance below helps you configure common
emulators so the diagnostics complete at full speed and the input UI is usable.

## Memory and timing

- Enable the Expansion Pak (8&nbsp;MiB RDRAM). The runtime allocator now reserves
  a 4&nbsp;MiB heap to profile inference allocations, so the default 4&nbsp;MiB layout
  is no longer sufficient.
- Leave PI timing at the default "instant"/"default" setting. The code issues
  32&nbsp;KiB DMA bursts back-to-back; throttled PI modes can make the diagnostics
  appear to hang for tens of seconds.
- Keep the CPU core in interpreter or cached interpreter mode when debugging.
  Aggressive dynarec settings sometimes skip the tight polling loops that gate
  the diagnostics and can hide early failures.

## Controller mapping

The UI only reads the D-Pad, `A`, `B`, and `START` buttons. Map them to
convenient keys so you can operate the on-screen keyboard:

- **D-Pad** &rarr; move the selection cursor.
- **A** &rarr; commit the highlighted character to the prompt buffer.
- **B** &rarr; delete the last character.
- **START** &rarr; submit the prompt to the tokenizer/inference engine.

No analog stick input is required; you can map it to a neutral key if the
emulator insists.

## Quick smoke test

1. Build the ROM with embedded weights (for example by running
   `./scripts/export_and_test.sh`).
2. Launch `scripts/emu_smoke.sh`; it verifies the assets, rebuilds if no ROM is
   present, and attempts to start `ares` or `mupen64plus`. If neither emulator is
   on your `PATH` the script prints the ROM path so you can open it manually.
3. Watch the boot log. You should see `[diag]` entries for the manifest probe,
   CRC sweep, and bandwidth benchmark before the keyboard UI appears.

If you need to re-run the diagnostics after the ROM has booted, reset the
emulator. Hot-resetting is enough; the ROM will redo the probe/CRC sequence on
startup.
