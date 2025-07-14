#!/bin/bash
# Focused test suite for v3.3.5 specific features
# Tests: hidden directory handling, entry-type filtering, time-aware tools

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

ST_BIN="${ST_BIN:-./target/release/st}"

echo -e "${BLUE}=== Smart Tree v3.3.5 Feature Tests ===${NC}"
echo ""

# Test 1: Hidden Directory Traversal Fix
echo -e "${YELLOW}Test 1: Hidden Directory Traversal${NC}"
echo "Creating test structure with hidden directories..."

TEST_DIR=$(mktemp -d)
mkdir -p "$TEST_DIR/.hidden/level1/level2"
echo "visible" > "$TEST_DIR/visible.txt"
echo "hidden" > "$TEST_DIR/.hidden/secret.txt"
echo "nested" > "$TEST_DIR/.hidden/level1/level2/deep.txt"

echo -n "Testing: Hidden directories are not traversed without -a flag... "
OUTPUT=$("$ST_BIN" "$TEST_DIR" --mode ai 2>&1)
if echo "$OUTPUT" | grep -q "deep.txt"; then
    echo -e "${RED}FAILED${NC} - Found deep.txt when it should be hidden"
    echo "$OUTPUT"
else
    echo -e "${GREEN}PASSED${NC}"
fi

echo -n "Testing: Hidden directories ARE traversed with --everything flag... "
OUTPUT=$("$ST_BIN" "$TEST_DIR" --mode ai --everything 2>&1)
if echo "$OUTPUT" | grep -q "deep.txt"; then
    echo -e "${GREEN}PASSED${NC}"
else
    echo -e "${RED}FAILED${NC} - deep.txt not found with --everything flag"
fi

# Test 2: Entry Type Filtering
echo -e "\n${YELLOW}Test 2: Entry Type Filtering${NC}"

mkdir -p "$TEST_DIR/mixed"
touch "$TEST_DIR/mixed/file1.txt" "$TEST_DIR/mixed/file2.txt"
mkdir -p "$TEST_DIR/mixed/dir1" "$TEST_DIR/mixed/dir2"

echo -n "Testing: --entry-type d shows only directories... "
OUTPUT=$("$ST_BIN" "$TEST_DIR/mixed" --find ".*" --entry-type d --mode ai 2>&1)
FILE_COUNT=$(echo "$OUTPUT" | grep -c "file[0-9].txt" || true)
DIR_COUNT=$(echo "$OUTPUT" | grep -c "dir[0-9]" || true)
if [ "$FILE_COUNT" -eq 0 ] && [ "$DIR_COUNT" -gt 0 ]; then
    echo -e "${GREEN}PASSED${NC}"
else
    echo -e "${RED}FAILED${NC} - Found $FILE_COUNT files, $DIR_COUNT dirs"
fi

echo -n "Testing: --entry-type f shows only files... "
OUTPUT=$("$ST_BIN" "$TEST_DIR/mixed" --find ".*" --entry-type f --mode ai 2>&1)
FILE_COUNT=$(echo "$OUTPUT" | grep -c "file[0-9].txt" || true)
DIR_COUNT=$(echo "$OUTPUT" | grep -c " dir[0-9]" || true)
if [ "$FILE_COUNT" -gt 0 ] && [ "$DIR_COUNT" -eq 0 ]; then
    echo -e "${GREEN}PASSED${NC}"
else
    echo -e "${RED}FAILED${NC} - Found $FILE_COUNT files, $DIR_COUNT dirs"
fi

# Test 3: MCP Time Awareness
echo -e "\n${YELLOW}Test 3: MCP Time Awareness${NC}"

echo -n "Testing: server_info includes current_time... "
MCP_OUTPUT=$(echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"server_info","arguments":{}},"id":1}' | timeout 5s "$ST_BIN" --mcp 2>&1 || true)
if echo "$MCP_OUTPUT" | grep -q "current_time"; then
    echo -e "${GREEN}PASSED${NC}"
    # Extract and show the time
    TIME_INFO=$(echo "$MCP_OUTPUT" | grep -o '"current_time":[^}]*}' | head -1)
    echo "  Found time info in response"
else
    echo -e "${RED}FAILED${NC}"
    echo "  No current_time found in server_info"
fi

echo -n "Testing: find_in_timespan tool exists... "
MCP_OUTPUT=$(echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}' | timeout 5s "$ST_BIN" --mcp 2>&1 || true)
if echo "$MCP_OUTPUT" | grep -q "find_in_timespan"; then
    echo -e "${GREEN}PASSED${NC}"
else
    echo -e "${RED}FAILED${NC}"
    echo "  find_in_timespan not found in tools list"
fi

# Test 4: LS Mode with Filtered Results
echo -e "\n${YELLOW}Test 4: LS Mode Path Display${NC}"

echo -n "Testing: LS mode shows directories containing matches... "
OUTPUT=$("$ST_BIN" "$TEST_DIR" --find "secret" --mode ls --everything 2>&1)
if echo "$OUTPUT" | grep -q ".hidden"; then
    echo -e "${GREEN}PASSED${NC}"
else
    echo -e "${RED}FAILED${NC}"
    echo "  Expected .hidden directory in ls output"
    echo "  Got: $OUTPUT"
fi

# Test 5: Date Range Search via MCP
echo -e "\n${YELLOW}Test 5: Date Range Search${NC}"

# Create files with specific dates
touch "$TEST_DIR/today.txt"
touch -d "2025-07-01" "$TEST_DIR/july1.txt"
touch -d "2025-06-01" "$TEST_DIR/june1.txt"

echo -n "Testing: find_in_timespan with date range... "
MCP_CMD=$(cat <<EOF
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"find_in_timespan","arguments":{"path":"$TEST_DIR","start_date":"2025-06-30","end_date":"2025-07-02"}},"id":3}
EOF
)
MCP_OUTPUT=$(echo "$MCP_CMD" | timeout 5s "$ST_BIN" --mcp 2>&1 || true)
if echo "$MCP_OUTPUT" | grep -q '"found": 1' || echo "$MCP_OUTPUT" | grep -q "july1.txt"; then
    echo -e "${GREEN}PASSED${NC}"
else
    echo -e "${RED}FAILED${NC}"
    echo "  Expected to find july1.txt in date range"
    # Debug output
    echo "$MCP_OUTPUT" | grep -o '{"jsonrpc".*}' | jq -r '.result.content[0].text' 2>/dev/null | head -20 || echo "  Could not parse output"
fi

# Cleanup
rm -rf "$TEST_DIR"

echo -e "\n${BLUE}=== Test Complete ===${NC}"
echo "All v3.3.5 specific features have been tested!"