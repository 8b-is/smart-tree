@echo off
REM Smart Tree Windows Batch Installer
REM This is a simple installer that downloads and installs Smart Tree
REM For a more robust installation, use install.ps1 (PowerShell script)

echo ========================================
echo   Smart Tree Windows Installer
echo ========================================
echo.

REM Check if PowerShell is available
where powershell >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo PowerShell detected. Running PowerShell installer...
    echo.
    powershell -ExecutionPolicy Bypass -File "%~dp0install.ps1"
    exit /b %ERRORLEVEL%
)

echo ERROR: PowerShell is required to install Smart Tree.
echo.
echo Please use one of these methods instead:
echo   1. Download the latest release manually from:
echo      https://github.com/8b-is/smart-tree/releases/latest
echo   2. Install PowerShell from:
echo      https://docs.microsoft.com/en-us/powershell/scripting/install/installing-powershell-on-windows
echo.
pause
exit /b 1
