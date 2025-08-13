# Golden Vector Test

This emulator-only test computes a checksum of the AI kernel output so that
hardware and emulator runs can be compared byte-for-byte.

Build on the host:

```
cc checksum.c ../../src/ai_kernel.c ../../src/console.c ../../src/dma.c ../../src/asset_stub.c -I../../include -o gv_test
./gv_test
```
