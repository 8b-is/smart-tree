// AI Output Discipline Module - Omni's Efficiency Manifesto Implementation
// When AI_TOOLS=1, all non-JSON output goes to stderr to keep stdout clean
// This enables perfect JSON parsing for AI consumers

/// Check if we're in AI mode
pub fn is_ai_mode() -> bool {
    std::env::var("AI_TOOLS").is_ok() || std::env::var("MCP_MODE").is_ok()
}

/// Print to stdout or stderr based on AI mode
#[macro_export]
macro_rules! ai_print {
    ($($arg:tt)*) => {
        if $crate::ai_output::is_ai_mode() {
            eprint!($($arg)*);
        } else {
            print!($($arg)*);
        }
    };
}

/// Println to stdout or stderr based on AI mode
#[macro_export]
macro_rules! ai_println {
    () => {
        if $crate::ai_output::is_ai_mode() {
            eprintln!();
        } else {
            println!();
        }
    };
    ($($arg:tt)*) => {
        if $crate::ai_output::is_ai_mode() {
            eprintln!($($arg)*);
        } else {
            println!($($arg)*);
        }
    };
}

/// Configuration for AI-optimized output
#[derive(Debug, Clone)]
pub struct AiOutputConfig {
    pub mode: String,
    pub compress: bool,
    pub no_emoji: bool,
    pub path_mode: String,
    pub deterministic_sort: bool,
    pub include_digest: bool,
    pub max_depth: Option<usize>,
}

impl Default for AiOutputConfig {
    fn default() -> Self {
        if is_ai_mode() {
            // Omni's recommended defaults for AI consumption
            Self {
                mode: "summary-ai".to_string(),
                compress: true,
                no_emoji: true,
                path_mode: "relative".to_string(),
                deterministic_sort: true,
                include_digest: true,
                max_depth: Some(5), // Reasonable default for overview
            }
        } else {
            // Human-friendly defaults
            Self {
                mode: "classic".to_string(),
                compress: false,
                no_emoji: false,
                path_mode: "off".to_string(),
                deterministic_sort: false,
                include_digest: false,
                max_depth: None,
            }
        }
    }
}

/// Generate a cache key for a given path and configuration
/// This enables AI clients to short-circuit repeated calls
pub fn generate_cache_key(path: &str, config: &AiOutputConfig) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("{:?}", config).hash(&mut hasher);
    
    let hash = hasher.finish();
    format!("st_cache_{:016x}", hash)
}

/// Standardized error response for AI self-correction
#[derive(serde::Serialize)]
pub struct AiError {
    pub code: String,
    pub message: String,
    pub classification: ErrorClass,
    pub hint: String,
    pub example: Option<String>,
    pub expected: Option<String>,
}

#[derive(serde::Serialize, Debug)]
pub enum ErrorClass {
    InvalidArg,
    Security,
    Resource,
    Timeout,
    TooLarge,
    Paginate,
}

impl AiError {
    pub fn invalid_arg(message: &str, hint: &str, example: Option<&str>) -> Self {
        Self {
            code: "INVALID_ARG".to_string(),
            message: message.to_string(),
            classification: ErrorClass::InvalidArg,
            hint: hint.to_string(),
            example: example.map(String::from),
            expected: None,
        }
    }
    
    pub fn security(message: &str, hint: &str) -> Self {
        Self {
            code: "SECURITY".to_string(),
            message: message.to_string(),
            classification: ErrorClass::Security,
            hint: hint.to_string(),
            example: None,
            expected: None,
        }
    }
    
    pub fn too_large(message: &str, hint: &str) -> Self {
        Self {
            code: "TOO_LARGE".to_string(),
            message: message.to_string(),
            classification: ErrorClass::TooLarge,
            hint: format!("Use pagination: {}", hint),
            example: Some("add 'limit: 100, cursor: \"next_page\"' to your request".to_string()),
            expected: None,
        }
    }
}

/// Response wrapper with usage stats and next best calls
#[derive(serde::Serialize)]
pub struct AiResponse<T> {
    pub data: T,
    pub cache_key: String,
    pub digest: Option<String>,
    pub usage: Usage,
    pub next_best_calls: Vec<NextCall>,
}

#[derive(serde::Serialize)]
pub struct Usage {
    pub file_count: usize,
    pub bytes_scanned: usize,
    pub elapsed_ms: u64,
}

#[derive(serde::Serialize)]
pub struct NextCall {
    pub tool: String,
    pub args: serde_json::Value,
    pub tip: String,
}

impl<T> AiResponse<T> {
    pub fn new(data: T, path: &str, config: &AiOutputConfig) -> Self {
        Self {
            data,
            cache_key: generate_cache_key(path, config),
            digest: None, // Set by caller if available
            usage: Usage {
                file_count: 0,
                bytes_scanned: 0,
                elapsed_ms: 0,
            },
            next_best_calls: vec![],
        }
    }
    
    pub fn with_digest(mut self, digest: String) -> Self {
        self.digest = Some(digest);
        self
    }
    
    pub fn with_usage(mut self, file_count: usize, bytes_scanned: usize, elapsed_ms: u64) -> Self {
        self.usage = Usage {
            file_count,
            bytes_scanned,
            elapsed_ms,
        };
        self
    }
    
    pub fn suggest_next(mut self, tool: &str, args: serde_json::Value, tip: &str) -> Self {
        self.next_best_calls.push(NextCall {
            tool: tool.to_string(),
            args,
            tip: tip.to_string(),
        });
        self
    }
}

/// Pagination support for list-style operations
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PaginationParams {
    pub limit: Option<usize>,
    pub cursor: Option<String>,
    pub fields: Option<Vec<String>>, // Field selector for token reduction
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            limit: if is_ai_mode() { Some(100) } else { None },
            cursor: None,
            fields: None,
        }
    }
}

/// Ensure all output follows Omni's discipline
pub fn setup_ai_output() {
    if is_ai_mode() {
        // Ensure panic messages go to stderr
        std::panic::set_hook(Box::new(|info| {
            eprintln!("Smart Tree panic: {}", info);
        }));
        
        // Log that we're in AI mode (to stderr!)
        eprintln!("# Smart Tree running in AI mode - JSON on stdout, logs on stderr");
    }
}