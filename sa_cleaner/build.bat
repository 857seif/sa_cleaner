@echo off
echo SA Cleaner v1.0 - By Seif Afandi
echo [Resizable | Dark Mode | Threaded]
set RUSTFLAGS=-C target-feature=+crt-static
cargo build --release
if errorlevel 1 exit /b 1
echo [+] Done: target/release/sa_cleaner.exe
pause
