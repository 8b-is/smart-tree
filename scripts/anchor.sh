#!/usr/bin/env bash
# 🌊 Smart Tree Memory Anchor Helper - Easy collaborative memory management!
# Usage: ./anchor.sh [command] [options]

set -euo pipefail

# ANSI color codes for that Trish-approved sparkle! ✨
readonly GREEN='\033[0;32m'
readonly BLUE='\033[0;34m'
readonly YELLOW='\033[1;33m'
readonly PURPLE='\033[0;35m'
readonly CYAN='\033[0;36m'
readonly RED='\033[0;31m'
readonly BOLD='\033[1m'
readonly RESET='\033[0m'

# Default values
PROJECT_PATH="."
ORIGIN="tandem:human:claude"

# Function to show usage
show_usage() {
    cat << EOF
${BOLD}${CYAN}🌊 Smart Tree Memory Anchor Helper${RESET}

${BOLD}USAGE:${RESET}
    anchor save <context> <keywords> [type]    Save a memory anchor
    anchor find <keywords>                     Find memory anchors
    anchor rapport [ai_tool]                   Check collaboration rapport
    anchor heatmap                             Show co-engagement heatmap
    anchor patterns [type]                     Find cross-domain patterns
    anchor insights <keywords>                 Get cross-session insights
    anchor invite <context>                    Invite a persona for help
    anchor list                                List recent anchors

${BOLD}TYPES:${RESET}
    pattern_insight    A pattern discovery
    solution          A problem solution
    breakthrough      A major breakthrough
    learning          Something learned
    joke              A shared joke/moment
    technical         Technical detail
    process           Process/workflow

${BOLD}EXAMPLES:${RESET}
    # Save a solution we discovered
    ${GREEN}anchor save "Fixed the bug by using Arc<Mutex<T>>" "arc,mutex,threading" solution${RESET}

    # Find all threading-related memories
    ${BLUE}anchor find "threading,concurrency"${RESET}

    # Check our partnership health
    ${PURPLE}anchor rapport claude${RESET}

    # Get insights for current work
    ${YELLOW}anchor insights "performance,optimization"${RESET}

${BOLD}ENVIRONMENT:${RESET}
    ST_PROJECT    Override project path (default: current directory)
    ST_ORIGIN     Override origin (default: tandem:human:claude)

${CYAN}✨ "Every anchor strengthens the waves of collaboration!" - Omni${RESET}
EOF
}

# Function to run MCP tool
run_mcp_tool() {
    local tool="$1"
    shift
    local args="$@"
    
    # Use st with MCP mode to execute tools
    echo "{\"tool\": \"$tool\", \"arguments\": $args}" | st --mcp 2>/dev/null | jq -r '.content // .error // "No response"'
}

# Save a memory anchor
save_anchor() {
    local context="$1"
    local keywords="$2"
    local anchor_type="${3:-learning}"
    
    echo -e "${CYAN}⚓ Anchoring memory...${RESET}"
    
    # Build keywords array
    IFS=',' read -ra kw_array <<< "$keywords"
    local kw_json=$(printf '"%s",' "${kw_array[@]}" | sed 's/,$//')
    
    local result=$(run_mcp_tool "anchor_collaborative_memory" "{
        \"context\": \"$context\",
        \"keywords\": [$kw_json],
        \"anchor_type\": \"$anchor_type\",
        \"origin\": \"$ORIGIN\",
        \"project_path\": \"$PROJECT_PATH\"
    }")
    
    echo -e "${GREEN}✓ Memory anchored successfully!${RESET}"
    echo -e "${BLUE}Context:${RESET} $context"
    echo -e "${BLUE}Keywords:${RESET} $keywords"
    echo -e "${BLUE}Type:${RESET} $anchor_type"
}

# Find memory anchors
find_anchors() {
    local keywords="$1"
    
    echo -e "${CYAN}🔍 Searching memory waves...${RESET}"
    
    # Build keywords array
    IFS=',' read -ra kw_array <<< "$keywords"
    local kw_json=$(printf '"%s",' "${kw_array[@]}" | sed 's/,$//')
    
    local result=$(run_mcp_tool "find_collaborative_memories" "{
        \"keywords\": [$kw_json],
        \"project_path\": \"$PROJECT_PATH\"
    }")
    
    echo "$result" | jq -r '
        if type == "array" then
            .[] | "${BOLD}[\(.anchor_type)]${RESET} \(.timestamp)\n${BLUE}\(.context)${RESET}\nKeywords: \(.keywords | join(", "))\nRelevance: \(.relevance)\n"
        else
            "No memories found for those keywords."
        end
    '
}

