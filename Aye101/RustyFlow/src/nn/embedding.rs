//! Embedding module.

use crate::nn::Module;
use crate::tensor::Tensor;

pub struct Embedding {
    weights: Tensor,
}

impl Embedding {
    pub fn new(vocab_size: usize, embed_dim: usize) -> Self {
        println!(
            "INFO: Initializing Embedding layer with vocab_size={}, embed_dim={}",
            vocab_size, embed_dim
        );
        // Initialize weights randomly. This matrix will be learned during training.
        let weights = Tensor::rand(vec![vocab_size, embed_dim]);
        Self { weights }
    }
}

impl Module for Embedding {
    fn forward(&self, input: &Tensor) -> Tensor {
        // Input can be 1D [seq_len] or 2D [batch_size, seq_len].
        let input_shape = input.shape();
        let embed_dim = self.weights.shape()[1];

        if input_shape.len() == 1 {
            // Handle 1D input (single sequence) for backward compatibility or inference.
            self.weights.gather(input, 0)
        } else if input_shape.len() == 2 {
            // Handle 2D input (batch of sequences).
            let batch_size = input_shape[0];
            let seq_len = input_shape[1];

            // 1. Reshape [batch_size, seq_len] -> [batch_size * seq_len]
            let flat_input = input.reshape(vec![batch_size * seq_len]);

            // 2. Gather embeddings: [batch_size * seq_len] -> [batch_size * seq_len, embed_dim]
            let gathered = self.weights.gather(&flat_input, 0);

            // 3. Reshape back to [batch_size, seq_len, embed_dim]
            gathered.reshape(vec![batch_size, seq_len, embed_dim])
        } else {
            panic!(
                "Embedding input must be a 1D or 2D tensor of token IDs, but got shape {:?}",
                input_shape
            );
        }
    }

    fn parameters(&self) -> Vec<Tensor> {
        vec![self.weights.clone()]
    }
}
