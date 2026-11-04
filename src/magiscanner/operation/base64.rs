use super::{ArgValue, Category, Operation, OperationError, OperationMeta};
use std::collections::HashMap;

use ::base64::prelude::*;

/// Decode base64-encoded data.
pub struct FromBase64;

// Static arg definitions need to live in a const context.
// We'll return them from meta() directly since static ArgDef with ArgValue isn't const-compatible.
impl Operation for FromBase64 {
    fn meta(&self) -> OperationMeta {
        OperationMeta {
            name: "from_base64",
            description: "Decode base64-encoded data (standard or URL-safe alphabet)",
            category: Category::DataFormat,
            args: &[],
        }
    }

    fn run(
        &self,
        input: &[u8],
        args: &HashMap<String, ArgValue>,
    ) -> Result<Vec<u8>, OperationError> {
        let alphabet = args
            .get("alphabet")
            .and_then(|v| v.as_str())
            .unwrap_or("standard");

        // Strip whitespace before decoding
        let cleaned: Vec<u8> = input
            .iter()
            .copied()
            .filter(|b| !b.is_ascii_whitespace())
            .collect();

        let decoded = match alphabet {
            "url_safe" => BASE64_URL_SAFE_NO_PAD
                .decode(&cleaned)
                .or_else(|_| BASE64_URL_SAFE.decode(&cleaned)),
            _ => BASE64_STANDARD_NO_PAD
                .decode(&cleaned)
                .or_else(|_| BASE64_STANDARD.decode(&cleaned)),
        };

        decoded.map_err(|e| OperationError::Failed(format!("base64 decode error: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_standard() {
        let op = FromBase64;
        let input = b"SGVsbG8gV29ybGQ=";
        let result = op.run(input, &HashMap::new()).unwrap();
        assert_eq!(result, b"Hello World");
    }

    #[test]
    fn test_decode_no_padding() {
        let op = FromBase64;
        let input = b"SGVsbG8";
        let result = op.run(input, &HashMap::new()).unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_decode_url_safe() {
        let op = FromBase64;
        // URL-safe base64 for bytes that would use + and /
        let mut args = HashMap::new();
        args.insert(
            "alphabet".to_string(),
            ArgValue::String("url_safe".to_string()),
        );
        let input = b"PDw_Pz4-"; // <<??>> in url-safe base64
        let result = op.run(input, &args).unwrap();
        assert_eq!(result, b"<<??>>");
    }

    #[test]
    fn test_decode_with_whitespace() {
        let op = FromBase64;
        let input = b"SGVs bG8g\nV29y bGQ=";
        let result = op.run(input, &HashMap::new()).unwrap();
        assert_eq!(result, b"Hello World");
    }
}
