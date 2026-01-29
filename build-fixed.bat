@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

echo 🔐 SecureChat Build Script (Fixed for Windows)
echo ===============================================
echo.

:: Check if running from correct directory
if not exist "core\Cargo.toml" (
    echo [!] Error: Please run this script from the securechat root directory
    echo     Current directory: %CD%
    exit /b 1
)

:: Check for Rust
echo [*] Checking Rust installation...
where rustc >nul 2>nul
if %errorlevel% neq 0 (
    echo [!] Rust not found. Please install Rust: https://rustup.rs/
    exit /b 1
)
echo [*] Rust found: 
rustc --version

:: Check for MSVC or GNU toolchain
echo [*] Checking for C compiler...
where cl >nul 2>nul
if %errorlevel% equ 0 (
    echo [*] MSVC compiler found
    set "COMPILER=msvc"
) else (
    where gcc >nul 2>nul
    if %errorlevel% equ 0 (
        echo [*] GCC found - will use GNU toolchain
        set "COMPILER=gnu"
    ) else (
        echo [!] WARNING: No C compiler found!
        echo.
        echo You have two options:
        echo.
        echo OPTION 1: Install Visual Studio Build Tools ^(Recommended^)
        echo   1. Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
        echo   2. Run the installer
        echo   3. Select: "Desktop development with C++"
        echo   4. Install and restart your terminal
        echo.
        echo OPTION 2: Use MinGW/GNU toolchain
        echo   Run: rustup target add x86_64-pc-windows-gnu
        echo   Then: cargo build --target x86_64-pc-windows-gnu
        echo.
        pause
        exit /b 1
    )
)

echo.
echo [*] Starting build...
echo.

:: Build core
echo [*] Building core library...
echo     This may take a while on first build...
cd /d "%~dp0core"

echo     Current directory: %CD%
echo     Running: cargo build --release

cargo build --release 2>&1
if %errorlevel% neq 0 (
    echo.
    echo [!] Core build failed!
    echo.
    echo Common fixes:
    echo 1. Install Visual Studio Build Tools:
    echo    https://visualstudio.microsoft.com/visual-cpp-build-tools/
    echo 2. Or switch to GNU toolchain:
    echo    rustup default stable-x86_64-pc-windows-gnu
    echo    rustup target add x86_64-pc-windows-gnu
    cd /d "%~dp0"
    pause
    exit /b 1
)

cd /d "%~dp0"
echo [*] Core library built successfully!
echo.

:: Build desktop
echo [*] Building desktop app...
cd /d "%~dp0desktop"

if not exist "src" (
    echo [!] Desktop src directory not found!
    cd /d "%~dp0"
    exit /b 1
)

cd src
echo [*] Installing Node dependencies...
call npm install 2>&1
if %errorlevel% neq 0 (
    echo [!] npm install failed
    cd /d "%~dp0"
    pause
    exit /b 1
)

cd ..
echo [*] Building with Tauri...
cargo tauri build 2>&1
if %errorlevel% neq 0 (
    echo [!] Desktop build failed
    cd /d "%~dp0"
    pause
    exit /b 1
)

cd /d "%~dp0"
echo.
echo ===============================================
echo [*] Build complete!
echo [*] Desktop app location:
echo     desktop\src-tauri\target\release\bundle\
echo ===============================================
pause
