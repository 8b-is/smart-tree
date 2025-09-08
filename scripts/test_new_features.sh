#!/bin/bash
# Test runner for new Smart Tree features
# Focuses on Claude integration, MCP session negotiation, and context mode

set -e  # Exit on error

echo "🧪 Smart Tree Feature Test Suite 🧪"
echo "===================================="
echo ""

# Colors for output (Trisha loves organized, colorful output!)
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counter
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "${BLUE}Running:${NC} $test_name"
    TESTS_RUN=$((TESTS_RUN + 1))
    
    if eval "$test_command" > /tmp/test_output.log 2>&1; then
        echo -e "  ${GREEN}✓ PASSED${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "  ${RED}✗ FAILED${NC}"
        echo -e "  ${YELLOW}Output:${NC}"
        cat /tmp/test_output.log | head -20
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
    echo ""
}

echo "📦 Building Smart Tree in release mode..."
cargo build --release --quiet

echo ""
echo "🧪 Unit Tests"
echo "-------------"

# Run Rust unit tests
run_test "Claude initialization tests" \
    "cargo test --test test_claude_integration --release"

run_test "MCP session negotiation tests" \
    "cargo test --test test_mcp_session --release"

run_test "Compression mode tests" \
    "cargo test compression --release"

run_test "Context mode formatter tests" \
    "cargo test context_mode --release"

echo ""
echo "🔧 Integration Tests"
echo "--------------------"

# Test the actual st binary
run_test "st --setup-claude command" \
    "cd /tmp && rm -rf st_test && mkdir st_test && cd st_test && \
     echo 'fn main() {}' > main.rs && \
     $PWD/../../../target/release/st --setup-claude && \
     test -f .claude/settings.json && test -f .claude/CLAUDE.md"

run_test "st --mode context output" \
    "$PWD/target/release/st --mode context . | grep -q 'Smart Tree Context'"

run_test "Environment variable compression" \
    "ST_COMPRESSION=quantum $PWD/target/release/st --mode context . | head -1 | grep -q '=== Smart Tree Context ==='"

echo ""
echo "🚀 MCP Tests"
echo "------------"

# Test MCP with session awareness
run_test "MCP session-aware mode" \
    "ST_SESSION_AWARE=1 timeout 1 $PWD/target/release/st --mcp < /dev/null 2>&1 | grep -q 'Starting Smart Tree MCP Server' || true"

run_test "MCP tools list" \
    "$PWD/target/release/st --mcp-tools | grep -q 'Available Tools'"

echo ""
echo "📊 Performance Tests"
echo "--------------------"

# Create test directories with different sizes
SMALL_DIR="/tmp/st_small_test"
LARGE_DIR="/tmp/st_large_test"

# Small project test
run_test "Small project compression (should use 'none')" \
    "rm -rf $SMALL_DIR && mkdir -p $SMALL_DIR && \
     touch $SMALL_DIR/file{1..10}.txt && \
     ST_COMPRESSION=auto $PWD/target/release/st $SMALL_DIR | wc -l | \
     awk '{if (\$1 < 20) exit 0; else exit 1}'"

# Large project simulation
run_test "Large project compression (should compress)" \
    "rm -rf $LARGE_DIR && mkdir -p $LARGE_DIR/{src,tests,docs}/{sub1,sub2} && \
     touch $LARGE_DIR/{src,tests,docs}/{sub1,sub2}/file{1..50}.rs && \
     ST_COMPRESSION=auto $PWD/target/release/st --mode quantum $LARGE_DIR | \
     head -1 | grep -q 'QUANTUM'"

echo ""
echo "🎨 Hook Integration Tests"
echo "-------------------------"

# Test hook configurations
run_test "Hook configuration in settings.json" \
    "cd /tmp/st_test && cat .claude/settings.json | \
     jq -e '.hooks.UserPromptSubmit[0].hooks[0].command | contains(\"st -m\")'"

run_test "Project type detection" \
    "cd /tmp && rm -rf py_test && mkdir py_test && cd py_test && \
     echo 'print()' > test.py && echo 'pytest' > requirements.txt && \
     $PWD/../../../target/release/st --setup-claude && \
     cat .claude/settings.json | grep -q '\"project_type\": \"Python\"'"

echo ""
echo "========================================"
echo "📈 Test Results Summary"
echo "========================================"
echo -e "Tests Run:    ${BLUE}$TESTS_RUN${NC}"
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo ""
    echo -e "${GREEN}🎉 All tests passed! Trisha would be proud of this audit!${NC}"
    echo "Aye, Aye! 🚢"
    exit 0
else
    echo ""
    echo -e "${RED}⚠️  Some tests failed. Check the output above for details.${NC}"
    echo "Don't worry Hue, we'll fix these together! 💪"
    exit 1
fi