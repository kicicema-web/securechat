# SecureChat CI/CD Guide

This document describes the Continuous Integration and Deployment setup for SecureChat.

## GitHub Actions Workflow

The `.github/workflows/build.yml` file defines automated builds for multiple platforms.

### Platforms

| Platform | Format | Runner | Notes |
|----------|--------|--------|-------|
| Linux | `.AppImage`, `.deb` | `ubuntu-latest` | GTK-based desktop app |
| Windows | `.msi`, `.exe` | `windows-latest` | MSI and NSIS installers |
| macOS | `.dmg`, `.app` | `macos-latest` | Desktop app bundle |
| Android | `.apk` | `ubuntu-latest` | Requires signing for release |
| iOS | `.xcframework` | `macos-latest` | Static library for Xcode integration |

### Triggers

The workflow runs on:
- **Push** to `main` or `develop` branches
- **Pull requests** to `main` branch
- **Tags** starting with `v*` (e.g., `v1.0.0`)

### Jobs

1. **test-core**: Runs unit tests on the Rust core library
2. **build-linux**: Builds Linux desktop app (AppImage and DEB)
3. **build-windows**: Builds Windows desktop app (MSI and NSIS installer)
4. **build-macos**: Builds macOS desktop app (DMG and APP bundle)
5. **build-android**: Builds Android APK
6. **build-ios**: Builds iOS XCFramework (static library)
7. **release**: Creates GitHub release with all artifacts (tag pushes only)

## Required Secrets

For signed releases, add these secrets to your GitHub repository:

### Android Signing

```
SIGNING_KEY          # Base64-encoded signing keystore (.jks)
KEY_ALIAS            # Key alias
KEY_STORE_PASSWORD   # Keystore password
KEY_PASSWORD         # Key password
```

### iOS/macOS Signing (for App Store distribution)

```
BUILD_CERTIFICATE_BASE64      # Base64-encoded .p12 certificate
P12_PASSWORD                  # Certificate password
BUILD_PROVISION_PROFILE_BASE64 # Base64-encoded .mobileprovision file
KEYCHAIN_PASSWORD             # Temporary keychain password
APPLE_TEAM_ID                 # Apple Developer Team ID
```

### Generating Android Signing Key

```bash
keytool -genkey -v -keystore securechat.keystore -alias securechat \
  -keyalg RSA -keysize 2048 -validity 10000

# Base64 encode for GitHub secret
base64 securechat.keystore | pbcopy  # macOS
base64 securechat.keystore | clip    # Windows (WSL)
```

### Exporting iOS Certificates

```bash
# Export certificate from Keychain as .p12
# Then base64 encode
cert_path="~/Desktop/build_certificate.p12"
base64 "$cert_path" | pbcopy

# Export provisioning profile
profile_path="~/Desktop/build_pp.mobileprovision"
base64 "$profile_path" | pbcopy
```

## Local Building

### Prerequisites

- **Rust**: https://rustup.rs/
- **Node.js**: https://nodejs.org/ (v20 recommended)
- **Tauri CLI**: `cargo install tauri-cli --locked`

### Quick Build

```powershell
# Build all platforms
.\build-all.ps1 -Release

# Build specific platforms only
.\build-all.ps1 -Release -Desktop
.\build-all.ps1 -Release -Android
```

### Manual Build

#### Core Library
```bash
cd core
cargo build --release
```

#### Desktop App
```bash
cd desktop
npm install
cargo tauri build
```

#### Android App
```bash
cd android
./gradlew assembleRelease
```

#### iOS Library (macOS only)
```bash
# Install targets
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios

# Build
cd core
cargo build --release --target aarch64-apple-ios
cargo build --release --target aarch64-apple-ios-sim
cargo build --release --target x86_64-apple-ios
```

## Artifacts

### Download Locations

- **Pull Requests**: Check the "Actions" tab → Select workflow run → Artifacts
- **Releases**: Automatically attached to GitHub releases for tagged versions

### Artifact Structure

```
securechat-linux-appimage/   # *.AppImage
securechat-linux-deb/        # *.deb
securechat-windows-msi/      # *.msi
securechat-windows-nsis/     # *.exe
securechat-macos-dmg/        # *.dmg
securechat-macos-app/        # *.app
securechat-android-apk/      # *.apk
securechat-ios-xcframework/  # SecureChatCore.xcframework
```

## Troubleshooting

### Linux Build Failures

Ensure all dependencies are installed:
```bash
sudo apt-get update
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.0-dev \
  libappindicator3-dev librsvg2-dev patchelf
```

### Windows Build Failures

- Ensure Windows SDK is installed
- Use `windows-latest` runner (includes required tools)

### macOS/iOS Build Failures

- Ensure Xcode Command Line Tools are installed: `xcode-select --install`
- For iOS, ensure valid signing certificates are configured

### Android Build Failures

- Verify JDK 17+ is installed
- Check `ANDROID_HOME` environment variable is set
- Ensure `gradlew` has execute permissions: `chmod +x gradlew`

## Release Checklist

Before creating a release:

1. [ ] Update version in:
   - `core/Cargo.toml`
   - `desktop/package.json`
   - `desktop/tauri.conf.json`
   - `android/app/build.gradle`

2. [ ] Update `CHANGELOG.md`

3. [ ] Create and push a tag:
   ```bash
   git tag -a v1.0.0 -m "Release version 1.0.0"
   git push origin v1.0.0
   ```

4. [ ] Wait for CI/CD to complete

5. [ ] Verify all artifacts are attached to the release

6. [ ] Test installation on target platforms

## Customization

### Disable Platforms

Edit `.github/workflows/build.yml` and comment out unwanted jobs:

```yaml
# build-ios:
#   runs-on: macos-latest
#   ...
```

### Add New Platforms

1. Add a new job following the existing pattern
2. Use appropriate `runs-on` runner
3. Add upload-artifact step
4. Update the `release` job to download the new artifact

### Change Trigger Conditions

Modify the `on:` section in the workflow file:

```yaml
on:
  push:
    branches: [main]  # Only main branch
  workflow_dispatch:   # Manual trigger
```
