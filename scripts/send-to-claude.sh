#!/bin/bash
# Send Smart Tree quantum output to Claude API
# This demonstrates the power of quantum compression for AI interactions

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if ANTHROPIC_API_KEY is set
if [ -z "$ANTHROPIC_API_KEY" ]; then
    echo -e "${RED}Error: ANTHROPIC_API_KEY environment variable not set${NC}"
    echo "Please set it with: export ANTHROPIC_API_KEY=your_key_here"
    exit 1
fi

# Get directory to analyze (default to current)
DIR="${1:-.}"

echo -e "${BLUE}🌳 Smart Tree Claude Integration${NC}"
echo -e "${YELLOW}Analyzing: $DIR${NC}"

# Generate quantum format
echo -e "\n${GREEN}Generating quantum format...${NC}"
QUANTUM_OUTPUT=$(st "$DIR" -m claude)

# Extract just the data size for display
DATA_SIZE=$(echo "$QUANTUM_OUTPUT" | grep '"data_size"' | grep -o '[0-9]*')
COMPRESSION=$(echo "$QUANTUM_OUTPUT" | grep '"compression_ratio"' | head -1 | cut -d'"' -f4)

echo -e "Compressed to: ${GREEN}$DATA_SIZE bytes${NC} (${YELLOW}$COMPRESSION${NC} of estimated original)"

# Create API request
echo -e "\n${BLUE}Sending to Claude API...${NC}"

# Build the request
REQUEST=$(cat <<EOF
{
  "model": "claude-3-sonnet-20240229",
  "messages": [{
    "role": "user",
    "content": "I'm sending you a directory structure in Smart Tree quantum format. This ultra-compressed format uses bitfield headers, token substitution, and ASCII control codes. Please analyze the structure and tell me: 1) What type of project is this? 2) What are the main components? 3) Any suggestions for organization?\n\n$QUANTUM_OUTPUT"
  }],
  "max_tokens": 2000,
  "temperature": 0.7
}
EOF
)

# Send to API
RESPONSE=$(curl -s https://api.anthropic.com/v1/messages \
    -H "x-api-key: $ANTHROPIC_API_KEY" \
    -H "anthropic-version: 2023-06-01" \
    -H "content-type: application/json" \
    -d "$REQUEST")

# Check for errors
if echo "$RESPONSE" | grep -q '"error"'; then
    echo -e "${RED}API Error:${NC}"
    echo "$RESPONSE" | jq -r '.error.message // .error' 2>/dev/null || echo "$RESPONSE"
    exit 1
fi

# Extract and display response
echo -e "\n${GREEN}Claude's Analysis:${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo "$RESPONSE" | jq -r '.content[0].text' 2>/dev/null || echo "Could not parse response"

# Show statistics
echo -e "\n${BLUE}Statistics:${NC}"
echo -e "- Original data (estimated): ~$(echo "$QUANTUM_OUTPUT" | jq -r '.context.benefits.original_size_estimate') bytes"
echo -e "- Compressed size: $DATA_SIZE bytes"
echo -e "- Compression ratio: $COMPRESSION"
echo -e "- API tokens used: ~$(echo "$RESPONSE" | jq -r '.usage.output_tokens // "unknown"')"

echo -e "\n${GREEN}✨ Quantum compression makes AI interactions more efficient!${NC}"