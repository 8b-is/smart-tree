//! GPU compute backend using wgpu.

use bytemuck;
use ndarray::{s, Array, ArrayD, Axis, Ix2};
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;
use wgpu::util::DeviceExt;

/// Holds the wgpu device, queue, and limits for GPU operations.
pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub limits: wgpu::Limits,
}

impl GpuContext {
    /// Tries to initialize a new GpuContext.
    /// This is an async function that is run inside a blocking context.
    fn new() -> Option<Self> {
        pollster::block_on(async {
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                })
                .await?;

            println!("INFO: Using GPU adapter: {}", adapter.get_info().name);
            let limits = adapter.limits();

            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("RustyFlow GPU Device"),
                        required_features: wgpu::Features::empty(),
                        required_limits: limits.clone(),
                    },
                    None,
                )
                .await
                .ok()?;

            Some(Self { device, queue, limits })
        })
    }
}

/// A lazily initialized, global GPU context.
/// It will attempt to find a GPU device the first time it's accessed.
pub static GPU_CONTEXT: Lazy<Option<GpuContext>> = Lazy::new(GpuContext::new);

/// A global flag to enable or disable GPU acceleration.
/// This is set by the application at startup.
pub static USE_GPU: AtomicBool = AtomicBool::new(false);

/// An atomic counter to track the total time spent waiting on GPU matmul kernels.
pub static TOTAL_GPU_TIME_NS: AtomicU64 = AtomicU64::new(0);

/// Performs 2D matrix multiplication on the GPU, with tiling for large matrices.
pub fn gpu_matmul_2d(
    gpu_context: &GpuContext,
    a: &ArrayD<f32>,
    b: &ArrayD<f32>,
) -> Result<ArrayD<f32>, &'static str> {
    let a_shape = a.shape();
    let b_shape = b.shape();

    if a.ndim() != 2 || b.ndim() != 2 {
        return Err("GPU matmul currently only supports 2D matrices.");
    }
    if a_shape[1] != b_shape[0] {
        return Err("Incompatible dimensions for matrix multiplication.");
    }

    let m = a_shape[0];
    let n = b_shape[1];

    let output_size_bytes = m * n * std::mem::size_of::<f32>();
    let limit = gpu_context.limits.max_storage_buffer_binding_size as usize;

    if output_size_bytes <= limit {
        // If it fits, run the operation in one go.
        return run_matmul_shader(gpu_context, a, b);
    }

    // --- Tiling logic ---
    println!(
        "INFO: [GPU] Output size ({:.2} MB) exceeds limit ({:.2} MB). Tiling matmul.",
        output_size_bytes as f32 / (1024.0 * 1024.0),
        limit as f32 / (1024.0 * 1024.0)
    );

    let bytes_per_output_column = m * std::mem::size_of::<f32>();
    if bytes_per_output_column > limit {
        return Err("Cannot tile matrix multiplication: a single output column is too large for GPU memory limits.");
    }
    let mut n_tile = limit / bytes_per_output_column;

    let workgroup_size_y = 8;
    n_tile = (n_tile / workgroup_size_y) * workgroup_size_y;
    if n_tile == 0 {
        // This can happen if limit / bytes_per_output_column is < 8.
        // We must process at least one column. The shader can handle this with boundary checks.
        n_tile = (limit / bytes_per_output_column).max(1);
    }

    let a_dyn = a.clone();
    let b_2d = b.clone().into_dimensionality::<Ix2>().unwrap();

    let mut result_tiles = Vec::new();
    for n_start in (0..n).step_by(n_tile) {
        let n_end = (n_start + n_tile).min(n);

        let b_tile_view = b_2d.slice(s![.., n_start..n_end]);
        let b_tile_owned = b_tile_view.to_owned().into_dyn();

        // This call is NOT recursive because `run_matmul_shader` doesn't do tiling.
        let c_tile = run_matmul_shader(gpu_context, &a_dyn, &b_tile_owned)?;
        result_tiles.push(c_tile.into_dimensionality::<Ix2>().unwrap());
    }

    // Concatenate tiles on CPU
    let views: Vec<_> = result_tiles.iter().map(|a| a.view()).collect();
    let final_result = ndarray::concatenate(Axis(1), &views)
        .map_err(|_| "Failed to concatenate result tiles on CPU.")?
        .into_dyn();

    Ok(final_result)
}