# Check rapport
check_rapport() {
    local ai_tool="${1:-claude}"
    
    echo -e "${PURPLE}💝 Checking partnership rapport with $ai_tool...${RESET}"
    
    local result=$(run_mcp_tool "get_collaboration_rapport" "{
        \"ai_tool\": \"$ai_tool\",
        \"project_path\": \"$PROJECT_PATH\"
    }")
    
    echo "$result"
}

# Show heatmap
show_heatmap() {
    echo -e "${YELLOW}🌡️ Co-engagement heatmap:${RESET}"
    
    local result=$(run_mcp_tool "get_co_engagement_heatmap" "{
        \"format\": \"visual\",
        \"project_path\": \"$PROJECT_PATH\"
    }")
    
    echo "$result"
}

# Find patterns
find_patterns() {
    local pattern_type="$1"
    
    echo -e "${CYAN}🔗 Finding cross-domain patterns...${RESET}"
    
    local args="{\"project_path\": \"$PROJECT_PATH\""
    if [[ -n "$pattern_type" ]]; then
        args="${args%\}}, \"pattern_type\": \"$pattern_type\"}"
    fi
    
    local result=$(run_mcp_tool "get_cross_domain_patterns" "$args")
    
    echo "$result"
}

# Get insights
get_insights() {
    local keywords="$1"
    
    echo -e "${YELLOW}💡 Gathering cross-session insights...${RESET}"
    
    # Build keywords array
    IFS=',' read -ra kw_array <<< "$keywords"
    local kw_json=$(printf '"%s",' "${kw_array[@]}" | sed 's/,$//')
    
    local result=$(run_mcp_tool "suggest_cross_session_insights" "{
        \"keywords\": [$kw_json],
        \"project_path\": \"$PROJECT_PATH\"
    }")
    
    echo "$result"
}

# Invite persona
invite_persona() {
    local context="$1"
    
    echo -e "${PURPLE}🎭 Inviting specialized persona...${RESET}"
    
    local result=$(run_mcp_tool "invite_persona" "{
        \"context\": \"$context\"
    }")
    
    echo "$result"
}

# List recent anchors (simple grep of history)
list_recent() {
    echo -e "${CYAN}📚 Recent memory anchors:${RESET}"
    
    # This is a simplified version - in reality would query the system
    echo "Use 'anchor find' with keywords to search memories"
}

# Main command handling
case "${1:-help}" in
    save)
        shift
        if [[ $# -lt 2 ]]; then
            echo -e "${RED}Error: save requires context and keywords${RESET}"
            echo "Usage: anchor save <context> <keywords> [type]"
            exit 1
        fi
        save_anchor "$@"
        ;;
    find)
        shift
        if [[ $# -lt 1 ]]; then
            echo -e "${RED}Error: find requires keywords${RESET}"
            echo "Usage: anchor find <keywords>"
            exit 1
        fi
        find_anchors "$1"
        ;;
    rapport)
        shift
        check_rapport "$@"
        ;;
    heatmap)
        show_heatmap
        ;;
    patterns)
        shift
        find_patterns "$@"
        ;;
    insights)
        shift
        if [[ $# -lt 1 ]]; then
            echo -e "${RED}Error: insights requires keywords${RESET}"
            echo "Usage: anchor insights <keywords>"
            exit 1
        fi
        get_insights "$1"
        ;;
    invite)
        shift
        if [[ $# -lt 1 ]]; then
            echo -e "${RED}Error: invite requires context${RESET}"
            echo "Usage: anchor invite <context>"
            exit 1
        fi
        invite_persona "$1"
        ;;
    list)
        list_recent
        ;;
    help|--help|-h)
        show_usage
        ;;
    *)
        echo -e "${RED}Unknown command: $1${RESET}"
        show_usage
        exit 1
        ;;
esac