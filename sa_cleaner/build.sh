#!/bin/bash
set -e
echo "=========================================="
echo "   SA Cleaner v1.0 - By Seif Afandi"
echo "   [Resizable | Dark Mode | Threaded]"
echo "=========================================="
for cmd in cargo x86_64-w64-mingw32-g++; do
  if ! command -v $cmd &>/dev/null; then echo "[!] Missing: $cmd"; exit 1; fi
done
if ! rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
  rustup target add x86_64-pc-windows-gnu
fi
export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
export CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++
export AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar
export RUSTFLAGS="-C target-feature=+crt-static"
cargo build --release --target x86_64-pc-windows-gnu
OUT="target/x86_64-pc-windows-gnu/release/sa_cleaner.exe"
if [ -f "$OUT" ]; then
  echo "[+] SUCCESS: $OUT"
  cp "$OUT" "../sa_cleaner.exe" 2>/dev/null || true
  echo "[+] Copied to: sa_cleaner.exe"
else
  echo "[!] Build failed"
fi
