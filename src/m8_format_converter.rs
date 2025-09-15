// M8 Format Converter - "No more format confusion!" 🔄
// Convert between .m8 (binary wave), .m8j (JSON), and .m8z (compressed)
// "One format to rule them all? Nah, let's have options!" - Hue

use anyhow::{Result, Context};
use std::path::Path;
use std::fs;
use std::io::{Read, Write};

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
        let ext = path.extension()
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
        target_format: Option<M8Format>
    ) -> Result<()> {
        // Read input file
        let data = fs::read(input_path)?;

        // Detect source format
        let source_format = M8Format::detect_from_content(&data);

        // Determine target format
        let target = target_format.unwrap_or_else(|| {
            M8Format::from_extension(output_path).unwrap_or(M8Format::Binary)
        });

        println!("🔄 Converting {:?} -> {:?}", source_format, target);

        // Perform conversion
        match (source_format, target) {
            // Same format - just copy
            (a, b) if a == b => {
                fs::copy(input_path, output_path)?;
            },

            // JSON to Binary
            (M8Format::Json, M8Format::Binary) => {
                Self::json_to_binary(&data, output_path)?;
            },

            // Binary to JSON
            (M8Format::Binary, M8Format::Json) => {
                Self::binary_to_json(&data, output_path)?;
            },

            // Compressed to anything - decompress first
            (M8Format::Compressed, target) => {
                let decompressed = Self::decompress(&data)?;
                let temp_format = M8Format::detect_from_content(&decompressed);

                // Recursive call with decompressed data
                let temp_path = format!("/tmp/temp.{}", temp_format.extension());
                fs::write(&temp_path, decompressed)?;
                Self::convert(Path::new(&temp_path), output_path, Some(target))?;
                fs::remove_file(&temp_path)?;
            },

            // Anything to Compressed
            (_, M8Format::Compressed) => {
                Self::compress(&data, output_path)?;
            },

            // Marqant conversions
            (M8Format::Marqant, M8Format::Json) => {
                Self::marqant_to_json(&data, output_path)?;
            },
            (M8Format::Json, M8Format::Marqant) => {
                Self::json_to_marqant(&data, output_path)?;
            },

            _ => {
                anyhow::bail!("Conversion from {:?} to {:?} not yet implemented",
                    source_format, target);
            }
        }

        println!("✅ Conversion complete: {}", output_path.display());
        Ok(())
    }

    /// Convert JSON to binary wave format
    fn json_to_binary(json_data: &[u8], output_path: &Path) -> Result<()> {
        use crate::mem8_binary::M8BinaryFile;

        let json_str = String::from_utf8_lossy(json_data);
        let value: serde_json::Value = serde_json::from_str(&json_str)?;

        let mut m8_file = M8BinaryFile::create(output_path)?;

        // Handle different JSON structures
        if let Some(contexts) = value.get("contexts").and_then(|c| c.as_array()) {
            for context in contexts {
                let content = serde_json::to_vec(context)?;
                let importance = context.get("score")
                    .and_then(|s| s.as_f64())
                    .unwrap_or(0.5) as f32;
                m8_file.append_block(&content, importance)?;
            }
        } else if let Some(array) = value.as_array() {
            for item in array {
                let content = serde_json::to_vec(item)?;
                m8_file.append_block(&content, 0.5)?;
            }
        } else {
            // Single object
            let content = serde_json::to_vec(&value)?;
            m8_file.append_block(&content, 1.0)?;
        }

        Ok(())
    }

    /// Convert binary to JSON
    fn binary_to_json(binary_data: &[u8], output_path: &Path) -> Result<()> {
        use crate::mem8_binary::M8BinaryFile;

        // Create temporary file from data
        let temp_path = "/tmp/temp_convert.m8";
        fs::write(temp_path, binary_data)?;

        let mut m8_file = M8BinaryFile::open(temp_path)?;
        let mut contexts = Vec::new();

        // Read all blocks
        while let Some(block) = m8_file.read_backwards()? {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&block.content) {
                contexts.push(json);
            }
        }

        // Create JSON structure
        let output = serde_json::json!({
            "format": "m8j",
            "version": 1,
            "contexts": contexts
        });

        fs::write(output_path, serde_json::to_string_pretty(&output)?)?;
        fs::remove_file(temp_path)?;

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
        target_format: M8Format
    ) -> Result<()> {
        fs::create_dir_all(output_dir)?;

        for entry in fs::read_dir(input_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Ok(_format) = M8Format::from_extension(&path) {
                    let file_name = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");

                    let output_path = output_dir.join(format!("{}.{}",
                        file_name, target_format.extension()));

                    println!("Converting: {} -> {}",
                        path.display(), output_path.display());

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
                    println!("  Renaming: {} -> {}",
                        file_path.display(), new_path.display());
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
    use tempfile::tempdir;

    #[test]
    fn test_format_detection() {
        assert_eq!(M8Format::detect_from_content(b"MEM8"), M8Format::Binary);
        assert_eq!(M8Format::detect_from_content(b"{\"test\":1}"), M8Format::Json);
        assert_eq!(M8Format::detect_from_content(b"\x78\x9c"), M8Format::Compressed);
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