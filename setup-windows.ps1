# SecureChat Windows Setup Script
# Run as Administrator: powershell -ExecutionPolicy Bypass -File setup-windows.ps1

param(
    [switch]$InstallDeps,
    [switch]$BuildOnly
)

function Write-Status($msg) { Write-Host "[*] $msg" -ForegroundColor Green }
function Write-Warning($msg) { Write-Host "[!] $msg" -ForegroundColor Yellow }
function Write-Error($msg) { Write-Host "[!] $msg" -ForegroundColor Red }

Write-Host "`nSecureChat Windows Setup`n" -ForegroundColor Cyan

# Check current directory
if (-not (Test-Path "core\Cargo.toml")) {
    Write-Error "Please run this script from the securechat root directory"
    exit 1
}

# Function to install dependencies
function Install-Dependencies {
    Write-Status "Checking dependencies..."
    
    # Check Rust
    $rust = Get-Command rustc -ErrorAction SilentlyContinue
    if (-not $rust) {
        Write-Warning "Rust not found! Installing..."
        $rustupPath = "$env:TEMP\rustup-init.exe"
        Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupPath
        & $rustupPath -y --default-toolchain stable
        $env:PATH = [Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [Environment]::GetEnvironmentVariable("PATH", "User")
    } else {
        Write-Status "Rust found: $(rustc --version)"
    }
    
    # Check for C compiler
    $cl = Get-Command cl -ErrorAction SilentlyContinue
    $gcc = Get-Command gcc -ErrorAction SilentlyContinue
    
    if (-not $cl -and -not $gcc) {
        Write-Warning "No C compiler found!"
        Write-Host ""
        Write-Host "Choose an option:"
        Write-Host "1. Install Visual Studio Build Tools (Recommended)"
        Write-Host "2. Install MinGW-w64 (Smaller)"
        Write-Host "3. Skip - I will install manually"
        Write-Host ""
        
        $choice = Read-Host "Enter choice (1-3)"
        
        if ($choice -eq "1") {
            Write-Status "Downloading Visual Studio Build Tools..."
            $vsPath = "$env:TEMP\vs_BuildTools.exe"
            Invoke-WebRequest -Uri "https://aka.ms/vs/17/release/vs_BuildTools.exe" -OutFile $vsPath
            & $vsPath --quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended
            Write-Status "Please restart your terminal after installation"
            exit 0
        }
        elseif ($choice -eq "2") {
            Write-Status "Installing MinGW via Chocolatey..."
            $choco = Get-Command choco -ErrorAction SilentlyContinue
            if (-not $choco) {
                Set-ExecutionPolicy Bypass -Scope Process -Force
                [System.Net.ServicePointManager]::SecurityProtocol = 3072
                Invoke-Expression ((New-Object Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
            }
            choco install mingw -y
            rustup default stable-x86_64-pc-windows-gnu
            rustup target add x86_64-pc-windows-gnu
            Write-Status "MinGW installed! Please restart your terminal."
        }
    } else {
        if ($cl) { Write-Status "MSVC compiler found" }
        if ($gcc) { Write-Status "GCC found" }
    }
    
    # Check Node.js
    $node = Get-Command node -ErrorAction SilentlyContinue
    if (-not $node) {
        Write-Warning "Node.js not found! Installing..."
        $nodePath = "$env:TEMP\nodejs.msi"
        Invoke-WebRequest -Uri "https://nodejs.org/dist/v20.11.0/node-v20.11.0-x64.msi" -OutFile $nodePath
        Start-Process msiexec.exe -ArgumentList "/i", $nodePath, "/quiet", "/norestart" -Wait
        $env:PATH = [Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [Environment]::GetEnvironmentVariable("PATH", "User")
    } else {
        Write-Status "Node.js found: $(node --version)"
    }
    
    Write-Host "`nAll dependencies installed!" -ForegroundColor Green
}

# Function to build the project
function Build-Project {
    Write-Status "Building SecureChat..."
    
    # Determine which toolchain to use
    $gcc = Get-Command gcc -ErrorAction SilentlyContinue
    $cl = Get-Command cl -ErrorAction SilentlyContinue
    
    if ($gcc -and -not $cl) {
        Write-Status "Using GNU toolchain"
        rustup default stable-x86_64-pc-windows-gnu 2>$null
    }
    
    # Build core
    Write-Status "Building core library..."
    Set-Location core
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Core build failed!"
        Set-Location ..
        return
    }
    Set-Location ..
    Write-Status "Core built successfully!"
    
    # Build desktop
    Write-Status "Building desktop app..."
    Set-Location desktop\src
    npm install
    Set-Location ..
    cargo tauri build
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "`nBuild complete!" -ForegroundColor Green
        Write-Status "Desktop app location: desktop\src-tauri\target\release\bundle\"
    } else {
        Write-Error "Desktop build failed!"
    }
    
    Set-Location ..
}

# Main execution
if ($InstallDeps) {
    Install-Dependencies
}

if ($BuildOnly -or -not $InstallDeps) {
    Build-Project
}

if (-not $InstallDeps -and -not $BuildOnly) {
    Install-Dependencies
    Build-Project
}

Read-Host "`nPress Enter to continue"
