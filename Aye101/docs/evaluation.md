# Evaluating Language Models in RustyFlow

Once a language model is trained, we need a way to measure its performance. This document explains the standard metric used for this purpose—**Perplexity**—and how to perform evaluation in the `RustyFlow` library.

## What is Perplexity?

**Perplexity (PPL)** is the standard metric for evaluating language models. It measures how well a probability model predicts a sample. In simple terms, perplexity is a measure of "surprise." A lower perplexity score indicates that the model is less surprised by the text in the test set, meaning it predicts the sequence of words more accurately.

### How it's Calculated

Perplexity is the exponentiated average cross-entropy loss.

-   **Cross-Entropy Loss**: For each token in a sequence, the loss measures the negative log probability the model assigned to the correct next token. A lower loss means the model was more confident about the correct token.
-   **Average Loss**: We calculate the average loss over an entire dataset (e.g., a validation or test set).
-   **Perplexity**: The final perplexity is calculated as `exp(average_loss)`.

The formula is:
`PPL = exp( (1/N) * Σ( -log P(word_i | context_i) ) )`

Where `N` is the number of tokens in the test set.

**Interpretation:** A perplexity of `K` means that, on average, the model is as confused as if it had to choose uniformly and independently from `K` possibilities for each token. A perfect model that assigns probability 1 to the correct next token would have a perplexity of 1.

## How to Evaluate in RustyFlow

The `language_model` example has been updated to automatically perform evaluation after the training loop completes.

### Train/Validation/Test Splits

Following standard machine learning practice, we split our data into three sets:

1.  **Training Set**: Used to train the model's parameters.
2.  **Validation Set**: Used to tune hyperparameters (like learning rate, model size, etc.) and check for overfitting during development. The model does not train on this data.
3.  **Test Set**: A held-out set used only once to report the final performance of the fully trained and tuned model. This provides an unbiased measure of the model's generalization ability.

### The `language_model` Example

-   **Training**: The model is trained on the training portion of the selected dataset.
-   **Validation**: After training, the example automatically runs an evaluation on the validation set and reports the average loss and perplexity.
-   **Dataset Handling**:
    -   For **`wikitext-2`**, the example uses the official `wiki.train.tokens` and `wiki.valid.tokens` files. To get a final benchmark score, you would evaluate on `wiki.test.tokens`.
    -   For **`tinyshakespeare`** and custom text files, the example performs a 90/10 split to create training and validation sets.

To run the demo and see the evaluation:
```bash
# This will train on TinyShakespeare and then evaluate it.
./run.sh demo
```

The output will include a final section with the validation loss and perplexity. This allows you to see if changes to the model or hyperparameters are improving its predictive power.
