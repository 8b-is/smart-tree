use super::{ArgValue, Category, Operation, OperationError, OperationMeta};
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

static URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)(?:https?|ftp)://[^\s<>"'\)\]\},;]+|(?:www\.)[a-zA-Z0-9][a-zA-Z0-9\-]*(?:\.[a-zA-Z]{2,})+[^\s<>"'\)\]\},;]*"#
    ).unwrap()
});

/// Extract URLs from text data.
pub struct ExtractUrls;

impl Operation for ExtractUrls {
    fn meta(&self) -> OperationMeta {
        OperationMeta {
            name: "extract_urls",
            description: "Extract URLs (http, https, ftp, www) from text data",
            category: Category::Extractors,
            args: &[],
        }
    }

    fn run(
        &self,
        input: &[u8],
        args: &HashMap<String, ArgValue>,
    ) -> Result<Vec<u8>, OperationError> {
        let unique = args.get("unique").and_then(|v| v.as_bool()).unwrap_or(true);

        let text = String::from_utf8_lossy(input);

        let mut urls: Vec<String> = URL_REGEX
            .find_iter(&text)
            .map(|m| m.as_str().to_string())
            .collect();

        if unique {
            let mut seen = std::collections::HashSet::new();
            urls.retain(|url| seen.insert(url.clone()));
        }

        let output = urls.join("\n");
        Ok(output.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_http_urls() {
        let op = ExtractUrls;
        let input = b"Visit https://example.com and http://test.org/path?q=1 for more info";
        let result = op.run(input, &HashMap::new()).unwrap();
        let output = String::from_utf8(result).unwrap();
        let urls: Vec<&str> = output.lines().collect();
        assert_eq!(urls.len(), 2);
        assert!(urls[0].contains("example.com"));
        assert!(urls[1].contains("test.org"));
    }

    #[test]
    fn test_extract_deduplicates() {
        let op = ExtractUrls;
        let input = b"https://evil.com https://evil.com https://good.com";
        let result = op.run(input, &HashMap::new()).unwrap();
        let output = String::from_utf8(result).unwrap();
        let urls: Vec<&str> = output.lines().collect();
        assert_eq!(urls.len(), 2);
    }

    #[test]
    fn test_no_urls() {
        let op = ExtractUrls;
        let input = b"no urls here, just plain text";
        let result = op.run(input, &HashMap::new()).unwrap();
        assert!(result.is_empty());
    }
}
