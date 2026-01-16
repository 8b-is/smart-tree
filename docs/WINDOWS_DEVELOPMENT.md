# Windows Development Guide for Smart Tree

This guide helps Windows developers build, test, and contribute to Smart Tree.

## Prerequisites

### 1. Install Rust

Download and install Rust from [rustup.rs](https://rustup.rs/):

```powershell
# Download and run rustup-init.exe
# Or use winget
winget install Rustlang.Rustup
```

This will install:
- `rustc` (Rust compiler)
- `cargo` (Rust package manager)
- MSVC toolchain (automatically via Visual Studio Build Tools)

### 2. Install Visual Studio Build Tools

Rust on Windows requires the Microsoft C++ build tools:

```powershell
# Option 1: Via winget
winget install Microsoft.VisualStudio.2022.BuildTools

# Option 2: Manual download
# https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
```

During installation, select:
- ✅ Desktop development with C++
- ✅ Windows 10/11 SDK

### 3. Install Git (Optional but Recommended)

```powershell
winget install Git.Git
```

### 4. Install Windows Terminal (Optional but Recommended)

For the best development experience:

```powershell
winget install Microsoft.WindowsTerminal
```

## Building Smart Tree

### Clone the Repository

```powershell
git clone https://github.com/8b-is/smart-tree
cd smart-tree
```

### Build Options

```powershell
# Debug build (faster compilation, slower execution)
cargo build

# Release build (optimized, production-ready)
cargo build --release

# Build with specific features
cargo build --release --features mem8

# Build all binaries
cargo build --release --bins
```

The compiled binaries will be in:
- Debug: `target\debug\st.exe`
- Release: `target\release\st.exe`

### Build Verification

```powershell
# Check if build succeeded
.\target\release\st.exe --version

# Test basic functionality
.\target\release\st.exe .
```

## Running Tests

### Unit Tests

```powershell
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in single thread (useful for debugging)
cargo test -- --test-threads=1
```

### Integration Tests

```powershell
# Run library tests
cargo test --lib

# Run doc tests
cargo test --doc

# Run specific integration test
cargo test --test integration_test_name
```

### Linting and Formatting

```powershell
# Check code formatting
cargo fmt -- --check

# Format code automatically
cargo fmt

# Run clippy (linter)
cargo clippy

# Clippy with warnings as errors
cargo clippy -- -D warnings
```

## Development Workflow

### Using the Management Script

Smart Tree includes a PowerShell management script for common tasks:

```powershell
# Build the project
.\scripts\manage.ps1 build

# Run with arguments
.\scripts\manage.ps1 run -- . --mode hex

# Run tests
.\scripts\manage.ps1 test

# Format code
.\scripts\manage.ps1 format

# Clean build artifacts
.\scripts\manage.ps1 clean

# Show project status
.\scripts\manage.ps1 status

# Install locally
.\scripts\manage.ps1 install

# Show examples
.\scripts\manage.ps1 examples
```

### Quick Development Commands

```powershell
# Watch for changes and rebuild (requires cargo-watch)
cargo install cargo-watch
cargo watch -x build

# Build and run in one command
cargo run --release -- .

# Build with verbose output
cargo build --release --verbose

# Check compilation without building
cargo check
```

## Debugging

### Visual Studio Code

1. Install the "rust-analyzer" extension
2. Install the "CodeLLDB" extension for debugging

Create `.vscode/launch.json`:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Smart Tree",
            "cargo": {
                "args": ["build", "--bin=st"],
                "filter": {
                    "name": "st",
                    "kind": "bin"
                }
            },
            "args": ["."],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

### Command-Line Debugging

```powershell
# Build with debug symbols
cargo build

# Run with backtrace on panic
$env:RUST_BACKTRACE = "1"
.\target\debug\st.exe .

# Full backtrace
$env:RUST_BACKTRACE = "full"
.\target\debug\st.exe .

# Enable logging
$env:RUST_LOG = "debug"
.\target\debug\st.exe .
```

## Common Issues and Solutions

### Issue: "link.exe not found"

**Solution**: Install Visual Studio Build Tools with C++ development tools.

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```

### Issue: Slow antivirus scanning

**Solution**: Add exclusions for:
- Your development directory (e.g., `C:\Users\YourName\Projects\smart-tree`)
- `target\release\st.exe`
- `%USERPROFILE%\.cargo`

```powershell
# Windows Defender exclusion (Run as Administrator)
Add-MpPreference -ExclusionPath "C:\Users\YourName\Projects\smart-tree"
Add-MpPreference -ExclusionPath "$env:USERPROFILE\.cargo"
```

### Issue: Long path errors

**Solution**: Enable long path support in Windows:

```powershell
# Run as Administrator
New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" `
    -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force
```

### Issue: Permission denied when accessing files

**Solution**: Smart Tree handles this gracefully. If you need to analyze protected directories:

```powershell
# Run as Administrator (not recommended for normal use)
Start-Process pwsh -Verb RunAs -ArgumentList "-Command", "cd '$PWD'; .\target\release\st.exe C:\Windows"
```

### Issue: Colors not showing in PowerShell

**Solution**: Use Windows Terminal or PowerShell 7+:

```powershell
# Install PowerShell 7
winget install Microsoft.PowerShell
```

## Testing Windows-Specific Features

### Path Handling

Test with various Windows paths:

```powershell
# UNC paths
.\target\release\st.exe \\server\share

# Drive letters
.\target\release\st.exe C:\

# Long paths
.\target\release\st.exe "\\?\C:\very\long\path\that\exceeds\260\characters"

# Mixed separators (should work)
.\target\release\st.exe C:/Users/Name/Documents
```

### File System Features

```powershell
# Test with NTFS features
# Create a junction (symlink-like)
New-Item -ItemType Junction -Path .\test-junction -Target C:\Windows

# Test st with junctions
.\target\release\st.exe .\test-junction

# Cleanup
Remove-Item .\test-junction
```

### Unicode and Emoji

```powershell
# Test with Unicode paths
mkdir ".\测试目录"
.\target\release\st.exe ".\测试目录"

# Test emoji output
.\target\release\st.exe . | Select-String "🌳"

# Cleanup
Remove-Item ".\测试目录"
```

## Performance Profiling

### Basic Benchmarking

```powershell
# Measure execution time
Measure-Command { .\target\release\st.exe . }

# Compare with debug build
Measure-Command { .\target\debug\st.exe . }

# Test on large directory
Measure-Command { .\target\release\st.exe C:\Windows\System32 --depth 3 }
```

### Memory Usage

```powershell
# Monitor memory usage with performance counter
$proc = Start-Process -FilePath ".\target\release\st.exe" -ArgumentList "C:\Windows" -PassThru
while (!$proc.HasExited) {
    $proc.Refresh()
    Write-Host "Memory: $($proc.WorkingSet64 / 1MB) MB"
    Start-Sleep -Seconds 1
}
```

## Contributing from Windows

### Code Style

Smart Tree follows Rust standard style:

```powershell
# Format before committing
cargo fmt

# Check lints
cargo clippy -- -D warnings
```

### Git Configuration

```powershell
# Set line endings to auto-convert
git config core.autocrlf true

# Set your identity
git config user.name "Your Name"
git config user.email "your.email@example.com"
```

### Creating Pull Requests

1. Fork the repository on GitHub
2. Clone your fork:
   ```powershell
   git clone https://github.com/YOUR_USERNAME/smart-tree
   cd smart-tree
   ```
3. Create a branch:
   ```powershell
   git checkout -b feature/my-windows-improvement
   ```
4. Make changes and commit:
   ```powershell
   cargo fmt
   cargo test
   git add .
   git commit -m "feat: improve Windows path handling"
   ```
5. Push and create PR:
   ```powershell
   git push origin feature/my-windows-improvement
   ```

## CI/CD on Windows

Smart Tree uses GitHub Actions for Windows CI:

```yaml
# .github/workflows/rust.yml includes Windows testing
- os: windows-latest
  name: Windows
  rust: stable
```

You can run the same tests locally:

```powershell
# Simulate CI environment
$env:SMART_TREE_NO_UPDATE_CHECK = "1"
$env:RUST_BACKTRACE = "1"
cargo test --release --verbose
```

## Advanced Topics

### Cross-Compilation

Build for different Windows architectures:

```powershell
# Install target
rustup target add x86_64-pc-windows-msvc
rustup target add aarch64-pc-windows-msvc

# Build for x86_64
cargo build --release --target x86_64-pc-windows-msvc

# Build for ARM64 (requires ARM64 tools)
cargo build --release --target aarch64-pc-windows-msvc
```

### Building with Different Toolchains

```powershell
# Install GNU toolchain (alternative to MSVC)
rustup toolchain install stable-x86_64-pc-windows-gnu

# Build with GNU
rustup run stable-x86_64-pc-windows-gnu cargo build --release
```

### Creating Windows Installers

```powershell
# Using cargo-wix (WiX Toolset)
cargo install cargo-wix
cargo wix init
cargo wix
```

## Resources

- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust
- [Cargo Book](https://doc.rust-lang.org/cargo/) - Cargo documentation
- [Smart Tree Documentation](https://github.com/8b-is/smart-tree) - Project docs
- [Windows Dev Center](https://developer.microsoft.com/windows/) - Windows development

## Getting Help

- GitHub Issues: https://github.com/8b-is/smart-tree/issues
- Discussions: https://github.com/8b-is/smart-tree/discussions
- Discord: (if available)

## Quick Reference Card

```powershell
# Essential commands for Windows developers
cargo build --release        # Build optimized binary
cargo test                   # Run all tests
cargo fmt                    # Format code
cargo clippy                 # Lint code
.\target\release\st.exe .   # Run the binary
.\scripts\manage.ps1 help   # Management script help
```

---

**Welcome to Smart Tree development on Windows! 🌳🪟**
