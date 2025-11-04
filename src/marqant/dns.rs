use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Resolve a dictionary ID from a DNS TXT record.
/// This implementation shells out to `dig`.
/// The expected format is a TXT record containing space-separated pairs
/// of `base64(key)=base64(value)`.
pub fn resolve_dns_dict(id: &str) -> Result<Option<HashMap<String, String>>> {
    // For testing, allow overriding the command
    let dig_cmd = std::env::var("MQ_DIG_CMD").unwrap_or_else(|_| "dig".to_string());
    let domain = format!("_mq.{}.mq.mem8.org", id);
    let output = std::process::Command::new(dig_cmd)
        .args(["+short", "TXT", &domain])
        .output()
        .map_err(|e| anyhow!("Failed to execute 'dig': {}. Is it in your PATH?", e))?;

    if !output.status.success() {
        // dig returns non-zero for NXDOMAIN etc.
        return Ok(None);
    }

    let txt_records = String::from_utf8(output.stdout)?;
    // Take the first line of TXT records if multiple are returned
    let Some(record) = txt_records.lines().next() else {
        return Ok(None);
    };

    // The record is wrapped in quotes, remove them.
    let record = record.trim().trim_matches('"');
    if record.is_empty() {
        return Ok(None);
    }

    let mut dict = HashMap::new();
    for pair in record.split_whitespace() {
        let Some((k_b64, v_b64)) = pair.split_once('=') else {
            return Err(anyhow!("Invalid DNS dict pair: {}", pair));
        };
        let key_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, k_b64)?;
        let val_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, v_b64)?;
        let key = String::from_utf8(key_bytes)?;
        let val = String::from_utf8(val_bytes)?;
        dict.insert(key, val);
    }

    if dict.is_empty() {
        Ok(None)
    } else {
        Ok(Some(dict))
    }
}
