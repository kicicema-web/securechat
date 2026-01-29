@echo off
echo 🚀 Push to GitHub
echo ==================
echo.

:: Check directory
if not exist ".git" (
    echo [!] Not a git repository!
    pause
    exit /b 1
)

:: Check for changes
git diff --quiet && git diff --cached --quiet
if %errorlevel% equ 0 (
    echo [*] No changes to push.
    exit /b 0
)

:: Show status
echo [*] Changes to push:
git status --short
echo.

:: Commit if needed
git diff --cached --quiet
if %errorlevel% equ 0 (
    set /p MSG="Enter commit message: "
    if "!MSG!"=="" set "MSG=Update %date%"
    git add -A
    git commit -m "%MSG%"
)

:: Push
echo.
echo [*] Pushing to GitHub...
echo.

:: Try to push
git push origin main 2>&1

if %errorlevel% neq 0 (
    echo.
    echo [!] Push failed!
    echo.
    echo You need to authenticate. Choose:
    echo.
    echo [1] Setup GitHub CLI (Recommended - easiest)
    echo [2] Enter Personal Access Token now
    echo [3] Show manual instructions
    echo.
    
    set /p CHOICE="Choice (1-3): "
    
    if "%CHOICE%"=="1" (
        call setup-github-auth.bat
    ) else if "%CHOICE%"=="2" (
        echo.
        echo Go to: https://github.com/settings/tokens
echo Generate a token with 'repo' permission
echo.
        set /p TOKEN="Paste your token: "
        git remote set-url origin https://kicicema-web:%TOKEN%@github.com/kicicema-web/securechat.git
        git push origin main
        git remote set-url origin https://github.com/kicicema-web/securechat.git
        echo.
        echo [!] Token removed from git config for security
    ) else (
        echo.
        echo To push manually:
        echo 1. Go to https://github.com/settings/tokens
echo 2. Generate new token with 'repo' scope
echo 3. Run: git push origin main
echo 4. Use the token as your password
    )
)

echo.
pause
