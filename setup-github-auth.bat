@echo off
echo 🔐 Setup GitHub Authentication
echo ===============================
echo.

:: Check for gh CLI
where gh >nul 2>&1
if %errorlevel% neq 0 (
    echo [*] GitHub CLI not found. Downloading...
    echo.
    
    :: Download GitHub CLI
    powershell -Command "Invoke-WebRequest -Uri 'https://github.com/cli/cli/releases/download/v2.43.1/gh_2.43.1_windows_amd64.msi' -OutFile '%TEMP%\gh.msi'"
    
    echo [*] Installing GitHub CLI...
    msiexec /i "%TEMP%\gh.msi" /quiet /norestart
    
    :: Refresh PATH
    set "PATH=%PATH%;C:\Program Files\GitHub CLI"
    
    echo [*] GitHub CLI installed!
    echo.
)

:: Login
echo [*] Starting GitHub login...
echo.
echo Follow the prompts:
echo   - Select HTTPS
echo   - Say YES to authenticate
echo   - A browser will open - login to GitHub
echo   - Copy the code shown
echo.
pause

call gh auth login

if %errorlevel% equ 0 (
    echo.
    echo ✅ Authentication successful!
    echo.
    echo You can now use:
    echo   build-and-push.bat
echo.
    echo And it will automatically push without asking for passwords!
) else (
    echo.
    echo [!] Authentication failed or cancelled.
    echo.
    echo Alternative: Use Personal Access Token
echo    1. Go to: https://github.com/settings/tokens
echo    2. Click "Generate new token (classic)"
    echo    3. Check the 'repo' box
echo    4. Generate and copy the token
echo    5. When pushing, use that token as password
)

pause
