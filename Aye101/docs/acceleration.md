# GPU Acceleration in RustyFlow

This document outlines the strategy for accelerating model training and inference in RustyFlow using GPU compute.

## The Need for Speed

Modern deep learning models, especially Transformers, involve an enormous number of calculations. The most dominant operation is matrix multiplication (`matmul`). While CPUs are excellent for a wide range of tasks, they are quickly overwhelmed by the sheer volume of parallelizable math required for training large models.

GPUs, with their thousands of cores, are designed for exactly this kind of massively parallel workload. Offloading computation to the GPU is essential for training models in a reasonable amount of time.

## Our Strategy: Cross-Platform GPU Compute with `wgpu`

To ensure RustyFlow remains portable, we are using the `wgpu` crate. `wgpu` is a Rust implementation of the WebGPU API standard, providing a modern, safe, and cross-platform way to interact with GPUs.

By using `wgpu`, our code can run on:
-   **macOS / iOS:** via the **Metal** backend (ideal for Apple Silicon).
-   **Linux / Android:** via the **Vulkan** backend.
-   **Windows:** via **Vulkan** or **DirectX 12**.

This approach avoids vendor lock-in (e.g., writing CUDA code that only runs on NVIDIA GPUs) and makes the library accessible to a wider range of hardware.

## Current Implementation

The integration of GPU acceleration is being done incrementally.

### 1. GPU Context
-   A new module, `src/gpu.rs`, has been introduced.
-   It uses `once_cell` to create a global `GpuContext` that is initialized lazily on its first use.
-   When the application starts and a GPU-accelerated operation is first called, RustyFlow will attempt to find a suitable GPU on your system. You will see a message like `INFO: Using GPU adapter: Apple M4` in your console.

### 2. Accelerated Operations
The primary focus of GPU acceleration is the `matmul` operation, which is the most computationally expensive part of a Transformer.

-   **2D Matmul (`[M, K] @ [K, N]`):** The baseline 2D matrix multiplication is implemented on the GPU.
-   **Batched 3D Matmul (`[B, M, K] @ [B, K, N]`):** This is crucial for self-attention and is now fully accelerated on the GPU.
-   **Batched 3D x 2D Matmul (`[B, M, K] @ [K, N]`):** This operation, common in feed-forward layers and projections, is also GPU-accelerated. It is handled by reshaping the 3D tensor to `[B*M, K]` and performing an efficient 2D matmul on the GPU.

All GPU operations are implemented using custom WGSL (WebGPU Shading Language) compute shaders.
- If a GPU operation is attempted but fails (e.g., due to non-contiguous memory), it safely falls back to the CPU and prints a `WARN` message to the console.
- If a GPU operation is not implemented for a specific tensor shape combination, it silently falls back to the CPU.

The primary confirmation that the GPU is being used is the performance difference recorded in `training_log.txt` between CPU and GPU runs. The per-operation `INFO: [GPU] Matmul...` logs have been removed to reduce verbosity.

### 3. Controlling GPU Usage and Profiling
-   **Configuration:** You can set default hyperparameters in your `config.env` file.
-   **Execution:** We now provide two dedicated training scripts:
    -   `./train_gpu.sh`: Attempts to use the GPU and falls back to CPU if a compatible one isn't found.
    -   `./train_cpu.sh`: Forces training on the CPU.
-   **Profiling & Logging:** After a training session, the scripts calculate the average time per epoch and append it to `training_log.txt`. This allows you to easily track and compare performance across different runs, devices, and model configurations.
-   **Model Naming:** The training scripts automatically append `-cpu` or `-gpu` to the model filename (e.g., `...-L2-H4-E128-gpu.bin`), so you know how the model was trained.

### 4. Profiling and Interpreting Performance

After a training run, the script now outputs a detailed profiling report that breaks down where time was spent. This helps to understand the performance characteristics and bottlenecks.

A real-world comparison between a CPU and GPU run on the `wikitext-2` dataset clearly illustrates the current situation:

**GPU Run Report:**
```
[Step 5.5: Profiling Report]
  - Total time in GPU matmul kernels: 268.39s
  - Total matmul time (CPU + GPU): 268.39s (30.5% of total training time)
  - Other CPU time (data loading, other ops, etc.): 612.50s (69.5% of total training time)
[AVG_EPOCH_TIME] 880.89
```

**CPU Run Report:**
```
[Step 5.5: Profiling Report]
  - Total time in CPU matmul fallback: 532.88s
  - Total matmul time (CPU + GPU): 532.88s (55.0% of total training time)
  - Other CPU time (data loading, other ops, etc.): 436.34s (45.0% of total training time)
[AVG_EPOCH_TIME] 969.22
```

**Why isn't the GPU providing a larger speedup? (Amdahl's Law in action)**

The profiling reports above clearly illustrate a common scenario in GPU acceleration and a classic example of **Amdahl's Law**. The law states that the maximum speedup of a program is limited by its sequential (non-parallelizable) parts.

