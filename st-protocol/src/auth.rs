//! Authentication and security
//!
//! ## Auth Block Format
//!
//! ```text
//! Protected operation:
//!   0x0E                  ; SO = AUTH_START
//!   [level: 1B]           ; 0x01=pin, 0x02=fido, 0x03=bio
//!   [session: 16B]        ; UUID
//!   [sig: 32B]            ; Ed25519 signature
//!   0x0F                  ; SI = AUTH_END
//!   [verb]                ; Actual operation
//!   ...payload...
//!   0x00                  ; END
//! ```
//!
//! ## Security Levels
//!
//! - Level 0x00: Read-only (SCAN, SEARCH, STATS) - no auth required
//! - Level 0x01: Local write (FORMAT output, temp files) - session required
//! - Level 0x02: Mutate (EDIT, DELETE) - requires FIDO
//! - Level 0x03: Admin (PERMIT, config changes) - requires FIDO + PIN

#[cfg(feature = "std")]
extern crate std as alloc;

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

use crate::{Verb, ProtocolError, ProtocolResult};

/// Authentication level required for operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum AuthLevel {
    /// No authentication required (read-only operations)
    None = 0x00,
    /// Session token required (local writes)
    Session = 0x01,
    /// FIDO/WebAuthn required (mutations)
    Fido = 0x02,
    /// FIDO + PIN required (admin operations)
    FidoPin = 0x03,
}

impl AuthLevel {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x00 => Some(AuthLevel::None),
            0x01 => Some(AuthLevel::Session),
            0x02 => Some(AuthLevel::Fido),
            0x03 => Some(AuthLevel::FidoPin),
            _ => None,
        }
    }

    pub fn as_byte(self) -> u8 {
        self as u8
    }

    /// Human readable name
    pub fn name(self) -> &'static str {
        match self {
            AuthLevel::None => "none",
            AuthLevel::Session => "session",
            AuthLevel::Fido => "fido",
            AuthLevel::FidoPin => "fido+pin",
        }
    }
}

/// Session UUID (16 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SessionId([u8; 16]);

impl SessionId {
    pub fn new(bytes: [u8; 16]) -> Self {
        SessionId(bytes)
    }

    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != 16 {
            return None;
        }
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(slice);
        Some(SessionId(bytes))
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Generate random session ID (std only)
    #[cfg(feature = "std")]
    pub fn random() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Simple pseudo-random based on time (for now)
        // Real implementation should use proper RNG
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let mut bytes = [0u8; 16];
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = ((now >> (i * 8)) & 0xFF) as u8;
        }
        SessionId(bytes)
    }
}


/// Ed25519 signature (32 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signature([u8; 32]);

impl Signature {
    pub fn new(bytes: [u8; 32]) -> Self {
        Signature(bytes)
    }

    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != 32 {
            return None;
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(slice);
        Some(Signature(bytes))
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Empty/null signature
    pub fn empty() -> Self {
        Signature([0u8; 32])
    }
}

impl Default for Signature {
    fn default() -> Self {
        Self::empty()
    }
}

/// Authentication block parsed from wire format
#[derive(Debug, Clone)]
pub struct AuthBlock {
    /// Required authentication level
    pub level: AuthLevel,
    /// Session identifier
    pub session: SessionId,
    /// Ed25519 signature over session + payload
    pub signature: Signature,
}

impl AuthBlock {
    /// Create a new auth block
    pub fn new(level: AuthLevel, session: SessionId, signature: Signature) -> Self {
        AuthBlock {
            level,
            session,
            signature,
        }
    }

    /// Create minimal auth block with just session
    pub fn with_session(session: SessionId) -> Self {
        AuthBlock {
            level: AuthLevel::Session,
            session,
            signature: Signature::empty(),
        }
    }

    /// Auth block size in bytes (1 + 16 + 32 = 49)
    pub const SIZE: usize = 1 + 16 + 32;

    /// Encode auth block (without SO/SI markers)
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn encode(&self) -> alloc::vec::Vec<u8> {
        let mut out = alloc::vec::Vec::with_capacity(Self::SIZE);
        out.push(self.level.as_byte());
        out.extend_from_slice(self.session.as_bytes());
        out.extend_from_slice(self.signature.as_bytes());
        out
    }

