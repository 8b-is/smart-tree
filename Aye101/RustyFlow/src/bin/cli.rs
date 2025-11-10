use rusty_flow::data::{DataLoader, Vocabulary};
use rusty_flow::gpu;
use rusty_flow::loss;
use rusty_flow::models::transformer::{Transformer, TransformerConfig};
use rusty_flow::nn::Module;
use rusty_flow::optimizer::{Optimizer, SGD};
use rusty_flow::tensor::Tensor;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::Path;
use std::time::Instant;
use sysinfo::{ProcessExt, System, SystemExt};

use bincode;
use ndarray::{s, ArrayD};
use serde::{Deserialize, Serialize};

// --- Serialization ---

/// A container for saving/loading the model, its config, and vocabulary.
#[derive(Serialize, Deserialize)]
struct SerializableModel {
    config: TransformerConfig,
    vocab: Vocabulary,
    params: Vec<ArrayD<f32>>,
    seq_len: usize,
}

/// Helper to save the model and vocab to a file.
fn save_model(
    model: &Transformer,
    config: &TransformerConfig,
    vocab: &Vocabulary,
    seq_len: usize,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure directory exists
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    let params_data: Vec<ArrayD<f32>> = model.parameters().iter().map(|p| p.data().clone()).collect();

    let serializable = SerializableModel {
        config: config.clone(),
        vocab: vocab.clone(),
        params: params_data,
        seq_len,
    };

    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    bincode::serialize_into(writer, &serializable)?;
    println!("  - Model saved to {}", path);
    Ok(())
}

/// Helper to load a model and vocab from a file.
fn load_model(path: &str) -> Result<(Transformer, Vocabulary, usize), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let loaded: SerializableModel = bincode::deserialize_from(reader)?;

    let model = Transformer::new(&loaded.config);
    let model_params = model.parameters();

    if loaded.params.len() != model_params.len() {
        return Err(format!(
            "Parameter count mismatch: loaded {} vs model {}",
            loaded.params.len(),
            model_params.len()
        )
        .into());
    }

    for (p, data) in model_params.iter().zip(loaded.params) {
        if p.shape() != data.shape() {
            return Err(format!(
                "Shape mismatch for parameter: model {:?} vs loaded {:?}",
                p.shape(),
                data.shape()
            )
            .into());
        }
        p.inner.borrow_mut().data = data;
    }
    println!("  - Model loaded successfully from {}", path);
    Ok((model, loaded.vocab, loaded.seq_len))
}

// --- Helper Functions ---

/// A simple helper function to parse command-line arguments.
fn get_arg_value(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == key)
        .and_then(|pos| args.get(pos + 1).cloned())
}

/// Reports the current process's memory usage.
fn report_memory_usage(system: &mut System, stage: &str) {
    if let Ok(pid) = sysinfo::get_current_pid() {
        system.refresh_process(pid);
        if let Some(process) = system.process(pid) {
            let memory_mb = process.memory() as f32 / 1024.0;
            println!("  - MEMORY USAGE ({}): {:.2} MB", stage, memory_mb);
        }
    }
}

/// Splits a corpus into training and validation sets by words.
fn split_corpus_by_words(corpus: &str, train_ratio: f32) -> (String, String) {
    let words: Vec<&str> = corpus.split_whitespace().collect();
    let train_len = (words.len() as f32 * train_ratio).floor() as usize;
    if train_len >= words.len() {
        return (corpus.to_string(), String::new());
    }
    (
        words[..train_len].join(" "),
        words[train_len..].join(" "),
    )
}

// --- Main Logic ---

fn main() {
    let args: Vec<String> = env::args().collect();

    // --- Global Setup: Handle GPU initialization early ---
    let use_gpu_arg = get_arg_value(&args, "--use-gpu")
        .map(|s| s.parse().unwrap_or(false))
        .unwrap_or(false);

    if use_gpu_arg {
        // Attempt to initialize GPU context and set the flag if successful.
        if let Some(_ctx) = &*gpu::GPU_CONTEXT {
            // The Lazy will print the adapter info on first access.
            gpu::USE_GPU.store(true, std::sync::atomic::Ordering::Relaxed);
        } else {
            println!("WARN: --use-gpu was specified, but no compatible GPU was found. Falling back to CPU.");
            gpu::USE_GPU.store(false, std::sync::atomic::Ordering::Relaxed);
        }
    }

    // --- Dispatch to appropriate mode ---
    if args.contains(&"--list-models".to_string()) {
        list_models();
    } else if args.contains(&"--chat".to_string()) {
        run_chat_session(args);
    } else {
        run_training_session(args);
    }
}