/// The core GPU matmul logic that runs a single compute shader.
fn run_matmul_shader(
    gpu_context: &GpuContext,
    a: &ArrayD<f32>,
    b: &ArrayD<f32>,
) -> Result<ArrayD<f32>, &'static str> {
    let GpuContext { device, queue, .. } = gpu_context;

    let a_shape = a.shape();
    let b_shape = b.shape();
    let m = a_shape[0];
    let k = a_shape[1];
    let n = b_shape[1];

    let a_2d = a.clone().into_dimensionality::<Ix2>().unwrap();
    let b_2d = b.clone().into_dimensionality::<Ix2>().unwrap();

    let shader_code = format!(
        r#"
        struct Matrix {{
            data: array<f32>,
        }};

        @group(0) @binding(0) var<storage, read> a: Matrix;
        @group(0) @binding(1) var<storage, read> b: Matrix;
        @group(0) @binding(2) var<storage, read_write> c: Matrix;

        @compute @workgroup_size(8, 8, 1)
        fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
            let m: u32 = {}u;
            let k: u32 = {}u;
            let n: u32 = {}u;

            if (global_id.x >= m || global_id.y >= n) {{
                return;
            }}

            var sum = 0.0;
            for (var i = 0u; i < k; i = i + 1u) {{
                sum = sum + a.data[global_id.x * k + i] * b.data[i * n + global_id.y];
            }}
            c.data[global_id.x * n + global_id.y] = sum;
        }}
    "#,
        m, k, n
    );

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Matmul Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_code.into()),
    });

    let a_contig = a_2d.as_standard_layout();
    let b_contig = b_2d.as_standard_layout();

    let a_slice = a_contig
        .as_slice()
        .ok_or("Matrix A is not contiguous in memory for GPU transfer")?;
    let b_slice = b_contig
        .as_slice()
        .ok_or("Matrix B is not contiguous in memory for GPU transfer")?;

    let a_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Matrix A Buffer"),
        contents: bytemuck::cast_slice(a_slice),
        usage: wgpu::BufferUsages::STORAGE,
    });

    let b_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Matrix B Buffer"),
        contents: bytemuck::cast_slice(b_slice),
        usage: wgpu::BufferUsages::STORAGE,
    });

    let c_buffer_size = (m * n * std::mem::size_of::<f32>()) as u64;
    let c_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Matrix C Buffer"),
        size: c_buffer_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Matmul Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Matmul Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Matmul Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: "main",
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Matmul Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: a_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: b_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: c_buffer.as_entire_binding() },
        ],
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Matmul Command Encoder"),
    });

    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Matmul Compute Pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        let workgroup_size_x = 8;
        let workgroup_size_y = 8;
        compute_pass.dispatch_workgroups(
            (m as u32 + workgroup_size_x - 1) / workgroup_size_x,
            (n as u32 + workgroup_size_y - 1) / workgroup_size_y,
            1,
        );
    }

    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size: c_buffer_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    encoder.copy_buffer_to_buffer(&c_buffer, 0, &staging_buffer, 0, c_buffer_size);
    queue.submit(std::iter::once(encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });

    let wait_start = Instant::now();
    device.poll(wgpu::Maintain::Wait);
    rx.recv().unwrap().unwrap();
    TOTAL_GPU_TIME_NS.fetch_add(
        wait_start.elapsed().as_nanos() as u64,
        Ordering::Relaxed,
    );

    let data = buffer_slice.get_mapped_range();
    let result_vec: Vec<f32> = bytemuck::cast_slice(&data).to_vec();

    drop(data);
    staging_buffer.unmap();

    Ok(ArrayD::from_shape_vec(vec![m, n], result_vec).unwrap())
}

