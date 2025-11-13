pub mod attention;
pub mod embedding;
pub mod feed_forward;
pub mod layer_norm;
pub mod patch_embedding;
pub mod linear;

use crate::tensor::Tensor;

/// A trait for a neural network module.
pub trait Module {
    /// Performs a forward pass on the module.
    fn forward(&self, input: &Tensor) -> Tensor;

    /// Returns a vector of all learnable parameters in the module.
    fn parameters(&self) -> Vec<Tensor>;

    /// Zeros out the gradients for all parameters in the module.
    fn zero_grad(&self) {
        for p in self.parameters() {
            p.zero_grad();
        }
    }
}
