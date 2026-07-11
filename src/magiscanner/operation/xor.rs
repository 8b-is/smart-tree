use super::{ArgValue, Category, Operation, OperationError, OperationMeta};
use std::collections::HashMap;

/// XOR data with a key. Supports null-preserving mode.
pub struct Xor;

impl Operation for Xor {
    fn meta(&self) -> OperationMeta {
        OperationMeta {
            name: "xor",
            description: "XOR data with a repeating key, with optional null-preserving mode",
            category: Category::Crypto,
            args: &[],
        }
    }

    fn run(
        &self,
        input: &[u8],
        args: &HashMap<String, ArgValue>,
    ) -> Result<Vec<u8>, OperationError> {
        let key = args
            .get("key")
            .and_then(|v| match v {
                ArgValue::Bytes(b) => Some(b.clone()),
                ArgValue::String(s) => Some(s.as_bytes().to_vec()),
                ArgValue::Int(i) => Some(vec![*i as u8]),
                _ => None,
            })
            .ok_or_else(|| OperationError::InvalidArg {
                name: "key".to_string(),
                reason: "XOR key is required (bytes, string, or single byte int)".to_string(),
            })?;

        if key.is_empty() {
            return Err(OperationError::InvalidArg {
                name: "key".to_string(),
                reason: "XOR key must not be empty".to_string(),
            });
        }

        let null_preserving = args
            .get("null_preserving")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let result: Vec<u8> = input
            .iter()
            .enumerate()
            .map(|(i, &byte)| {
                let key_byte = key[i % key.len()];
                if null_preserving && (byte == 0x00 || byte == key_byte) {
                    byte
                } else {
                    byte ^ key_byte
                }
            })
            .collect();

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_single_byte() {
        let op = Xor;
        let mut args = HashMap::new();
        args.insert("key".to_string(), ArgValue::Int(0x42));

        let input = b"Hello";
        let encrypted = op.run(input, &args).unwrap();
        // XOR is its own inverse
        let decrypted = op.run(&encrypted, &args).unwrap();
        assert_eq!(decrypted, b"Hello");
    }

    #[test]
    fn test_xor_multi_byte_key() {
        let op = Xor;
        let mut args = HashMap::new();
        args.insert("key".to_string(), ArgValue::String("secret".to_string()));

        let input = b"Hello World!";
        let encrypted = op.run(input, &args).unwrap();
        assert_ne!(&encrypted, input);
        let decrypted = op.run(&encrypted, &args).unwrap();
        assert_eq!(decrypted, b"Hello World!");
    }

    #[test]
    fn test_xor_null_preserving() {
        let op = Xor;
        let mut args = HashMap::new();
        args.insert("key".to_string(), ArgValue::Int(0xFF));
        args.insert("null_preserving".to_string(), ArgValue::Bool(true));

        let input = &[0x00, 0x42, 0xFF, 0x01];
        let result = op.run(input, &args).unwrap();
        // 0x00 preserved, 0x42 ^ 0xFF = 0xBD, 0xFF preserved (== key), 0x01 ^ 0xFF = 0xFE
        assert_eq!(result, vec![0x00, 0xBD, 0xFF, 0xFE]);
    }

    #[test]
    fn test_xor_missing_key() {
        let op = Xor;
        let result = op.run(b"test", &HashMap::new());
        assert!(result.is_err());
    }
}
