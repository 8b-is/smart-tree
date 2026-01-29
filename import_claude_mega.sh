#!/bin/bash
# Import Claude mega chat into Smart Tree memory system
# "Every conversation becomes a wave!" - Hue & Claude

CONVERSATIONS_FILE="/aidata/wrAIth/MEMZ/m8c/data/seeds/Claude/conversations.json"
ST="./target/release/st"

echo "🧠 Importing Claude Mega Chat..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Count conversations
CONV_COUNT=$(jq 'length' "$CONVERSATIONS_FILE")
echo "Found $CONV_COUNT conversations"

# Import first 10 conversations as memories
for i in $(seq 0 1000); do
    # Extract conversation details
    NAME=$(jq -r ".[$i].name // \"Conversation $i\"" "$CONVERSATIONS_FILE" | head -c 100)
    DATE=$(jq -r ".[$i].created_at // \"unknown\"" "$CONVERSATIONS_FILE" | cut -d'T' -f1)

    # Get first message as context (if exists)
    FIRST_MSG=$(jq -r ".[$i].chat_messages[0].text // \"No message\"" "$CONVERSATIONS_FILE" 2>/dev/null | head -c 200)

    # Create keywords from name
    KEYWORDS=$(echo "$NAME" | tr ' ' '\n' | head -3 | tr '\n' ',' | sed 's/,$//')

    if [ -n "$NAME" ] && [ "$NAME" != "null" ]; then
        echo ""
        echo "[$((i+1))] Importing: $NAME"
        echo "    Date: $DATE"
        echo "    Preview: ${FIRST_MSG:0:50}..."

        # Anchor as memory
        $ST --memory-anchor "claude-conversation" "$KEYWORDS,claude,chat" "$NAME - $DATE: $FIRST_MSG" 2>/dev/null

        echo "    ✓ Imported as memory"
    fi
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✨ Import complete!"
echo ""

# Show stats
$ST --memory-stats

echo ""
echo "💡 Search memories with:"
echo "   st --memory-find claude"
echo "   st --memory-find conversation"
