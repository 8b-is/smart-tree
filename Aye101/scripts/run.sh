#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# --- Helper Functions ---
# Function to display help message
usage() {
    echo "Usage: $0 <command> [options]"
    echo
    echo "This is the main control script for the RustyFlow project."
    echo
echo "Core Commands:"
    echo "  train_cpu [config_file]     Run training on CPU. See 'train_cpu.sh' for details."
    echo "  train_gpu [config_file]     Run training on GPU. See 'train_gpu.sh' for details."
    echo "  chat_cpu [config_file]      Start a chat session on CPU. See 'chat_cpu.sh' for details."
    echo "  chat_gpu [config_file]      Start a chat session on GPU. See 'chat_gpu.sh' for details."
    echo "  setup                     Create a default 'config.env' file if it doesn't exist."
    echo
    echo "Development & Deployment Commands:"
    echo "  build                     Compile the project in release mode."
    echo "  check                     Check the project for errors without building."
    echo "  test                      Run tests."
    echo "  get-data [dataset]        Download a dataset (e.g., 'tinyshakespeare')."
    echo "  help                      Show this help message."
    echo
    exit 1
}

# --- Main Script Logic ---
# Check if any command is provided
if [ $# -eq 0 ]; then
    usage
fi

case "$1" in
    setup)
        echo "Setting up project..."
        if [ -f "config.env" ]; then
            echo "Info: 'config.env' already exists. No action taken."
        else
            echo "Creating default 'config.env'..."
            # Using cat with a heredoc to create the file
            cat > config.env <<'EOF'
# --- Configuration for RustyFlow Training and Chatting ---

# This file is sourced by train.sh and chat.sh.
# You can create multiple config files and pass the path as an argument.
# e.g., ./train.sh my_wikitext_config.env

# --- Common Settings ---

# Dataset to use: tinyshakespeare, wikitext-2, short, or a path to a text file.
DATASET="tinyshakespeare"

# Path to save the trained model to, or load from for chatting.
MODEL_PATH="models/model.bin"

# Sequence length for training and context for chat.
SEQ_LEN=64

# --- Training Hyperparameters ---
NUM_EPOCHS=10
BATCH_SIZE=16
LEARNING_RATE=0.01

# --- Chat Hyperparameters ---
# (SEQ_LEN from above is used for chat context)
EOF
            echo "Default 'config.env' created. Please review it before training."
        fi
        ;;
    build)
        echo "Building project in release mode..."
        cargo build --release
        echo "Build complete. Executable in target/release/"
        ;;
    train_cpu)
        echo "Delegating to train_cpu.sh..."
        shift # remove 'train_cpu' from args
        ./train_cpu.sh "$@"
        ;;
    train_gpu)
        echo "Delegating to train_gpu.sh..."
        shift # remove 'train_gpu' from args
        ./train_gpu.sh "$@"
        ;;
    chat_cpu)
        echo "Delegating to chat_cpu.sh for CPU chat..."
        shift # remove 'chat_cpu' from args
        ./chat_cpu.sh "$@"
        ;;
    chat_gpu)
        echo "Delegating to chat_gpu.sh for GPU chat..."
        shift # remove 'chat_gpu' from args
        ./chat_gpu.sh "$@"
        ;;
    check)
        echo "Checking project..."
        cargo check
        ;;
    test)
        echo "Running tests..."
        cargo test
        ;;
    get-data)
        DATASET=${2:-tinyshakespeare}
        case "$DATASET" in
            tinyshakespeare)
                echo "Downloading TinyShakespeare dataset..."
                sh scripts/download_tinyshakespeare.sh
                ;;
            wikitext-2)
                echo "Info: The WikiText-2 dataset is included with the project in 'data/wikitext-2'."
                echo "No download is necessary."
                ;;
            *)
                echo "Error: Unknown dataset '$DATASET' for get-data command."
                echo "Supported dataset for download is 'tinyshakespeare'."
                exit 1
                ;;
        esac
        ;;
    help | *)
        usage
        ;;
esac

echo "Script finished successfully."
