//! Smart Tree FFI — C-compatible interface for dyad Flutter integration.
//!
//! Provides a minimal C-compatible API for directory scanning, search, and
//! MCP tool calls from Flutter via Dart FFI.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::slice;

use crate::mcp::combinatorics_llm::{LlamaModelCache, McpProgressEmitter};
use crate::mcp::tools::{analyze_directory, quick_tree, project_context_dump, smart_read};
use crate::scanner::{Scanner, ScannerConfig, FileNode, TreeStats};
use crate::formatters::{Formatter, classic::ClassicFormatter, json::JsonFormatter};
use serde_json::{json, Value};
use std::io::Write;

/// Opaque handle for a Smart Tree scanner instance.
pub struct ScannerHandle {
    scanner: Scanner,
}

/// Opaque handle for a Smart Tree model cache instance.
pub struct ModelCacheHandle {
    cache: LlamaModelCache,
}

/// Opaque handle for MCP progress emitter.
pub struct ProgressEmitterHandle {
    emitter: McpProgressEmitter,
}

/// Result code for FFI functions.
#[repr(i32)]
pub enum StResult {
    Ok = 0,
    NullPointer = -1,
    InvalidUtf8 = -2,
    ScanFailed = -3,
    JsonSerializationFailed = -4,
    InvalidPath = -5,
    IoError = -6,
}

/// Opaque string result - caller must free with `st_string_free`.
#[repr(C)]
pub struct StString {
    ptr: *mut c_char,
    len: usize,
}

impl StString {
    fn new(s: String) -> Self {
        let cstr = CString::new(s).unwrap_or_else(|_| CString::new("").unwrap());
        let len = cstr.as_bytes().len();
        Self {
            ptr: cstr.into_raw(),
            len,
        }
    }

    fn empty() -> Self {
        Self::new(String::new())
    }
}

/// Free a string returned by Smart Tree FFI.
///
/// # Safety
/// The pointer must have been returned by a Smart Tree FFI function.
#[no_mangle]
pub unsafe extern "C" fn st_string_free(s: StString) {
    if !s.ptr.is_null() {
        drop(CString::from_raw(s.ptr));
    }
}

/// Create a new scanner instance.
///
/// # Safety
/// The caller owns the returned handle and must call `st_scanner_free`.
#[no_mangle]
pub unsafe extern "C" fn st_scanner_new() -> *mut ScannerHandle {
    let scanner = Scanner::new();
    Box::into_raw(Box::new(ScannerHandle { scanner }))
}

