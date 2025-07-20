#!/bin/bash
# MCP Test Suite for Smart Tree v3.3.5
# Tests all MCP functionality to ensure AI assistants can use the tools correctly

set -e  # Exit on error

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Test counters
PASSED=0
FAILED=0

# Smart Tree binary path
ST_BIN="${ST_BIN:-./target/release/st}"

# Check if binary exists
if [ ! -f "$ST_BIN" ]; then
    echo -e "${RED}Error: Smart Tree binary not found at $ST_BIN${NC}"
    echo "Please build with: cargo build --release"
    exit 1
fi

# Function to run an MCP command
run_mcp_cmd() {
    local cmd="$1"
    local description="$2"
    echo -n "Testing: $description... "
    
    # Create a unique request ID
    local req_id=$((RANDOM % 10000))
    
    # Run the command and capture output
    local output=$(echo "$cmd" | timeout 5s "$ST_BIN" --mcp 2>&1 || true)
    
    # Check if we got a valid JSON-RPC response
    if echo "$output" | grep -q '"jsonrpc".*"2.0"'; then
        # Extract just the JSON-RPC response
        local json_response=$(echo "$output" | grep -o '{"jsonrpc".*"id":[0-9]*}' | head -1)
        
        # Check for error in response
        if echo "$json_response" | grep -q '"error"'; then
            echo -e "${RED}FAILED${NC}"
            echo "Error response: $json_response"
            ((FAILED++))
            return 1
        else
            echo -e "${GREEN}PASSED${NC}"
            ((PASSED++))
            return 0
        fi
    else
        echo -e "${RED}FAILED${NC}"
        echo "Invalid response: $output"
        ((FAILED++))
        return 1
    fi
}

# Function to test a tool with expected content
test_tool_content() {
    local cmd="$1"
    local description="$2"
    local expected="$3"
    
    echo -n "Testing: $description... "
    
    local output=$(echo "$cmd" | timeout 5s "$ST_BIN" --mcp 2>&1 || true)
    
    if echo "$output" | grep -q "$expected"; then
        echo -e "${GREEN}PASSED${NC}"
        ((PASSED++))
        return 0
    else
        echo -e "${RED}FAILED${NC}"
        echo "Expected to find: $expected"
        echo "Got: $output"
        ((FAILED++))
        return 1
    fi
}

echo "=== Smart Tree MCP Test Suite v3.3.5 ==="
echo "Binary: $ST_BIN"
echo ""

# Create test directory structure
TEST_DIR=$(mktemp -d)
echo "Creating test directory: $TEST_DIR"

# Create test structure
mkdir -p "$TEST_DIR/.hidden/subdir"
mkdir -p "$TEST_DIR/visible/code"
mkdir -p "$TEST_DIR/docs"

# Create test files with different dates
echo "test content" > "$TEST_DIR/recent.txt"
echo "old content" > "$TEST_DIR/old.txt"
touch -d "2024-01-01" "$TEST_DIR/old.txt"

echo "hidden file" > "$TEST_DIR/.hidden/secret.txt"
echo "# README" > "$TEST_DIR/docs/README.md"
echo "console.log('test');" > "$TEST_DIR/visible/code/test.js"
echo "TODO: implement this" > "$TEST_DIR/visible/code/app.js"

# Test 1: Server Info with Current Time
echo -e "\n${YELLOW}Test Group 1: Server Info${NC}"
test_tool_content \
    '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"server_info","arguments":{}},"id":1}' \
    "server_info shows current time" \
    "current_time"

test_tool_content \
    '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"server_info","arguments":{}},"id":2}' \
    "server_info includes date format hint" \
    "YYYY-MM-DD"

# Test 2: Basic Directory Analysis
echo -e "\n${YELLOW}Test Group 2: Basic Directory Analysis${NC}"
run_mcp_cmd \
    "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"analyze_directory\",\"arguments\":{\"path\":\"$TEST_DIR\",\"mode\":\"ai\"}},\"id\":3}" \
    "analyze_directory with AI mode"

run_mcp_cmd \
    "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"quick_tree\",\"arguments\":{\"path\":\"$TEST_DIR\"}},\"id\":4}" \
    "quick_tree for fast overview"

# Test 3: File Finding with Entry Type
echo -e "\n${YELLOW}Test Group 3: Entry Type Filtering${NC}"
run_mcp_cmd \
    "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"find_files\",\"arguments\":{\"path\":\"$TEST_DIR\",\"pattern\":\".*\",\"entry_type\":\"d\"}},\"id\":5}" \
    "find_files with directories only (entry_type='d')"

