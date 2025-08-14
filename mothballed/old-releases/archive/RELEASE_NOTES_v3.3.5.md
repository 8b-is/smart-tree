# Smart Tree v3.3.5 Release Notes

**Release Date**: 2025-01-13  
**Focus**: Critical Bug Fixes & Cross-Platform Stability

## 🐛 Critical Bug Fixes

### Fixed: ST_DEFAULT_MODE Environment Variable Precedence
- **Issue**: `ST_DEFAULT_MODE` incorrectly overrode explicit `--mode` command line arguments
- **Impact**: Users couldn't override their environment variable with command line flags
- **Solution**: 
  - Changed default mode from "classic" to "auto" to detect explicit usage
  - Updated mode selection logic to prioritize command line arguments
  - Now `ST_DEFAULT_MODE=hex st --mode ls` correctly shows ls mode ✅

**Correct precedence order:**
1. AI_TOOLS environment variable (highest)
2. --semantic flag
3. **--mode command line argument** (now properly prioritized)
4. ST_DEFAULT_MODE environment variable
5. Default (classic mode)

### Fixed: MCP Integration Tests on GitHub Actions
- **Issue**: MCP tests failed in CI because they hardcoded `./target/release/st` path
- **Impact**: All GitHub Actions builds were failing on test phase
- **Solution**: Made tests robust by falling back to debug binary when release isn't available
- **Result**: Tests now work in both local development and CI environments ✅

### Fixed: Entry Type Filtering Test
- **Issue**: `test_entry_type_filtering` expected 2 directories but found 3 (including root)
- **Impact**: Test failures when filtering directories with `entry_type=d`
- **Solution**: Updated test to properly filter out temporary root directory
- **Result**: Test now correctly validates directory-only filtering ✅

### Fixed: Windows Hidden Directory Test
- **Issue**: Test failed on Windows due to hardcoded Unix paths and hidden directory handling
- **Impact**: All Windows builds failing in CI
- **Solution**: 
  - Made hidden directory creation graceful (handles Windows differences)
  - Removed hardcoded `/tmp/` paths that don't exist on Windows
  - Added cross-platform fallback logic with helpful debug output
- **Result**: Tests now pass on Windows, macOS, and Linux ✅

## 🏗️ Internal Improvements

### Enhanced File Type Detection
- **Improvement**: Fixed emoji spacing issues in ls formatter
- **Details**: Some emojis were consuming the space before filenames
- **Solution**: Ensured consistent spacing with `format!("{} ", emoji)`
- **Result**: Better visual alignment in ls mode output

### Cross-Platform Test Robustness
- **Improvement**: All MCP integration tests now handle platform differences gracefully
- **Coverage**: Tests work across Unix, macOS, and Windows environments
- **Debugging**: Added helpful debug output for Windows-specific issues

## 🔧 Technical Details

### Mode Selection Logic Refactor
```rust
// New approach: Check if user explicitly provided --mode
else if args.mode != OutputMode::Auto {
    // User explicitly specified a mode - this takes precedence!
    (args.mode, args.compress)
} else if let Some(env_mode) = default_mode_env {
    // Fall back to ST_DEFAULT_MODE if no explicit mode
    (env_mode, args.compress)
} else {
    // Default to classic mode
    (OutputMode::Classic, args.compress)
}
```

### Cross-Platform Test Strategy
- **Fallback logic**: Try release binary first, then debug binary
- **Windows compatibility**: Graceful handling of file system differences
- **Debug output**: Helpful diagnostics when tests encounter platform issues

## 📊 Quality Metrics

- **Test Coverage**: All 4 MCP integration tests passing ✅
- **Cross-Platform**: Tests verified on Ubuntu, macOS, and Windows ✅
- **Backward Compatibility**: All existing functionality preserved ✅
- **Performance**: No performance regressions introduced ✅

## 🚀 Upgrade Notes

This is a **recommended upgrade** for all users, especially if you:
- Use `ST_DEFAULT_MODE` environment variable
- Run smart-tree in CI/CD environments
- Use Windows for development
- Rely on MCP integration tests

### Upgrading
```bash
# Using the install script
curl -sSL https://raw.githubusercontent.com/8b-is/smart-tree/main/scripts/install.sh | bash

# Using cargo
cargo install --git https://github.com/8b-is/smart-tree

# Using the manage script (for developers)
./scripts/manage.sh install
```

## 🙏 Acknowledgments

Special thanks to the GitHub Actions runners that helped us catch these cross-platform issues! The Windows runner that kept failing was actually helping us identify real compatibility problems.

---

**Full Changelog**: [v3.3.5...v3.3.5](https://github.com/8b-is/smart-tree/compare/v3.3.5...v3.3.5)  
**Download**: [GitHub Releases](https://github.com/8b-is/smart-tree/releases/tag/v3.3.5)