//! Multi-Head Attention module.

use crate::models::transformer::TransformerConfig;
use crate::nn::Module;
use crate::tensor::Tensor;

pub struct MultiHeadAttention {
    num_heads: usize,
    d_k: usize,
    w_q: Tensor,
    w_k: Tensor,
    w_v: Tensor,
    w_o: Tensor,
}

impl MultiHeadAttention {
    pub fn new(config: &TransformerConfig) -> Self {
        println!("INFO: Initializing MultiHeadAttention with {} heads.", config.num_heads);
        assert_eq!(config.embed_dim % config.num_heads, 0, "embed_dim must be divisible by num_heads");

        let embed_dim = config.embed_dim;
        let num_heads = config.num_heads;
        let d_k = embed_dim / num_heads;

        Self {
            num_heads,
            d_k,
            w_q: Tensor::rand(vec![embed_dim, embed_dim]),
            w_k: Tensor::rand(vec![embed_dim, embed_dim]),
            w_v: Tensor::rand(vec![embed_dim, embed_dim]),
            w_o: Tensor::rand(vec![embed_dim, embed_dim]),
        }
    }
}

impl Module for MultiHeadAttention {
    fn forward(&self, input: &Tensor) -> Tensor {
        // Input shape: [batch_size, seq_len, embed_dim]
        let batch_size = input.shape()[0];
        let seq_len = input.shape()[1];
        let embed_dim = input.shape()[2];

        // 1. Project to Q, K, V - uses 3D x 2D matmul
        // Output shape: [batch_size, seq_len, embed_dim]
        let q = input.matmul(&self.w_q);
        let k = input.matmul(&self.w_k);
        let v = input.matmul(&self.w_v);

        // 2. Reshape and transpose for multi-head processing
        // [batch, seq, embed] -> [batch, seq, heads, d_k] -> [batch, heads, seq, d_k]
        let q_multihead = q.reshape(vec![batch_size, seq_len, self.num_heads, self.d_k]).transpose(1, 2);
        let k_multihead = k.reshape(vec![batch_size, seq_len, self.num_heads, self.d_k]).transpose(1, 2);
        let v_multihead = v.reshape(vec![batch_size, seq_len, self.num_heads, self.d_k]).transpose(1, 2);

        // For batched matmul, flatten batch and head dimensions
        // [batch, heads, seq, d_k] -> [batch * heads, seq, d_k]
        let q_flat = q_multihead.reshape(vec![batch_size * self.num_heads, seq_len, self.d_k]);
        let k_flat = k_multihead.reshape(vec![batch_size * self.num_heads, seq_len, self.d_k]);
        let v_flat = v_multihead.reshape(vec![batch_size * self.num_heads, seq_len, self.d_k]);
        
        // 3. Scaled dot-product attention
        // [b*h, seq, d_k] @ [b*h, d_k, seq] -> [b*h, seq, seq]
        let scores = q_flat.matmul(&k_flat.transpose(1, 2));
        let scaled_scores = scores / (self.d_k as f32).sqrt();
        let attention_weights = scaled_scores.softmax(2); // Softmax over the last dim (keys)
        
        // [b*h, seq, seq] @ [b*h, seq, d_k] -> [b*h, seq, d_k]
        let context_flat = attention_weights.matmul(&v_flat);

        // 4. Concatenate heads and final projection
        // [b*h, seq, d_k] -> [batch, heads, seq, d_k] -> [batch, seq, heads, d_k] -> [batch, seq, embed]
        let context = context_flat
            .reshape(vec![batch_size, self.num_heads, seq_len, self.d_k])
            .transpose(1, 2)
            .reshape(vec![batch_size, seq_len, embed_dim]);
            
        // Final projection: [batch, seq, embed] @ [embed, embed] -> [batch, seq, embed]
        let output = context.matmul(&self.w_o);
        
        output
    }

    fn parameters(&self) -> Vec<Tensor> {
        vec![
            self.w_q.clone(),
            self.w_k.clone(),
            self.w_v.clone(),
            self.w_o.clone(),
        ]
    }
}
