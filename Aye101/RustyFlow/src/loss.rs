//! Loss functions.

use crate::tensor::Tensor;

/// Computes the cross-entropy loss between logits and targets.
///
/// This is the standard loss function for multi-class classification.
/// It is equivalent to applying `log_softmax` to the logits and then `nll_loss`.
///
/// # Arguments
/// * `logits` - The raw, unnormalized scores from the model. Shape `[..., num_classes]`.
/// * `targets` - The ground truth labels (class indices). Shape `[...]`.
///
/// The shapes of logits and targets must be broadcastable, with `logits` having one extra dimension.
pub fn cross_entropy_loss(logits: &Tensor, targets: &Tensor) -> Tensor {
    let logits_shape = logits.shape();
    let num_classes = logits_shape[logits_shape.len() - 1];

    // Reshape logits to 2D ([N, num_classes]) and targets to 1D ([N])
    // where N is the product of all other dimensions (e.g., batch_size * seq_len).
    let n: usize = logits_shape[..logits_shape.len() - 1].iter().product();
    let logits_2d = logits.reshape(vec![n, num_classes]);
    let targets_1d = targets.reshape(vec![n]);

    // 1. Calculate log_softmax of the logits.
    let log_probs = logits_2d.log_softmax(1); // Softmax over the class dimension.

    // 2. Compute Negative Log Likelihood loss.
    log_probs.nll_loss(&targets_1d)
}
