//! Layer Normalization module.

use crate::nn::Module;
use crate::tensor::Tensor;
pub struct LayerNorm {
    gamma: Tensor, // Learnable gain
    beta: Tensor,  // Learnable bias
    epsilon: f32,
}

impl LayerNorm {
    /// Creates a new LayerNorm module.
    /// `dim` is the size of the last dimension (the feature dimension).
    pub fn new(dim: usize) -> Self {
        println!("INFO: Initializing LayerNorm with dim={}", dim);
        Self {
            // Initialize gain to 1s and bias to 0s, which are standard starting points.
            gamma: Tensor::new(vec![1.0; dim], vec![dim]),
            beta: Tensor::zeros(vec![dim]),
            epsilon: 1e-5, // Small value to prevent division by zero.
        }
    }
}

impl Module for LayerNorm {
    fn forward(&self, input: &Tensor) -> Tensor {
        // Normalize over the last dimension (the feature dimension, e.g., embed_dim).
        let axis = input.shape().len() - 1;

        // Calculate mean and variance using graph-aware operations.
        // `keep_dims` is true to allow for broadcasting.
        let mean = input.mean_axis(axis, true);
        let variance = input.var_axis(axis, true);

        // Normalize the input: (x - mean) / sqrt(variance + epsilon)
        let normalized = (input - &mean) / &((&variance + self.epsilon).sqrt());

        // Apply scale (gamma) and shift (beta).
        // `gamma` and `beta` are broadcasted across the other dimensions.
        let output = &(&self.gamma * &normalized) + &self.beta;

        output
    }

    fn parameters(&self) -> Vec<Tensor> {
        vec![self.gamma.clone(), self.beta.clone()]
    }
}
