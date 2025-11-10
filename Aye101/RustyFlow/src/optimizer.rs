//! Optimizers for updating model parameters.

use crate::tensor::Tensor;
use std::vec::Vec;

/// A trait for optimizers.
pub trait Optimizer {
    /// Clears the gradients of all parameters managed by the optimizer.
    fn zero_grad(&mut self);
    /// Updates the parameters based on their gradients.
    fn step(&mut self);
}

/// Stochastic Gradient Descent (SGD) optimizer.
pub struct SGD {
    params: Vec<Tensor>,
    lr: f32,
}

impl SGD {
    /// Creates a new SGD optimizer.
    /// `params`: A list of learnable tensors (model parameters).
    /// `lr`: The learning rate.
    pub fn new(params: Vec<Tensor>, lr: f32) -> Self {
        println!("INFO: Initializing SGD optimizer with learning_rate={}", lr);
        Self { params, lr }
    }
}

impl Optimizer for SGD {
    fn zero_grad(&mut self) {
        for p in &self.params {
            p.zero_grad();
        }
    }

    fn step(&mut self) {
        // Gradient clipping threshold. This is a common technique to prevent exploding gradients.
        const CLIP_NORM: f32 = 1.0;

        for p in &self.params {
            if let Some(grad_tensor) = p.grad() {
                let grad_data = grad_tensor.data();
                let norm = grad_data.iter().map(|&x| x * x).sum::<f32>().sqrt();

                if norm > CLIP_NORM {
                    // If the norm of the gradient is greater than the clip value, scale it down.
                    let clipped_grad_data = grad_data.mapv(|g| g * CLIP_NORM / norm);
                    p.set_grad(Tensor::from_data(clipped_grad_data));
                }
            }
            p.update(self.lr);
        }
    }
}
