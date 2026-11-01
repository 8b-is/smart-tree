#!/bin/bash
# Simple doc bundler for testing

set -e

cd "$(dirname "$0")/.."

echo "Creating documentation bundle..."

# Create output directory
mkdir -p bundles

# Create a simple aggregate file
OUTPUT="bundles/docs_bundle_$(date +%Y%m%d_%H%M%S).md"

echo "# Smart Tree Documentation Bundle" > "$OUTPUT"
echo "" >> "$OUTPUT"
echo "Generated on: $(date)" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# Count files
TOTAL=$(find docs -name "*.md" -type f | wc -l)
echo "Total documents: $TOTAL" >> "$OUTPUT"
echo "" >> "$OUTPUT"
echo "---" >> "$OUTPUT"

# Add each file
COUNT=0
for file in $(find docs -name "*.md" -type f | sort); do
    ((COUNT++))
    echo "Processing [$COUNT/$TOTAL]: $file"
    
    echo "" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    echo "════════════════════════════════════════════════════════════════════════" >> "$OUTPUT"
    echo "# 📄 $file" >> "$OUTPUT"
    echo "════════════════════════════════════════════════════════════════════════" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    
    cat "$file" >> "$OUTPUT"
done

# Get sizes
ORIG_SIZE=$(stat -c%s "$OUTPUT" 2>/dev/null || wc -c < "$OUTPUT")
ORIG_MB=$(echo "scale=2; $ORIG_SIZE / 1048576" | bc -l)

echo ""
echo "Aggregate created: $OUTPUT ($ORIG_MB MB)"
echo ""
echo "Compressing with mq..."

# Compress it
MQ_OUTPUT="${OUTPUT%.md}.mq"
./target/release/mq compress "$OUTPUT" -o "$MQ_OUTPUT"

# Get compressed size
MQ_SIZE=$(stat -c%s "$MQ_OUTPUT" 2>/dev/null || wc -c < "$MQ_OUTPUT")
MQ_KB=$(echo "scale=2; $MQ_SIZE / 1024" | bc -l)
RATIO=$(echo "scale=2; 100 - ($MQ_SIZE * 100 / $ORIG_SIZE)" | bc -l)

echo ""
echo "✅ Bundle created successfully!"
echo "   Original: $ORIG_MB MB"
echo "   Compressed: $MQ_KB KB ($RATIO% reduction)"
echo "   Output: $MQ_OUTPUT"