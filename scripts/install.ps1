# Smart Tree Windows Installer Script
# 
# This script installs the 'st' binary on Windows
#
# Usage:
#   iwr -useb https://raw.githubusercontent.com/8b-is/smart-tree/main/scripts/install.ps1 | iex
#
# You can customize the installation by setting environment variables:
#   - $env:INSTALL_DIR: The directory to install the binary to (default: $env:LOCALAPPDATA\Programs\st)
#   - $env:VERSION: The version to install (default: latest)

param(
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\st",
    [string]$Version = ""
)

# Configuration
$GitHubRepo = "8b-is/smart-tree"
$BinaryName = "st.exe"

# Terminal colors
function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
    exit 1
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

# Function to check if a release has assets
function Test-ReleaseAssets {
    param([string]$VersionTag)
    
    try {
        $releaseInfo = Invoke-RestMethod -Uri "https://api.github.com/repos/$GitHubRepo/releases/tags/$VersionTag"
        return $releaseInfo.assets.Count -gt 0
    } catch {
        return $false
    }
}

# Function to get releases with assets
function Get-ReleasesWithAssets {
    try {
        # Get first 10 releases with pagination for efficiency
        $releases = Invoke-RestMethod -Uri "https://api.github.com/repos/$GitHubRepo/releases?per_page=10"
        return $releases | Where-Object { $_.assets.Count -gt 0 } | Select-Object -ExpandProperty tag_name
    } catch {
        Write-Error "Failed to fetch releases from GitHub API"
    }
}

# Function to select version interactively
function Select-Version {
    param([string]$LatestVersion)
    
    Write-Warning "The latest version ($LatestVersion) doesn't have any release binaries yet."
    Write-Info "Fetching other available versions with binaries..."
    
    $versions = Get-ReleasesWithAssets
    
    if ($versions.Count -eq 0) {
        Write-Error "No releases with binaries found!"
    }
    
    Write-Info "Available versions with binaries:"
    for ($i = 0; $i -lt $versions.Count; $i++) {
        Write-Host "  $($i + 1). $($versions[$i])"
    }
    
    while ($true) {
        $selection = Read-Host "Select a version (1-$($versions.Count)) or 'q' to quit"
        
        if ($selection -eq 'q') {
            Write-Info "Installation cancelled."
            exit 0
        }
        
        # Safe integer parsing
        $selectionNum = 0
        if ([System.Int32]::TryParse($selection, [ref]$selectionNum) -and 
            $selectionNum -ge 1 -and $selectionNum -le $versions.Count) {
            return $versions[$selectionNum - 1]
        } else {
            Write-Error "Invalid selection. Please try again."
        }
    }
}

# Main Installation Logic

Write-Info "Smart Tree Windows Installer"
Write-Info "Repository: $GitHubRepo"
Write-Info ""

# 1. Detect Architecture
$arch = $env:PROCESSOR_ARCHITECTURE
switch ($arch) {
    "AMD64" { $targetArch = "x86_64" }
    "ARM64" { $targetArch = "aarch64" }
    default {
        Write-Error "Unsupported architecture: $arch"
    }
}

Write-Info "Detected architecture: $targetArch"

# 2. Determine Version to Install
if ([string]::IsNullOrEmpty($Version)) {
    Write-Info "Fetching the latest version number..."
    try {
        $latestRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/$GitHubRepo/releases/latest"
        $latestVersion = $latestRelease.tag_name
        
        if ([string]::IsNullOrEmpty($latestVersion)) {
            Write-Error "Could not fetch the latest version. Please check the repository path and your network connection."
        }
        
        # Check if latest version has assets
        if (-not (Test-ReleaseAssets -VersionTag $latestVersion)) {
            $Version = Select-Version -LatestVersion $latestVersion
        } else {
            $Version = $latestVersion
            Write-Info "Latest version is $Version"
        }
    } catch {
        Write-Error "Failed to fetch the latest version: $_"
    }
} else {
    Write-Info "Installing specified version: $Version"
    
    # Check if specified version has assets
    if (-not (Test-ReleaseAssets -VersionTag $Version)) {
        Write-Warning "Version $Version doesn't have any release binaries."
        $response = Read-Host "Would you like to select another version? (y/n)"
        if ($response -eq 'y' -or $response -eq 'Y') {
            $Version = Select-Version -LatestVersion $Version
        } else {
            Write-Error "Cannot install version without binaries."
        }
    }
}

# 3. Construct Download URL
$archiveName = "st-$Version-$targetArch-pc-windows-msvc.zip"
$downloadUrl = "https://github.com/$GitHubRepo/releases/download/$Version/$archiveName"

# 4. Download and Extract
$tempDir = New-Item -ItemType Directory -Path "$env:TEMP\st-install-$(Get-Random)" -Force
$archivePath = Join-Path $tempDir $archiveName

Write-Info "Downloading from $downloadUrl"
try {
    # Use WebRequest for better progress display
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath -UseBasicParsing
    $ProgressPreference = 'Continue'
} catch {
    Write-Error "Download failed: $_"
}

Write-Info "Extracting the binary..."
try {
    Expand-Archive -Path $archivePath -DestinationPath $tempDir -Force
} catch {
    Write-Error "Extraction failed: $_"
}

# 5. Install the binary
Write-Info "Installing to $InstallDir..."

# Create install directory if it doesn't exist
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$binaryPath = Join-Path $tempDir $BinaryName
if (-not (Test-Path $binaryPath)) {
    # Sometimes the binary is in a subdirectory
    $binaryPath = Get-ChildItem -Path $tempDir -Filter $BinaryName -Recurse | Select-Object -First 1 -ExpandProperty FullName
    if ([string]::IsNullOrEmpty($binaryPath)) {
        Write-Error "Could not find the '$BinaryName' binary in the downloaded archive."
    }
}

$destinationPath = Join-Path $InstallDir $BinaryName
Copy-Item -Path $binaryPath -Destination $destinationPath -Force

# 6. Update PATH if necessary
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$InstallDir*") {
    Write-Info "Adding $InstallDir to your PATH..."
    $newPath = "$userPath;$InstallDir"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Success "Added to PATH. Please restart your terminal for changes to take effect."
    
    # Update current session PATH
    $env:Path = "$env:Path;$InstallDir"
} else {
    Write-Info "$InstallDir is already in your PATH"
}

# 7. Verify Installation
Write-Info "Verifying installation..."
try {
    $versionOutput = & $destinationPath --version
    Write-Success "Successfully installed st to $destinationPath"
    Write-Info "Version: $versionOutput"
    Write-Info "You can now use the 'st' command!"
} catch {
    Write-Warning "Installation completed but verification failed. You may need to restart your terminal."
}

# Clean up
Remove-Item -Path $tempDir -Recurse -Force

# Show next steps
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Smart Tree installed successfully! 🌳" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Info "Next steps:"
Write-Host "  1. Restart your terminal (or open a new one) to use 'st'" -ForegroundColor Yellow
Write-Host "  2. Try: st --help" -ForegroundColor Yellow
Write-Host "  3. Try: st --version" -ForegroundColor Yellow
Write-Host "  4. Try: st . (analyze current directory)" -ForegroundColor Yellow
Write-Host ""
Write-Host "For MCP integration with Claude Desktop:" -ForegroundColor Cyan
Write-Host "  Run: st --mcp-config" -ForegroundColor Yellow
Write-Host ""
Write-Host "Thank you for installing Smart Tree! 🎸" -ForegroundColor Green
