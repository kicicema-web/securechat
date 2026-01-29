# SecureChat Multi-Platform Build Script
# Run this script to build for all supported platforms

param(
    [switch]$Release,
    [switch]$Android,
    [switch]$IOS,
    [switch]$Desktop,
    [switch]$Help
)

function Show-Help {
    Write-Host @"
SecureChat Multi-Platform Build Script

Usage: .\build-all.ps1 [Options]

Options:
    -Release    Build in release mode (optimized)
    -Android    Build Android only
    -IOS        Build iOS only (requires macOS)
    -Desktop    Build Desktop (Linux/Windows/macOS) only
    -Help       Show this help message

Examples:
    .\build-all.ps1 -Release                    # Build all platforms in release mode
    .\build-all.ps1 -Desktop -Release           # Build desktop only
    .\build-all.ps1 -Android -Release          # Build Android only
"@
}

if ($Help) {
    Show-Help
    exit 0
}

$buildType = if ($Release) { "release" } else { "debug" }
$mode = if ($Release) { "--release" } else { "" }

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "SecureChat Multi-Platform Build" -ForegroundColor Cyan
Write-Host "Build type: $buildType" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Check prerequisites
Write-Host "`n[*] Checking prerequisites..." -ForegroundColor Green

$rust = Get-Command rustc -ErrorAction SilentlyContinue
if (-not $rust) {
    Write-Error "Rust not found! Please install Rust from https://rustup.rs/"
    exit 1
}

$node = Get-Command node -ErrorAction SilentlyContinue
if (-not $node) {
    Write-Error "Node.js not found! Please install Node.js from https://nodejs.org/"
    exit 1
}

Write-Host "    Rust: $(rustc --version)" -ForegroundColor Gray
Write-Host "    Node: $(node --version)" -ForegroundColor Gray

# Build Core Library
if (-not $Android -and -not $IOS) {
    Write-Host "`n[*] Building core library..." -ForegroundColor Green
    Set-Location core
    cargo build $mode
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Core build failed!"
        Set-Location ..
        exit 1
    }
    Set-Location ..
    Write-Host "    Core built successfully!" -ForegroundColor Green
}

# Build Desktop
if ((-not $Android -and -not $IOS) -or $Desktop) {
    Write-Host "`n[*] Building desktop app..." -ForegroundColor Green
    
    # Check for Tauri CLI
    $tauri = Get-Command cargo-tauri -ErrorAction SilentlyContinue
    if (-not $tauri) {
        Write-Host "    Installing Tauri CLI..." -ForegroundColor Yellow
        cargo install tauri-cli --locked
    }
    
    Set-Location desktop
    
    Write-Host "    Installing npm dependencies..." -ForegroundColor Gray
    npm install
    if ($LASTEXITCODE -ne 0) {
        Write-Error "npm install failed!"
        Set-Location ..
        exit 1
    }
    
    Write-Host "    Building Tauri app..." -ForegroundColor Gray
    cargo tauri build
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Tauri build failed!"
        Set-Location ..
        exit 1
    }
    
    Set-Location ..
    Write-Host "    Desktop app built successfully!" -ForegroundColor Green
}

# Build Android
if ($Android -or (-not $Desktop -and -not $IOS)) {
    Write-Host "`n[*] Building Android app..." -ForegroundColor Green
    
    # Check for Java
    $java = Get-Command java -ErrorAction SilentlyContinue
    if (-not $java) {
        Write-Error "Java not found! Please install JDK 17 or later."
        exit 1
    }
    
    Set-Location android
    
    if (Test-Path "gradlew") {
        Write-Host "    Building with Gradle..." -ForegroundColor Gray
        if ($Release) {
            .\gradlew assembleRelease
        } else {
            .\gradlew assembleDebug
        }
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Android build failed!"
            Set-Location ..
            exit 1
        }
        Write-Host "    Android app built successfully!" -ForegroundColor Green
    } else {
        Write-Warning "Gradle wrapper not found. Skipping Android build."
    }
    
    Set-Location ..
}

# Build iOS (macOS only)
if (($IOS -or (-not $Android -and -not $Desktop)) -and $IsMacOS) {
    Write-Host "`n[*] Building iOS library..." -ForegroundColor Green
    
    # Install iOS targets
    rustup target add aarch64-apple-ios
    rustup target add aarch64-apple-ios-sim
    rustup target add x86_64-apple-ios
    
    Set-Location core
    
    Write-Host "    Building for iOS device (arm64)..." -ForegroundColor Gray
    cargo build --release --target aarch64-apple-ios
    
    Write-Host "    Building for iOS simulator (arm64)..." -ForegroundColor Gray
    cargo build --release --target aarch64-apple-ios-sim
    
    Write-Host "    Building for iOS simulator (x86_64)..." -ForegroundColor Gray
    cargo build --release --target x86_64-apple-ios
    
    # Create XCFramework
    Write-Host "    Creating XCFramework..." -ForegroundColor Gray
    mkdir -p target/ios
    
    lipo -create `
        target/aarch64-apple-ios-sim/release/libsecurechat_core.a `
        target/x86_64-apple-ios/release/libsecurechat_core.a `
        -output target/ios/libsecurechat_core_sim.a
    
    xcodebuild -create-xcframework `
        -library target/aarch64-apple-ios/release/libsecurechat_core.a `
        -library target/ios/libsecurechat_core_sim.a `
        -output target/ios/SecureChatCore.xcframework
    
    Set-Location ..
    Write-Host "    iOS library built successfully!" -ForegroundColor Green
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "Build Complete!" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Show output locations
Write-Host "`nOutput locations:" -ForegroundColor Green
if (-not $Android -and -not $IOS) {
    Write-Host "  Core Library: core/target/$buildType/" -ForegroundColor Gray
}
if ((-not $Android -and -not $IOS) -or $Desktop) {
    Write-Host "  Desktop App:" -ForegroundColor Gray
    Write-Host "    - Windows: desktop/src-tauri/target/release/bundle/" -ForegroundColor Gray
    Write-Host "    - Linux:   desktop/src-tauri/target/release/bundle/" -ForegroundColor Gray
    Write-Host "    - macOS:   desktop/src-tauri/target/release/bundle/" -ForegroundColor Gray
}
if ($Android -or (-not $Desktop -and -not $IOS)) {
    Write-Host "  Android APK: android/app/build/outputs/apk/$buildType/" -ForegroundColor Gray
}
if (($IOS -or (-not $Android -and -not $Desktop)) -and $IsMacOS) {
    Write-Host "  iOS XCFramework: core/target/ios/SecureChatCore.xcframework" -ForegroundColor Gray
}

Write-Host ""
