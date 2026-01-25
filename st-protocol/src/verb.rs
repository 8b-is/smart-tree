//! Verb definitions using control ASCII codes as opcodes
//!
//! Each verb is a single byte in the control ASCII range (0x01-0x1F).
//! 0x00 is reserved for END marker.

/// Protocol verbs mapped to control ASCII codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Verb {
    // === Core Operations ===
    /// SOH (0x01) - Scan directory
    Scan = 0x01,
    /// STX (0x02) - Format output
    Format = 0x02,
    /// ETX (0x03) - Search files/content
    Search = 0x03,
    /// EOT (0x04) - End stream/session
    EndStream = 0x04,

    // === Heartbeat & Status ===
    /// ENQ (0x05) - Ping/health check
    Ping = 0x05,
    /// ACK (0x06) - OK/acknowledgment
    Ok = 0x06,
    /// BEL (0x07) - Alert/notification
    Alert = 0x07,

    // === Navigation & History ===
    /// BS (0x08) - Back/undo
    Back = 0x08,
    /// HT (0x09) - Request context
    Context = 0x09,
    /// LF (0x0A) - Next item in sequence
    Next = 0x0A,

    // === Stats & Completion ===
    /// VT (0x0B) - Statistics request
    Stats = 0x0B,
    /// FF (0x0C) - Clear/reset
    Clear = 0x0C,
    /// CR (0x0D) - Complete/commit
    Complete = 0x0D,

    // === Authentication ===
    /// SO (0x0E) - Start auth block
    AuthStart = 0x0E,
    /// SI (0x0F) - End auth block
    AuthEnd = 0x0F,

    // === Access Control ===
    /// DC1 (0x11) - Permit access
    Permit = 0x11,
    /// DC2 (0x12) - Deny access
    Deny = 0x12,
    /// DC3 (0x13) - Elevate privileges
    Elevate = 0x13,
    /// DC4 (0x14) - Audit log
    Audit = 0x14,

    // === Error & Subscription ===
    /// NAK (0x15) - Error response
    Error = 0x15,
    /// SYN (0x16) - Subscribe to updates
    Subscribe = 0x16,
    /// ETB (0x17) - Unsubscribe
    Unsubscribe = 0x17,
    /// CAN (0x18) - Cancel operation
    Cancel = 0x18,

    // === M8 Memory Operations ===
    /// EM (0x19) - Wave signal (M8 memory)
    M8Wave = 0x19,
    /// SUB (0x1A) - Remember/store
    Remember = 0x1A,
    /// FS (0x1C) - Recall from memory
    Recall = 0x1C,
    /// GS (0x1D) - Forget/delete
    Forget = 0x1D,

    // === Session Management ===
    /// RS (0x1E) - Session control
    Session = 0x1E,
    /// US (0x1F) - User identification
    User = 0x1F,
}

impl Verb {
    /// Create verb from raw byte
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Verb::Scan),
            0x02 => Some(Verb::Format),
            0x03 => Some(Verb::Search),
            0x04 => Some(Verb::EndStream),
            0x05 => Some(Verb::Ping),
            0x06 => Some(Verb::Ok),
            0x07 => Some(Verb::Alert),
            0x08 => Some(Verb::Back),
            0x09 => Some(Verb::Context),
            0x0A => Some(Verb::Next),
            0x0B => Some(Verb::Stats),
            0x0C => Some(Verb::Clear),
            0x0D => Some(Verb::Complete),
            0x0E => Some(Verb::AuthStart),
            0x0F => Some(Verb::AuthEnd),
            0x11 => Some(Verb::Permit),
            0x12 => Some(Verb::Deny),
            0x13 => Some(Verb::Elevate),
            0x14 => Some(Verb::Audit),
            0x15 => Some(Verb::Error),
            0x16 => Some(Verb::Subscribe),
            0x17 => Some(Verb::Unsubscribe),
            0x18 => Some(Verb::Cancel),
            0x19 => Some(Verb::M8Wave),
            0x1A => Some(Verb::Remember),
            0x1C => Some(Verb::Recall),
            0x1D => Some(Verb::Forget),
            0x1E => Some(Verb::Session),
            0x1F => Some(Verb::User),
            _ => None,
        }
    }

    /// Get raw byte value
    #[inline]
    pub fn as_byte(self) -> u8 {
        self as u8
    }

    /// Check if verb requires authentication
    pub fn requires_auth(self) -> bool {
        matches!(
            self,
            Verb::Format
                | Verb::Clear
                | Verb::Permit
                | Verb::Deny
                | Verb::Elevate
                | Verb::Remember
                | Verb::Forget
        )
    }

    /// Get the security level required for this verb
    pub fn security_level(self) -> u8 {
        match self {
            // Level 0: Read-only (no auth)
            Verb::Scan | Verb::Search | Verb::Stats | Verb::Ping |
            Verb::Context | Verb::Recall => 0,

            // Level 1: Local write (session required)
            Verb::Format | Verb::Clear | Verb::Remember |
            Verb::Subscribe | Verb::Unsubscribe => 1,

            // Level 2: Mutate (FIDO required)
            Verb::Forget | Verb::Elevate => 2,

            // Level 3: Admin (FIDO + PIN)
            Verb::Permit | Verb::Deny | Verb::Audit => 3,

            // Everything else: session required
            _ => 1,
        }
    }

    /// Human-readable name
    pub fn name(self) -> &'static str {
        match self {
            Verb::Scan => "SCAN",
            Verb::Format => "FORMAT",
            Verb::Search => "SEARCH",
            Verb::EndStream => "END_STREAM",
            Verb::Ping => "PING",
            Verb::Ok => "OK",
            Verb::Alert => "ALERT",
            Verb::Back => "BACK",
            Verb::Context => "CONTEXT",
            Verb::Next => "NEXT",
            Verb::Stats => "STATS",
            Verb::Clear => "CLEAR",
            Verb::Complete => "COMPLETE",
            Verb::AuthStart => "AUTH_START",
            Verb::AuthEnd => "AUTH_END",
            Verb::Permit => "PERMIT",
            Verb::Deny => "DENY",
            Verb::Elevate => "ELEVATE",
            Verb::Audit => "AUDIT",
            Verb::Error => "ERROR",
            Verb::Subscribe => "SUBSCRIBE",
            Verb::Unsubscribe => "UNSUBSCRIBE",
            Verb::Cancel => "CANCEL",
            Verb::M8Wave => "M8_WAVE",
            Verb::Remember => "REMEMBER",
            Verb::Recall => "RECALL",
            Verb::Forget => "FORGET",
            Verb::Session => "SESSION",
            Verb::User => "USER",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verb_roundtrip() {
        for b in 0x01..=0x1F {
            if let Some(verb) = Verb::from_byte(b) {
                assert_eq!(verb.as_byte(), b);
            }
        }
    }

    #[test]
    fn test_ping_is_0x05() {
        assert_eq!(Verb::Ping.as_byte(), 0x05);
    }

    #[test]
    fn test_security_levels() {
        // Read-only operations are level 0
        assert_eq!(Verb::Scan.security_level(), 0);
        assert_eq!(Verb::Search.security_level(), 0);
        assert_eq!(Verb::Ping.security_level(), 0);

        // Admin operations are level 3
        assert_eq!(Verb::Permit.security_level(), 3);
        assert_eq!(Verb::Deny.security_level(), 3);
    }
}