- The GPU successfully cut the `matmul` time in half (from 533s to 268s), a significant acceleration.
- However, this `matmul` time only represented 55% of the total time in the CPU-only run.
- In the GPU run, the `matmul` time shrinks to just **30.5%** of the total. The other **69.5%** of the time is now spent on the CPU, becoming the new, dominant bottleneck.

In our case:
- The **accelerated part** is matrix multiplication (`matmul`), which accounts for **30.5%** of the total time.
- The **sequential part** is "Other CPU time", which accounts for a massive **69.5%** of the total time.

Even if we had an infinitely fast GPU that made `matmul` take zero time, the total training time would only decrease by 30.5%. The remaining 69.5% of the time, spent on the CPU, becomes the new bottleneck.

The "Other CPU time" consists of two main components:

1.  **Data Transfer and Synchronization Overhead:** For *every single GPU matmul call*, data must be made available to the GPU, and the result must be brought back for the next CPU operation. This involves significant overhead:
    *   **On Discrete GPUs (e.g., NVIDIA, AMD):** This is a physical copy of data from the CPU's RAM to the GPU's dedicated VRAM over a bus like PCIe. This is a well-known and significant bottleneck.
    *   **On Unified Memory (e.g., Apple Silicon):** You might ask, "Why is there overhead if the memory is shared?" This is an excellent question. While the CPU and GPU share physical RAM, they are still separate processors. The API (`wgpu`, and Metal underneath) must ensure memory coherency and prevent race conditions. When we "copy" data, we are performing a *synchronization* operation: telling the driver to transfer ownership of that memory region, flush caches, and make it visible to the other processor. While faster than a PCIe transfer, this is not a zero-cost operation and contributes to the "Other CPU time".

    This constant back-and-forth movement of data and ownership is called "ping-ponging" and is computationally expensive. It can often take more time than the `matmul` operation itself, especially for smaller models where the computation is quick.

2.  **Limited Operation Coverage:** Currently, only `matmul` is offloaded to the GPU. All other operations in the model—`relu`, `softmax`, `layer_norm`, element-wise additions for biases and residual connections, `transpose`, `reshape`—are still running on the CPU. This forces the data to return to the CPU after every `matmul`, creating the data transfer and synchronization overhead mentioned above.

Consider a simple `FeedForward` layer:
```rust
// In FeedForward::forward
let x = input.matmul(&self.linear1_weights); // 1. CPU -> GPU -> matmul -> GPU -> CPU
let x = &x + &self.linear1_bias;             // 2. CPU operation
let x = x.relu();                            // 3. CPU operation
let x = x.matmul(&self.linear2_weights); // 4. CPU -> GPU -> matmul -> GPU -> CPU
let x = &x + &self.linear2_bias;             // 5. CPU operation
```
Steps 1 and 4 involve expensive data transfers. To achieve significant speedups, we must reduce these transfers by moving steps 2, 3, and 5 to the GPU as well.

## Future Work: The Path to True Acceleration

The current implementation is a foundational first step. The profiling report tells us exactly where to focus our efforts. The path to unlocking significant GPU performance involves tackling the "Other CPU time".

1.  **Expand GPU Operation Coverage:** The most immediate next step is to implement more compute shaders for other common neural network operations. Good candidates include:
    -   Element-wise operations (`add`, `mul`, `relu`).
    -   Reduction operations (`sum`).
    -   Complex, fused kernels for `softmax` and `layer_norm`.
    Each operation moved to the GPU reduces the amount of data ping-ponging.

2.  **"Device-Aware" Tensors:** This is the most critical architectural change for high performance. The `Tensor` struct should be modified to know which device (CPU or GPU) its data resides on.
    -   A `Tensor` could hold either an `ArrayD<f32>` (for CPU) or a `wgpu::Buffer` (for GPU).
    -   When an operation is called (e.g., `a.add(b)`), the implementation would check the device of `a` and `b`.
    -   If both are on the GPU, it would dispatch a GPU `add` kernel without any CPU data transfer.
    -   If one is on the CPU and one is on the GPU, it would automatically transfer one to match the other.
    -   This allows for **chaining multiple GPU operations together** entirely on the GPU, only bringing the final result back to the CPU when absolutely necessary. This is how libraries like PyTorch and TensorFlow achieve their speed.

3.  **Is it straightforward?** Implementing these changes in Rust with `wgpu` is powerful and cross-platform, but it is not trivial. It requires careful management of GPU buffers, memory layouts, bind groups, and shader code. Each new GPU kernel adds complexity. The "Device-Aware" Tensor is a significant redesign of the core `Tensor` API.

    Furthermore, to truly optimize for platforms like Apple Silicon, one might explore more advanced `wgpu` features or even platform-specific APIs to leverage zero-copy buffers, which adds another layer of complexity and can compromise portability. For now, our goal is to stick to the standard, portable `wgpu` patterns while reducing CPU-GPU synchronization points by expanding GPU operation coverage. This is the standard and necessary path for building a high-performance deep learning library from scratch.

By following this incremental approach, we can progressively reduce the "Other CPU time" and unlock the true potential of the GPU.
