//! Data loading and processing utilities.

use crate::tensor::Tensor;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

/// Manages the vocabulary for tokenization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vocabulary {
    pub word_to_id: HashMap<String, u32>,
    pub id_to_word: HashMap<u32, String>,
}

impl Vocabulary {
    /// Creates a new vocabulary from a corpus of text.
    pub fn new(corpus: &str) -> Self {
        let mut vocab_set: BTreeSet<&str> = corpus.split_whitespace().collect();
        vocab_set.insert("<pad>");
        vocab_set.insert("<unk>"); // Add unknown token

        let mut word_to_id = HashMap::new();
        let mut id_to_word = HashMap::new();
        for (i, &word) in vocab_set.iter().enumerate() {
            let id = i as u32;
            word_to_id.insert(word.to_string(), id);
            id_to_word.insert(id, word.to_string());
        }

        Self {
            word_to_id,
            id_to_word,
        }
    }

    /// Returns the size of the vocabulary.
    pub fn size(&self) -> usize {
        self.word_to_id.len()
    }

    /// Tokenizes a sentence into a vector of token IDs.
    pub fn tokenize(&self, sentence: &str) -> Vec<u32> {
        let unk_id = *self.word_to_id.get("<unk>").unwrap();
        sentence
            .split_whitespace()
            .map(|word| *self.word_to_id.get(word).unwrap_or(&unk_id))
            .collect()
    }
}

/// Represents a batch of data for training.
#[derive(Debug)]
pub struct DataBatch {
    pub inputs: Tensor,
    pub targets: Tensor,
}

/// An iterator that yields batches of data.
pub struct DataLoader {
    inputs: Vec<Vec<u32>>,
    targets: Vec<Vec<u32>>,
    batch_size: usize,
    seq_len: usize,
    current_pos: usize,
}

impl DataLoader {
    /// Creates a new DataLoader.
    /// It tokenizes the corpus, creates input/target pairs, and prepares for batching.
    pub fn new(corpus: &str, vocab: &Vocabulary, batch_size: usize, seq_len: usize) -> Self {
        let tokens = vocab.tokenize(corpus);

        let mut inputs = Vec::new();
        let mut targets = Vec::new();

        // Create non-overlapping sequences of `seq_len`. This is much more memory-efficient
        // for large datasets than creating a sliding window with a stride of 1.
        if tokens.len() > seq_len {
            for i in (0..(tokens.len() - seq_len)).step_by(seq_len) {
                let input_seq = tokens[i..i + seq_len].to_vec();
                let target_seq = tokens[i + 1..i + 1 + seq_len].to_vec();
                inputs.push(input_seq);
                targets.push(target_seq);
            }
        }

        Self {
            inputs,
            targets,
            batch_size,
            seq_len,
            current_pos: 0,
        }
    }

    /// Returns the total number of sequences created by the loader.
    pub fn num_sequences(&self) -> usize {
        self.inputs.len()
    }
}

impl Iterator for DataLoader {
    type Item = DataBatch;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pos >= self.inputs.len() {
            // Don't reset; the iterator is exhausted. A new DataLoader is created for each epoch.
            return None;
        }

        let end = (self.current_pos + self.batch_size).min(self.inputs.len());
        let input_batch_vecs = &self.inputs[self.current_pos..end];
        let target_batch_vecs = &self.targets[self.current_pos..end];

        // If the last batch is smaller than batch_size, we skip it for simplicity.
        if input_batch_vecs.len() < self.batch_size {
            // End the iteration; don't return a partial batch.
            return None;
        }

        // Flatten and convert to f32 for Tensor creation
        let flat_inputs: Vec<f32> = input_batch_vecs
            .iter()
            .flatten()
            .map(|&id| id as f32)
            .collect();
        let flat_targets: Vec<f32> = target_batch_vecs
            .iter()
            .flatten()
            .map(|&id| id as f32)
            .collect();

        let batch_size = input_batch_vecs.len();
        let inputs = Tensor::new(flat_inputs, vec![batch_size, self.seq_len]);
        let targets = Tensor::new(flat_targets, vec![batch_size, self.seq_len]);

        self.current_pos = end;

        Some(DataBatch { inputs, targets })
    }
}
