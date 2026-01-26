//! Frame encoding/decoding
//!
//! ## Frame Format
//!
//! ```text
//! ┌──────┬─────────────────┬──────┐
//! │ verb │     payload     │ 0x00 │
//! │ 1B   │   N bytes       │ END  │
//! └──────┴─────────────────┴──────┘
//! ```

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::{Verb, Payload, END, ESC, ProtocolError, ProtocolResult, MAX_FRAME_SIZE};

/// A complete protocol frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    verb: Verb,
    payload: Payload,
}

impl Frame {
    /// Create a new frame
    pub fn new(verb: Verb, payload: Payload) -> Self {
        Frame { verb, payload }
    }

    /// Create a simple frame with no payload
    pub fn simple(verb: Verb) -> Self {
        Frame {
            verb,
            payload: Payload::empty(),
        }
    }

    /// Get the verb
    pub fn verb(&self) -> Verb {
        self.verb
    }

    /// Get the payload
    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    /// Consume and return the payload
    pub fn into_payload(self) -> Payload {
        self.payload
    }

    /// Encode frame for wire transmission
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn encode(&self) -> Vec<u8> {
        let encoded_payload = self.payload.encode();
        let mut out = Vec::with_capacity(encoded_payload.len() + 2);

        out.push(self.verb.as_byte());
        out.extend_from_slice(&encoded_payload);
        out.push(END);

        out
    }

    /// Decode frame from wire format
    pub fn decode(data: &[u8]) -> ProtocolResult<Self> {
        // Minimum frame: verb + END = 2 bytes
        if data.len() < 2 {
            return Err(ProtocolError::FrameTooShort);
        }

        if data.len() > MAX_FRAME_SIZE {
            return Err(ProtocolError::FrameTooLarge);
        }

        // Last byte must be END
        if data[data.len() - 1] != END {
            return Err(ProtocolError::MissingEndMarker);
        }

        // First byte is verb
        let verb = Verb::from_byte(data[0]).ok_or(ProtocolError::InvalidVerb(data[0]))?;

        // Payload is everything between verb and END
        let payload_data = &data[1..data.len() - 1];
        let payload = Payload::decode(payload_data)?;

        Ok(Frame { verb, payload })
    }

    /// Check if frame is valid (verb + END marker present)
    pub fn is_valid(data: &[u8]) -> bool {
        if data.len() < 2 {
            return false;
        }
        if data[data.len() - 1] != END {
            return false;
        }
        Verb::from_byte(data[0]).is_some()
    }

    /// Find the end of a frame in a buffer (returns length including END)
    pub fn find_end(data: &[u8]) -> Option<usize> {
        let mut i = 1; // Skip verb byte

        while i < data.len() {
            match data[i] {
                END => return Some(i + 1),
                ESC if i + 1 < data.len() => i += 2, // Skip escape sequence
                ESC => return None, // Incomplete escape
                _ => i += 1,
            }
        }

        None // No END found
    }
}

/// Builder for constructing frames
pub struct FrameBuilder {
    verb: Verb,
    payload: Payload,
}

impl FrameBuilder {
    pub fn new(verb: Verb) -> Self {
        FrameBuilder {
            verb,
            payload: Payload::new(),
        }
    }

    /// Add a byte to payload
    pub fn byte(mut self, b: u8) -> Self {
        self.payload.push_byte(b);
        self
    }

    /// Add a string to payload
    pub fn string(mut self, s: &str) -> Self {
        self.payload.push_str(s);
        self
    }

    /// Add bytes to payload
    pub fn bytes(mut self, data: &[u8]) -> Self {
        for &b in data {
            self.payload.push_byte(b);
        }
        self
    }

    /// Add u16 LE to payload
    pub fn u16_le(mut self, v: u16) -> Self {
        self.payload.push_u16_le(v);
        self
    }

    /// Add u32 LE to payload
    pub fn u32_le(mut self, v: u32) -> Self {
        self.payload.push_u32_le(v);
        self
    }

    /// Build the frame
    pub fn build(self) -> Frame {
        Frame {
            verb: self.verb,
            payload: self.payload,
        }
    }
}

// Convenience constructors for common frames
impl Frame {
    /// Create a PING frame
    pub fn ping() -> Self {
        Frame::simple(Verb::Ping)
    }

    /// Create a STATS frame
    pub fn stats() -> Self {
        Frame::simple(Verb::Stats)
    }

    /// Create an OK/ACK frame
    pub fn ok() -> Self {
        Frame::simple(Verb::Ok)
    }

    /// Create an ERROR frame with message
    pub fn error(message: &str) -> Self {
        Frame::new(Verb::Error, Payload::from_string(message))
    }

    /// Create a SCAN frame
    pub fn scan(path: &str, depth: u8) -> Self {
        let mut payload = Payload::new();
        // Length-prefixed path
        let len = path.len();
        if len <= 126 {
            payload.push_byte((len as u8) + 0x80);
        } else {
            payload.push_byte(0xFF);
            payload.push_u16_le(len as u16);
        }
        payload.push_str(path);
        payload.push_byte(depth);

        Frame::new(Verb::Scan, payload)
    }

    /// Create a SEARCH frame (simple - pattern only)
    pub fn search(pattern: &str) -> Self {
        Frame::new(Verb::Search, Payload::from_string(pattern))
    }

