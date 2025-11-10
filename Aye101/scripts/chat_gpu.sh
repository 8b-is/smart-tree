#!/bin/bash
set -e

# --- Script to chat with a RustyFlow model on GPU ---
CONFIG_FILE="${1:-config.env}"

if [ ! -f "$CONFIG_FILE" ]; then
    echo "Error: Config file '$CONFIG_FILE' not found."
    echo "A default can be created by running './run.sh setup'."
    exit 1
fi

# Source the configuration file
source "$CONFIG_FILE"

DEVICE="gpu"
# Inserts device name before the extension, e.g., model.bin -> model-gpu.bin
FINAL_MODEL_PATH="${MODEL_PATH%.bin}-${DEVICE}.bin"

echo "--- Starting GPU Chat Session ---"
echo "Config File: $CONFIG_FILE"
echo "Loading model from: $FINAL_MODEL_PATH"
echo "Temperature: $TEMPERATURE, Top-p: $TOP_P"
echo "-----------------------------"

if [ ! -f "$FINAL_MODEL_PATH" ]; then
    echo "Error: Model file not found at '$FINAL_MODEL_PATH'."
    echo "Please run './train_gpu.sh' first to train and save a model."
    echo "Available models:"
    cargo run --release --bin cli -- --list-models
    exit 1
fi

# The seq_len for chat is loaded from the model file, but we pass it as a default.
cargo run --release --bin cli -- \
    --chat \
    --use-gpu "true" \
    --load-path "$FINAL_MODEL_PATH" \
    --seq-len "$SEQ_LEN" \
    --temperature "$TEMPERATURE" \
    --top-p "$TOP_P"

echo "--- Chat Session Finished ---"
