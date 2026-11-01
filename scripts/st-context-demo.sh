#!/usr/bin/env bash
# 🌟 Smart Tree Context-Aware Demo
# "The assistant who remembers what you're doing!" - The Cheet 🎸

set -euo pipefail

# Colors for beautiful output
readonly GREEN='\033[0;32m'
readonly BLUE='\033[0;34m'
readonly YELLOW='\033[1;33m'
readonly PURPLE='\033[0;35m'
readonly CYAN='\033[0;36m'
readonly RED='\033[0;31m'
readonly BOLD='\033[1m'
readonly RESET='\033[0m'

# Demo header
echo -e "${BOLD}${CYAN}🌟 Smart Tree Context-Aware System Demo 🌟${RESET}"
echo -e "${PURPLE}\"Like a roadie who knows what guitar you need!\"${RESET}\n"

# Function to simulate ST operations
simulate_st() {
    local operation="$1"
    echo -e "${YELLOW}▶ Running: st $operation${RESET}"
    ./target/release/st $operation 2>/dev/null | head -10
    echo ""
    sleep 1
}

# Function to show context
show_context() {
    echo -e "${CYAN}📊 Current Context Analysis:${RESET}"
    echo "Based on your recent operations, ST detects you are:"
}

# Function to show suggestions
show_suggestions() {
    echo -e "${GREEN}💡 Smart Suggestions:${RESET}"
}

echo -e "${BOLD}Scenario 1: Exploring a New Codebase${RESET}"
echo -e "${BLUE}You just cloned a project and want to understand it...${RESET}\n"

simulate_st "--mode summary-ai"
show_context
echo "  🔍 EXPLORING - You're getting familiar with the codebase"
show_suggestions
echo "  • st --mode semantic        # See how files are grouped"
echo "  • st --mode stats          # Get detailed statistics"
echo "  • st --search main --type rs  # Find entry points"
echo ""

echo -e "${BOLD}Scenario 2: Hunting for a Bug${RESET}"
echo -e "${BLUE}You're searching for the source of an error...${RESET}\n"

simulate_st "--search \"TODO\" --type rs"
simulate_st "--search \"FIXME\" --type rs"
simulate_st "--search \"error\" --type rs"

show_context
echo "  🔍 HUNTING - You're tracking down something specific"
show_suggestions
echo "  • st --search \"error\" --mode ai    # Get AI-optimized results"
echo "  • st --newer-than 1 --sort newest  # Check recent changes"
echo "  • st --mode relations             # See code dependencies"
echo ""

echo -e "${BOLD}Scenario 3: Active Development${RESET}"
echo -e "${BLUE}You're writing new features...${RESET}\n"

echo -e "${YELLOW}▶ Simulating edits to src/main.rs${RESET}"
echo -e "${YELLOW}▶ Running tests...${RESET}"
simulate_st "--search test --type rs"

show_context
echo "  💻 CODING - You're actively developing in Rust"
show_suggestions
echo "  • st --mode relations --focus src/main.rs  # See impact"
echo "  • st --search test --type rs              # Find related tests"
echo "  • st --mode quantum-semantic src/         # Deep analysis"
echo ""

echo -e "${BOLD}Scenario 4: Optimizing Performance${RESET}"
echo -e "${BLUE}You're looking for performance bottlenecks...${RESET}\n"

simulate_st "--sort largest --top 10"
simulate_st "--mode waste"

show_context
echo "  ⚡ OPTIMIZING - You're improving performance"
show_suggestions
echo "  • st --mode waste              # Find optimization targets"
echo "  • st --sort largest --top 20   # Find large files"
echo "  • st --search \"TODO.*perf\"     # Find performance TODOs"
echo ""

echo -e "${BOLD}Context Benefits:${RESET}"
echo -e "${GREEN}✅ Automatic depth selection${RESET} - Exploring? Get depth 3. Debugging? Get depth 0 (auto)."
echo -e "${GREEN}✅ Smart mode selection${RESET} - Coding? Get AI mode. Exploring? Get semantic mode."
echo -e "${GREEN}✅ Relevant suggestions${RESET} - Get commands that match what you're doing."
echo -e "${GREEN}✅ Learning patterns${RESET} - ST remembers your common searches and hot directories."
echo -e "${GREEN}✅ Workflow optimization${RESET} - Less typing, more doing!"

echo -e "\n${BOLD}${CYAN}Smart Tree: It's not just a tree, it's your coding companion! 🌳✨${RESET}"
echo -e "${PURPLE}\"The tool that hands you exactly what you need!\" - Omni${RESET}"