run_mcp_cmd \
    "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"find_files\",\"arguments\":{\"path\":\"$TEST_DIR\",\"pattern\":\".*\",\"entry_type\":\"f\"}},\"id\":6}" \
    "find_files with files only (entry_type='f')"

# Test 4: Hidden Directory Handling
echo -e "\n${YELLOW}Test Group 4: Hidden Directory Handling${NC}"
run_mcp_cmd \
    "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"analyze_directory\",\"arguments\":{\"path\":\"$TEST_DIR\",\"show_hidden\":false}},\"id\":7}" \
    "analyze_directory without hidden files"

run_mcp_cmd \
    "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"analyze_directory\",\"arguments\":{\"path\":\"$TEST_DIR\",\"show_hidden\":true}},\"id\":8}" \
    "analyze_directory with hidden files"

# Test 5: Time-based Searches
echo -e "\n${YELLOW}Test Group 5: Time-based File Searches${NC}"
YESTERDAY=$(date -d "yesterday" +%Y-%m-%d)
TOMORROW=$(date -d "tomorrow" +%Y-%m-%d)
LAST_YEAR=$(date -d "last year" +%Y-%m-%d)

run_mcp_cmd \
    "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"find_recent_changes\",\"arguments\":{\"path\":\"$TEST_DIR\",\"days\":7}},\"id\":9}" \
    "find_recent_changes (last 7 days)"

run_mcp_cmd \
    "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"find_in_timespan\",\"arguments\":{\"path\":\"$TEST_DIR\",\"start_date\":\"$YESTERDAY\",\"end_date\":\"$TOMORROW\"}},\"id\":10}" \
    "find_in_timespan (yesterday to tomorrow)"

run_mcp_cmd \
    "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"find_in_timespan\",\"arguments\":{\"path\":\"$TEST_DIR\",\"start_date\":\"$LAST_YEAR\"}},\"id\":11}" \
    "find_in_timespan (from last year)"

# Test 6: Content Search
echo -e "\n${YELLOW}Test Group 6: Content Search${NC}"
run_mcp_cmd \
    "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"search_in_files\",\"arguments\":{\"path\":\"$TEST_DIR\",\"keyword\":\"TODO\"}},\"id\":12}" \
    "search_in_files for TODO"

# Test 7: File Type Searches
echo -e "\n${YELLOW}Test Group 7: File Type Searches${NC}"
run_mcp_cmd \
    "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"find_code_files\",\"arguments\":{\"path\":\"$TEST_DIR\"}},\"id\":13}" \
    "find_code_files"

run_mcp_cmd \
    "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"find_documentation\",\"arguments\":{\"path\":\"$TEST_DIR\"}},\"id\":14}" \
    "find_documentation"

# Test 8: Statistics
echo -e "\n${YELLOW}Test Group 8: Statistics and Analysis${NC}"
run_mcp_cmd \
    "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"get_statistics\",\"arguments\":{\"path\":\"$TEST_DIR\"}},\"id\":15}" \
    "get_statistics"

run_mcp_cmd \
    "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"directory_size_breakdown\",\"arguments\":{\"path\":\"$TEST_DIR\"}},\"id\":16}" \
    "directory_size_breakdown"

# Test 9: Advanced Features
echo -e "\n${YELLOW}Test Group 9: Advanced Features${NC}"
run_mcp_cmd \
    "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"semantic_analysis\",\"arguments\":{\"path\":\"$TEST_DIR\"}},\"id\":17}" \
    "semantic_analysis"

# Test 10: Error Handling
echo -e "\n${YELLOW}Test Group 10: Error Handling${NC}"
run_mcp_cmd \
    "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"analyze_directory\",\"arguments\":{\"path\":\"/nonexistent/path\"}},\"id\":18}" \
    "analyze_directory with invalid path (should fail gracefully)"

# Cleanup
echo -e "\nCleaning up test directory..."
rm -rf "$TEST_DIR"

# Summary
echo -e "\n=== Test Summary ==="
echo -e "Total Tests: $((PASSED + FAILED))"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"

if [ $FAILED -eq 0 ]; then
    echo -e "\n${GREEN}All tests passed! 🎉${NC}"
    exit 0
else
    echo -e "\n${RED}Some tests failed. Please fix before releasing.${NC}"
    exit 1
fi