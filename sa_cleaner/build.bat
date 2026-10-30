@echo off
echo ========================================
echo   SA Cleaner v2.0 - Advanced Build
echo ========================================
set RUSTFLAGS=-C target-feature=+crt-static
cargo build --release
if errorlevel 1 (
    echo [!] Build failed
    exit /b 1
)
echo [+] SUCCESS: target\release\sa_cleaner.exe
copy /Y target\release\sa_cleaner.exe sa_cleaner_v2.exe >nul
echo [+] Copied to sa_cleaner_v2.exe
pause
