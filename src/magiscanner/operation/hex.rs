use super::{ArgValue, Category, Operation, OperationError, OperationMeta};
use std::collections::HashMap;

/// Decode hex-encoded data.
pub struct FromHex;

impl Operation for FromHex {
    fn meta(&self) -> OperationMeta {
        OperationMeta {
            name: "from_hex",
            description: "Decode hex-encoded data, stripping whitespace and common delimiters",
            category: Category::DataFormat,
            args: &[],
        }
    }

    fn run(
        &self,
        input: &[u8],
        _args: &HashMap<String, ArgValue>,
    ) -> Result<Vec<u8>, OperationError> {
        // Strip whitespace, colons, dashes, and 0x prefixes
        let input_str = std::str::from_utf8(input)
            .map_err(|e| OperationError::Failed(format!("input is not valid UTF-8: {e}")))?;

        let cleaned: String = input_str
            .replace("0x", "")
            .replace("0X", "")
            .chars()
            .filter(|c| c.is_ascii_hexdigit())
            .collect();

        ::hex::decode(&cleaned)
            .map_err(|e| OperationError::Failed(format!("hex decode error: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_hex() {
        let op = FromHex;
        let input = b"48656c6c6f";
        let result = op.run(input, &HashMap::new()).unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_decode_hex_with_spaces() {
        let op = FromHex;
        let input = b"48 65 6c 6c 6f";
        let result = op.run(input, &HashMap::new()).unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_decode_hex_with_colons() {
        let op = FromHex;
        let input = b"48:65:6c:6c:6f";
        let result = op.run(input, &HashMap::new()).unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_decode_hex_with_0x_prefix() {
        let op = FromHex;
        let input = b"0x48656c6c6f";
        let result = op.run(input, &HashMap::new()).unwrap();
        assert_eq!(result, b"Hello");
    }
}
