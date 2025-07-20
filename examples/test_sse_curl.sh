#!/bin/bash
# Test SSE functionality with curl

echo "🧪 Smart Tree SSE Curl Tests"
echo "============================"
echo

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Base URL
BASE_URL="http://localhost:8420"

echo -e "${BLUE}Test 1: Basic SSE Connection${NC}"
echo "Command: curl -N -H 'Accept: text/event-stream' $BASE_URL/sse"
echo -e "${YELLOW}This will stream events indefinitely. Press Ctrl+C to stop.${NC}"
echo "----------------------------------------"
curl -N -H "Accept: text/event-stream" "$BASE_URL/sse"
echo
echo

echo -e "${BLUE}Test 2: SSE with Timeout (10 seconds)${NC}"
echo "Command: timeout 10 curl -N -H 'Accept: text/event-stream' $BASE_URL/sse"
echo "----------------------------------------"
timeout 10 curl -N -H "Accept: text/event-stream" "$BASE_URL/sse"
echo
echo

echo -e "${BLUE}Test 3: SSE with Pretty Printing${NC}"
echo "Command: curl -N -H 'Accept: text/event-stream' $BASE_URL/sse 2>/dev/null | while IFS= read -r line; do"
echo "  [[ \$line =~ ^data: ]] && echo \$line | cut -d' ' -f2- | jq ."
echo "done"
echo "----------------------------------------"
timeout 15 curl -N -H "Accept: text/event-stream" "$BASE_URL/sse" 2>/dev/null | while IFS= read -r line; do
    if [[ $line =~ ^data: ]]; then
        echo "$line" | cut -d' ' -f2- | jq . 2>/dev/null || echo "$line"
    elif [[ $line =~ ^event: ]]; then
        echo -e "${GREEN}Event Type: ${line#event: }${NC}"
    elif [[ $line =~ ^id: ]]; then
        echo -e "${YELLOW}Event ID: ${line#id: }${NC}"
    fi
done
echo
echo

echo -e "${BLUE}Test 4: Filter Specific Event Types${NC}"
echo "Command: curl -N -H 'Accept: text/event-stream' $BASE_URL/sse | grep -E '^event: (created|modified|deleted)' -A2"
echo "----------------------------------------"
timeout 20 curl -N -H "Accept: text/event-stream" "$BASE_URL/sse" 2>/dev/null | grep -E '^event: (created|modified|deleted)' -A2
echo
echo

echo -e "${BLUE}Test 5: Save Events to File${NC}"
echo "Command: curl -N -H 'Accept: text/event-stream' $BASE_URL/sse > sse_events.log"
echo "----------------------------------------"
timeout 10 curl -N -H "Accept: text/event-stream" "$BASE_URL/sse" > sse_events.log 2>/dev/null
echo -e "${GREEN}Events saved to sse_events.log${NC}"
echo "First few events:"
head -20 sse_events.log
echo
echo

echo -e "${BLUE}Test 6: Process Events with AWK${NC}"
echo "Command: curl -N -H 'Accept: text/event-stream' $BASE_URL/sse | awk '/^event:/ {event=\$2} /^data:/ {print \"Event:\", event, \"Data:\", substr(\$0, 7)}'"
echo "----------------------------------------"
timeout 10 curl -N -H "Accept: text/event-stream" "$BASE_URL/sse" 2>/dev/null | awk '/^event:/ {event=$2} /^data:/ {print "Event:", event, "Data:", substr($0, 7)}'
echo
echo

echo -e "${GREEN}✅ SSE Tests Complete!${NC}"
echo
echo "Additional curl options for SSE:"
echo "  -N, --no-buffer     : Disable buffering for real-time streaming"
echo "  -H 'Accept: text/event-stream' : Set proper SSE header"
echo "  --compressed        : Enable compression"
echo "  -m, --max-time <seconds> : Maximum time for the operation"
echo "  -v, --verbose       : Show detailed connection info"
echo
echo "Example with all options:"
echo "  curl -N -H 'Accept: text/event-stream' -H 'Cache-Control: no-cache' --compressed -v $BASE_URL/sse"