/// Performs batched 3D matrix multiplication on the GPU.
/// Shapes: a[B, M, K], b[B, K, N] -> c[B, M, N]
pub fn gpu_matmul_3d(
    gpu_context: &GpuContext,
    a: &ArrayD<f32>,
    b: &ArrayD<f32>,
) -> Result<ArrayD<f32>, &'static str> {
    let GpuContext { device, queue, .. } = gpu_context;
    let a_shape = a.shape();
    let b_shape = b.shape();

    if a.ndim() != 3 || b.ndim() != 3 {
        return Err("GPU matmul_3d requires 3D matrices.");
    }
    if a_shape[0] != b_shape[0] || a_shape[2] != b_shape[1] {
        return Err("Incompatible dimensions for batched matrix multiplication.");
    }

    // Note: This 3D matmul could also be tiled if B * M * N is too large.
    // For now, we assume it fits, as the most common bottleneck is 2D matmul with large vocab.

    let batch_size = a_shape[0];
    let m = a_shape[1];
    let k = a_shape[2];
    let n = b_shape[2];

    let shader_code = format!(
        r#"
        struct Matrix {{
            data: array<f32>,
        }};

        @group(0) @binding(0) var<storage, read> a: Matrix;
        @group(0) @binding(1) var<storage, read> b: Matrix;
        @group(0) @binding(2) var<storage, read_write> c: Matrix;

        @compute @workgroup_size(8, 8, 1)
        fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
            let B: u32 = {}u;
            let M: u32 = {}u;
            let K: u32 = {}u;
            let N: u32 = {}u;

            let b_idx = global_id.z;
            let m_idx = global_id.x;
            let n_idx = global_id.y;

            if (m_idx >= M || n_idx >= N || b_idx >= B) {{
                return;
            }}

            var sum = 0.0;
            for (var k_idx = 0u; k_idx < K; k_idx = k_idx + 1u) {{
                let a_index = b_idx * M * K + m_idx * K + k_idx;
                let b_index = b_idx * K * N + k_idx * N + n_idx;
                sum = sum + a.data[a_index] * b.data[b_index];
            }}
            let c_index = b_idx * M * N + m_idx * N + n_idx;
            c.data[c_index] = sum;
        }}
    "#,
        batch_size, m, k, n
    );

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Batched Matmul Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_code.into()),
    });

    let a_contig = a.as_standard_layout();
    let b_contig = b.as_standard_layout();

    let a_slice = a_contig
        .as_slice()
        .ok_or("Matrix A is not contiguous in memory for GPU transfer")?;
    let b_slice = b_contig
        .as_slice()
        .ok_or("Matrix B is not contiguous in memory for GPU transfer")?;

    let a_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Matrix A Buffer (3D)"),
        contents: bytemuck::cast_slice(a_slice),
        usage: wgpu::BufferUsages::STORAGE,
    });

    let b_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Matrix B Buffer (3D)"),
        contents: bytemuck::cast_slice(b_slice),
        usage: wgpu::BufferUsages::STORAGE,
    });

    let c_buffer_size = (batch_size * m * n * std::mem::size_of::<f32>()) as u64;
    let c_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Matrix C Buffer (3D)"),
        size: c_buffer_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Batched Matmul Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Batched Matmul Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Batched Matmul Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: "main",
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Batched Matmul Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: a_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: b_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: c_buffer.as_entire_binding() },
        ],
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Batched Matmul Command Encoder"),
    });

    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Batched Matmul Compute Pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        let workgroup_size_x = 8;
        let workgroup_size_y = 8;
        compute_pass.dispatch_workgroups(
            (m as u32 + workgroup_size_x - 1) / workgroup_size_x,
            (n as u32 + workgroup_size_y - 1) / workgroup_size_y,
            batch_size as u32,
        );
    }

    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer (3D)"),
        size: c_buffer_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    encoder.copy_buffer_to_buffer(&c_buffer, 0, &staging_buffer, 0, c_buffer_size);
    queue.submit(std::iter::once(encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });

    let wait_start = Instant::now();
    device.poll(wgpu::Maintain::Wait);
    rx.recv().unwrap().unwrap();
    TOTAL_GPU_TIME_NS.fetch_add(
        wait_start.elapsed().as_nanos() as u64,
        Ordering::Relaxed,
    );

    let data = buffer_slice.get_mapped_range();
    let result_vec: Vec<f32> = bytemuck::cast_slice(&data).to_vec();

    drop(data);
    staging_buffer.unmap();

    Ok(ArrayD::from_shape_vec(vec![batch_size, m, n], result_vec).unwrap())
}
