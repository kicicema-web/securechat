@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

echo 🔐 SecureChat Build Script
echo ==========================

:: Check dependencies
echo [*] Checking dependencies...

where rustc >nul 2>nul
if %errorlevel% neq 0 (
    echo [!] Rust not found. Please install Rust: https://rustup.rs/
    exit /b 1
)

where node >nul 2>nul
if %errorlevel% neq 0 (
    echo [!] Node.js not found. Please install Node.js 20+
    exit /b 1
)

where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [!] Cargo not found. Please install Rust
    exit /b 1
)

echo [*] All dependencies found!

:: Default to all
if "%~1"=="" set "target=all"
set "target=%~1"

if "%target%"=="core" goto build_core
if "%target%"=="desktop" goto build_desktop
if "%target%"=="test" goto run_tests
if "%target%"=="all" goto build_all

echo Usage: %0 [core^|desktop^|test^|all]
exit /b 1

:build_core
echo [*] Building core library...
cd core
cargo build --release
if %errorlevel% neq 0 (
    echo [!] Core build failed
    exit /b 1
)
cd ..
echo [*] Core library built successfully!
if "%target%"=="core" goto :eof
goto build_desktop

:build_desktop
echo [*] Building desktop app...
cd desktop

echo [*] Installing Node dependencies...
cd src
call npm install
if %errorlevel% neq 0 (
    echo [!] npm install failed
    exit /b 1
)
cd ..

echo [*] Building with Tauri...
cargo tauri build
if %errorlevel% neq 0 (
    echo [!] Desktop build failed
    exit /b 1
)

cd ..
echo [*] Desktop app built successfully!
if "%target%"=="desktop" goto :eof
goto :eof

:run_tests
echo [*] Running tests...
cd core
cargo test
if %errorlevel% neq 0 (
    echo [!] Tests failed
    exit /b 1
)
cd ..
echo [*] Tests passed!
goto :eof

:build_all
call :run_tests
call :build_core
call :build_desktop
echo [*] Build complete!
echo [*] Desktop app: desktop\src-tauri\target\release\bundle\
goto :eof
