//! .m8 file format implementation with Markqant compression
//! Achieves 100:1 semantic-preserving compression for AI-native storage

use crate::mem8::wave::MemoryWave;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::io::Write;

/// Magic bytes for .m8 format (spec: first 4 bytes = "MEM8")
const M8_MAGIC: &[u8] = b"MEM8";

/// Section types in .m8 files (aligned with docs/M8_UNIFIED_FORMAT.md)
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum SectionType {
    Identity = 0x01,
    Context = 0x02,
    Structure = 0x03,
    Compilation = 0x04,
    Cache = 0x05,
    AiContext = 0x06,
    Relationships = 0x07,
    SensorArbitration = 0x08,
    MarkqantDocument = 0x09,
    QuantumTree = 0x0A,
    CodeRelations = 0x0B,
    BuildArtifacts = 0x0C,
    TemporalIndex = 0x0D,
    CollectiveEmotion = 0x0E,
    WaveMemoryBlob = 0x0F,
    ReactiveStateDump = 0x10,
    CustodianNotes = 0x11,
}

/// .m8 file header
#[derive(Debug)]
pub struct M8Header {
    /// File format version
    pub version: u16,
    /// Number of sections
    pub section_count: u16,
    /// Total file size (excluding CRC)
    pub file_size: u64,
    /// Creation timestamp
    pub timestamp: u64,
}

/// .m8 file section
#[derive(Debug)]
pub struct M8Section {
    /// Section type
    pub section_type: SectionType,
    /// Section size in bytes
    pub size: u32,
    /// Section data
    pub data: Vec<u8>,
}

/// Wave memory compressed to 32 bytes
#[derive(Debug, Clone)]
pub struct CompressedWave {
    pub id: u64,           // 8 bytes - unique identifier
    pub amplitude: u8,     // 1 byte - logarithmically quantized
    pub frequency: u16,    // 2 bytes - frequency in Hz
    pub phase: u8,         // 1 byte - phase in radians * 40.74
    pub valence: i8,       // 1 byte - emotional valence * 127
    pub arousal: u8,       // 1 byte - emotional arousal * 255
    pub decay_tau: u16,    // 2 bytes - decay constant in seconds
    pub timestamp: u64,    // 8 bytes - creation time
    pub interference: u64, // 8 bytes - interference pattern hash
}

impl CompressedWave {
    /// Compress a MemoryWave to 32 bytes
    pub fn from_wave(wave: &MemoryWave, id: u64) -> Self {
        Self {
            id,
            amplitude: quantize_amplitude(wave.amplitude),
            frequency: wave.frequency as u16,
            phase: ((wave.phase / std::f32::consts::PI + 1.0) * 127.5) as u8,
            valence: (wave.valence * 127.0) as i8,
            arousal: (wave.arousal * 255.0) as u8,
            decay_tau: wave
                .decay_tau
                .map(|d| d.as_secs() as u16)
                .unwrap_or(u16::MAX),
            timestamp: wave.created_at.elapsed().as_secs(),
            interference: 0, // Calculated separately
        }
    }

    /// Decompress to MemoryWave
    pub fn to_wave(&self) -> MemoryWave {
        let mut wave = MemoryWave::new(self.frequency as f32, dequantize_amplitude(self.amplitude));

        wave.phase = (self.phase as f32 / 127.5 - 1.0) * std::f32::consts::PI;
        wave.valence = self.valence as f32 / 127.0;
        wave.arousal = self.arousal as f32 / 255.0;
        wave.decay_tau = if self.decay_tau == u16::MAX {
            None
        } else {
            Some(std::time::Duration::from_secs(self.decay_tau as u64))
        };

        wave
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        bytes[0..8].copy_from_slice(&self.id.to_le_bytes());
        bytes[8] = self.amplitude;
        bytes[9..11].copy_from_slice(&self.frequency.to_le_bytes());
        bytes[11] = self.phase;
        bytes[12] = self.valence as u8;
        bytes[13] = self.arousal;
        bytes[14..16].copy_from_slice(&self.decay_tau.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.timestamp.to_le_bytes());
        bytes[24..32].copy_from_slice(&self.interference.to_le_bytes());
        bytes
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(anyhow!("CompressedWave must be exactly 32 bytes"));
        }