    /// Create a SEARCH frame with path, pattern, and max results
    pub fn search_path(path: &str, pattern: &str, max_results: u8) -> Self {
        let mut payload = Payload::new();

        // Length-prefixed path
        let path_len = path.len();
        if path_len <= 126 {
            payload.push_byte((path_len as u8) + 0x80);
        } else {
            payload.push_byte(0xFF);
            payload.push_u16_le(path_len as u16);
        }
        payload.push_str(path);

        // Length-prefixed pattern
        let pattern_len = pattern.len();
        if pattern_len <= 126 {
            payload.push_byte((pattern_len as u8) + 0x80);
        } else {
            payload.push_byte(0xFF);
            payload.push_u16_le(pattern_len as u16);
        }
        payload.push_str(pattern);

        // Max results
        payload.push_byte(max_results);

        Frame::new(Verb::Search, payload)
    }

    /// Create a FORMAT frame with just mode (lists formats)
    pub fn format(mode: &str) -> Self {
        Frame::new(Verb::Format, Payload::from_string(mode))
    }

    /// Create a FORMAT frame with mode, path, and depth (scans and formats)
    pub fn format_path(mode: &str, path: &str, depth: u8) -> Self {
        let mut payload = Payload::new();

        // Length-prefixed mode
        let mode_len = mode.len();
        if mode_len <= 126 {
            payload.push_byte((mode_len as u8) + 0x80);
        } else {
            payload.push_byte(0xFF);
            payload.push_u16_le(mode_len as u16);
        }
        payload.push_str(mode);

        // Length-prefixed path
        let path_len = path.len();
        if path_len <= 126 {
            payload.push_byte((path_len as u8) + 0x80);
        } else {
            payload.push_byte(0xFF);
            payload.push_u16_le(path_len as u16);
        }
        payload.push_str(path);

        // Depth
        payload.push_byte(depth);

        Frame::new(Verb::Format, payload)
    }

    /// Create a REMEMBER frame
    pub fn remember(content: &str, keywords: &str, memory_type: &str) -> Self {
        let mut payload = Payload::new();

        // Length-prefixed content
        let content_len = content.len();
        if content_len <= 126 {
            payload.push_byte((content_len as u8) + 0x80);
        } else {
            payload.push_byte(0xFF);
            payload.push_u16_le(content_len as u16);
        }
        payload.push_str(content);

        // Length-prefixed keywords
        let keywords_len = keywords.len();
        if keywords_len <= 126 {
            payload.push_byte((keywords_len as u8) + 0x80);
        } else {
            payload.push_byte(0xFF);
            payload.push_u16_le(keywords_len as u16);
        }
        payload.push_str(keywords);

        // Length-prefixed type
        let type_len = memory_type.len();
        if type_len <= 126 {
            payload.push_byte((type_len as u8) + 0x80);
        } else {
            payload.push_byte(0xFF);
            payload.push_u16_le(type_len as u16);
        }
        payload.push_str(memory_type);

        // Default emotional state (neutral)
        payload.push_byte(128); // valence = 0.0
        payload.push_byte(128); // arousal = 0.5

        Frame::new(Verb::Remember, payload)
    }

    /// Create a RECALL frame
    pub fn recall(keywords: &str, max_results: u8) -> Self {
        let mut payload = Payload::new();

        // Length-prefixed keywords
        let keywords_len = keywords.len();
        if keywords_len <= 126 {
            payload.push_byte((keywords_len as u8) + 0x80);
        } else {
            payload.push_byte(0xFF);
            payload.push_u16_le(keywords_len as u16);
        }
        payload.push_str(keywords);

        payload.push_byte(max_results);

        Frame::new(Verb::Recall, payload)
    }

    /// Create a FORGET frame
    pub fn forget(memory_id: &str) -> Self {
        Frame::new(Verb::Forget, Payload::from_string(memory_id))
    }

    /// Create an M8_WAVE frame (get memory stats)
    pub fn m8_wave() -> Self {
        Frame::simple(Verb::M8Wave)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_frame() {
        let frame = Frame::ping();
        let encoded = frame.encode();
        assert_eq!(encoded, vec![0x05, 0x00]); // ENQ + END
    }

    #[test]
    fn test_error_frame() {
        let frame = Frame::error("not found");
        let encoded = frame.encode();
        assert_eq!(encoded[0], 0x15); // NAK
        assert_eq!(encoded[encoded.len() - 1], 0x00); // END
    }

    #[test]
    fn test_frame_roundtrip() {
        let original = Frame::scan("/home/hue", 3);
        let encoded = original.encode();
        let decoded = Frame::decode(&encoded).unwrap();

        assert_eq!(decoded.verb(), Verb::Scan);
    }

    #[test]
    fn test_find_end() {
        let data = vec![0x01, 0x41, 0x42, 0x00]; // SCAN "AB" END
        assert_eq!(Frame::find_end(&data), Some(4));
    }

    #[test]
    fn test_find_end_with_escape() {
        let data = vec![0x01, 0x1B, 0x00, 0x00]; // SCAN ESC-NULL END
        assert_eq!(Frame::find_end(&data), Some(4));
    }

    #[test]
    fn test_builder() {
        let frame = FrameBuilder::new(Verb::Search)
            .string("*.rs")
            .build();

        assert_eq!(frame.verb(), Verb::Search);
        assert_eq!(frame.payload().as_str(), Some("*.rs"));
    }
}
