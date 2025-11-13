# RustyFlow: A Pure Rust Machine Learning Library

RustyFlow is an experimental machine learning library written entirely in Rust, designed for building and training neural network models, starting with a Transformer-based language model. The project aims to provide a clear, efficient, and robust foundation for deep learning in Rust, leveraging `ndarray` for numerical operations and `wgpu` for cross-platform GPU acceleration.

## Features

-   **Transformer Architecture**: Implements key components of the Transformer model, including Multi-Head Attention, Feed-Forward Networks, Layer Normalization, and Positional Embeddings.
-   **Automatic Differentiation (Autograd)**: A custom autograd engine enables automatic gradient computation for training neural networks.
-   **Optimizers**: Stochastic Gradient Descent (SGD) is currently implemented.
-   **Data Handling**: Utilities for vocabulary building, tokenization, and batching.
-   **Cross-Platform GPU Acceleration**: Leverages `wgpu` (WebGPU) for GPU-accelerated matrix multiplications, supporting Metal (macOS/iOS), Vulkan (Linux/Android), and DirectX 12 (Windows).
-   **Model Serialization**: Save and load trained models for persistence and inference.
-   **Interactive Chat**: Engage with trained language models in a conversational interface.
-   **Profiling**: Basic profiling tools to analyze performance bottlenecks (CPU vs. GPU).

## Getting Started

### Prerequisites

-   **Rust and Cargo**: If you don't have Rust installed, you can get it from [rustup.rs](https://rustup.rs/).
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
-   **`git`**: For cloning the repository.

### 1. Clone the Repository

```bash
git clone https://github.com/cekim7/RustyFlow.git
cd RustyFlow
```

### 2. Build the Project

Compile the project in release mode. This will create the `cli` executable in `target/release/`.

```bash
./run.sh build
```

### 3. Setup Configuration and Download Data

A `config.env` file is used to manage hyperparameters and settings. You can create a default one and download necessary datasets:

```bash
# Create a default config.env if it doesn't exist
./run.sh setup

# Download the TinyShakespeare dataset (required for tinyshakespeare training)
./run.sh get-data tinyshakespeare
```

The `wikitext-2` dataset is included directly in the repository under `data/wikitext-2`.
For more details on data handling, see [`docs/data_handling.md`](docs/data_handling.md).

## Usage

RustyFlow provides separate scripts for training and chatting on CPU or GPU, all configurable via `config.env`.

### Configuration (`config.env`)

The `config.env` file (created by `./run.sh setup`) allows you to customize:
-   `DATASET`: `tinyshakespeare`, `wikitext-2`, `short`, or a path to a custom text file.
-   `SEQ_LEN`: Sequence length for training and context for chat.
-   `NUM_EPOCHS`, `BATCH_SIZE`, `LEARNING_RATE`: Training hyperparameters.
-   `EMBED_DIM`, `NUM_HEADS`, `NUM_LAYERS`: Model architecture.
-   `MODEL_PATH`: Base path for saving/loading models. (Scripts will append `-cpu.bin` or `-gpu.bin`).
-   `TEMPERATURE`, `TOP_P`: Sampling parameters for chat.

You can create multiple `.env` files (e.g., `wikitext.env`) and pass them as an argument to the scripts:
```bash
# Example: Use a custom config file for training
./train_gpu.sh wikitext.env
```

### Training

Training scripts will automatically save the trained model to the path specified in `config.env` (with `-cpu.bin` or `-gpu.bin` appended) and log performance metrics to `training_log.txt`.

#### Train on CPU

```bash
./train_cpu.sh
```

#### Train on GPU (Apple Silicon, NVIDIA, AMD, Intel)

```bash
./train_gpu.sh
```

**Note on GPU Acceleration**: While GPU acceleration is implemented, achieving significant speedups requires offloading more operations than just matrix multiplication. The `wgpu` backend provides cross-platform compatibility but introduces overhead due to data transfers between CPU and GPU for each operation. For a detailed explanation, refer to [`docs/acceleration.md`](docs/acceleration.md).

### Chatting

After training a model, you can interact with it using the chat scripts.

#### Chat on CPU

```bash
./chat_cpu.sh
```

#### Chat on GPU

```bash
./chat_gpu.sh
```

When running `chat_cpu.sh` or `chat_gpu.sh` without a trained model, it will list available models in the `models/` directory.

### Other Commands

The `run.sh` script also offers other utility commands:

-   `./run.sh build`: Compiles the project in release mode.
-   `./run.sh check`: Checks the project for errors without building.
-   `./run.sh test`: Runs all unit and integration tests.
-   `./run.sh get-data [dataset]`: Downloads specified datasets (e.g., `tinyshakespeare`).
-   `./run.sh help`: Displays usage information.

## Documentation

-   **Data Handling**: [`docs/data_handling.md`](docs/data_handling.md)
-   **GPU Acceleration**: [`docs/acceleration.md`](docs/acceleration.md)
-   **Model Evaluation**: [`docs/evaluation.md`](docs/evaluation.md)

## Contributing

RustyFlow is a work in progress. Contributions, feedback, and suggestions are welcome! Please open an issue or submit a pull request on GitHub.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
