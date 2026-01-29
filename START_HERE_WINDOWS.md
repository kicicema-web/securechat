# 🚀 START HERE - Windows Users

## The Problem: `link.exe not found`

You're seeing this error because **Rust needs a C compiler** to build the project.

## The Solution (Choose ONE)

---

### ✅ OPTION 1: MinGW (Recommended - Fast, Small)

**Time:** 10 minutes | **Size:** 200MB | **No Admin Required**

#### Step 1: Download MinGW

1. Go to: https://winlibs.com/
2. Find the section: **"GCC 13.2.0 (with POSIX threads) + LLVM/Clang/LLD/LLDB 17.0.6 + MinGW-w64 11.0.1 (UCRT)"**
3. Click: **Zip archive** under the **Win64** column
4. Save the file

#### Step 2: Extract

1. Open the downloaded ZIP file
2. Extract to: `C:\mingw64`
3. You should see `C:\mingw64\bin\gcc.exe`

#### Step 3: Add to PATH

1. Press `Win + R`, type `sysdm.cpl`, press Enter
2. Click **"Environment Variables"**
3. Under **"User variables"**, find **"Path"**, click **"Edit"**
4. Click **"New"**, type: `C:\mingw64\bin`
5. Click **OK** on all windows
6. **Close and reopen your terminal** (IMPORTANT!)

#### Step 4: Verify

```cmd
gcc --version
```

You should see version info. If not, PATH wasn't set correctly.

#### Step 5: Build

```cmd
cd C:\Users\onyx\securechat
build-mingw.bat
```

---

### OPTION 2: Visual Studio Build Tools

**Time:** 30 minutes | **Size:** 6GB

1. Download: https://visualstudio.microsoft.com/visual-cpp-build-tools/
2. Run the installer
3. Check **"Desktop development with C++"**
4. Click Install
5. **Restart your computer**
6. Build:
   ```cmd
   build-fixed.bat
   ```

---

## 🔃 Auto-Push to GitHub

Once build works, use this to build AND push:

```cmd
build-and-push.bat
```

### First Time GitHub Setup

Since the old token was deleted, you need to authenticate once:

**Method A: GitHub CLI (Easiest)**
```cmd
setup-github-auth.bat
```

**Method B: Personal Access Token**
1. Go to: https://github.com/settings/tokens
2. Click **"Generate new token (classic)"**
3. Check the **"repo"** box
4. Click **Generate token**
5. **Copy the token** (you won't see it again!)
6. When `build-and-push.bat` asks for password, paste the token

---

## 📋 Quick Commands Reference

| Command | Purpose |
|---------|---------|
| `build-mingw.bat` | Build using MinGW |
| `build-fixed.bat` | Build using Visual Studio |
| `build-and-push.bat` | Build + push to GitHub |
| `push-to-github.bat` | Push only (if already built) |
| `setup-github-auth.bat` | Setup GitHub authentication |

---

## ❓ Troubleshooting

### "gcc not found"
→ Restart your terminal after adding MinGW to PATH

### "npm not found"
→ Install Node.js: https://nodejs.org/

### "cargo not found"
→ Reinstall Rust: https://rustup.rs/

### Build succeeds but push fails
→ Run `setup-github-auth.bat` or create a Personal Access Token

### "I installed MinGW but still get link.exe error"
→ Run: `rustup default stable-x86_64-pc-windows-gnu`

---

## 📁 After Successful Build

Your app will be in:
- **Installer:** `desktop\src-tauri\target\release\bundle\msi\SecureChat_0.1.0_x64_en-US.msi`
- **Portable EXE:** `desktop\src-tauri\target\release\SecureChat.exe`

---

## 🆘 Still Need Help?

Open an issue: https://github.com/kicicema-web/securechat/issues

Or check the detailed guides:
- `BUILD_WINDOWS.md` - Detailed build documentation
- `QUICK_START_WINDOWS.md` - Quick reference
