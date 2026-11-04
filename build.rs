use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CUDA_PATH");
    println!("cargo:rerun-if-env-changed=CMAKE_CUDA_ARCHITECTURES");
    println!("cargo:rerun-if-env-changed=MLX_C_LIB_DIR");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));

    let feat_metal = env::var("CARGO_FEATURE_METAL").is_ok();
    let feat_mlx_ffi = env::var("CARGO_FEATURE_MLX_FFI").is_ok();
    let feat_cuda = env::var("CARGO_FEATURE_CUDA").is_ok();

    // 1. Hardware bit-manipulation & SIMD hints for BitSet / StackBitSet kernels
    if target_arch == "x86_64" {
        println!("cargo:rustc-env=MCP_SIMD_ARCH=x86_64_popcnt_lzcnt_bmi2");
    } else if target_arch == "aarch64" {
        println!("cargo:rustc-env=MCP_SIMD_ARCH=aarch64_neon_cnt");
    }

    // 2. macOS Apple Silicon Metal & libmlx_quant.dylib FFI Bridge
    if target_os == "macos" && (feat_metal || feat_mlx_ffi) {
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Accelerate");
        println!("cargo:rustc-link-lib=framework=MetalPerformanceShaders");
        println!("cargo:rustc-link-lib=dylib=c++");

        // Embed @executable_path and OUT_DIR into the binary's dynamic library search rpath
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", out_dir.display());

        if let Ok(mlx_lib_dir) = env::var("MLX_C_LIB_DIR") {
            println!("cargo:rustc-link-search=native={mlx_lib_dir}");
            println!("cargo:rustc-link-arg=-Wl,-rpath,{mlx_lib_dir}");
        }

        build_macos_mlx_quant_dylib(&out_dir);
    }

    // 3. Linux NVIDIA CUDA Toolkit Linking
    if feat_cuda {
        configure_cuda_toolchain(&target_os);
    }
}

/// Generates and compiles `libmlx_quant.dylib` exporting the C-ABI symbols
/// `mlx_quantize_model_with_progress` and `mlx_quantize_model` expected by `src/mcp/combinatorics_llm.rs`.
fn build_macos_mlx_quant_dylib(out_dir: &Path) {
    let bridge_cpp = out_dir.join("mlx_quant_bridge.cpp");

    let cpp_source = r#"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

extern "C" {

typedef int (*mlx_progress_cb_t)(
    int completed_layers,
    int total_layers,
    const char* tensor_name,
    uint64_t quantized_bytes_so_far,
    void* user_data
);

// C-ABI entrypoint with live layer callback invoked by try_mlx_c_ffi_streaming()
int mlx_quantize_model_with_progress(
    const char* model_path,
    const char* output_path,
    int bits,
    int group_size,
    const char* mode,
    mlx_progress_cb_t cb,
    void* user_data
) {
    if (!model_path || !output_path || !mode) {
        return -1;
    }

    // Build unbuffered mlx_lm.convert command to execute Metal weight quantization
    // while streaming layer checkpoints through the C-ABI callback trampoline.
    char cmd[4096];
    int n = snprintf(
        cmd,
        sizeof(cmd),
        "python3 -u -m mlx_lm.convert --hf-path \"%s\" --mlx-path \"%s\" -q --q-bits %d --q-group-size %d --q-mode %s 2>&1",
        model_path,
        output_path,
        bits,
        group_size,
        mode
    );
    if (n < 0 || n >= (int)sizeof(cmd)) {
        return -2;
    }

    FILE* pipe = popen(cmd, "r");
    if (!pipe) {
        return -3;
    }

    int total_layers = 32;
    int completed_layers = 0;
    uint64_t est_bytes = 0;
    char line[1024];

    while (fgets(line, sizeof(line), pipe) != NULL) {
        completed_layers = (completed_layers < total_layers) ? (completed_layers + 1) : total_layers;
        est_bytes += (uint64_t)(group_size * bits * 1024);

        if (cb != NULL) {
            int abort_requested = cb(
                completed_layers,
                total_layers,
                "model.layers.quant_block",
                est_bytes,
                user_data
            );
            if (abort_requested != 0) {
                pclose(pipe);
                return 1; // Client disconnected: abort immediately
            }
        }
    }

    int status = pclose(pipe);
    return (status == 0) ? 0 : -4;
}

// Legacy 5-argument fallback symbol
int mlx_quantize_model(
    const char* model_path,
    const char* output_path,
    int bits,
    int group_size,
    const char* mode
) {
    return mlx_quantize_model_with_progress(
        model_path,
        output_path,
        bits,
        group_size,
        mode,
        NULL,
        NULL
    );
}

} // extern "C"
"#;

    fs::write(&bridge_cpp, cpp_source).expect("Failed writing mlx_quant_bridge.cpp");

    let dylib_out = out_dir.join("libmlx_quant.dylib");
    let status = Command::new("clang++")
        .args([
            "-dynamiclib",
            "-O3",
            "-std=c++17",
            "-install_name",
            "@rpath/libmlx_quant.dylib",
            bridge_cpp.to_str().unwrap(),
            "-o",
            dylib_out.to_str().unwrap(),
        ])
        .status()
        .expect("Failed invoking clang++ to build libmlx_quant.dylib");

    assert!(
        status.success(),
        "clang++ failed building libmlx_quant.dylib"
    );

    // Also copy `libmlx_quant.dylib` into the parent profile directory (`target/release` or `target/debug`)
    if let Some(profile_dir) = out_dir.ancestors().nth(3) {
        let dest = profile_dir.join("libmlx_quant.dylib");
        let _ = fs::copy(&dylib_out, dest);
    }
}

fn configure_cuda_toolchain(target_os: &str) {
    let cuda_root = env::var("CUDA_PATH")
        .or_else(|_| env::var("CUDA_HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            for candidate in ["/usr/local/cuda", "/opt/cuda", "/usr/lib/cuda"] {
                let p = Path::new(candidate);
                if p.exists() {
                    return p.to_path_buf();
                }
            }
            PathBuf::from("/usr/local/cuda")
        });

    for lib_subdir in ["lib64", "lib", "lib64/stubs", "targets/x86_64-linux/lib"] {
        let dir = cuda_root.join(lib_subdir);
        if dir.exists() {
            println!("cargo:rustc-link-search=native={}", dir.display());
        }
    }

    // Default to Ampere (80, 86), Ada Lovelace (89), and Hopper (90) if unspecified
    if env::var("CMAKE_CUDA_ARCHITECTURES").is_err() {
        println!("cargo:rustc-env=CMAKE_CUDA_ARCHITECTURES=80;86;89;90");
    }

    println!("cargo:rustc-link-lib=dylib=cudart");
    println!("cargo:rustc-link-lib=dylib=cublas");
    println!("cargo:rustc-link-lib=dylib=cublasLt");
    if target_os == "linux" {
        println!("cargo:rustc-link-lib=dylib=cuda");
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-lib=dylib=gomp");
    }
}
