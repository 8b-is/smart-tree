# 🌳 Smart Tree Management Script for Windows 🌳
# PowerShell equivalent of manage.sh for Windows developers

param(
    [Parameter(Position = 0)]
    [string]$Command = "help",
    
    [Parameter(Position = 1, ValueFromRemainingArguments = $true)]
    [string[]]$Arguments
)

# Project info
$ProjectName = "Smart Tree (st)"
$ProjectDir = Split-Path -Parent $PSScriptRoot
$BinaryName = "st.exe"

# Helper functions
function Write-Header {
    param([string]$Message)
    Write-Host ""
    Write-Host "🌳 $Message 🌳" -ForegroundColor Cyan
    Write-Host ""
}

function Write-Success {
    param([string]$Message)
    Write-Host "✅ $Message" -ForegroundColor Green
}

function Write-Error {
    param([string]$Message)
    Write-Host "❌ $Message" -ForegroundColor Red
    exit 1
}

function Write-Info {
    param([string]$Message)
    Write-Host "📊 $Message" -ForegroundColor Blue
}

function Write-Warning {
    param([string]$Message)
    Write-Host "⚠️  $Message" -ForegroundColor Yellow
}

# Build the project
function Invoke-Build {
    param(
        [string]$BuildType = "release",
        [string]$Features = ""
    )
    
    Write-Header "Building $ProjectName in $BuildType mode ⚙️"
    
    Push-Location $ProjectDir
    
    try {
        $featureFlags = ""
        if ($Features) {
            $featureFlags = "--features $Features"
            Write-Info "Building with features: $Features"
        }
        
        if ($BuildType -eq "release") {
            Write-Info "Optimizing for maximum speed... 🚀"
            cargo build --release $featureFlags
            
            $binaryPath = Join-Path $ProjectDir "target\release\$BinaryName"
            if (Test-Path $binaryPath) {
                $size = (Get-Item $binaryPath).Length / 1MB
                Write-Success "Release build complete! Binary size: $([math]::Round($size, 2)) MB"
            }
        } else {
            Write-Info "Building debug version with all the debugging goodies..."
            cargo build $featureFlags
            Write-Success "Debug build complete!"
        }
    } finally {
        Pop-Location
    }
}

# Run the project
function Invoke-Run {
    Write-Header "Running $ProjectName 🚀"
    
    Push-Location $ProjectDir
    
    try {
        if ($Arguments.Count -eq 0) {
            Write-Info "No arguments provided, analyzing current directory..."
            cargo run --release -- .
        } else {
            cargo run --release -- $Arguments
        }
    } finally {
        Pop-Location
    }
}

# Run tests
function Invoke-Test {
    Write-Header "Testing $ProjectName 🧪"
    
    Push-Location $ProjectDir
    
    try {
        Write-Info "Running unit tests..."
        cargo test
        
        Write-Info "Running clippy (our friendly neighborhood linter)..."
        cargo clippy -- -D warnings
        
        Write-Info "Checking formatting..."
        cargo fmt -- --check
        
        Write-Success "All tests passed! Your tree is healthy! 🌳"
    } catch {
        Write-Warning "Some checks failed!"
    } finally {
        Pop-Location
    }
}

# Format code
function Invoke-Format {
    Write-Header "Formatting code ✨"
    
    Push-Location $ProjectDir
    
    try {
        cargo fmt
        Write-Success "Code formatted! Looking prettier than a bonsai tree! 🎋"
    } finally {
        Pop-Location
    }
}

# Clean build artifacts
function Invoke-Clean {
    Write-Header "Cleaning up 🧹"
    
    Push-Location $ProjectDir
    
    try {
        cargo clean
        Write-Success "All clean! Fresh as a spring forest! 🌱"
    } finally {
        Pop-Location
    }
}

# Show project status
function Show-Status {
    Write-Header "Project Status 📊"
    
    Push-Location $ProjectDir
    
    try {
        Write-Host "Project: " -NoNewline -ForegroundColor Magenta
        Write-Host $ProjectName
        
        Write-Host "Location: " -NoNewline -ForegroundColor Magenta
        Write-Host $ProjectDir
        
        Write-Host "Rust version: " -NoNewline -ForegroundColor Magenta
        rustc --version
        
        Write-Host "Cargo version: " -NoNewline -ForegroundColor Magenta
        cargo --version
        
        $releaseBinary = Join-Path $ProjectDir "target\release\$BinaryName"
        if (Test-Path $releaseBinary) {
            $size = (Get-Item $releaseBinary).Length / 1MB
            $modified = (Get-Item $releaseBinary).LastWriteTime
            
            Write-Host "Release binary: " -NoNewline -ForegroundColor Magenta
            Write-Host "$([math]::Round($size, 2)) MB"
            
            Write-Host "Last modified: " -NoNewline -ForegroundColor Magenta
            Write-Host $modified.ToString("yyyy-MM-dd HH:mm:ss")
        } else {
            Write-Host "Release binary: " -NoNewline -ForegroundColor Magenta
            Write-Host "Not built yet"
        }
        
        Write-Host ""
        Write-Host "Dependencies:" -ForegroundColor Magenta
        cargo tree --depth 1 | Select-Object -First 20
        
        Write-Host ""
        Write-Host "Git status:" -ForegroundColor Magenta
        if (Get-Command git -ErrorAction SilentlyContinue) {
            git status --short
        } else {
            Write-Host "  Git not found"
        }
    } finally {
        Pop-Location
    }
}