    /// Decode auth block (without SO/SI markers)
    pub fn decode(data: &[u8]) -> ProtocolResult<Self> {
        if data.len() < Self::SIZE {
            return Err(ProtocolError::InvalidAuthBlock);
        }

        let level = AuthLevel::from_byte(data[0])
            .ok_or(ProtocolError::InvalidAuthBlock)?;
        let session = SessionId::from_slice(&data[1..17])
            .ok_or(ProtocolError::InvalidAuthBlock)?;
        let signature = Signature::from_slice(&data[17..49])
            .ok_or(ProtocolError::InvalidAuthBlock)?;

        Ok(AuthBlock {
            level,
            session,
            signature,
        })
    }
}

/// Security context for a connection
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Current session (if authenticated)
    session: Option<SessionId>,
    /// Authenticated level
    level: AuthLevel,
    /// User identifier (if known)
    user: Option<[u8; 32]>,
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityContext {
    /// Create unauthenticated context
    pub fn new() -> Self {
        SecurityContext {
            session: None,
            level: AuthLevel::None,
            user: None,
        }
    }

    /// Create authenticated context
    pub fn authenticated(session: SessionId, level: AuthLevel) -> Self {
        SecurityContext {
            session: Some(session),
            level,
            user: None,
        }
    }

    /// Get current session
    pub fn session(&self) -> Option<&SessionId> {
        self.session.as_ref()
    }

    /// Get authentication level
    pub fn level(&self) -> AuthLevel {
        self.level
    }

    /// Check if a verb is permitted
    pub fn can_execute(&self, verb: Verb) -> bool {
        let required = verb.security_level();
        self.level as u8 >= required
    }

    /// Elevate to higher auth level
    pub fn elevate(&mut self, level: AuthLevel, session: SessionId) {
        if level > self.level {
            self.level = level;
            self.session = Some(session);
        }
    }

    /// Check if authenticated at all
    pub fn is_authenticated(&self) -> bool {
        self.level > AuthLevel::None
    }

    /// Set user identifier
    pub fn set_user(&mut self, user: [u8; 32]) {
        self.user = Some(user);
    }
}

/// Protected paths that require elevation
pub const PROTECTED_PATHS: &[&str] = &[
    "~/.claude/settings.json",
    "~/.claude/",
    "~/.config/",
    "~/.ssh/",
    "~/.gnupg/",
    "/etc/",
];

/// Check if a path requires elevated access
pub fn is_protected_path(path: &str) -> bool {
    for protected in PROTECTED_PATHS {
        if path.starts_with(protected) || path.contains(protected) {
            return true;
        }
    }
    false
}

/// Get required auth level for a path
pub fn path_auth_level(path: &str) -> AuthLevel {
    if path.contains("/.claude/") || path.contains("/.ssh/") || path.contains("/.gnupg/") {
        AuthLevel::Fido // FIDO required for sensitive configs
    } else if path.starts_with("/etc/") {
        AuthLevel::FidoPin // Admin required for system files
    } else if is_protected_path(path) {
        AuthLevel::Session // At least session for other protected paths
    } else {
        AuthLevel::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_levels() {
        assert!(AuthLevel::FidoPin > AuthLevel::Fido);
        assert!(AuthLevel::Fido > AuthLevel::Session);
        assert!(AuthLevel::Session > AuthLevel::None);
    }

    #[test]
    fn test_auth_block_roundtrip() {
        let original = AuthBlock {
            level: AuthLevel::Fido,
            session: SessionId::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
            signature: Signature::new([42u8; 32]),
        };

        let encoded = original.encode();
        let decoded = AuthBlock::decode(&encoded).unwrap();

        assert_eq!(decoded.level, original.level);
        assert_eq!(decoded.session.as_bytes(), original.session.as_bytes());
        assert_eq!(decoded.signature.as_bytes(), original.signature.as_bytes());
    }

    #[test]
    fn test_security_context() {
        let mut ctx = SecurityContext::new();

        // Can always execute read-only
        assert!(ctx.can_execute(Verb::Scan));
        assert!(ctx.can_execute(Verb::Ping));

        // Cannot execute protected operations
        assert!(!ctx.can_execute(Verb::Permit));

        // Elevate
        ctx.elevate(AuthLevel::FidoPin, SessionId::default());
        assert!(ctx.can_execute(Verb::Permit));
    }

    #[test]
    fn test_protected_paths() {
        assert!(is_protected_path("~/.claude/settings.json"));
        assert!(is_protected_path("/etc/passwd"));
        assert!(!is_protected_path("/home/user/projects/foo"));
    }

    #[test]
    fn test_path_auth_level() {
        assert_eq!(path_auth_level("/home/user/file.txt"), AuthLevel::None);
        assert_eq!(path_auth_level("~/.claude/settings.json"), AuthLevel::Fido);
        assert_eq!(path_auth_level("/etc/hosts"), AuthLevel::FidoPin);
    }
}
