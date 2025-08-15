#!/usr/bin/env bash
set -euo pipefail

TOOLCHAIN="+nightly"
REPO="https://github.com/rust-console/cargo-n64"

echo "[cargo-n64] Trying upstream main first…"
if cargo ${TOOLCHAIN} install cargo-n64 --git "${REPO}" --branch main --locked; then
  echo "[cargo-n64] Installed from upstream main."
  exit 0
fi

echo "[cargo-n64] Upstream main failed, applying tiny patch to remove .backtrace()…"
workdir="$(mktemp -d)"
git clone --depth=1 "${REPO}" "${workdir}/cargo-n64"
pushd "${workdir}/cargo-n64" >/dev/null

# 1) Remove the now-useless crate attribute.
perl -0777 -pe 's/#!\[feature\(backtrace\)\]\n?//s' -i src/lib.rs

# 2) Strip the entire `if let Some(backtrace) = error.backtrace() { … }` block.
perl -0777 -pe 's/if\s+let\s+Some\([^)]*\)\s*=\s*error\.backtrace\(\)\s*\{[^}]*\}\s*//s' -i src/lib.rs

# Optional: print the error chain instead (harmless if not present in file layout)
# perl -0777 -pe 's/eprintln!\(\{error\}\);\s*/eprintln!("{error}"); let mut s = error.source(); while let Some(c)=s { eprintln!("caused by: {c}"); s=c.source(); }/s' -i src/lib.rs || true

cargo ${TOOLCHAIN} install --path . --locked
popd >/dev/null
echo "[cargo-n64] Installed from patched tree."

