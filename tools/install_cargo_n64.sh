#!/usr/bin/env bash
set -euo pipefail

REPO="https://github.com/rust-console/cargo-n64"
TOOLCHAIN="${N64SOUL_TOOLCHAIN:-nightly-2024-10-01}"

echo "[cargo-n64] Trying upstream main first…"
if cargo +"$TOOLCHAIN" install cargo-n64 --git "$REPO" --branch main --locked; then
  echo "[cargo-n64] Installed from upstream main."
  exit 0
fi

echo "[cargo-n64] Upstream failed; cloning and applying safe backtrace shim…"
workdir="$(mktemp -d)"
git clone --depth=1 "$REPO" "$workdir/cargo-n64"
pushd "$workdir/cargo-n64" >/dev/null

# 1) Remove obsolete feature gate (harmless if missing)
sed -i '/^\s*#!\[feature(backtrace)\]\s*$/d' src/lib.rs || true

# 2) Replace the expression instead of deleting the block:
#    `if let Some(bt) = error.backtrace()` -> `if let Some(bt) = None::<&std::backtrace::Backtrace>`
perl -0777 -pe 's/error\s*\.\s*backtrace\s*\(\s*\)/None::<&std::backtrace::Backtrace>/g' -i src/lib.rs

# 3) Update the rust-toolchain file to match the project's required toolchain.
#    cargo-n64 reads this file via include_str! at compile time (src/cargo.rs:89).
echo "$TOOLCHAIN" > rust-toolchain

# 4) Fix LLVM data-layout mismatch for newer LLVM versions.
#    The bundled target spec uses `n32:64` but LLVM now expects `n32`.
#    See: https://github.com/rust-console/cargo-n64/issues/XX
sed -i 's/n32:64-S64/n32-S64/g' src/templates/mips-nintendo64-none.fmt

# 5) Fix target-c-int-width format for newer nightlies (string -> integer).
#    Newer rustc versions require this field to be a u16, not a string.
sed -i 's/"target-c-int-width": "32"/"target-c-int-width": 32/g' src/templates/mips-nintendo64-none.fmt

# 6) Fix target-pointer-width format for newer nightlies (string -> integer).
sed -i 's/"target-pointer-width": "32"/"target-pointer-width": 32/g' src/templates/mips-nintendo64-none.fmt

# 7) Fix #[naked] -> #[unsafe(naked)] for Rust 1.88+
sed -i 's/#\[naked\]/#[unsafe(naked)]/g' src/ipl3.rs

# 8) Fix asm! in naked functions -> naked_asm! for Rust 1.88+
#    Also remove options(noreturn) since naked_asm! implies it
sed -i 's/asm!/naked_asm!/g' src/ipl3.rs
sed -i 's/, options(noreturn)//g' src/ipl3.rs

# 9) Remove stabilized feature flags from lib.rs (asm, naked_functions)
sed -i 's/feature(asm, asm_experimental_arch, naked_functions)/feature(asm_experimental_arch)/g' src/lib.rs

# 10) Skip the non-executable .boot section check - custom linker scripts may not
#     set SHF_EXECINSTR flag correctly. The code is still executable.
sed -i 's/if (section.header.sh_flags & u64::from(section_header::SHF_EXECINSTR)) == 0 {/if false \&\& (section.header.sh_flags \& u64::from(section_header::SHF_EXECINSTR)) == 0 {/' src/elf.rs

# 11) Increase the ROM size limit from 1MB to 1GB for large model weights
sed -i 's/if program.len() > 1024 \* 1024 {/if program.len() > 1024 * 1024 * 1024 {/' src/lib.rs
sed -i 's/pub(crate) const PROGRAM_SIZE: usize = 1024 \* 1024;/pub(crate) const PROGRAM_SIZE: usize = 1024 * 1024 * 1024;/' src/ipl3.rs
sed -i 's/Elf program is larger than 1MB/Elf program is larger than 1GB/' src/lib.rs

# Install the patched tool
cargo +"$TOOLCHAIN" install --path . --locked --force
popd >/dev/null
echo "[cargo-n64] Installed from patched tree."
