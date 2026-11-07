// M8 Format Converter - "No more format confusion!" 🔄
// Convert between .m8 (binary wave), .m8j (JSON), and .m8z (compressed)
// "One format to rule them all? Nah, let's have options!" - Hue

use anyhow::{Context, Result};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

/// Supported M8 formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum M8Format {
    /// Binary wave-based format (the REAL .m8)
    Binary,

    /// JSON context format (.m8j)
    Json,

    /// Compressed format (.m8z)
    Compressed,

    /// Marqant quantum-compressed (.mq)
    Marqant,
}

impl M8Format {
    /// Detect format from file extension
    pub fn from_extension(path: &Path) -> Result<Self> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .context("No file extension")?;

        match ext {
            "m8" => Ok(M8Format::Binary),
            "m8j" => Ok(M8Format::Json),
            "m8z" => Ok(M8Format::Compressed),
            "mq" => Ok(M8Format::Marqant),
            _ => anyhow::bail!("Unknown M8 format: .{}", ext),
        }
    }

    /// Detect format from file content
    pub fn detect_from_content(data: &[u8]) -> Self {
        // Check magic bytes
        if data.starts_with(b"MEM8") {
            M8Format::Binary
        } else if data.starts_with(b"{") || data.starts_with(b"[") {
            M8Format::Json
        } else if data.starts_with(b"\x78\x9c") || data.starts_with(b"\x78\xda") {
            // zlib compressed
            M8Format::Compressed
        } else if data.starts_with(b"MARQANT") {
            M8Format::Marqant
        } else {
            // Default to JSON
            M8Format::Json
        }
    }

    /// Get recommended extension
    pub fn extension(&self) -> &str {
        match self {
            M8Format::Binary => "m8",
            M8Format::Json => "m8j",
            M8Format::Compressed => "m8z",
            M8Format::Marqant => "mq",
        }
    }
}

/// Convert between M8 formats
pub struct M8Converter;

impl M8Converter {
    /// Convert any M8 format to another
    pub fn convert(
        input_path: &Path,
        output_path: &Path,
        target_format: Option<M8Format>,
    ) -> Result<()> {
        // Read input file
        let data = fs::read(input_path)?;

        // Detect source format
        let source_format = M8Format::detect_from_content(&data);

        // Determine target format
        let target = target_format
            .unwrap_or_else(|| M8Format::from_extension(output_path).unwrap_or(M8Format::Binary));

        println!("🔄 Converting {:?} -> {:?}", source_format, target);

        // Perform conversion
        match (source_format, target) {
            // Same format - just copy
            (a, b) if a == b => {
                fs::copy(input_path, output_path)?;
            }

            // JSON to Binary
            (M8Format::Json, M8Format::Binary) => {
                Self::json_to_binary(&data, output_path)?;
            }

            // Binary to JSON
            (M8Format::Binary, M8Format::Json) => {
                Self::binary_to_json(&data, output_path)?;
            }

            // Compressed to anything - decompress first
            (M8Format::Compressed, target) => {
                let decompressed = Self::decompress(&data)?;
                let temp_format = M8Format::detect_from_content(&decompressed);

                // Recursive call with decompressed data
                let temp_path = format!("/tmp/temp.{}", temp_format.extension());
                fs::write(&temp_path, decompressed)?;
                Self::convert(Path::new(&temp_path), output_path, Some(target))?;
                fs::remove_file(&temp_path)?;
            }

            // Anything to Compressed
            (_, M8Format::Compressed) => {
                Self::compress(&data, output_path)?;
            }

            // Marqant conversions
            (M8Format::Marqant, M8Format::Json) => {
                Self::marqant_to_json(&data, output_path)?;
            }
            (M8Format::Json, M8Format::Marqant) => {
                Self::json_to_marqant(&data, output_path)?;
            }
            (M8Format::Binary, M8Format::Marqant) => {
                if let Some(mq) = crate::m8_consciousness::extract_m8_marqant_section(&data, false)
                {
                    fs::write(output_path, mq)?;
                } else {
                    let decoded = crate::m8_consciousness::decode_m8_bytes(&data)?;
                    Self::json_to_marqant(&decoded, output_path)?;
                }
            }
            (M8Format::Marqant, M8Format::Binary) => {
                let decoded = crate::m8_consciousness::decode_m8_bytes(&data)?;
                let tight = crate::m8_consciousness::encode_json_bytes_as_tight_m8(
                    &decoded,
                    42.73,
                    &String::from_utf8_lossy(&decoded),
                )?;
                fs::write(output_path, tight)?;
            }

            _ => {
                anyhow::bail!(
                    "Conversion from {:?} to {:?} not yet implemented",
                    source_format,
                    target
                );
            }
        }

        println!("✅ Conversion complete: {}", output_path.display());
        Ok(())
    }

    /// Convert JSON to tight MEM8 binary wave format
    fn json_to_binary(json_data: &[u8], output_path: &Path) -> Result<()> {
        let json_str = String::from_utf8_lossy(json_data);
        let value: serde_json::Value = serde_json::from_str(&json_str)?;
        let freq = value
            .get("frequency")
            .and_then(|f| f.as_f64())
            .unwrap_or(42.73);
        let essence = value
            .get("essence")
            .and_then(|e| e.as_str())
            .unwrap_or("Converted M8 consciousness");
        let compact_bytes = serde_json::to_vec(&value)?;
        let tight = crate::m8_consciousness::encode_json_bytes_as_tight_m8(
            &compact_bytes,
            freq,
            &format!("# {:.2}Hz — {}", freq, essence),
        )?;
        fs::write(output_path, tight)?;
        Ok(())
    }