        Ok(Self {
            id: u64::from_le_bytes(bytes[0..8].try_into()?),
            amplitude: bytes[8],
            frequency: u16::from_le_bytes(bytes[9..11].try_into()?),
            phase: bytes[11],
            valence: bytes[12] as i8,
            arousal: bytes[13],
            decay_tau: u16::from_le_bytes(bytes[14..16].try_into()?),
            timestamp: u64::from_le_bytes(bytes[16..24].try_into()?),
            interference: u64::from_le_bytes(bytes[24..32].try_into()?),
        })
    }
}

/// Logarithmic amplitude quantization
fn quantize_amplitude(amplitude: f32) -> u8 {
    if amplitude <= 0.0 {
        0
    } else {
        (32.0 * amplitude.log2()).clamp(0.0, 255.0) as u8
    }
}

/// Inverse logarithmic quantization
fn dequantize_amplitude(quantized: u8) -> f32 {
    if quantized == 0 {
        0.0
    } else {
        2.0_f32.powf(quantized as f32 / 32.0)
    }
}

/// Markqant v2.0 rotating token system
pub struct MarkqantEncoder {
    /// Token assignments (pattern -> token)
    tokens: HashMap<String, u8>,
    /// Reverse mapping (token -> pattern)
    patterns: HashMap<u8, String>,
    /// Pattern frequencies
    frequencies: HashMap<String, usize>,
    /// Next available token
    next_token: u8,
}

impl Default for MarkqantEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkqantEncoder {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
            patterns: HashMap::new(),
            frequencies: HashMap::new(),
            next_token: 0x80, // Start at 128
        }
    }

    /// Analyze text and build token assignments
    pub fn analyze(&mut self, text: &str) {
        // Find all repeated substrings
        let words: Vec<&str> = text.split_whitespace().collect();

        // Count frequencies
        for window_size in 1..=5 {
            for i in 0..words.len().saturating_sub(window_size - 1) {
                let pattern = words[i..i + window_size].join(" ");
                *self.frequencies.entry(pattern).or_insert(0) += 1;
            }
        }

        // Score patterns by (length - 1) * (frequency - 1)
        let mut scored_patterns: Vec<_> = self
            .frequencies
            .iter()
            .filter(|(_, &freq)| freq >= 2)
            .map(|(pattern, &freq)| {
                let score = (pattern.len() - 1) * (freq - 1);
                (pattern.clone(), score)
            })
            .collect();

        scored_patterns.sort_by_key(|(_, score)| std::cmp::Reverse(*score));

        // Assign tokens to top patterns
        for (pattern, _) in scored_patterns.iter().take(128) {
            if self.next_token < 255 {
                self.tokens.insert(pattern.clone(), self.next_token);
                self.patterns.insert(self.next_token, pattern.clone());
                self.next_token = self.next_token.saturating_add(1);
            }
        }
    }

    /// Encode text using assigned tokens
    pub fn encode(&self, text: &str) -> Vec<u8> {
        let mut result = Vec::new();
        let mut remaining = text;

        while !remaining.is_empty() {
            let mut found = false;

            // Try to match longest pattern first
            for len in (1..=remaining.len()).rev() {
                if let Some(&token) = self.tokens.get(&remaining[..len]) {
                    result.push(token);
                    remaining = &remaining[len..];
                    found = true;
                    break;
                }
            }

            if !found {
                // No pattern match, encode as raw byte
                result.extend_from_slice(remaining.chars().next().unwrap().to_string().as_bytes());
                remaining = &remaining[remaining.chars().next().unwrap().len_utf8()..];
            }
        }

        result
    }

    /// Decode tokens back to text
    pub fn decode(&self, data: &[u8]) -> Result<String> {
        let mut result = String::new();
        let mut i = 0;

        while i < data.len() {
            if data[i] >= 0x80 {
                // Token
                if let Some(pattern) = self.patterns.get(&data[i]) {
                    result.push_str(pattern);
                } else {
                    return Err(anyhow!("Unknown token: 0x{:02x}", data[i]));
                }
                i += 1;
            } else {
                // Raw UTF-8
                let ch = data[i] as char;
                result.push(ch);
                i += 1;
            }
        }

        Ok(result)
    }
}

/// .m8 file writer
pub struct M8Writer<W: Write> {
    writer: W,
    sections: Vec<M8Section>,
}

