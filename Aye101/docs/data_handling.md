# Data Handling in RustyFlow

This document provides a guide on how to handle datasets for training models with the RustyFlow library.

## Configuring a Training Run

Training is configured via a central `config.env` file and executed with the `train.sh` script. This approach allows you to manage all hyperparameters and settings in one place.

### The `config.env` File

If `config.env` doesn't exist, you can create a default version by running:
```bash
./run.sh setup
```

This file contains key-value pairs for settings like:
- `DATASET`: The dataset to use (`tinyshakespeare`, `wikitext-2`, `short`, or a path to a file).
- `MODEL_PATH`: Where to save the trained model.
- `NUM_EPOCHS`, `BATCH_SIZE`, `SEQ_LEN`, `LEARNING_RATE`: Training hyperparameters.

### Starting a Training Session

To start training, simply run the `train.sh` script. It will automatically pick up the settings from `config.env`.

```bash
./train.sh
```

You can also manage multiple configurations by creating different `.env` files and passing the path as an argument:
```bash
# Create a custom config for wikitext
cp config.env wikitext_config.env
# (edit wikitext_config.env to set DATASET="wikitext-2")

# Run training with the custom config
./train.sh wikitext_config.env
```

### Available Pre-configured Datasets

-   `short`: A small, in-memory paragraph for quick sanity checks. No download needed.
-   `tinyshakespeare`: (Default) The complete works of Shakespeare. A good starting point. **Requires download.**
-   `wikitext-2`: A more standard language modeling benchmark. **Included with the project.**

## Dataset Sources

### WikiText-2 (Included)

The **WikiText-2** dataset is a well-known benchmark for language modeling. The pre-tokenized version is **included** directly in this project under the `data/wikitext-2/` directory and is tracked by Git.

-   **Files**: `wiki.train.tokens`, `wiki.valid.tokens`, `wiki.test.tokens`.
-   **Usage**: No download is necessary. To use it, set `DATASET="wikitext-2"` in your `config.env` and run `./train.sh`.

### TinyShakespeare (Downloadable)

The **TinyShakespeare** dataset contains the complete works of Shakespeare. It's a good dataset for initial testing.

-   **Download**: You need to download it using the provided script. Ensure you have `curl` installed.
    ```bash
    ./run.sh get-data tinyshakespeare
    ```
    This will create `data/tinyshakespeare/input.txt`. The `data/tinyshakespeare` directory is ignored by Git.
-   **Usage**: Set `DATASET="tinyshakespeare"` in your `config.env` and run `./train.sh`.

**Note:** The `language_model` example uses the pre-tokenized `wiki.train.tokens` for training and `wiki.valid.tokens` for validation. A proper benchmark result should be reported on `wiki.test.tokens` after model development is complete. See `docs/evaluation.md` for more details.

## A Note on Data in Version Control

The `wikitext-2` dataset is currently tracked by Git. This decision was made to ensure the project works "out of the box" after a fresh clone, without requiring users to download and set up data files separately.

**Trade-offs:**
-   **Pros**: Maximum convenience. Anyone can clone the repository and immediately run training on a standard benchmark.
-   **Cons**: Increases the repository size. The `wikitext-2` dataset is approximately 5MB. For much larger datasets, this approach would be unsuitable.

**Alternative (Compression):**
An alternative approach would be to store a compressed archive (e.g., `data/wikitext-2.zip`) in Git and add the uncompressed directory (`data/wikitext-2/`) to `.gitignore`. A setup script would then be required to extract the data before the first training run.

For this project, convenience was prioritized, but for projects with larger datasets, the compression method is recommended.

## How the `DataLoader` Works

The current `DataLoader` (`src/data.rs`) is designed for simplicity. Here's a summary of its operation:

1.  **Initialization**: `DataLoader::new(corpus, vocab, batch_size, seq_len)`
    - It takes the entire text corpus as a single `&str`.
    - It tokenizes the corpus into a flat vector of token IDs.
    - It then creates overlapping sequences of `seq_len` tokens. For a given sequence `tokens[i..i+seq_len]`, the target is `tokens[i+1..i+1+seq_len]`.

2.  **Iteration**:
    - When you iterate over the `DataLoader`, it yields `DataBatch` structs.
    - Each `DataBatch` contains `inputs` and `targets` `Tensor`s, with a shape of `[batch_size, seq_len]`.
    - It processes the data in chunks of `batch_size`. For simplicity, it drops the final partial batch if the total number of sequences is not perfectly divisible by `batch_size`.

### Limitations and Future Improvements

- **Memory Usage**: The current implementation loads the entire corpus into memory and generates all possible input/target sequences at once. This is fine for datasets like TinyShakespeare, but will not scale to gigabyte-sized datasets.
- **Future Work**: A more advanced `DataLoader` would stream data from disk, tokenize on-the-fly, and use techniques like memory-mapped files to handle very large corpora efficiently.

## Using Your Own Dataset

You can easily train on a custom text file.

1.  **Prepare your data**: Place your training data into a single `.txt` file (e.g., `my_corpus.txt`).
2.  **Update your config**: In `config.env`, set `DATASET="path/to/your/my_corpus.txt"`.
3.  **Adjust Hyperparameters**: In the same `config.env` file, adjust `NUM_EPOCHS`, `BATCH_SIZE`, `SEQ_LEN`, etc., to suit the size and complexity of your dataset.
4.  **Run training**:
    ```bash
    ./train.sh
    ```

This approach allows for flexible experimentation with different text sources while keeping all configuration in one place.
