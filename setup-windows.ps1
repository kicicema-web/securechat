# SecureChat Windows Setup Script
# Run as Administrator: powershell -ExecutionPolicy Bypass -File setup-windows.ps1

param(
    [switch]$InstallDeps,
    [switch]$BuildOnly
)

$Red = "`e[91m"
$Green = "`e[92m"
$Yellow = "`e[93m"
$Reset = "`e[0m"

function Write-Status($msg) { Write-Host "$Green[*]$Reset $msg" }
function Write-Warning($msg) { Write-Host "$Yellow[!]$Reset $msg" }
function Write-Error($msg) { Write-Host "$Red[!]$Reset $msg" }

Write-Host "`n🔐 SecureChat Windows Setup`n" -ForegroundColor Cyan

# Check if running as admin
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

# Check current directory
if (-not (Test-Path "core\Cargo.toml")) {
    Write-Error "Please run this script from the securechat root directory"
    Write-Host "Current directory: $(Get-Location)"
    exit 1
}

# Function to install dependencies
function Install-Dependencies {
    Write-Status "Checking dependencies...`n"
    
    # Check Rust
    $rust = Get-Command rustc -ErrorAction SilentlyContinue
    if (-not $rust) {
        Write-Warning "Rust not found!"
        Write-Host "Installing Rust..."
        
        # Download and run rustup
        $rustupUrl = "https://win.rustup.rs/x86_64"
        $rustupPath = "$env:TEMP\rustup-init.exe"
        
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath
        & $rustupPath -y --default-toolchain stable
        
        # Reload PATH
        $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")
    } else {
        Write-Status "Rust found: $(rustc --version)"
    }
    
    # Check for C compiler
    $cl = Get-Command cl -ErrorAction SilentlyContinue
    $gcc = Get-Command gcc -ErrorAction SilentlyContinue
    
    if (-not $cl -and -not $gcc) {
        Write-Warning "No C compiler found!`n"
        
        Write-Host "Choose an option:`n" -ForegroundColor Yellow
        Write-Host "1. Install Visual Studio Build Tools (Recommended - better compatibility)"
        Write-Host "2. Install MinGW-w64 (Smaller, no admin required after install)"
        Write-Host "3. Skip - I'll install manually`n"
        
        $choice = Read-Host "Enter choice (1-3)"
        
        switch ($choice) {
            "1" {
                Write-Status "Downloading Visual Studio Build Tools..."
                $vsUrl = "https://aka.ms/vs/17/release/vs_BuildTools.exe"
                $vsPath = "$env:TEMP\vs_BuildTools.exe"
                
                Invoke-WebRequest -Uri $vsUrl -OutFile $vsPath
                
                Write-Status "Installing Visual Studio Build Tools..."
                Write-Host "This will install C++ build tools. Please wait...`n" -ForegroundColor Yellow
                
                & $vsPath --quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended
                
                if ($LASTEXITCODE -eq 0) {
                    Write-Status "Visual Studio Build Tools installed successfully!"
                    Write-Warning "Please restart your terminal and run this script again"
                    exit 0
                } else {
                    Write-Error "Installation failed. Please install manually from:"
                    Write-Host "https://visualstudio.microsoft.com/visual-cpp-build-tools/"
                }
            }
            "2" {
                Write-Status "Installing MinGW via Chocolatey..."
                
                # Check for Chocolatey
                $choco = Get-Command choco -ErrorAction SilentlyContinue
                if (-not $choco) {
                    Write-Status "Installing Chocolatey..."
                    Set-ExecutionPolicy Bypass -Scope Process -Force
                    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
                    Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
                    $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")
                }
                
                choco install mingw -y
                
                # Add to PATH
                $mingwPath = "C:\ProgramData\mingw64\mingw64\bin"
                if (Test-Path $mingwPath) {
                    [System.Environment]::SetEnvironmentVariable("PATH", $env:PATH + ";$mingwPath", "User")
                    $env:PATH += ";$mingwPath"
                }
                
                # Switch Rust to GNU toolchain
                rustup default stable-x86_64-pc-windows-gnu
                rustup target add x86_64-pc-windows-gnu
                
                Write-Status "MinGW installed! Please restart your terminal."
            }
            default {
                Write-Host "`nManual installation instructions:"
                Write-Host "Option 1: Visual Studio Build Tools"
                Write-Host "  https://visualstudio.microsoft.com/visual-cpp-build-tools/"
                Write-Host "Option 2: MinGW via MSYS2"
                Write-Host "  https://www.msys2.org/"
                exit 0
            }
        }
    } else {
        if ($cl) { Write-Status "MSVC compiler found" }
        if ($gcc) { Write-Status "GCC found" }
    }
    
    # Check Node.js
    $node = Get-Command node -ErrorAction SilentlyContinue
    if (-not $node) {
        Write-Warning "Node.js not found!"
        Write-Host "Installing Node.js..."
        
        # Download Node.js installer
        $nodeUrl = "https://nodejs.org/dist/v20.11.0/node-v20.11.0-x64.msi"
        $nodePath = "$env:TEMP\nodejs.msi"
        
        Invoke-WebRequest -Uri $nodeUrl -OutFile $nodePath
        Start-Process msiexec.exe -ArgumentList "/i", $nodePath, "/quiet", "/norestart" -Wait
        
        $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")
        Write-Status "Node.js installed!"
    } else {
        Write-Status "Node.js found: $(node --version)"
    }
    
    Write-Host "`n✅ All dependencies installed!`n" -ForegroundColor Green
}

# Function to build the project
function Build-Project {
    Write-Status "Building SecureChat...`n"
    
    # Determine which toolchain to use
    $gcc = Get-Command gcc -ErrorAction SilentlyContinue
    $cl = Get-Command cl -ErrorAction SilentlyContinue
    
    $cargoArgs = "--release"
    if ($gcc -and -not $cl) {
        Write-Status "Using GNU toolchain"
        rustup default stable-x86_64-pc-windows-gnu 2>$null
        $cargoArgs = "--release --target x86_64-pc-windows-gnu"
    }
    
    # Build core
    Write-Status "Building core library..."
    Set-Location core
    $buildOutput = cargo build $cargoArgs.Split() 2>&1
    $buildResult = $LASTEXITCODE
    Set-Location ..
    
    if ($buildResult -ne 0) {
        Write-Error "Core build failed!`n"
        Write-Host $buildOutput
        
        Write-Host "`nTroubleshooting:" -ForegroundColor Yellow
        Write-Host "1. If you see 'linker link.exe not found', install Visual Studio Build Tools:"
        Write-Host "   https://visualstudio.microsoft.com/visual-cpp-build-tools/"
        Write-Host "2. Or switch to GNU toolchain: rustup default stable-x86_64-pc-windows-gnu"
        return
    }
    
    Write-Status "Core built successfully!`n"
    
    # Build desktop
    Write-Status "Building desktop app..."
    Set-Location desktop\src
    
    Write-Status "Installing npm dependencies..."
    npm install 2>&1 | Out-Null
    
    Set-Location ..
    cargo tauri build 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "`n✅ Build complete!`n" -ForegroundColor Green
        Write-Status "Desktop app location:"
        Write-Host "  desktop\src-tauri\target\release\bundle\`n"
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

Write-Host "Press any key to continue..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