    /// Convert binary MEM8 to JSON (supports both tight MEM8 section containers and legacy 4KB block files)
    fn binary_to_json(binary_data: &[u8], output_path: &Path) -> Result<()> {
        let decoded = crate::m8_consciousness::decode_m8_bytes(binary_data)?;
        if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&decoded) {
            fs::write(output_path, serde_json::to_string_pretty(&value)?)?;
        } else {
            let output = serde_json::json!({
                "format": "m8j",
                "version": 1,
                "content": String::from_utf8_lossy(&decoded)
            });
            fs::write(output_path, serde_json::to_string_pretty(&output)?)?;
        }
        Ok(())
    }

    /// Compress data with zlib
    fn compress(data: &[u8], output_path: &Path) -> Result<()> {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;

        let file = fs::File::create(output_path)?;
        let mut encoder = ZlibEncoder::new(file, Compression::default());
        encoder.write_all(data)?;
        encoder.finish()?;

        Ok(())
    }

    /// Decompress zlib data
    fn decompress(data: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::ZlibDecoder;

        let mut decoder = ZlibDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;

        Ok(decompressed)
    }

    /// Convert Marqant to JSON
    fn marqant_to_json(mq_data: &[u8], output_path: &Path) -> Result<()> {
        use crate::formatters::marqant::MarqantFormatter;

        let mq_str = String::from_utf8_lossy(mq_data);
        let markdown = MarqantFormatter::decompress_marqant(&mq_str)?;

        let json = serde_json::json!({
            "format": "markdown",
            "content": markdown
        });

        fs::write(output_path, serde_json::to_string_pretty(&json)?)?;
        Ok(())
    }

    /// Convert JSON to Marqant
    fn json_to_marqant(json_data: &[u8], output_path: &Path) -> Result<()> {
        use crate::formatters::marqant::MarqantFormatter;

        let json_str = String::from_utf8_lossy(json_data);
        let value: serde_json::Value = serde_json::from_str(&json_str)?;

        let markdown = if let Some(content) = value.get("content").and_then(|c| c.as_str()) {
            content.to_string()
        } else {
            // Convert JSON to markdown representation
            format!("```json\n{}\n```", serde_json::to_string_pretty(&value)?)
        };

        let compressed = MarqantFormatter::compress_markdown(&markdown)?;
        fs::write(output_path, compressed)?;

        Ok(())
    }

    /// Batch convert all files in a directory
    pub fn convert_directory(
        input_dir: &Path,
        output_dir: &Path,
        target_format: M8Format,
    ) -> Result<()> {
        fs::create_dir_all(output_dir)?;

        for entry in fs::read_dir(input_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Ok(_format) = M8Format::from_extension(&path) {
                    let file_name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");

                    let output_path =
                        output_dir.join(format!("{}.{}", file_name, target_format.extension()));

                    println!(
                        "Converting: {} -> {}",
                        path.display(),
                        output_path.display()
                    );

                    Self::convert(&path, &output_path, Some(target_format))?;
                }
            }
        }

        Ok(())
    }
}

/// Fix all misnamed .m8 files in the system
pub fn fix_m8_extensions() -> Result<()> {
    println!("🔧 Fixing .m8 file extensions...");

    let dirs = [
        "~/.mem8",
        "~/.mem8/projects",
        "~/.mem8/users",
        "~/.mem8/llms",
    ];

    for dir in &dirs {
        let expanded = shellexpand::tilde(dir);
        let path = Path::new(expanded.as_ref());

        if !path.exists() {
            continue;
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.extension().and_then(|e| e.to_str()) == Some("m8") {
                // Read first few bytes to detect format
                let mut file = fs::File::open(&file_path)?;
                let mut buffer = [0u8; 16];
                file.read_exact(&mut buffer)?;

                let detected = M8Format::detect_from_content(&buffer);

                if detected != M8Format::Binary {
                    // Rename file with correct extension
                    let new_path = file_path.with_extension(detected.extension());
                    println!(
                        "  Renaming: {} -> {}",
                        file_path.display(),
                        new_path.display()
                    );
                    fs::rename(&file_path, &new_path)?;
                }
            }
        }
    }

    println!("✅ Extension fix complete!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(M8Format::detect_from_content(b"MEM8"), M8Format::Binary);
        assert_eq!(
            M8Format::detect_from_content(b"{\"test\":1}"),
            M8Format::Json
        );
        assert_eq!(
            M8Format::detect_from_content(b"\x78\x9c"),
            M8Format::Compressed
        );
        assert_eq!(M8Format::detect_from_content(b"MARQANT"), M8Format::Marqant);
    }

    #[test]
    fn test_extension_mapping() {
        let path = Path::new("test.m8");
        assert_eq!(M8Format::from_extension(path).unwrap(), M8Format::Binary);

        let path = Path::new("test.m8j");
        assert_eq!(M8Format::from_extension(path).unwrap(), M8Format::Json);
    }
}
