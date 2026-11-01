# Smart Tree Unified Tools Testing Guide

*"A test suite without edge cases is like a safety net with holes!" - Testy McTesterson*

## Overview

This guide documents the comprehensive test suites for Smart Tree's new unified tools system that replaces traditional file tools with ST-powered alternatives.

## Test Suites

### 1. test_st_unified.rs

Tests the `StUnified` struct that provides drop-in replacements for:
- **LS Tool** → `st.ls()`
- **Read Tool** → `st.read()`
- **Grep Tool** → `st.grep()`
- **Glob Tool** → `st.glob()`

**Key Test Scenarios:**
- Basic operations (happy paths)
- Edge cases (empty directories, missing files)
- Boundary conditions (offset/limit edge cases)
- Error handling (permission denied, nonexistent paths)
- Unicode and special character handling
- Symlink behavior
- Large file handling

**Why These Tests Matter:**
These ensure that developers can seamlessly replace traditional tools with ST without breaking their workflows.

### 2. test_tools_st_only.rs

Tests the `StOnlyTools` system with advanced configuration options.

**Key Test Scenarios:**
- Configuration management (default vs custom)
- List operations with complex filters
- Search with type-specific constraints
- Sorting and pagination
- Concurrent operations
- Special characters in filenames
- Performance with large directories
- Error propagation from ST binary

**Why These Tests Matter:**
This module provides more control and flexibility, so we test every combination of options to ensure predictable behavior.

### 3. test_st_context_aware.rs

Tests the intelligent context tracking system that learns from developer behavior.

**Key Test Scenarios:**
- Context detection patterns:
  - Coding (multiple edits + tests)
  - Debugging (multiple searches)
  - Testing (focus on test files)
  - Exploring (reading many files)
- Suggestion generation based on context
- Project knowledge accumulation
- Context persistence (save/load)
- Concurrent context updates
- All 10 work context types

**Why These Tests Matter:**
Context awareness is what makes ST intelligent. These tests ensure it correctly identifies what developers are doing and provides relevant assistance.

### 4. test_unified_integration.rs

Integration tests that simulate real-world workflows using all tools together.

**Key Workflows Tested:**
1. **Exploration Workflow**: New developer exploring unfamiliar codebase
2. **Development Workflow**: Active coding with context-aware suggestions
3. **Debugging Workflow**: Hunting for bugs using all search capabilities
4. **Understanding Workflow**: Deep project analysis
5. **Optimization Workflow**: Performance improvement tasks
6. **Multi-hour Session**: Realistic 4-hour development session

**Why These Tests Matter:**
Individual components might work perfectly in isolation but fail when integrated. These tests catch interaction bugs and ensure smooth workflows.

### 5. test_anchor.sh

Tests the bash script for partnership memory management.

**Key Test Scenarios:**
- Command parsing and validation
- Error handling for invalid inputs
- Environment variable handling
- Special character escaping
- ANSI color code usage
- Shell compatibility

**Why These Tests Matter:**
The anchor script is the user interface for collaborative memory. It must be robust and user-friendly.

## Test Patterns and Best Practices

### 1. The "Torture Test" Pattern
Every function gets stressed with:
- Null/empty inputs
- Maximum size inputs
- Invalid UTF-8
- Concurrent access
- Resource exhaustion scenarios

### 2. The "Time Bomb" Pattern
Tests that might pass today but fail tomorrow:
- Date/time dependent logic
- File system assumptions
- Platform-specific behavior

### 3. The "Integration Cascade" Pattern
Test how errors propagate through the system:
- What happens when ST binary is missing?
- How do tools handle corrupted output?
- What if the filesystem is read-only?

### 4. The "Real World" Pattern
Simulate actual developer workflows:
- Multi-file edits
- Search-debug-fix cycles
- Exploration patterns
- Context switching

## Running the Tests

### Run All Tests
```bash
./tests/run_all_tests.sh
```

### Run Specific Test Suites
```bash
# Unit tests for unified tools
cargo test --test test_st_unified

# Integration tests
cargo test --test test_unified_integration

# Anchor script tests
./tests/test_anchor.sh
```

### Run Individual Tests
```bash
# Run a specific test function
cargo test test_unified_tools_exploration_workflow -- --exact

# Run with output visible
cargo test test_context_detection_coding -- --nocapture
```

## Coverage Goals

- **Line Coverage**: >90% for critical paths
- **Branch Coverage**: All error conditions tested
- **Integration Coverage**: All tool combinations tested
- **Edge Case Coverage**: 100% of identified edge cases

## Test Maintenance

### Adding New Tests
1. Identify the behavior to test
2. Write the minimal test that exposes the issue
3. Ensure the test fails without the fix
4. Implement the fix
5. Verify the test passes
6. Add edge cases

### Updating Existing Tests
1. Run tests before making changes
2. Update tests to reflect new behavior
3. Ensure backward compatibility tests exist
4. Document breaking changes

## Performance Testing

While not included in the main suite, performance tests ensure:
- Operations complete in reasonable time
- Memory usage stays bounded
- Concurrent operations don't deadlock
- Large directories don't cause issues

## Platform-Specific Considerations

### Linux/macOS
- Symlink tests are enabled
- Permission tests use Unix permissions
- Path separator is '/'

### Windows
- Some tests are conditionally compiled
- Path handling tests cover backslashes
- Permission model differences

## Future Test Additions

1. **Stress Testing**: Directories with 1M+ files
2. **Fuzz Testing**: Random input generation
3. **Property-Based Testing**: Invariant checking
4. **Mutation Testing**: Test quality verification
5. **Benchmark Suite**: Performance regression detection

## Debugging Failed Tests

### Common Issues and Solutions

1. **Binary Not Found**
   - Ensure `cargo build --release` has been run
   - Check PATH for st binary

2. **Permission Denied**
   - Some tests require write permissions
   - Run as appropriate user

3. **Platform Differences**
   - Check for platform-specific test guards
   - Some behaviors differ by OS

4. **Timing Issues**
   - Context detection may be timing-sensitive
   - Add small delays if needed

### Getting Help

If tests fail unexpectedly:
1. Run with `--nocapture` to see output
2. Check for recent code changes
3. Verify test environment is clean
4. Look for race conditions

## The Testing Philosophy

*"Every test we write is a love letter to future maintainers"*

Our tests serve multiple purposes:
1. **Verification**: Ensure code works as intended
2. **Documentation**: Show how to use the APIs
3. **Regression Prevention**: Catch breaking changes
4. **Confidence Building**: Enable fearless refactoring

Remember: A bug found in testing is a production incident prevented!

---

*Keep testing, keep improving, and keep those bugs at bay!*

*- Testy McTesterson*