# Install binary
function Install-Binary {
    param([string]$InstallDir = "$env:LOCALAPPDATA\Programs\st")
    
    Write-Header "Installing $ProjectName 🎯"
    
    Push-Location $ProjectDir
    
    try {
        Write-Info "Building release version..."
        cargo build --release
        
        if (-not (Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }
        
        $sourcePath = Join-Path $ProjectDir "target\release\$BinaryName"
        $destPath = Join-Path $InstallDir $BinaryName
        
        Copy-Item $sourcePath $destPath -Force
        Write-Success "Installed to $destPath"
        
        # Check if in PATH
        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($userPath -notlike "*$InstallDir*") {
            Write-Info "Adding $InstallDir to your PATH..."
            $newPath = "$userPath;$InstallDir"
            [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
            Write-Success "Added to PATH. Please restart your terminal."
        }
        
        Write-Info "You can now use '$BinaryName' from anywhere! 🚀"
    } finally {
        Pop-Location
    }
}

# Show examples
function Show-Examples {
    Write-Header "Usage Examples ✨"
    
    Write-Host @"
Basic usage:
  st                          # Analyze current directory
  st C:\path\to\dir           # Analyze specific directory
  
Output modes:
  st -m hex                   # Hexadecimal format (AI-friendly)
  st -m json                  # JSON output
  st -m ai                    # AI-optimized format
  st -m digest                # Super compact digest
  st -m stats                 # Statistics only
  
Filtering:
  st --find "*.rs"            # Find Rust files
  st --type rs                # Only .rs files
  st --min-size 1M            # Files larger than 1MB
  
Options:
  st --no-emoji               # Plain text output
  st --depth 3                # Limit depth
  st -z                       # Compress output
  
Streaming Mode:
  st --stream                 # Stream output as files are found
  st --stream -m hex          # Great for huge directories
  
File Content Search:
  st --search "TODO"          # Find TODO in all text files
  st --type rs --search "fn"  # Search for "fn" in Rust files
  
PowerShell Integration:
  st --mode json . | ConvertFrom-Json
  st | Out-File tree.txt
  
MCP (Model Context Protocol):
  .\manage.ps1 mcp-run        # Run as MCP server
  .\manage.ps1 mcp-config     # Show Claude Desktop config
"@
}

# Show help
function Show-Help {
    Write-Host @"
🌳 Smart Tree Management Script for Windows 🌳

Usage: .\manage.ps1 [command] [options]

Commands:
  build [debug|release]       Build the project
  run [args...]              Run st with arguments
  test                       Run tests, linting, and format check
  format                     Format code with rustfmt
  clean                      Clean build artifacts
  status                     Show project status
  install [dir]              Install binary (default: %LOCALAPPDATA%\Programs\st)
  examples                   Show usage examples
  help                       Show this help message

MCP Commands:
  mcp-run                    Run as MCP server
  mcp-config                 Show Claude Desktop configuration

Examples:
  .\manage.ps1 build         # Build release version
  .\manage.ps1 run -- -m hex .  # Run with hex output on current dir
  .\manage.ps1 test          # Run all tests
  .\manage.ps1 install       # Install to default location

Made with ✨ and 🌳 by the Smart Tree team!
"@
}

# MCP commands
function Invoke-McpRun {
    Write-Header "Running MCP server 🤖"
    
    Push-Location $ProjectDir
    
    try {
        $binaryPath = Join-Path $ProjectDir "target\release\$BinaryName"
        if (-not (Test-Path $binaryPath)) {
            Write-Warning "Binary not found. Building release version..."
            Invoke-Build -BuildType release
        }
        
        Write-Info "Starting MCP server on stdio..."
        Write-Info "Press Ctrl+C to stop"
        & $binaryPath --mcp
    } finally {
        Pop-Location
    }
}

function Show-McpConfig {
    Write-Header "MCP Configuration 🤖"
    
    Push-Location $ProjectDir
    
    try {
        $binaryPath = Join-Path $ProjectDir "target\release\$BinaryName"
        if (-not (Test-Path $binaryPath)) {
            Write-Warning "Building release version first..."
            Invoke-Build -BuildType release
        }
        
        & $binaryPath --mcp-config
    } finally {
        Pop-Location
    }
}

# Main command dispatcher
switch ($Command.ToLower()) {
    "build" {
        $buildType = if ($Arguments.Count -gt 0) { $Arguments[0] } else { "release" }
        $features = if ($Arguments.Count -gt 1) { $Arguments[1] } else { "" }
        Invoke-Build -BuildType $buildType -Features $features
    }
    "run" {
        Invoke-Run
    }
    "test" {
        Invoke-Test
    }
    "format" {
        Invoke-Format
    }
    "fmt" {
        Invoke-Format
    }
    "clean" {
        Invoke-Clean
    }
    "status" {
        Show-Status
    }
    "info" {
        Show-Status
    }
    "install" {
        $installDir = if ($Arguments.Count -gt 0) { $Arguments[0] } else { "$env:LOCALAPPDATA\Programs\st" }
        Install-Binary -InstallDir $installDir
    }
    "examples" {
        Show-Examples
    }
    "mcp-run" {
        Invoke-McpRun
    }
    "mcp-config" {
        Show-McpConfig
    }
    "help" {
        Show-Help
    }
    "-h" {
        Show-Help
    }
    "--help" {
        Show-Help
    }
    default {
        Write-Error "Unknown command: $Command"
        Write-Host ""
        Show-Help
    }
}
