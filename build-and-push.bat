@echo off
echo 🔐 SecureChat Build and Push to GitHub
echo ======================================
echo.

:: Check directory
if not exist "core\Cargo.toml" (
    echo [!] ERROR: Run this from the securechat folder!
    pause
    exit /b 1
)

:: Check for git
where git >nul 2>&1
if %errorlevel% neq 0 (
    echo [!] Git not found! Install from: https://git-scm.com/
    pause
    exit /b 1
)

:: Check for gcc or cl
call gcc --version >nul 2>&1
if %errorlevel% equ 0 (
    echo [*] Using MinGW/GNU build
    set "BUILD_SCRIPT=build-mingw.bat"
) else (
    call cl >nul 2>&1
    if %errorlevel% equ 0 (
        echo [*] Using MSVC build
        set "BUILD_SCRIPT=build-fixed.bat"
    ) else (
        echo [!] No C compiler found!
        echo.
        echo Please install ONE of:
        echo 1. MinGW: https://winlibs.com/ ^(200MB, faster^)
        echo 2. Visual Studio Build Tools: https://visualstudio.microsoft.com/visual-cpp-build-tools/ ^(6GB^)
        echo.
        echo Then run this script again.
        pause
        exit /b 1
    )
)

:: Run build
echo.
echo [*] Starting build...
call %BUILD_SCRIPT%

if %errorlevel% neq 0 (
    echo [!] Build failed! Fix errors above before pushing.
    pause
    exit /b 1
)

echo.
echo [*] Build successful!
echo.

:: Check git status
echo [*] Checking for changes...
git status --short

:: Check if there are changes
git diff --quiet
if %errorlevel% equ 0 (
    git diff --cached --quiet
    if %errorlevel% equ 0 (
        echo.
        echo [!] No changes to commit.
        echo Your code is already up to date!
        pause
        exit /b 0
    )
)

:: Commit changes
echo.
echo [*] Committing changes...

set /p COMMIT_MSG="Enter commit message (or press Enter for default): "

if "%COMMIT_MSG%"=="" (
    set "COMMIT_MSG=Update: Windows build %date% %time%"
)

git add -A
git commit -m "%COMMIT_MSG%"

:: Push to GitHub
echo.
echo [*] Pushing to GitHub...
echo.
echo If prompted for password, use your Personal Access Token!
echo Get one at: https://github.com/settings/tokens
echo.

git push origin main 2>&1

if %errorlevel% neq 0 (
    echo.
    echo [!] Push failed!
    echo.
    echo Common fixes:
    echo 1. Create a Personal Access Token:
    echo    https://github.com/settings/tokens ^> Generate new token ^> check 'repo'
    echo 2. Use that token as your password when prompted
    echo.
    echo Or use GitHub CLI:
    echo    gh auth login
    pause
    exit /b 1
)

echo.
echo ======================================
echo ✅ BUILD AND PUSH SUCCESSFUL!
echo ======================================
echo.
echo Your code is now on GitHub:
echo   https://github.com/kicicema-web/securechat
echo.
pause
