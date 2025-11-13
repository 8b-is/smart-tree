#!/bin/bash
# This script downloads the TinyShakespeare dataset.

set -e

# The directory where the data will be stored.
DATA_DIR="data/tinyshakespeare"
# The URL from which to download the dataset.
URL="https://raw.githubusercontent.com/karpathy/char-rnn/master/data/tinyshakespeare/input.txt"
# The output file path.
OUTPUT_FILE="$DATA_DIR/input.txt"

# Create the directory if it doesn't exist.
mkdir -p "$DATA_DIR"

echo "Downloading TinyShakespeare dataset to $OUTPUT_FILE..."

# Use curl to download the file.
# -L: follow redirects
# -o: specify output file
curl -L "$URL" -o "$OUTPUT_FILE"

echo "Download complete."
echo "You can now run the 'language_model' example."