fn list_models() {
    println!("--- Available Models ---");
    let models_dir = "models";
    if let Ok(entries) = fs::read_dir(models_dir) {
        let mut models_found = false;
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("bin") {
                    println!("  - {}", path.display());
                    models_found = true;
                }
            }
        }
        if !models_found {
            println!("  No models found in '{}' directory.", models_dir);
        }
    } else {
        println!("  '{}' directory not found or could not be read.", models_dir);
        println!("  (You might need to create it: mkdir models)");
    }
    println!("\nTo chat with a model, update MODEL_PATH in your config file or run './train.sh' to create one.");
}

fn run_training_session(args: Vec<String>) {
    println!("--- RustyFlow Language Model Training ---");

    // --- 1. Define Dataset and Hyperparameters ---
    println!("\n[Step 1: Dataset and Hyperparameters]");
    let dataset_arg =
        get_arg_value(&args, "--dataset").unwrap_or_else(|| "tinyshakespeare".to_string());
    let save_path = get_arg_value(&args, "--save-path");
    let load_path = get_arg_value(&args, "--load-path");

    let batch_size = get_arg_value(&args, "--batch-size")
        .and_then(|s| s.parse().ok())
        .unwrap_or(16);
    let seq_len = get_arg_value(&args, "--seq-len")
        .and_then(|s| s.parse().ok())
        .unwrap_or(64);
    let num_epochs = get_arg_value(&args, "--num-epochs")
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    let learning_rate = get_arg_value(&args, "--learning-rate")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.01);

    println!("  - Dataset: {}", dataset_arg);
    println!("  - Batch Size: {}", batch_size);
    println!("  - Sequence Length: {}", seq_len);
    println!("  - Epochs: {}", num_epochs);
    println!("  - Learning Rate: {}", learning_rate);

    // --- 2. Load Data and Build Vocabulary ---
    println!("\n[Step 2: Vocabulary and Data Loading]");
    let (train_corpus, valid_corpus, data_source_display) = match dataset_arg.as_str() {
        "short" => (
            "A short paragraph for testing. It has a small vocabulary.".to_string(),
            String::new(),
            "in-memory short paragraph".to_string(),
        ),
        "tinyshakespeare" => {
            let path = "data/tinyshakespeare/input.txt";
            if !Path::new(path).exists() {
                panic!("Dataset not found at '{}'. Please run './run.sh get-data tinyshakespeare'.", path);
            }
            let corpus = fs::read_to_string(path).expect("Failed to read dataset file.");
            let (train, valid) = split_corpus_by_words(&corpus, 0.9);
            (train, valid, path.to_string())
        }
        "wikitext-2" => {
            let (train_path, valid_path) = ("data/wikitext-2/wiki.train.tokens", "data/wikitext-2/wiki.valid.tokens");
            if !Path::new(train_path).exists() {
                panic!("WikiText-2 dataset not found. The project should include these files.");
            }
            let train = fs::read_to_string(train_path).expect("Failed to read train file.");
            let valid = fs::read_to_string(valid_path).expect("Failed to read valid file.");
            (train, valid, "wikitext-2 (tokenized)".to_string())
        }
        path if Path::new(path).exists() => {
            let corpus = fs::read_to_string(path).expect("Failed to read custom dataset file.");
            let (train, valid) = split_corpus_by_words(&corpus, 0.9);
            (train, valid, path.to_string())
        }
        _ => panic!("Dataset '{}' not found.", dataset_arg),
    };

    println!("  - Using data source: {}", data_source_display);
    let vocab = Vocabulary::new(&train_corpus);
    println!("  - Vocabulary built with {} unique tokens.", vocab.size());

    // --- 3. Model Configuration ---
    println!("\n[Step 3: Model Configuration]");
    let embed_dim = get_arg_value(&args, "--embed-dim")
        .and_then(|s| s.parse().ok())
        .unwrap_or(128);
    let num_heads = get_arg_value(&args, "--num-heads")
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);
    let num_layers = get_arg_value(&args, "--num-layers")
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);

    let config = TransformerConfig {
        vocab_size: vocab.size(),
        embed_dim,
        num_heads,
        num_layers,
    };
    println!("  - Vocabulary Size: {}", config.vocab_size);
    println!("  - Embedding Dimension: {}", config.embed_dim);
    println!("  - Number of Heads: {}", config.num_heads);
    println!("  - Number of Layers: {}", config.num_layers);

    // --- 4. Instantiate Model and Optimizer ---
    println!("\n[Step 4: Instantiate Model and Optimizer]");
    let model = Transformer::new(&config);

    if let Some(path) = load_path {
        println!("\n[INFO] Loading model from '{}' to continue training...", path);
        let file = File::open(&path).expect("Failed to open model file.");
        let reader = BufReader::new(file);
        let loaded: SerializableModel = bincode::deserialize_from(reader).expect("Failed to deserialize model.");
        
        let model_params = model.parameters();
        if loaded.params.len() == model_params.len() {
            for (p, data) in model_params.iter().zip(loaded.params) {
                if p.shape() == data.shape() {
                    p.inner.borrow_mut().data = data;
                } else {
                    panic!("Shape mismatch while loading model for continued training.");
                }
            }
            println!("  - Loaded weights successfully.");
        } else {
            panic!("Parameter count mismatch while loading model for continued training.");
        }
    }

    let mut optimizer = SGD::new(model.parameters(), learning_rate);
    println!("  - Full Transformer model and SGD optimizer created.");

    // --- 5. Training Loop ---
    println!("\n[Step 5: Training Loop]");
    // Reset profiling counters before training
    gpu::TOTAL_GPU_TIME_NS.store(0, std::sync::atomic::Ordering::Relaxed);
    rusty_flow::tensor::CPU_MATMUL_TIME_NS.store(0, std::sync::atomic::Ordering::Relaxed);

    let mut system = System::new_all();
    report_memory_usage(&mut system, "Before Training");

    let training_start_time = Instant::now();
    for epoch in 0..num_epochs {
        let mut epoch_loss = 0.0;
        let mut num_batches = 0;
        let data_loader = DataLoader::new(&train_corpus, &vocab, batch_size, seq_len);

        if epoch == 0 {
            println!(
                "  - DataLoader created {} training sequences.",
                data_loader.num_sequences()
            );
        }

        for (i, batch) in data_loader.enumerate() {
            let logits = model.forward(&batch.inputs);
            let loss = loss::cross_entropy_loss(&logits, &batch.targets);
            epoch_loss += *loss.data().first().unwrap();
            num_batches += 1;

            optimizer.zero_grad();
            loss.backward();
            optimizer.step();

            if (i + 1) % 50 == 0 {
                println!(
                    "  - Epoch {:>3}/{} | Batch {:>4} | Batch Loss = {:.6}",
                    epoch + 1,
                    num_epochs,
                    i + 1,
                    *loss.data().first().unwrap()
                );
            }
        }

        if num_batches > 0 {
            println!(
                "  - Epoch {:>3}/{}: Average Loss = {:.6}",
                epoch + 1,
                num_epochs,
                epoch_loss / num_batches as f32
            );
        }
        report_memory_usage(&mut system, &format!("End of Epoch {}", epoch + 1));
    }
    let training_duration = training_start_time.elapsed();
    println!("  - Training finished in {:?}.", training_duration);

    // --- 5.5 Profiling Report ---
    println!("\n[Step 5.5: Profiling Report]");
    let use_gpu = gpu::USE_GPU.load(std::sync::atomic::Ordering::Relaxed);
    let gpu_time_ns = gpu::TOTAL_GPU_TIME_NS.load(std::sync::atomic::Ordering::Relaxed);
    let cpu_matmul_time_ns =
        rusty_flow::tensor::CPU_MATMUL_TIME_NS.load(std::sync::atomic::Ordering::Relaxed);

    if use_gpu && gpu_time_ns > 0 {
        let gpu_time_s = gpu_time_ns as f64 / 1e9;
        println!(
            "  - Total time in GPU matmul kernels: {:.4}s",
            gpu_time_s
        );
    }
    if cpu_matmul_time_ns > 0 {
        let cpu_time_s = cpu_matmul_time_ns as f64 / 1e9;
        println!(
            "  - Total time in CPU matmul fallback: {:.4}s",
            cpu_time_s
        );
    }

    let total_training_s = training_duration.as_secs_f64();
    let total_matmul_ns = gpu_time_ns + cpu_matmul_time_ns;
    let total_matmul_s = total_matmul_ns as f64 / 1e9;

    if total_training_s > 0.0 && total_matmul_s > 0.0 {
        println!(
            "  - Total matmul time (CPU + GPU): {:.4}s ({:.1}% of total training time)",
            total_matmul_s,
            (total_matmul_s / total_training_s) * 100.0
        );
        let other_time_s = (total_training_s - total_matmul_s).max(0.0);
        println!(
            "  - Other CPU time (data loading, other ops, etc.): {:.4}s ({:.1}% of total training time)",
            other_time_s,
            (other_time_s / total_training_s) * 100.0
        );
    }
    // --- End Profiling Report ---

    // Add parseable output for logging
    if num_epochs > 0 {
        let avg_epoch_time_secs = training_duration.as_secs_f32() / num_epochs as f32;
        println!("[AVG_EPOCH_TIME] {:.4}", avg_epoch_time_secs);
    }

    // --- 6. Evaluation ---
    println!("\n[Step 6: Evaluation]");
    let eval_start_time = Instant::now();
    run_evaluation(&model, &valid_corpus, &vocab, batch_size, seq_len);
    println!("  - Evaluation finished in {:?}.", eval_start_time.elapsed());

    // --- 7. Save Model ---
    if let Some(path) = save_path {
        println!("\n[Step 7: Saving Model]");
        save_model(&model, &config, &vocab, seq_len, &path).expect("Failed to save model.");
    }

    println!("\n--- Training Finished ---");
}

