// Decoder framework - Convert quantum format to other representations
// All formats are now just views into the quantum stream

use anyhow::Result;
use std::io::Write;

pub mod classic;
pub mod hex;
pub mod json;

/// Quantum entry components after parsing
#[derive(Debug, Clone)]
pub struct QuantumEntry {
    pub header: u8,
    pub size: Option<u64>,
    pub perms_delta: Option<u16>,
    pub time_delta: Option<i64>,
    pub owner_delta: Option<(u32, u32)>,
    pub name: String,
    pub is_dir: bool,
    pub is_link: bool,
    pub traversal: TraversalCode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TraversalCode {
    Same,    // Continue at same level
    Deeper,  // Enter directory
    Back,    // Exit directory
    Summary, // Summary follows
}

impl From<u8> for TraversalCode {
    fn from(byte: u8) -> Self {
        match byte {
            0x0B => TraversalCode::Same,
            0x0E => TraversalCode::Deeper,
            0x0F => TraversalCode::Back,
            0x0C => TraversalCode::Summary,
            _ => TraversalCode::Same, // Default
        }
    }
}

/// Base trait for all quantum decoders
pub trait QuantumDecoder: Send {
    /// Initialize the decoder
    fn init(&mut self, writer: &mut dyn Write) -> Result<()>;

    /// Process a quantum entry
    fn decode_entry(&mut self, entry: &QuantumEntry, writer: &mut dyn Write) -> Result<()>;

    /// Finalize output
    fn finish(&mut self, writer: &mut dyn Write) -> Result<()>;
}

/// Parse a quantum stream and decode to target format
pub fn decode_quantum_stream<D: QuantumDecoder>(
    quantum_data: &[u8],
    decoder: &mut D,
    writer: &mut dyn Write,
) -> Result<()> {
    decoder.init(writer)?;

    // Parse quantum entries from binary data
    let mut offset = 0;
    while offset < quantum_data.len() {
        let (entry, new_offset) = parse_quantum_entry(quantum_data, offset)?;
        if let Some(entry) = entry {
            decoder.decode_entry(&entry, writer)?;
        }
        offset = new_offset;
    }

    decoder.finish(writer)?;
    Ok(())
}

/// Parse a single quantum entry from binary data
fn parse_quantum_entry(data: &[u8], offset: usize) -> Result<(Option<QuantumEntry>, usize)> {
    if offset >= data.len() {
        return Ok((None, offset));
    }

    let header = data[offset];
    let mut offset = offset + 1;

    let mut entry = QuantumEntry {
        header,
        size: None,
        perms_delta: None,
        time_delta: None,
        owner_delta: None,
        name: String::new(),
        is_dir: (header & 0x10) != 0,
        is_link: (header & 0x20) != 0,
        traversal: TraversalCode::Same,
    };

    // Parse size if present
    if (header & 0x01) != 0 {
        let (size, new_offset) = decode_size(data, offset)?;
        entry.size = Some(size);
        offset = new_offset;
    }

    // Parse permissions delta if present
    if (header & 0x02) != 0 && offset + 2 <= data.len() {
        entry.perms_delta = Some((data[offset] as u16) << 8 | data[offset + 1] as u16);
        offset += 2;
    }

    // TODO: Parse time, owner/group deltas

    // Parse name (ends with traversal code)
    let name_start = offset;
    while offset < data.len() && !is_traversal_code(data[offset]) {
        offset += 1;
    }

    entry.name = String::from_utf8_lossy(&data[name_start..offset]).into_owned();

    // Parse traversal code
    if offset < data.len() {
        entry.traversal = data[offset].into();
        offset += 1;
    }

    Ok((Some(entry), offset))
}

/// Check if a byte is a traversal code
fn is_traversal_code(byte: u8) -> bool {
    matches!(byte, 0x0B | 0x0E | 0x0F | 0x0C)
}

/// Decode variable-length size encoding
fn decode_size(data: &[u8], offset: usize) -> Result<(u64, usize)> {
    if offset >= data.len() {
        anyhow::bail!("Unexpected end of data while decoding size");
    }

    let prefix = data[offset];
    match prefix {
        0x00 => {
            if offset + 1 >= data.len() {
                anyhow::bail!("Incomplete size encoding");
            }
            Ok((data[offset + 1] as u64, offset + 2))
        }
        0x01 => {
            if offset + 2 >= data.len() {
                anyhow::bail!("Incomplete size encoding");
            }
            let size = u16::from_le_bytes([data[offset + 1], data[offset + 2]]) as u64;
            Ok((size, offset + 3))
        }
        0x02 => {
            if offset + 4 >= data.len() {
                anyhow::bail!("Incomplete size encoding");
            }
            let size = u32::from_le_bytes([
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
            ]) as u64;
            Ok((size, offset + 5))
        }
        0x03 => {
            if offset + 8 >= data.len() {
                anyhow::bail!("Incomplete size encoding");
            }
            let size = u64::from_le_bytes([
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
                data[offset + 8],
            ]);
            Ok((size, offset + 9))
        }
        _ => {
            // Check if it's a size token
            if (0xA0..=0xAF).contains(&prefix) {
                // Size range tokens
                let size = match prefix {
                    0xA0 => 0,                // TOKEN_SIZE_ZERO
                    0xA1 => 512,              // TOKEN_SIZE_TINY (average)
                    0xA2 => 50 * 1024,        // TOKEN_SIZE_SMALL (average)
                    0xA3 => 5 * 1024 * 1024,  // TOKEN_SIZE_MEDIUM (average)
                    0xA4 => 50 * 1024 * 1024, // TOKEN_SIZE_LARGE (average)
                    _ => 0,
                };
                Ok((size, offset + 1))
            } else {
                anyhow::bail!("Invalid size prefix: 0x{:02x}", prefix);
            }
        }
    }
}
