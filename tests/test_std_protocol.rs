//! Integration test for ST daemon protocol
//!
//! Tests the binary protocol communication with the daemon.

use st_protocol::{Frame, PayloadEncoder, Verb};

#[test]
fn test_ping_frame_encoding() {
    let frame = Frame::ping();
    let encoded = frame.encode();
    assert_eq!(encoded, vec![0x05, 0x00]); // ENQ + END
}

#[test]
fn test_scan_frame_encoding() {
    let frame = Frame::scan("/tmp", 2);
    let encoded = frame.encode();

    // Verify structure
    assert_eq!(encoded[0], 0x01); // SCAN verb
    assert_eq!(encoded[encoded.len() - 1], 0x00); // END marker
}

#[test]
fn test_frame_roundtrip() {
    let original = Frame::scan("/home/hue", 3);
    let encoded = original.encode();
    let decoded = Frame::decode(&encoded).unwrap();

    assert_eq!(decoded.verb(), Verb::Scan);
}

#[test]
fn test_error_frame() {
    let frame = Frame::error("test error");
    let encoded = frame.encode();

    assert_eq!(encoded[0], 0x15); // NAK/Error verb
    let decoded = Frame::decode(&encoded).unwrap();
    assert_eq!(decoded.verb(), Verb::Error);
    assert_eq!(decoded.payload().as_str(), Some("test error"));
}

#[test]
fn test_search_frame() {
    let frame = Frame::search("*.rs");
    let encoded = frame.encode();

    assert_eq!(encoded[0], 0x03); // ETX/Search verb
    let decoded = Frame::decode(&encoded).unwrap();
    assert_eq!(decoded.verb(), Verb::Search);
    assert_eq!(decoded.payload().as_str(), Some("*.rs"));
}

#[test]
fn test_payload_encoder() {
    let payload = PayloadEncoder::new()
        .string("/home/hue")
        .byte(3)
        .build();

    // Verify length-prefixed string
    let bytes = payload.as_bytes();
    assert_eq!(bytes[0], 0x89); // 9 + 0x80 = length prefix
}