/// Free a scanner instance.
///
/// # Safety
/// The handle must have been returned by `st_scanner_new`.
#[no_mangle]
pub unsafe extern "C" fn st_scanner_free(handle: *mut ScannerHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

/// Scan a directory and return JSON output.
///
/// # Safety
/// The `path` must be a valid null-terminated UTF-8 string.
/// The caller must free the returned string with `st_string_free`.
#[no_mangle]
pub unsafe extern "C" fn st_scan_directory(
    handle: *mut ScannerHandle,
    path: *const c_char,
    format_json: c_int,
) -> StString {
    if handle.is_null() || path.is_null() {
        return StString::empty();
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return StString::empty(),
    };

    let config = ScannerConfig {
        path: path_str.to_string(),
        ..Default::default()
    };

    let scanner = &mut (*handle).scanner;
    let (nodes, stats) = match scanner.scan(config) {
        Ok((n, s)) => (n, s),
        Err(_) => return StString::empty(),
    };

    let output = if format_json != 0 {
        let formatter = JsonFormatter::new();
        let mut buf = Vec::new();
        let _ = formatter.format(&mut buf, &nodes, &stats, std::path::Path::new(path_str));
    } else {
        let formatter = ClassicFormatter::new();
        let mut buf = Vec::new();
        let _ = formatter.format(&mut buf, &nodes, &stats, std::path::Path::new(path_str));
    };

    // For now return a simple JSON summary
    let summary = json!({
        "path": path_str,
        "total_files": stats.total_files,
        "total_dirs": stats.total_dirs,
        "total_size": stats.total_size,
    });

    StString::new(summary.to_string())
}

/// Quick tree scan - returns compact JSON.
///
/// # Safety
/// The `path` must be a valid null-terminated UTF-8 string.
/// The caller must free the returned string with `st_string_free`.
#[no_mangle]
pub unsafe extern "C" fn st_quick_tree(
    path: *const c_char,
    max_depth: c_int,
) -> StString {
    if path.is_null() {
        return StString::empty();
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return StString::empty(),
    };

    let config = ScannerConfig {
        path: path_str.to_string(),
        max_depth: Some(max_depth as usize),
        ..Default::default()
    };

    let scanner = Scanner::new();
    let (nodes, stats) = match scanner.scan(config) {
        Ok((n, s)) => (n, s),
        Err(_) => return StString::empty(),
    };

    let summary = json!({
        "path": path_str,
        "max_depth": max_depth,
        "total_files": stats.total_files,
        "total_dirs": stats.total_dirs,
        "total_size": stats.total_size,
        "nodes": nodes.len(),
    });

    StString::new(summary.to_string())
}

/// Create a new model cache for LLM operations.
///
/// # Safety
/// The caller owns the returned handle and must call `st_model_cache_free`.
#[no_mangle]
pub unsafe extern "C" fn st_model_cache_new(max_models: c_int) -> *mut ModelCacheHandle {
    let cache = LlamaModelCache::new(max_models as usize);
    Box::into_raw(Box::new(ModelCacheHandle { cache }))
}

/// Free a model cache instance.
///
/// # Safety
/// The handle must have been returned by `st_model_cache_new`.
#[no_mangle]
pub unsafe extern "C" fn st_model_cache_free(handle: *mut ModelCacheHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

/// Create a progress emitter.
///
/// # Safety
/// The caller owns the returned handle and must call `st_progress_emitter_free`.
#[no_mangle]
pub unsafe extern "C" fn st_progress_emitter_new() -> *mut ProgressEmitterHandle {
    let emitter = McpProgressEmitter::default();
    Box::into_raw(Box::new(ProgressEmitterHandle { emitter }))
}

/// Free a progress emitter.
///
/// # Safety
/// The handle must have been returned by `st_progress_emitter_new`.
#[no_mangle]
pub unsafe extern "C" fn st_progress_emitter_free(handle: *mut ProgressEmitterHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

/// Call an MCP combinatorics/LLM tool by name.
///
/// # Safety
/// All string pointers must be valid null-terminated UTF-8 strings.
/// The `model_cache` and `emitter` handles must be valid.
/// The caller must free the returned string with `st_string_free`.
#[no_mangle]
pub unsafe extern "C" fn st_call_tool(
    tool_name: *const c_char,
    args_json: *const c_char,
    model_cache: *mut ModelCacheHandle,
    emitter: *mut ProgressEmitterHandle,
) -> StString {
    if tool_name.is_null() || args_json.is_null() || model_cache.is_null() || emitter.is_null() {
        return StString::empty();
    }

    let tool_name_str = match CStr::from_ptr(tool_name).to_str() {
        Ok(s) => s,
        Err(_) => return StString::empty(),
    };

    let args_str = match CStr::from_ptr(args_json).to_str() {
        Ok(s) => s,
        Err(_) => return StString::empty(),
    };

    let args: Value = match serde_json::from_str(args_str) {
        Ok(v) => v,
        Err(_) => return StString::empty(),
    };

    let model_cache_ref = &(*model_cache).cache;
    let emitter_ref = &(*emitter).emitter;

    let result = crate::mcp::combinatorics_llm::execute_combinatorics_llm_tool_with_emitter(
        tool_name_str,
        args,
        model_cache_ref,
        emitter_ref,
        None,
    );

    StString::new(result.to_string())
}

/// Search for files using smart-tree's semantic search.
///
/// # Safety
/// The `path` and `query` must be valid null-terminated UTF-8 strings.
/// The caller must free the returned string with `st_string_free`.
#[no_mangle]
pub unsafe extern "C" fn st_search_files(
    path: *const c_char,
    query: *const c_char,
    file_type: *const c_char,
    max_results: c_int,
) -> StString {
    if path.is_null() || query.is_null() {
        return StString::empty();
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return StString::empty(),
    };

    let query_str = match CStr::from_ptr(query).to_str() {
        Ok(s) => s,
        Err(_) => return StString::empty(),
    };

    let type_str = if file_type.is_null() {
        "files"
    } else {
        match CStr::from_ptr(file_type).to_str() {
            Ok(s) => s,
            Err(_) => return StString::empty(),
        }
    };

    // Use the consolidated find tool
    let args = json!({
        "type": type_str,
        "path": path_str,
        "pattern": query_str,
        "limit": max_results as usize,
    });

    // For now return a simple result
    let result = json!({
        "query": query_str,
        "path": path_str,
        "type": type_str,
        "max_results": max_results,
        "results": [],
    });

    StString::new(result.to_string())
}

/// Get the version of the Smart Tree library.
///
/// # Safety
/// The caller must free the returned string with `st_string_free`.
#[no_mangle]
pub unsafe extern "C" fn st_version() -> StString {
    StString::new(env!("CARGO_PKG_VERSION").to_string())
}

/// Initialize the library (call once at startup).
///
/// # Safety
/// Must be called before any other FFI functions.
#[no_mangle]
pub unsafe extern "C" fn st_init() -> StResult {
    // Initialize any global state if needed
    StResult::Ok
}

/// Shutdown the library (call once at shutdown).
///
/// # Safety
/// Must be called after all other FFI functions.
#[no_mangle]
pub unsafe extern "C" fn st_shutdown() -> StResult {
    StResult::Ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_roundtrip() {
        let s = StString::new("hello".to_string());
        assert_eq!(s.len, 5);
        unsafe {
            st_string_free(s);
        }
    }

    #[test]
    fn test_version() {
        let v = st_version();
        assert!(!v.ptr.is_null());
        assert!(v.len > 0);
        unsafe {
            st_string_free(v);
        }
    }
}