# Building SecureChat on Windows

This guide helps you build SecureChat on Windows and fix the common `link.exe not found` error.

## Quick Start

### Option 1: Automatic Setup (Recommended)

Run PowerShell as Administrator and execute:

```powershell
# Navigate to the project folder
cd C:\path\to\securechat

# Run the setup script
powershell -ExecutionPolicy Bypass -File setup-windows.ps1
```

This will automatically install all required dependencies.

### Option 2: Manual Installation

#### Step 1: Install Rust

Download and run:
```
https://win.rustup.rs/
```

#### Step 2: Install Visual Studio Build Tools

Download from:
```
https://visualstudio.microsoft.com/visual-cpp-build-tools/
```

During installation, select **"Desktop development with C++"** workload.

#### Step 3: Install Node.js

Download from:
```
https://nodejs.org/ (LTS version)
```

#### Step 4: Build

Open a new terminal (to get updated PATH) and run:

```cmd
build-fixed.bat
```

---

## The `link.exe not found` Error Explained

This error occurs because Rust on Windows needs a C compiler to link binaries:

```
error: linker `link.exe` not found
```

### Solution 1: Install Visual Studio Build Tools (Recommended)

1. Download: https://visualstudio.microsoft.com/visual-cpp-build-tools/
2. Run the installer
3. Check **"Desktop development with C++"**
4. Click Install (requires ~6GB space)
5. **Restart your terminal** after installation

### Solution 2: Use MinGW (Smaller, No Admin)

If you don't want to install Visual Studio:

```cmd
# Install MinGW
# Via Chocolatey (if you have it):
choco install mingw

# Via MSYS2:
# 1. Download from https://www.msys2.org/
# 2. Install and open MSYS2 terminal
# 3. Run: pacman -S mingw-w64-x86_64-toolchain
# 4. Add C:\msys64\mingw64\bin to your PATH

# Then switch Rust to GNU toolchain:
rustup default stable-x86_64-pc-windows-gnu
rustup target add x86_64-pc-windows-gnu

# Build using the GNU script:
build-gnu.bat
```

---

## Alternative: Without Visual Studio

If you cannot install Visual Studio, you can:

1. **Build only the core library** (command-line only):
   ```cmd
   cd core
   cargo build --release
   ```

2. **Use WSL2** (Windows Subsystem for Linux):
   ```powershell
   wsl --install
   # Then build in Linux environment
   ```

3. **Use Docker**:
   ```dockerfile
   # Create a Dockerfile for building
   FROM rust:latest
   WORKDIR /app
   COPY . .
   RUN cargo build --release
   ```

---

## Verification

After fixing the linker issue, verify with:

```cmd
# Check Rust
rustc --version

# Check C compiler
cl        # For MSVC
gcc       # For MinGW

# Check Node
node --version
npm --version
```

---

## Build Outputs

After successful build, find your executables at:

| Platform | Location |
|----------|----------|
| Windows MSI | `desktop\src-tauri\target\release\bundle\msi\` |
| Windows EXE | `desktop\src-tauri\target\release\` |
| Core Library | `core\target\release\securechat_core.dll` |

---

## Troubleshooting

### "cargo is not recognized"

Restart your terminal or run:
```cmd
refreshenv
```

### "npm not found"

Make sure Node.js is installed and restart your terminal.

### Build still fails after installing VS Build Tools

Try running from "Developer Command Prompt for VS 2022" which sets up proper environment variables.

### Still having issues?

1. Check Rust installation:
   ```cmd
   rustup show
   rustup update
   ```

2. Repair Visual Studio installation:
   - Open Visual Studio Installer
   - Click "Repair"

3. Use the GNU toolchain instead:
   ```cmd
   rustup default stable-x86_64-pc-windows-gnu
   rustup target add x86_64-pc-windows-gnu
   ```

---

## Need Help?

Open an issue at: https://github.com/kicicema-web/securechat/issues
