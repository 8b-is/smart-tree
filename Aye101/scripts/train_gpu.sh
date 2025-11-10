#!/bin/bash
set -e

# --- Script to train a RustyFlow model on GPU ---
CONFIG_FILE="${1:-config.env}"
LOG_FILE="training_log.txt"

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

echo "--- Starting GPU Training Session ---"
echo "Config File: $CONFIG_FILE"
echo "Dataset: $DATASET"
echo "Device: $DEVICE"
echo "Model will be saved to: $FINAL_MODEL_PATH"
echo "Epochs: $NUM_EPOCHS, Batch Size: $BATCH_SIZE, Seq Len: $SEQ_LEN, LR: $LEARNING_RATE"
echo "---------------------------------"

# Run training, print to terminal, and capture output to a variable
TRAINING_OUTPUT=$(cargo run --release --bin cli -- \
    --use-gpu "true" \
    --dataset "$DATASET" \
    --save-path "$FINAL_MODEL_PATH" \
    --num-epochs "$NUM_EPOCHS" \
    --batch-size "$BATCH_SIZE" \
    --seq-len "$SEQ_LEN" \
    --learning-rate "$LEARNING_RATE" \
    --embed-dim "${EMBED_DIM:-128}" \
    --num-heads "${NUM_HEADS:-4}" \
    --num-layers "${NUM_LAYERS:-2}" | tee /dev/tty)

# Extract average epoch time and log it
AVG_EPOCH_TIME=$(echo "$TRAINING_OUTPUT" | grep '\[AVG_EPOCH_TIME\]' | awk '{print $2}')

if [ -n "$AVG_EPOCH_TIME" ]; then
    echo "Average epoch time: ${AVG_EPOCH_TIME}s"
    # Create log header if file doesn't exist
    if [ ! -f "$LOG_FILE" ]; then
        printf "timestamp\tdevice\tdataset\tavg_epoch_time_s\tmodel_path\n" > "$LOG_FILE"
    fi
    # Use a portable date format
    TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    printf "${TIMESTAMP}\t${DEVICE}\t${DATASET}\t${AVG_EPOCH_TIME}\t${FINAL_MODEL_PATH}\n" >> "$LOG_FILE"
    echo "Logged performance to $LOG_FILE"
else
    echo "Warning: Could not extract average epoch time from training output."
fi

echo "--- Training Finished ---"
