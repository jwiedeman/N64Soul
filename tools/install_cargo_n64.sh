#!/usr/bin/env bash
set -euo pipefail

REPO="https://github.com/rust-console/cargo-n64"

echo "[cargo-n64] Trying upstream main first…"
if cargo +nightly install cargo-n64 --git "$REPO" --branch main --locked; then
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

# Install the patched tool
cargo +nightly install --path . --locked
popd >/dev/null
echo "[cargo-n64] Installed from patched tree."
