//! ST Protocol - Binary Wire Protocol for Smart Tree Daemon
//!
//! A tight, 6502-inspired binary protocol using control ASCII (0x00-0x1F) as opcodes.
//! No JSON in the core path. Every byte means something.
//!
//! ## Frame Format
//!
//! ```text
//! ┌──────┬─────────────────┬──────┐
//! │ verb │     payload     │ 0x00 │
//! │ 1B   │   N bytes       │ END  │
//! └──────┴─────────────────┴──────┘
//! ```
//!
//! ## Escape Sequences
//!
//! - `0x1B 0x1B` = literal `0x1B` in payload
//! - `0x1B 0x00` = literal `0x00` in payload
//!
//! ## Network Addressing
//!
//! Single byte prefix for routing:
//! - `0x00` = local daemon (Unix socket)
//! - `0x01-0x7F` = cached host index
//! - `0x80-0xFE` = inline address (len = byte - 0x80)
//! - `0xFF` = broadcast/discover

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std as alloc;

mod verb;
mod frame;
mod payload;
mod address;
mod error;
mod auth;

pub use verb::Verb;
pub use frame::{Frame, FrameBuilder};
pub use payload::{Payload, PayloadEncoder, PayloadDecoder};
pub use address::{Address, AddressString, HostCache};
pub use error::{ProtocolError, ProtocolResult};
pub use auth::{AuthLevel, AuthBlock, SecurityContext, SessionId, Signature};
pub use auth::{is_protected_path, path_auth_level, PROTECTED_PATHS};

/// Protocol version
pub const VERSION: u8 = 1;

/// End of frame marker
pub const END: u8 = 0x00;

/// Escape byte
pub const ESC: u8 = 0x1B;

/// Maximum payload length (64KB)
pub const MAX_PAYLOAD_LEN: usize = 65535;

/// Maximum frame size including overhead
pub const MAX_FRAME_SIZE: usize = MAX_PAYLOAD_LEN + 4;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_frame() {
        // PING is just ENQ + END = 2 bytes
        let frame = Frame::new(Verb::Ping, Payload::empty());
        let encoded = frame.encode();
        assert_eq!(encoded, vec![0x05, 0x00]);
    }

    #[test]
    fn test_scan_frame() {
        // SCAN /home/hue depth=3
        let mut payload = Payload::new();
        payload.push_str("/home/hue");
        payload.push_byte(3); // depth

        let frame = Frame::new(Verb::Scan, payload);
        let encoded = frame.encode();

        // Verify structure
        assert_eq!(encoded[0], 0x01); // SCAN verb
        assert_eq!(encoded[encoded.len() - 1], 0x00); // END marker
    }

    #[test]
    fn test_escape_sequence() {
        // Payload containing literal 0x00 and 0x1B
        let mut payload = Payload::new();
        payload.push_byte(0x00); // Should become 0x1B 0x00
        payload.push_byte(0x1B); // Should become 0x1B 0x1B
        payload.push_byte(0x42); // Normal byte

        let encoded = payload.encode();
        assert_eq!(encoded, vec![0x1B, 0x00, 0x1B, 0x1B, 0x42]);
    }

    #[test]
    fn test_roundtrip() {
        let original = Frame::new(Verb::Search, Payload::from_string("*.rs"));
        let encoded = original.encode();
        let decoded = Frame::decode(&encoded).unwrap();

        assert_eq!(decoded.verb(), original.verb());
        assert_eq!(decoded.payload().as_bytes(), original.payload().as_bytes());
    }
}
