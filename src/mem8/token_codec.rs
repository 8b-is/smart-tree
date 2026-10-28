//! Lossless dictionary tokens for exact text and structured data.
//! Codes 0..255 are bytes; subsequent codes reference previous token + byte.
//! The dictionary is local to each record, so records decode independently.

use anyhow::{bail, ensure, Result};
use std::collections::HashMap;

const LIMIT: usize = 65536;

pub fn encode(input: &[u8]) -> Vec<u8> {
    let Some((&first, rest)) = input.split_first() else {
        return Vec::new();
    };
    let mut dictionary = HashMap::new();
    let mut next = 256u32;
    let mut prefix = u32::from(first);
    let mut output = Vec::new();
    for &byte in rest {
        if let Some(&code) = dictionary.get(&(prefix, byte)) {
            prefix = code;
        } else {
            write_token(&mut output, prefix);
            if next < LIMIT as u32 {
                dictionary.insert((prefix, byte), next);
                next += 1;
            }
            prefix = u32::from(byte);
        }
    }
    write_token(&mut output, prefix);
    output
}

pub fn decode(input: &[u8], expected: usize) -> Result<Vec<u8>> {
    let mut cursor = input;
    let mut dictionary: Vec<Vec<u8>> = (0..256).map(|byte| vec![byte as u8]).collect();
    let mut previous: Option<u32> = None;
    let mut output = Vec::with_capacity(expected.min(1024 * 1024));
    while !cursor.is_empty() {
        let code = read_token(&mut cursor)?;
        let entry = if let Some(entry) = dictionary.get(code as usize) {
            entry.clone()
        } else if code as usize == dictionary.len() && previous.is_some() {
            let mut entry = dictionary[previous.unwrap_or(0) as usize].clone();
            entry.push(entry[0]);
            entry
        } else {
            bail!("Invalid dictionary token {code}");
        };
        ensure!(
            entry.len() <= expected.saturating_sub(output.len()),
            "Token payload exceeds declared size"
        );
        output.extend_from_slice(&entry);
        if let Some(previous) = previous {
            if dictionary.len() < LIMIT {
                let mut added = dictionary[previous as usize].clone();
                added.push(entry[0]);
                dictionary.push(added);
            }
        }
        previous = Some(code);
    }
    ensure!(output.len() == expected, "Incomplete token payload");
    Ok(output)
}

fn write_token(output: &mut Vec<u8>, mut code: u32) {
    while code >= 128 {
        output.push((code as u8 & 127) | 128);
        code >>= 7;
    }
    output.push(code as u8);
}

fn read_token(input: &mut &[u8]) -> Result<u32> {
    let mut code = 0;
    for shift in [0, 7, 14] {
        let (&byte, rest) = input
            .split_first()
            .ok_or_else(|| anyhow::anyhow!("Truncated dictionary token"))?;
        *input = rest;
        code |= u32::from(byte & 127) << shift;
        if byte < 128 {
            ensure!(code < LIMIT as u32, "Dictionary token exceeds limit");
            return Ok(code);
        }
    }
    bail!("Invalid dictionary token length")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_binary_unicode_and_repeated_text() {
        for bytes in [
            Vec::new(),
            vec![0; 5000],
            (0..=255).collect(),
            "🌳 path/name.rs conversation memory "
                .repeat(1000)
                .into_bytes(),
        ] {
            let tokens = encode(&bytes);
            assert_eq!(decode(&tokens, bytes.len()).unwrap(), bytes);
        }
        let bytes = b"project/src/repeated/path".repeat(1000);
        assert!(encode(&bytes).len() < bytes.len() / 4);
    }

    #[test]
    fn malformed_tokens_and_expansion_limits_fail() {
        for bytes in [vec![128], vec![255, 255, 255, 1], vec![128, 2]] {
            assert!(decode(&bytes, 20).is_err());
        }
        assert!(decode(&encode(&[42; 100]), 2).is_err());
        assert!(decode(&encode(b"text"), 5).is_err());
    }
}
