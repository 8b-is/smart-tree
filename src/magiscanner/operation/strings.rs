use super::{ArgValue, Category, Operation, OperationError, OperationMeta};
use std::collections::HashMap;

/// Extract printable ASCII strings from binary data.
pub struct ExtractStrings;

impl Operation for ExtractStrings {
    fn meta(&self) -> OperationMeta {
        OperationMeta {
            name: "extract_strings",
            description: "Extract runs of printable ASCII characters from data",
            category: Category::Extractors,
            args: &[],
        }
    }

    fn run(
        &self,
        input: &[u8],
        args: &HashMap<String, ArgValue>,
    ) -> Result<Vec<u8>, OperationError> {
        let min_length = args.get("min_length").and_then(|v| v.as_int()).unwrap_or(4) as usize;

        let mut strings = Vec::new();
        let mut current = Vec::new();

        for &byte in input {
            if byte.is_ascii_graphic() || byte == b' ' {
                current.push(byte);
            } else {
                if current.len() >= min_length {
                    strings.push(current.clone());
                }
                current.clear();
            }
        }
        // Don't forget trailing string
        if current.len() >= min_length {
            strings.push(current);
        }

        let output = strings
            .into_iter()
            .map(|s| String::from_utf8_lossy(&s).to_string())
            .collect::<Vec<_>>()
            .join("\n");

        Ok(output.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_from_binary() {
        let op = ExtractStrings;
        let mut input = Vec::new();
        input.extend_from_slice(b"Hello");
        input.extend_from_slice(&[0x00, 0x01, 0x02]);
        input.extend_from_slice(b"World!");
        input.extend_from_slice(&[0x00]);
        input.extend_from_slice(b"ab"); // too short (< 4)

        let result = op.run(&input, &HashMap::new()).unwrap();
        let output = String::from_utf8(result).unwrap();
        assert_eq!(output, "Hello\nWorld!");
    }

    #[test]
    fn test_custom_min_length() {
        let op = ExtractStrings;
        let mut args = HashMap::new();
        args.insert("min_length".to_string(), ArgValue::Int(2));

        let mut input = Vec::new();
        input.extend_from_slice(b"Hi");
        input.push(0x00);
        input.extend_from_slice(b"X"); // too short even for min=2
        input.push(0x00);
        input.extend_from_slice(b"OK");

        let result = op.run(&input, &args).unwrap();
        let output = String::from_utf8(result).unwrap();
        assert_eq!(output, "Hi\nOK");
    }
}
