#!/bin/bash
# 🎸 The Cheet's Doc Bundler - "All your docs in one hot .mq!" 🔥
# Bundles all smart-tree documentation into a single Marqant file

set -e

# Colors for our rockstar output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DOCS_DIR="$PROJECT_ROOT/docs"
OUTPUT_DIR="$PROJECT_ROOT/bundles"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo -e "${PURPLE}🎸 The Cheet's Doc Bundler - Let's rock! 🎸${NC}"
echo -e "${CYAN}═══════════════════════════════════════════${NC}"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Check if mq binary exists
if [ ! -f "$PROJECT_ROOT/target/release/mq" ]; then
    echo -e "${YELLOW}Building mq binary...${NC}"
    cd "$PROJECT_ROOT"
    cargo build --release --bin mq
fi

# Output file
BUNDLE_MQ="$OUTPUT_DIR/smart_tree_docs_$TIMESTAMP.mq"

echo -e "${BLUE}📚 Collecting documentation files...${NC}"

# Count total docs and calculate original size
TOTAL_DOCS=0
ORIGINAL_SIZE=0
DOC_COUNT=0

# Create arrays to store file info
declare -a DOC_FILES
declare -a DOC_SIZES

# Collect all markdown files
echo -e "${CYAN}Scanning for markdown files...${NC}"
while IFS= read -r -d '' file; do
    DOC_FILES+=("$file")
    SIZE=$(stat -c%s "$file" 2>/dev/null || stat -f%z "$file" 2>/dev/null || wc -c < "$file")
    DOC_SIZES+=($SIZE)
    ((ORIGINAL_SIZE += SIZE))
    ((TOTAL_DOCS++))
    echo -ne "\rFound $TOTAL_DOCS files..."
done < <(find "$DOCS_DIR" -name "*.md" -type f -print0 | sort -z)
echo -e "\r${GREEN}Found $TOTAL_DOCS files!${NC}    "

ORIGINAL_SIZE_MB=$(awk "BEGIN {printf \"%.2f\", $ORIGINAL_SIZE / 1048576}")
echo -e "${YELLOW}Found $TOTAL_DOCS documentation files (${ORIGINAL_SIZE_MB} MB total)${NC}\n"

# Show files being processed
DOC_COUNT=0
for i in "${!DOC_FILES[@]}"; do
    file="${DOC_FILES[$i]}"
    size="${DOC_SIZES[$i]}"
    relative_path="${file#$DOCS_DIR/}"
    size_kb=$(awk "BEGIN {printf \"%.1f\", $size / 1024}")
    ((DOC_COUNT++))
    echo -e "${GREEN}  [$DOC_COUNT/$TOTAL_DOCS]${NC} ${relative_path} (${size_kb} KB)"
done

# Use mq aggregate command
echo -e "\n${BLUE}🎯 Running Marqant aggregation...${NC}"

# Check if aggregate command exists
if ! "$PROJECT_ROOT/target/release/mq" aggregate --help &>/dev/null; then
    echo -e "${YELLOW}Aggregate command not found, using compress instead...${NC}"
    
    # Create a manual aggregate
    TEMP_AGGREGATE="$OUTPUT_DIR/.temp_aggregate_$TIMESTAMP.md"
    
    # Header
    cat > "$TEMP_AGGREGATE" << EOF
# Smart Tree Documentation Bundle

> Generated on: $(date '+%Y-%m-%d %H:%M:%S')
> Total documents: $TOTAL_DOCS
> Original size: ${ORIGINAL_SIZE_MB} MB

---

EOF
    
    # Add each file
    for i in "${!DOC_FILES[@]}"; do
        file="${DOC_FILES[$i]}"
        relative_path="${file#$DOCS_DIR/}"
        
        echo -e "\n\n════════════════════════════════════════════════════════════════════════" >> "$TEMP_AGGREGATE"
        echo "# 📄 $relative_path" >> "$TEMP_AGGREGATE"
        echo "════════════════════════════════════════════════════════════════════════" >> "$TEMP_AGGREGATE"
        echo "" >> "$TEMP_AGGREGATE"
        cat "$file" >> "$TEMP_AGGREGATE"
    done
    
    # Footer
    cat >> "$TEMP_AGGREGATE" << 'EOF'

---

## Bundle Information

This documentation bundle was created using Smart Tree's Marqant compression format.

### The Cheet Says...

> "That's all the docs, compressed and ready to rock! 🎸"

EOF
    
    # Compress the aggregate
    "$PROJECT_ROOT/target/release/mq" compress "$TEMP_AGGREGATE" -o "$BUNDLE_MQ"
    rm -f "$TEMP_AGGREGATE"
else
    # Use the aggregate command directly
    "$PROJECT_ROOT/target/release/mq" aggregate -o "$BUNDLE_MQ" "${DOC_FILES[@]}"
fi

# Check if the bundle was created
if [ ! -f "$BUNDLE_MQ" ]; then
    echo -e "${RED}❌ Failed to create bundle!${NC}"
    exit 1
fi

# Get compressed size
MQ_SIZE=$(stat -c%s "$BUNDLE_MQ" 2>/dev/null || stat -f%z "$BUNDLE_MQ" 2>/dev/null || wc -c < "$BUNDLE_MQ")
MQ_SIZE_KB=$(awk "BEGIN {printf \"%.2f\", $MQ_SIZE / 1024}")
MQ_RATIO=$(awk "BEGIN {printf \"%.2f\", 100 - ($MQ_SIZE * 100 / $ORIGINAL_SIZE)}")

# Create stats report
STATS_FILE="$OUTPUT_DIR/compression_stats_$TIMESTAMP.txt"
cat > "$STATS_FILE" << EOF
Smart Tree Documentation Bundle - Compression Statistics
═══════════════════════════════════════════════════════

Bundle created: $(date '+%Y-%m-%d %H:%M:%S')
Total documents: $TOTAL_DOCS

Original size: $ORIGINAL_SIZE bytes ($ORIGINAL_SIZE_MB MB)
Compressed size: $MQ_SIZE bytes ($MQ_SIZE_KB KB)
Compression ratio: ${MQ_RATIO}%

Bundle file: $BUNDLE_MQ

To decompress:
  mq decompress "$BUNDLE_MQ" -o docs_restored.md

To inspect:
  mq inspect "$BUNDLE_MQ"

EOF

# Display results
echo -e "\n${GREEN}✅ Documentation bundle created successfully!${NC}"
echo -e "${CYAN}═══════════════════════════════════════════${NC}"
echo -e "${YELLOW}📊 Compression Results:${NC}"
echo -e "   Original: ${ORIGINAL_SIZE_MB} MB"
echo -e "   Compressed: ${MQ_SIZE_KB} KB (${GREEN}${MQ_RATIO}% reduction${NC})"
echo -e "\n${PURPLE}📦 Output files:${NC}"
echo -e "   ${BUNDLE_MQ}"
echo -e "   ${STATS_FILE}"

# Test with inspect command
echo -e "\n${BLUE}🔍 Inspecting bundle...${NC}"
"$PROJECT_ROOT/target/release/mq" inspect "$BUNDLE_MQ" | head -20

echo -e "\n${GREEN}🎉 Done! Your docs are bundled and ready to ship! 🚀${NC}"
echo -e "${CYAN}🎸 The Cheet says: \"From ${ORIGINAL_SIZE_MB} MB to ${MQ_SIZE_KB} KB - that's compression that rocks! 🤘\"${NC}"