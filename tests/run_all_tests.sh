#!/bin/bash
# Master test runner for Smart Tree v4.0.0 - Now with Unified Tools!
# "Testing all the things, because untested code is broken code!" - Testy McTesterson

set -e

BLUE='\033[0;34m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}=== Smart Tree v4.0.0 Comprehensive Test Suite ===${NC}"
echo -e "${CYAN}Testing the new unified tools that replace traditional file tools!${NC}"
echo ""

# Check if binary exists
if [ ! -f "./target/release/st" ]; then
    echo -e "${YELLOW}Building Smart Tree...${NC}"
    cargo build --release
fi

# Run all Rust unit tests
echo -e "\n${YELLOW}Running all Rust unit tests...${NC}"
cargo test

# Run specific unified tool tests
echo -e "\n${PURPLE}Running ST Unified Tool tests...${NC}"
cargo test --test test_st_unified

echo -e "\n${PURPLE}Running Tools ST Only tests...${NC}"
cargo test --test test_tools_st_only

echo -e "\n${PURPLE}Running ST Context Aware tests...${NC}"
cargo test --test test_st_context_aware

echo -e "\n${PURPLE}Running Unified Integration tests...${NC}"
cargo test --test test_unified_integration

# Run anchor.sh tests
echo -e "\n${PURPLE}Running Partnership Memory Helper tests...${NC}"
./tests/test_anchor.sh

# Run Rust MCP integration tests
echo -e "\n${YELLOW}Running Rust MCP integration tests...${NC}"
cargo test --test mcp_integration

# Run partnership tests
echo -e "\n${YELLOW}Running partnership collaboration tests...${NC}"
cargo test --test test_partnership

# Run smart edit tests
echo -e "\n${YELLOW}Running smart edit tests...${NC}"
cargo test --test test_smart_edit

# Run v3.3.5 specific feature tests
echo -e "\n${YELLOW}Running v3.3.5 feature tests...${NC}"
if [ -f "./tests/test_v3.3.5_features.sh" ]; then
    ./tests/test_v3.3.5_features.sh
else
    echo -e "${YELLOW}Skipping v3.3.5 tests (file not found)${NC}"
fi

# Run comprehensive MCP test suite
echo -e "\n${YELLOW}Running comprehensive MCP test suite...${NC}"
if [ -f "./tests/mcp_test_suite.sh" ]; then
    ./tests/mcp_test_suite.sh
else
    echo -e "${YELLOW}Skipping MCP test suite (file not found)${NC}"
fi

# Summary
echo -e "\n${GREEN}=== All Tests Complete! ===${NC}"
echo -e "${CYAN}✨ Smart Tree v4.0.0 Unified Tools are battle-tested and ready! ✨${NC}"
echo ""
echo -e "${GREEN}Test Coverage Summary:${NC}"
echo "  ✓ ST Unified Tool System"
echo "  ✓ Tools ST Only Implementation"
echo "  ✓ Context-Aware Intelligence"
echo "  ✓ Integration Workflows"
echo "  ✓ Partnership Memory System"
echo "  ✓ MCP Protocol Compliance"
echo "  ✓ Smart Edit Functionality"
echo ""
echo -e "${BLUE}🎸 Rock on with confidence - these tools are solid! 🎸${NC}"