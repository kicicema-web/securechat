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

# Function to check and install C compiler
function Install-CCompiler {
    $cl = Get-Command cl -ErrorAction SilentlyContinue
    $gcc = Get-Command gcc -ErrorAction SilentlyContinue
    
    if ($cl) {
        Write-Status "MSVC compiler (cl) found"
        return $true
    }
    
    if ($gcc) {
        Write-Status "GCC compiler found"
        Write-Status "Switching Rust to GNU toolchain..."
        rustup default stable-x86_64-pc-windows-gnu 2>$null
        rustup target add x86_64-pc-windows-gnu 2>$null
        return $true
    }
    
    Write-Warning "No C compiler found!"
    Write-Host ""
    Write-Host "You MUST install a C compiler to build SecureChat."
    Write-Host ""
    Write-Host "Option 1: MinGW (Recommended - 200MB, fast install)"
    Write-Host "  1. Download from: https://winlibs.com/"
    Write-Host "  2. Extract to C:\mingw64"
    Write-Host "  3. Add C:\mingw64\bin to your PATH"
    Write-Host "  4. Restart this terminal and run again"
    Write-Host ""
    Write-Host "Option 2: Visual Studio Build Tools (6GB)"
    Write-Host "  1. Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/"
    Write-Host "  2. Select Desktop development with C++"
    Write-Host "  3. Install and restart"
    Write-Host ""
    Write-Host "Option 3: Auto-install MinGW via Chocolatey"
    
    $choice = Read-Host "Install via Chocolatey now? (y/n)"
    
    if ($choice -eq "y" -or $choice -eq "Y") {
        # Check for Chocolatey
        $choco = Get-Command choco -ErrorAction SilentlyContinue
        if (-not $choco) {
            Write-Status "Installing Chocolatey..."
            Set-ExecutionPolicy Bypass -Scope Process -Force
            [System.Net.ServicePointManager]::SecurityProtocol = 3072
            Invoke-Expression ((New-Object Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
            $env:PATH = [Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [Environment]::GetEnvironmentVariable("PATH", "User")
        }
        
        Write-Status "Installing MinGW..."
        choco install mingw -y
        
        if ($LASTEXITCODE -eq 0) {
            # Refresh PATH
            $env:PATH = [Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [Environment]::GetEnvironmentVariable("PATH", "User")
            
            # Verify
            $gcc = Get-Command gcc -ErrorAction SilentlyContinue
            if ($gcc) {
                Write-Status "MinGW installed successfully!"
                Write-Status "Switching Rust to GNU toolchain..."
                rustup default stable-x86_64-pc-windows-gnu
                rustup target add x86_64-pc-windows-gnu
                return $true
            }
        }
    }
    
    return $false
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
    
    # Check for C compiler - REQUIRED
    if (-not (Install-CCompiler)) {
        Write-Error "C compiler is required. Please install one and try again."
        exit 1
    }
    
    # Check Node.js
    $node = Get-Command node -ErrorAction SilentlyContinue
    if (-not $node) {
        Write-Warning "Node.js not found! Installing..."
        $nodePath = "$env:TEMP\nodejs.msi"
        Invoke-WebRequest -Uri "https://nodejs.org/dist/v20.11.0/node-v20.11.0-x64.msi" -OutFile $nodePath
        Start-Process msiexec.exe -ArgumentList "/i", $nodePath, "/quiet", "/norestart" -Wait
        $env:PATH = [Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [Environment]::GetEnvironmentVariable("PATH", "User")
        Write-Status "Node.js installed"
    } else {
        Write-Status "Node.js found: $(node --version)"
    }
    
    Write-Host "`nAll dependencies ready!" -ForegroundColor Green
}

# Function to build the project
function Build-Project {
    # First check if we have a C compiler
    $cl = Get-Command cl -ErrorAction SilentlyContinue
    $gcc = Get-Command gcc -ErrorAction SilentlyContinue
    
    if (-not $cl -and -not $gcc) {
        Write-Error "No C compiler found! Cannot build."
        Write-Host "Run with -InstallDeps first, or install MinGW/Visual Studio manually."
        return
    }
    
    if ($gcc -and -not $cl) {
        Write-Status "Using GNU toolchain (MinGW)"
        rustup default stable-x86_64-pc-windows-gnu 2>$null
    } else {
        Write-Status "Using MSVC toolchain"
    }
    
    Write-Status "Building SecureChat..."
    
    # Build core
    Write-Status "Building core library..."
    Set-Location core
    cargo build --release
    $coreResult = $LASTEXITCODE
    Set-Location ..
    
    if ($coreResult -ne 0) {
        Write-Error "Core build failed!"
        return
    }
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
elseif ($BuildOnly) {
    Build-Project
}
else {
    # Default: check deps then build
    Install-Dependencies
    Build-Project
}

Read-Host "`nPress Enter to continue"