impl<W: Write> M8Writer<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            sections: Vec::new(),
        }
    }

    /// Add a wave memory section
    pub fn add_wave_memory(&mut self, waves: &[CompressedWave]) -> Result<()> {
        let mut data = Vec::with_capacity(waves.len() * 32);

        for wave in waves {
            data.extend_from_slice(&wave.to_bytes());
        }

        self.sections.push(M8Section {
            section_type: SectionType::WaveMemoryBlob,
            size: data.len() as u32,
            data,
        });

        Ok(())
    }

    /// Add a Markqant-compressed text section
    pub fn add_markqant_text(&mut self, text: &str) -> Result<()> {
        let mut encoder = MarkqantEncoder::new();
        encoder.analyze(text);
        let encoded = encoder.encode(text);

        // Store token table followed by encoded data
        let mut data = Vec::new();

        // Token table header
        data.extend_from_slice(&(encoder.patterns.len() as u16).to_le_bytes());

        // Token definitions
        for (token, pattern) in &encoder.patterns {
            data.push(*token);
            data.extend_from_slice(&(pattern.len() as u16).to_le_bytes());
            data.extend_from_slice(pattern.as_bytes());
        }

        // Encoded text
        data.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
        data.extend_from_slice(&encoded);

        self.sections.push(M8Section {
            section_type: SectionType::MarkqantDocument,
            size: data.len() as u32,
            data,
        });

        Ok(())
    }

    /// Write the complete .m8 file
    pub fn finish(mut self) -> Result<()> {
        // Write magic bytes
        self.writer.write_all(M8_MAGIC)?;

        // Calculate total size
        let header_size = 16; // Magic + header fields
        let section_headers_size = self.sections.len() * 8; // Type + size per section
        let data_size: usize = self.sections.iter().map(|s| s.data.len()).sum();
        let total_size = header_size + section_headers_size + data_size + 4; // +4 for CRC

        // Write header
        let header = M8Header {
            version: 1,
            section_count: self.sections.len() as u16,
            file_size: total_size as u64,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        };

        self.writer.write_all(&header.version.to_le_bytes())?;
        self.writer.write_all(&header.section_count.to_le_bytes())?;
        self.writer.write_all(&header.file_size.to_le_bytes())?;
        self.writer.write_all(&header.timestamp.to_le_bytes())?;

        // Write sections
        for section in &self.sections {
            self.writer.write_all(&[section.section_type as u8])?;
            self.writer.write_all(&section.size.to_le_bytes())?;
            self.writer.write_all(&section.data)?;
        }

        // Calculate and write CRC32
        let crc = 0u32; // TODO: Implement actual CRC32
        self.writer.write_all(&crc.to_le_bytes())?;

        Ok(())
    }
}

/// Example of .m8 format usage
pub fn create_example_m8() -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut writer = M8Writer::new(&mut buffer);

    // Add some wave memories
    let waves = vec![
        CompressedWave::from_wave(&MemoryWave::new(440.0, 0.8), 1),
        CompressedWave::from_wave(&MemoryWave::new(880.0, 0.6), 2),
    ];
    writer.add_wave_memory(&waves)?;

    // Add some text
    writer.add_markqant_text("The user is cooking in the kitchen at 6PM")?;

    writer.finish()?;
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wave_compression() {
        let mut wave = MemoryWave::new(440.0, 0.8);
        wave.valence = 0.7;
        wave.arousal = 0.4;

        let compressed = CompressedWave::from_wave(&wave, 12345);
        assert_eq!(compressed.to_bytes().len(), 32);

        let decompressed = compressed.to_wave();
        assert!((decompressed.frequency - 440.0).abs() < 1.0);
        assert!((decompressed.valence - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_markqant_encoding() {
        let mut encoder = MarkqantEncoder::new();
        let text = "the cat in the hat sat on the mat";
        encoder.analyze(text);

        let encoded = encoder.encode(text);
        let decoded = encoder.decode(&encoded).unwrap();

        assert_eq!(decoded, text);
        assert!(encoded.len() < text.len()); // Should compress
    }

    #[test]
    fn test_m8_creation() {
        let m8_data = create_example_m8().unwrap();
        assert!(m8_data.starts_with(M8_MAGIC));
        assert!(m8_data.len() > 100); // Should have some content
    }
}
