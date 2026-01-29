# Quick Start for Windows

## ⚠️ YOU MUST INSTALL A C COMPILER FIRST

Rust needs a C linker to compile. Choose ONE option:

### Option A: MinGW (Small, Fast, Recommended) - 200MB

1. **Download MinGW** from: https://winlibs.com/
   - Choose: `GCC 13.2.0 (with POSIX threads) + LLVM/Clang/LLD/LLDB 17.0.6 + MinGW-w64 11.0.1 (UCRT) - release 1`
   - Download the `Win64` zip

2. **Extract** to `C:\mingw64`

3. **Add to PATH**:
   - Press Win+R, type `sysdm.cpl`, press Enter
   - Click "Environment Variables"
   - Under "User variables", find "Path", click "Edit"
   - Click "New", add: `C:\mingw64\bin`
   - Click OK on all windows
   - **Restart your terminal**

4. **Switch Rust to GNU**:
   ```cmd
   rustup default stable-x86_64-pc-windows-gnu
   rustup target add x86_64-pc-windows-gnu
   ```

5. **Build**:
   ```cmd
   build-mingw.bat
   ```

### Option B: Visual Studio Build Tools - 6GB

1. Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/

2. Run installer, select **"Desktop development with C++"**

3. Install and **restart your terminal**

4. Build:
   ```cmd
   build-fixed.bat
   ```

---

## 🚀 Build & Auto-Push Script

After installing the compiler above, use this single command to build AND push:

```cmd
build-and-push.bat
```

This will:
1. Build the core library
2. Build the desktop app
3. Push to GitHub (will ask for login)

---

## First Time GitHub Login

Since the old token was revoked, you'll need to log in again:

### Method 1: Personal Access Token (Easiest)
1. Go to: https://github.com/settings/tokens
2. Click "Generate new token (classic)"
3. Select `repo` scope
4. Copy the token
5. When pushing, use this token as your password

### Method 2: GitHub CLI (Recommended for long term)
```cmd
# Install GitHub CLI from: https://cli.github.com/
# Then:
gh auth login
# Follow prompts, select HTTPS, then Yes to authenticate
```

---

## Troubleshooting

### "link.exe not found"
→ You didn't install the C compiler. See Option A or B above.

### "gcc not found"
→ MinGW is not in your PATH. Restart terminal after adding to PATH.

### "npm not found"
→ Install Node.js: https://nodejs.org/

### Git push asks for password
→ Use your Personal Access Token as the password (not your GitHub password)
