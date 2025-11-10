//! A basic linear layer.

use crate::nn::Module;
use crate::tensor::Tensor;

pub struct Linear {
    weights: Tensor,
    bias: Tensor,
}

impl Linear {
    pub fn new(in_features: usize, out_features: usize) -> Self {
        println!(
            "INFO: Initializing Linear layer with in_features={}, out_features={}",
            in_features, out_features
        );
        Self {
            weights: Tensor::rand(vec![in_features, out_features]),
            bias: Tensor::zeros(vec![out_features]),
        }
    }
}

impl Module for Linear {
    fn forward(&self, input: &Tensor) -> Tensor {
        let x = input.matmul(&self.weights);
        &x + &self.bias
    }

    fn parameters(&self) -> Vec<Tensor> {
        vec![self.weights.clone(), self.bias.clone()]
    }
}