fn run_chat_session(args: Vec<String>) {
    println!("--- RustyFlow Interactive Chat ---");
    let load_path = get_arg_value(&args, "--load-path").expect("--load-path is required for chat mode.");
    let temperature = get_arg_value(&args, "--temperature")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.8);
    let top_p = get_arg_value(&args, "--top-p")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.9);

    println!("\n[Step 1: Loading Model]");
    let (model, vocab, seq_len) = load_model(&load_path).expect("Failed to load model.");
    println!("  - Context length from model file: {}", seq_len);


    println!("\n[Step 2: Start Chatting]");
    println!("Model loaded. You can now chat with the model.");
    println!("Type a prompt and press Enter. Type 'exit' or 'quit' to end.");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut prompt = String::new();
        io::stdin().read_line(&mut prompt).expect("Failed to read line");
        let prompt = prompt.trim();

        if prompt == "exit" || prompt == "quit" { break; }
        if prompt.is_empty() { continue; }

        generate_text(&model, &vocab, prompt, seq_len, 50, temperature, top_p);
    }
}

// --- Standalone Functions (used by both modes) ---

fn run_evaluation(model: &Transformer, corpus: &str, vocab: &Vocabulary, batch_size: usize, seq_len: usize) {
    if corpus.trim().is_empty() {
        println!("  - Validation set is empty, skipping evaluation.");
        return;
    }
    println!("  - Running evaluation on validation set...");
    let mut total_loss = 0.0;
    let mut num_batches = 0;
    let data_loader = DataLoader::new(corpus, vocab, batch_size, seq_len);
    model.zero_grad();

    for batch in data_loader {
        let logits = model.forward(&batch.inputs);
        let loss = loss::cross_entropy_loss(&logits, &batch.targets);
        total_loss += *loss.data().first().unwrap();
        num_batches += 1;
        loss.backward(); // Clear graph
    }

    if num_batches > 0 {
        let avg_loss = total_loss / num_batches as f32;
        let perplexity = avg_loss.exp();
        println!("  ------------------------------------");
        println!("  - Validation Average Loss: {:.4}", avg_loss);
        println!("  - Validation Perplexity:   {:.4}", perplexity);
        println!("  ------------------------------------");
    }
}

