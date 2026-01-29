@echo off
echo 🔐 SecureChat Build (MinGW/GNU Toolchain)
echo ==========================================
echo.

:: Check if we're in the right directory
if not exist "core\Cargo.toml" (
    echo [!] ERROR: Run this from the securechat folder!
    echo     Current: %CD%
    pause
    exit /b 1
)

:: Check for gcc
call gcc --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [!] ERROR: gcc not found!
    echo.
    echo You need to install MinGW:
    echo 1. Download from: https://winlibs.com/
    echo 2. Extract to C:\mingw64
    echo 3. Add C:\mingw64\bin to your PATH
    echo 4. Restart this terminal
    echo.
    pause
    exit /b 1
)

echo [*] GCC found:
gcc --version | find "gcc"

:: Switch Rust to GNU toolchain
echo [*] Setting up Rust GNU toolchain...
rustup default stable-x86_64-pc-windows-gnu >nul 2>&1
rustup target add x86_64-pc-windows-gnu >nul 2>&1

:: Build core
echo.
echo [*] Building core library (this takes time on first run)...
cd /d "%~dp0core" || (
    echo [!] Failed to enter core directory
    exit /b 1
)

cargo build --release 2>&1
if %errorlevel% neq 0 (
    echo.
    echo [!] BUILD FAILED
    cd /d "%~dp0"
    pause
    exit /b 1
)

cd /d "%~dp0"
echo [*] Core library built!

:: Build desktop
echo.
echo [*] Building desktop app...

if not exist "desktop\src\package.json" (
    echo [!] Desktop files not found
    pause
    exit /b 1
)

cd /d "%~dp0desktop\src"

echo [*] Installing dependencies...
call npm install
if %errorlevel% neq 0 (
    echo [!] npm install failed
    cd /d "%~dp0"
    pause
    exit /b 1
)

cd /d "%~dp0desktop"

echo [*] Building with Tauri (this may take several minutes)...
cargo tauri build 2>&1
if %errorlevel% neq 0 (
    echo [!] Desktop build failed
    cd /d "%~dp0"
    pause
    exit /b 1
)

cd /d "%~dp0"

echo.
echo ==========================================
echo ✅ BUILD SUCCESSFUL!
echo ==========================================
echo.
echo Your app is here:
echo   desktop\src-tauri\target\release\bundle\msi\
echo   desktop\src-tauri\target\release\SecureChat.exe
echo.
pause
