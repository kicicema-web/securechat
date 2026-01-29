@echo off
echo 🔐 SecureChat Build Script (GNU Toolchain - No Visual Studio Needed)
echo =====================================================================
echo.

:: Switch to GNU toolchain
echo [*] Setting up GNU toolchain (no MSVC required)...
rustup default stable-x86_64-pc-windows-gnu 2>nul
rustup target add x86_64-pc-windows-gnu 2>nul

:: Check for MinGW
where gcc >nul 2>nul
if %errorlevel% neq 0 (
    echo [!] MinGW not found!
    echo.
    echo Please install MinGW-w64:
    echo   Option 1: Using MSYS2:
    echo     1. Download from: https://www.msys2.org/
    echo     2. Install and open MSYS2 terminal
    echo     3. Run: pacman -S mingw-w64-x86_64-toolchain
    echo     4. Add C:\msys64\mingw64\bin to your PATH
    echo.
    echo   Option 2: Using Chocolatey:
    echo     choco install mingw
    echo.
    pause
    exit /b 1
)

echo [*] GCC found
gcc --version

echo.
echo [*] Building core library...
cd core
cargo build --release --target x86_64-pc-windows-gnu 2>&1
if %errorlevel% neq 0 (
    echo [!] Core build failed
    cd ..
    pause
    exit /b 1
)
cd ..
echo [*] Core built successfully!

echo.
echo [*] Installing Node dependencies...
cd desktop\src
call npm install 2>&1
if %errorlevel% neq 0 (
    echo [!] npm install failed
    cd ..\..
    pause
    exit /b 1
)
cd ..\..

echo.
echo [*] Build complete!
echo.
echo Note: Desktop app requires proper Tauri setup.
echo To build desktop, you still need:
echo   - WebView2 Runtime (usually pre-installed on Windows 10/11)
echo   - Or run: cargo tauri build from the desktop directory
pause