fn generate_text(
    model: &Transformer,
    vocab: &Vocabulary,
    prompt: &str,
    seq_len: usize,
    num_tokens_to_generate: usize,
    temperature: f32,
    top_p: f32,
) {
    println!("  - Generating {} tokens...", num_tokens_to_generate);

    let mut generated_tokens = vocab.tokenize(prompt);
    let mut result_string = prompt.to_string();
    let pad_id = vocab.word_to_id["<pad>"];

    for _ in 0..num_tokens_to_generate {
        let context_start = generated_tokens.len().saturating_sub(seq_len);
        let mut input_tokens = generated_tokens[context_start..].to_vec();

        let padding_needed = seq_len.saturating_sub(input_tokens.len());
        if padding_needed > 0 {
            input_tokens.splice(0..0, vec![pad_id; padding_needed]);
        }

        let input_f32: Vec<f32> = input_tokens.iter().map(|&id| id as f32).collect();
        let input_tensor = Tensor::new(input_f32, vec![1, seq_len]);

        let logits = model.forward(&input_tensor);

        // Scope the data borrow so it's released before backward() is called.
        let next_token_id = {
            let logits_data = logits.data();
            let last_token_logits_slice = logits_data.slice(s![0, seq_len - 1, ..]);

            // --- Advanced Sampling (Temperature and Top-P) ---

            // 1. Apply temperature
            let scaled_logits: Vec<f32> = last_token_logits_slice
                .iter()
                .map(|&l| l / temperature)
                .collect();

            // 2. Calculate softmax probabilities
            let max_logit = scaled_logits
                .iter()
                .fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            let exps: Vec<f32> = scaled_logits
                .iter()
                .map(|&l| (l - max_logit).exp())
                .collect();
            let sum_exps: f32 = exps.iter().sum();
            let probs: Vec<f32> = exps.iter().map(|&e| e / sum_exps).collect();

            // --- Top-P (Nucleus) Sampling ---
            let mut sampled_id: u32;

            if top_p < 1.0 && top_p > 0.0 {
                // Create a vector of (index, probability) pairs to sort
                let mut indexed_probs: Vec<(usize, f32)> =
                    probs.iter().enumerate().map(|(i, &p)| (i, p)).collect();
                // Sort by probability in descending order
                indexed_probs.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

                // Find the nucleus of tokens
                let mut cumulative_prob = 0.0;
                let mut nucleus_indices = Vec::new();
                for &(index, prob) in &indexed_probs {
                    nucleus_indices.push((index, prob));
                    cumulative_prob += prob;
                    if cumulative_prob >= top_p {
                        break;
                    }
                }

                // Re-normalize the probabilities of the nucleus tokens
                let nucleus_sum_probs: f32 = nucleus_indices.iter().map(|&(_, p)| p).sum();
                let nucleus_probs: Vec<(usize, f32)> = nucleus_indices
                    .iter()
                    .map(|&(i, p)| (i, p / nucleus_sum_probs))
                    .collect();

                // Sample from the nucleus distribution
                let r: f32 = rand::random();
                let mut cum_prob = 0.0;
                sampled_id = nucleus_probs.last().unwrap().0 as u32; // Default to last in nucleus
                for &(index, prob) in &nucleus_probs {
                    cum_prob += prob;
                    if r < cum_prob {
                        sampled_id = index as u32;
                        break;
                    }
                }
            } else {
                // Original inverse transform sampling on the full distribution
                let r: f32 = rand::random();
                let mut cum_prob = 0.0;
                sampled_id = (vocab.size() - 1) as u32; // Default to last token
                for (i, &prob) in probs.iter().enumerate() {
                    cum_prob += prob;
                    if r < cum_prob {
                        sampled_id = i as u32;
                        break;
                    }
                }
            }

            sampled_id
        };

        // Clear the computation graph to free memory.
        logits.backward();

        if next_token_id == pad_id || next_token_id == vocab.word_to_id["<unk>"] {
            break;
        }

        generated_tokens.push(next_token_id);
        if let Some(word) = vocab.id_to_word.get(&next_token_id) {
            result_string.push(' ');
            result_string.push_str(word);
        }
    }
    println!("  - Result: \"{}\"", result_string);
}
