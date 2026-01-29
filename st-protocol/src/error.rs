//! Protocol error types

/// Protocol-specific errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    /// Invalid or unknown verb byte
    InvalidVerb(u8),
    /// Frame is too short to be valid
    FrameTooShort,
    /// Frame exceeds maximum size
    FrameTooLarge,
    /// Missing END marker
    MissingEndMarker,
    /// Invalid escape sequence
    InvalidEscape,
    /// Payload decode error
    PayloadError,
    /// Invalid address format
    InvalidAddress,
    /// Authentication required
    AuthRequired,
    /// Authentication failed
    AuthFailed,
    /// Invalid auth block structure
    InvalidAuthBlock,
    /// Insufficient privileges
    InsufficientPrivileges,
    /// Session expired or invalid
    InvalidSession,
    /// Unknown host in cache
    UnknownHost(u8),
    /// I/O error (std only)
    #[cfg(feature = "std")]
    Io(String),
}

impl core::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ProtocolError::InvalidVerb(b) => write!(f, "invalid verb byte: 0x{:02X}", b),
            ProtocolError::FrameTooShort => write!(f, "frame too short"),
            ProtocolError::FrameTooLarge => write!(f, "frame exceeds maximum size"),
            ProtocolError::MissingEndMarker => write!(f, "missing END marker (0x00)"),
            ProtocolError::InvalidEscape => write!(f, "invalid escape sequence"),
            ProtocolError::PayloadError => write!(f, "payload decode error"),
            ProtocolError::InvalidAddress => write!(f, "invalid address format"),
            ProtocolError::AuthRequired => write!(f, "authentication required"),
            ProtocolError::AuthFailed => write!(f, "authentication failed"),
            ProtocolError::InvalidAuthBlock => write!(f, "invalid auth block structure"),
            ProtocolError::InsufficientPrivileges => write!(f, "insufficient privileges"),
            ProtocolError::InvalidSession => write!(f, "invalid or expired session"),
            ProtocolError::UnknownHost(idx) => write!(f, "unknown host at index {}", idx),
            #[cfg(feature = "std")]
            ProtocolError::Io(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ProtocolError {}

/// Result type for protocol operations
pub type ProtocolResult<T> = Result<T, ProtocolError>;
