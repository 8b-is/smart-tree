//! Payload encoding/decoding with escape sequences
//!
//! ## Length Prefix Format
//!
//! First byte after verb determines payload format:
//! - `0x20-0x7E` = ASCII string starts (printable chars)
//! - `0x80-0xFE` = Length prefix (len = byte - 0x80, max 126)
//! - `0xFF` = Extended length (next 2 bytes = u16 LE)
//!
//! ## Escape Sequences
//!
//! - `0x1B 0x1B` = literal `0x1B` in payload
//! - `0x1B 0x00` = literal `0x00` in payload

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::{ESC, END, ProtocolError, ProtocolResult};

/// Raw payload data with escape handling
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payload {
    /// Unescaped raw bytes
    #[cfg(feature = "std")]
    data: Vec<u8>,
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    data: alloc::vec::Vec<u8>,
    #[cfg(all(not(feature = "alloc"), not(feature = "std")))]
    data: [u8; 256],
    #[cfg(all(not(feature = "alloc"), not(feature = "std")))]
    len: usize,
}

impl Default for Payload {
    fn default() -> Self {
        Self::new()
    }
}

impl Payload {
    /// Create empty payload
    pub fn new() -> Self {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            Payload { data: Vec::new() }
        }
        #[cfg(all(not(feature = "alloc"), not(feature = "std")))]
        {
            Payload {
                data: [0u8; 256],
                len: 0,
            }
        }
    }

    /// Create empty payload
    pub fn empty() -> Self {
        Self::new()
    }

    /// Create from string slice
    pub fn from_string(s: &str) -> Self {
        let mut p = Self::new();
        p.push_str(s);
        p
    }

    /// Create from bytes
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut p = Self::new();
        for &b in bytes {
            p.push_byte(b);
        }
        p
    }

    /// Push a single byte (unescaped)
    pub fn push_byte(&mut self, b: u8) {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            self.data.push(b);
        }
        #[cfg(all(not(feature = "alloc"), not(feature = "std")))]
        {
            if self.len < 256 {
                self.data[self.len] = b;
                self.len += 1;
            }
        }
    }

    /// Push bytes from a string
    pub fn push_str(&mut self, s: &str) {
        for b in s.bytes() {
            self.push_byte(b);
        }
    }

    /// Push a u16 as little-endian
    pub fn push_u16_le(&mut self, v: u16) {
        self.push_byte((v & 0xFF) as u8);
        self.push_byte((v >> 8) as u8);
    }

    /// Push a u32 as little-endian
    pub fn push_u32_le(&mut self, v: u32) {
        self.push_byte((v & 0xFF) as u8);
        self.push_byte(((v >> 8) & 0xFF) as u8);
        self.push_byte(((v >> 16) & 0xFF) as u8);
        self.push_byte((v >> 24) as u8);
    }

    /// Get raw bytes (unescaped)
    pub fn as_bytes(&self) -> &[u8] {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            &self.data
        }
        #[cfg(all(not(feature = "alloc"), not(feature = "std")))]
        {
            &self.data[..self.len]
        }
    }

    /// Length of unescaped data
    pub fn len(&self) -> usize {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            self.data.len()
        }
        #[cfg(all(not(feature = "alloc"), not(feature = "std")))]
        {
            self.len
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Encode payload with escape sequences for wire format
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.len() * 2); // worst case: all escapes

        for &b in self.as_bytes() {
            match b {
                END => {
                    out.push(ESC);
                    out.push(END);
                }
                ESC => {
                    out.push(ESC);
                    out.push(ESC);
                }
                _ => out.push(b),
            }
        }

        out
    }

    /// Decode payload from wire format (with escape sequences)
    pub fn decode(data: &[u8]) -> ProtocolResult<Self> {
        let mut payload = Self::new();
        let mut i = 0;

        while i < data.len() {
            let b = data[i];

            if b == ESC {
                // Escape sequence
                if i + 1 >= data.len() {
                    return Err(ProtocolError::InvalidEscape);
                }

                let next = data[i + 1];
                match next {
                    END => payload.push_byte(END),  // 0x1B 0x00 = literal 0x00
                    ESC => payload.push_byte(ESC),  // 0x1B 0x1B = literal 0x1B
                    _ => return Err(ProtocolError::InvalidEscape),
                }
                i += 2;
            } else {
                payload.push_byte(b);
                i += 1;
            }
        }

        Ok(payload)
    }

    /// Try to interpret as UTF-8 string
    pub fn as_str(&self) -> Option<&str> {
        core::str::from_utf8(self.as_bytes()).ok()
    }

    /// Read a u16 LE at offset
    pub fn read_u16_le(&self, offset: usize) -> Option<u16> {
        let bytes = self.as_bytes();
        if offset + 2 > bytes.len() {
            return None;
        }
        Some(u16::from_le_bytes([bytes[offset], bytes[offset + 1]]))
    }

    /// Read a u32 LE at offset
    pub fn read_u32_le(&self, offset: usize) -> Option<u32> {
        let bytes = self.as_bytes();
        if offset + 4 > bytes.len() {
            return None;
        }
        Some(u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]))
    }
}

/// Payload encoder for building complex payloads
pub struct PayloadEncoder {
    payload: Payload,
}

impl Default for PayloadEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl PayloadEncoder {
    pub fn new() -> Self {
        PayloadEncoder {
            payload: Payload::new(),
        }
    }

