#!/bin/bash
# Master test runner for Smart Tree v3.3.5
# Runs all test suites before release

set -e

BLUE='\033[0;34m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m'

echo -e "${BLUE}=== Smart Tree v4.0.0 Release Test Suite ===${NC}"
echo ""

# Check if binary exists
if [ ! -f "./target/release/st" ]; then
    echo -e "${YELLOW}Building Smart Tree...${NC}"
    cargo build --release
fi

# Run Rust tests
echo -e "\n${YELLOW}Running Rust unit tests...${NC}"
cargo test

# Run Rust MCP integration tests
echo -e "\n${YELLOW}Running Rust MCP integration tests...${NC}"
cargo test --test mcp_integration

# Run v3.3.5 specific feature tests
echo -e "\n${YELLOW}Running v3.3.5 feature tests...${NC}"
./tests/test_v3.3.5_features.sh

# Run comprehensive MCP test suite
echo -e "\n${YELLOW}Running comprehensive MCP test suite...${NC}"
./tests/mcp_test_suite.sh

echo -e "\n${GREEN}=== All Tests Complete! ===${NC}"
echo "Smart Tree v4.0.0 is ready for release! 🎉"