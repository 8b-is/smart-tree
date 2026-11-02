#!/bin/bash
# 🎸 The Cheet's Doc Bundler - Direct Approach 🔥

set -e

# Colors
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${PURPLE}🎸 The Cheet's Doc Bundler - Let's rock! 🎸${NC}"
echo -e "${CYAN}═══════════════════════════════════════════${NC}"

# Setup paths
cd "$(dirname "$0")/.."
PROJECT_ROOT="$(pwd)"
DOCS_DIR="$PROJECT_ROOT/docs"
OUTPUT_DIR="$PROJECT_ROOT/bundles"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Output files
AGGREGATE_FILE="$OUTPUT_DIR/docs_aggregate_$TIMESTAMP.md"
BUNDLE_MQ="$OUTPUT_DIR/smart_tree_docs_$TIMESTAMP.mq"

# Create aggregate file header
cat > "$AGGREGATE_FILE" << EOF
# Smart Tree Documentation Bundle

> Generated on: $(date '+%Y-%m-%d %H:%M:%S')
> This bundle contains all Smart Tree documentation in a single Marqant-compressed file.

---

EOF

# Count files
TOTAL_DOCS=$(find "$DOCS_DIR" -name "*.md" -type f | wc -l)
echo -e "${YELLOW}Found $TOTAL_DOCS documentation files${NC}"

# Process each file
COUNT=0
TOTAL_SIZE=0

find "$DOCS_DIR" -name "*.md" -type f | sort | while read -r file; do
    ((COUNT++))
    
    # Get file info
    relative_path="${file#$DOCS_DIR/}"
    file_size=$(stat -c%s "$file" 2>/dev/null || wc -c < "$file")
    size_kb=$(echo "scale=1; $file_size / 1024" | bc -l 2>/dev/null || echo "?")
    
    echo -e "${GREEN}  [$COUNT/$TOTAL_DOCS]${NC} Processing: $relative_path (${size_kb} KB)"
    
    # Add file to aggregate
    {
        echo ""
        echo ""
        echo "════════════════════════════════════════════════════════════════════════"
        echo "# 📄 $relative_path"
        echo "════════════════════════════════════════════════════════════════════════"
        echo ""
        cat "$file"
    } >> "$AGGREGATE_FILE"
done

# Add footer
cat >> "$AGGREGATE_FILE" << 'EOF'

---

## Bundle Information

This documentation bundle was created using Smart Tree's Marqant v2.0 compression format.

### The Cheet Says...

> "That's all the docs, compressed and ready to rock! 🎸 From heavyweight docs to 
> lightweight .mq files - it's like going from vinyl to streaming!"

EOF

# Get original size
ORIG_SIZE=$(stat -c%s "$AGGREGATE_FILE" 2>/dev/null || wc -c < "$AGGREGATE_FILE")
ORIG_MB=$(echo "scale=2; $ORIG_SIZE / 1048576" | bc -l 2>/dev/null || echo "$((ORIG_SIZE / 1048576))")

echo -e "\n${YELLOW}📊 Original aggregate size: ${ORIG_MB} MB${NC}"

# Compress with mq
echo -e "${BLUE}🗜️  Compressing to .mq format...${NC}"

if [ -f "$PROJECT_ROOT/target/release/mq" ]; then
    "$PROJECT_ROOT/target/release/mq" compress "$AGGREGATE_FILE" -o "$BUNDLE_MQ"
else
    echo -e "${YELLOW}Warning: mq binary not found, keeping uncompressed${NC}"
    cp "$AGGREGATE_FILE" "$BUNDLE_MQ"
fi

# Get compressed size
if [ -f "$BUNDLE_MQ" ]; then
    MQ_SIZE=$(stat -c%s "$BUNDLE_MQ" 2>/dev/null || wc -c < "$BUNDLE_MQ")
    MQ_KB=$(echo "scale=2; $MQ_SIZE / 1024" | bc -l 2>/dev/null || echo "$((MQ_SIZE / 1024))")
    RATIO=$(echo "scale=1; 100 - ($MQ_SIZE * 100 / $ORIG_SIZE)" | bc -l 2>/dev/null || echo "?")
    
    echo -e "\n${GREEN}✅ Documentation bundle created successfully!${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════${NC}"
    echo -e "${YELLOW}📊 Compression Results:${NC}"
    echo -e "   Original: ${ORIG_MB} MB"
    echo -e "   Compressed: ${MQ_KB} KB (${GREEN}${RATIO}% reduction${NC})"
    echo -e "\n${PURPLE}📦 Output files:${NC}"
    echo -e "   Aggregate: ${AGGREGATE_FILE}"
    echo -e "   Bundle: ${BUNDLE_MQ}"
    
    # Test with inspect
    if [ -f "$PROJECT_ROOT/target/release/mq" ]; then
        echo -e "\n${BLUE}🔍 Bundle preview:${NC}"
        "$PROJECT_ROOT/target/release/mq" inspect "$BUNDLE_MQ" 2>/dev/null | head -15 || echo "Inspect not available"
    fi
    
    echo -e "\n${CYAN}🎸 The Cheet says: \"That's compression that rocks! 🤘\"${NC}"
else
    echo -e "${RED}❌ Failed to create bundle${NC}"
fi

# Cleanup
rm -f "$AGGREGATE_FILE"