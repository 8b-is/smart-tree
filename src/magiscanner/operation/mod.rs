pub mod base64;
pub mod hex;
pub mod strings;
pub mod url_extract;
pub mod xor;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Categories mirroring CyberChef's groupings, filtered to security-relevant ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    DataFormat,
    Crypto,
    Hashing,
    Networking,
    Extractors,
    Compression,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DataFormat => write!(f, "Data Format"),
            Self::Crypto => write!(f, "Crypto"),
            Self::Hashing => write!(f, "Hashing"),
            Self::Networking => write!(f, "Networking"),
            Self::Extractors => write!(f, "Extractors"),
            Self::Compression => write!(f, "Compression"),
        }
    }
}

/// Typed parameter value for operation arguments (CyberChef's "Ingredient").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArgValue {
    String(String),
    Int(i64),
    Bool(bool),
    Bytes(Vec<u8>),
}

impl ArgValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Bytes(b) => Some(b),
            _ => None,
        }
    }
}

/// Definition of one argument/ingredient an operation accepts.
pub struct ArgDef {
    pub name: &'static str,
    pub description: &'static str,
    pub default: ArgValue,
}

/// Metadata about an operation.
pub struct OperationMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub category: Category,
    pub args: &'static [ArgDef],
}

/// The core operation trait. Every transform implements this.
/// Mirrors CyberChef's `run(input, args) -> output` pattern.
pub trait Operation: Send + Sync {
    fn meta(&self) -> OperationMeta;

    /// Execute the transform: input bytes + args -> output bytes.
    fn run(
        &self,
        input: &[u8],
        args: &HashMap<String, ArgValue>,
    ) -> Result<Vec<u8>, OperationError>;
}

#[derive(Debug, thiserror::Error)]
pub enum OperationError {
    #[error("invalid argument '{name}': {reason}")]
    InvalidArg { name: String, reason: String },
    #[error("operation failed: {0}")]
    Failed(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Registry mapping operation names to factory functions.
pub struct OperationRegistry {
    factories: HashMap<&'static str, fn() -> Box<dyn Operation>>,
}

impl OperationRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            factories: HashMap::new(),
        };
        reg.register_defaults();
        reg
    }

    fn register_defaults(&mut self) {
        self.register("from_base64", || Box::new(base64::FromBase64));
        self.register("from_hex", || Box::new(hex::FromHex));
        self.register("xor", || Box::new(xor::Xor));
        self.register("extract_strings", || Box::new(strings::ExtractStrings));
        self.register("extract_urls", || Box::new(url_extract::ExtractUrls));
    }

    pub fn register(&mut self, name: &'static str, factory: fn() -> Box<dyn Operation>) {
        self.factories.insert(name, factory);
    }

    pub fn create(&self, name: &str) -> Option<Box<dyn Operation>> {
        self.factories.get(name).map(|f| f())
    }

    pub fn list(&self) -> Vec<&'static str> {
        let mut names: Vec<_> = self.factories.keys().copied().collect();
        names.sort();
        names
    }
}

impl Default for OperationRegistry {
    fn default() -> Self {
        Self::new()
    }
}