    /// Add length-prefixed string (short form: 0x80-0xFE)
    pub fn string(mut self, s: &str) -> Self {
        let len = s.len();
        if len <= 126 {
            self.payload.push_byte((len as u8) + 0x80);
        } else {
            self.payload.push_byte(0xFF);
            self.payload.push_u16_le(len as u16);
        }
        self.payload.push_str(s);
        self
    }

    /// Add raw bytes with length prefix
    pub fn bytes(mut self, data: &[u8]) -> Self {
        let len = data.len();
        if len <= 126 {
            self.payload.push_byte((len as u8) + 0x80);
        } else {
            self.payload.push_byte(0xFF);
            self.payload.push_u16_le(len as u16);
        }
        for &b in data {
            self.payload.push_byte(b);
        }
        self
    }

    /// Add single byte
    pub fn byte(mut self, b: u8) -> Self {
        self.payload.push_byte(b);
        self
    }

    /// Add u16 little-endian
    pub fn u16_le(mut self, v: u16) -> Self {
        self.payload.push_u16_le(v);
        self
    }

    /// Add u32 little-endian
    pub fn u32_le(mut self, v: u32) -> Self {
        self.payload.push_u32_le(v);
        self
    }

    /// Finish building and return payload
    pub fn build(self) -> Payload {
        self.payload
    }
}

/// Payload decoder for parsing complex payloads
pub struct PayloadDecoder<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> PayloadDecoder<'a> {
    pub fn new(payload: &'a Payload) -> Self {
        PayloadDecoder {
            data: payload.as_bytes(),
            pos: 0,
        }
    }

    /// Remaining bytes
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    /// Read a single byte
    pub fn byte(&mut self) -> Option<u8> {
        if self.pos < self.data.len() {
            let b = self.data[self.pos];
            self.pos += 1;
            Some(b)
        } else {
            None
        }
    }

    /// Read u16 little-endian
    pub fn u16_le(&mut self) -> Option<u16> {
        if self.pos + 2 <= self.data.len() {
            let v = u16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]]);
            self.pos += 2;
            Some(v)
        } else {
            None
        }
    }

    /// Read u32 little-endian
    pub fn u32_le(&mut self) -> Option<u32> {
        if self.pos + 4 <= self.data.len() {
            let v = u32::from_le_bytes([
                self.data[self.pos],
                self.data[self.pos + 1],
                self.data[self.pos + 2],
                self.data[self.pos + 3],
            ]);
            self.pos += 4;
            Some(v)
        } else {
            None
        }
    }

    /// Read length-prefixed string
    pub fn string(&mut self) -> Option<&'a str> {
        let len_byte = self.byte()?;

        let len = if len_byte == 0xFF {
            self.u16_le()? as usize
        } else if len_byte >= 0x80 {
            (len_byte - 0x80) as usize
        } else {
            // Printable ASCII - read until non-printable or end
            self.pos -= 1;
            let start = self.pos;
            while self.pos < self.data.len() && self.data[self.pos] >= 0x20 && self.data[self.pos] <= 0x7E {
                self.pos += 1;
            }
            let s = core::str::from_utf8(&self.data[start..self.pos]).ok()?;
            return Some(s);
        };

        if self.pos + len > self.data.len() {
            return None;
        }

        let s = core::str::from_utf8(&self.data[self.pos..self.pos + len]).ok()?;
        self.pos += len;
        Some(s)
    }

    /// Read length-prefixed bytes
    pub fn bytes(&mut self) -> Option<&'a [u8]> {
        let len_byte = self.byte()?;

        let len = if len_byte == 0xFF {
            self.u16_le()? as usize
        } else if len_byte >= 0x80 {
            (len_byte - 0x80) as usize
        } else {
            return None; // Raw bytes must have length prefix
        };

        if self.pos + len > self.data.len() {
            return None;
        }

        let data = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Some(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_roundtrip() {
        let original = Payload::from_bytes(&[0x00, 0x1B, 0x42, 0x00, 0x1B]);
        let encoded = original.encode();
        let decoded = Payload::decode(&encoded).unwrap();
        assert_eq!(decoded.as_bytes(), original.as_bytes());
    }

    #[test]
    fn test_string_encoding() {
        let payload = PayloadEncoder::new()
            .string("/home/hue")
            .byte(3) // depth
            .build();

        let mut decoder = PayloadDecoder::new(&payload);
        assert_eq!(decoder.string(), Some("/home/hue"));
        assert_eq!(decoder.byte(), Some(3));
    }

    #[test]
    fn test_short_length_prefix() {
        // String "abc" should encode as: 0x83 'a' 'b' 'c'
        let payload = PayloadEncoder::new().string("abc").build();
        let bytes = payload.as_bytes();
        assert_eq!(bytes[0], 0x83); // 3 + 0x80
        assert_eq!(&bytes[1..4], b"abc");
    }

    #[test]
    fn test_extended_length() {
        // 200-byte string needs extended length
        let long_str = "x".repeat(200);
        let payload = PayloadEncoder::new().string(&long_str).build();
        let bytes = payload.as_bytes();
        assert_eq!(bytes[0], 0xFF); // Extended marker
        assert_eq!(bytes[1], 200); // Low byte
        assert_eq!(bytes[2], 0);   // High byte
    }
}
