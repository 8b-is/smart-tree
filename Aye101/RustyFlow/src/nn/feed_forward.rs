//! A position-wise feed-forward network.

use crate::models::transformer::TransformerConfig;
use crate::nn::Module;
use crate::tensor::Tensor;

pub struct FeedForward {
    linear1_weights: Tensor,
    linear1_bias: Tensor,
    linear2_weights: Tensor,
    linear2_bias: Tensor,
}

impl FeedForward {
    pub fn new(config: &TransformerConfig) -> Self {
        println!("INFO: Initializing FeedForward network.");
        let embed_dim = config.embed_dim;
        let ff_dim = embed_dim * 4; // Common practice

        Self {
            linear1_weights: Tensor::rand(vec![embed_dim, ff_dim]),
            linear1_bias: Tensor::zeros(vec![ff_dim]),
            linear2_weights: Tensor::rand(vec![ff_dim, embed_dim]),
            linear2_bias: Tensor::zeros(vec![embed_dim]),
        }
    }
}

impl Module for FeedForward {
    fn forward(&self, input: &Tensor) -> Tensor {
        let x = input.matmul(&self.linear1_weights);
        let x = &x + &self.linear1_bias;
        let x = x.relu();
        let x = x.matmul(&self.linear2_weights);
        let x = &x + &self.linear2_bias;

        x
    }

    fn parameters(&self) -> Vec<Tensor> {
        vec![
            self.linear1_weights.clone(),
            self.linear1_bias.clone(),
            self.linear2_weights.clone(),
            self.linear2_bias.clone(),
        ]
    }
}
