// Classic Decoder - Convert quantum format to human-readable tree
// TODO: Implement classic tree visualization from quantum stream

use super::{QuantumDecoder, QuantumEntry};
use std::io::Write;
use anyhow::Result;

pub struct ClassicDecoder;

impl ClassicDecoder {
    pub fn new() -> Self {
        Self
    }
}

impl QuantumDecoder for ClassicDecoder {
    fn init(&mut self, _writer: &mut dyn Write) -> Result<()> {
        // TODO: Implement initialization
        Ok(())
    }
    
    fn decode_entry(&mut self, _entry: &QuantumEntry, _writer: &mut dyn Write) -> Result<()> {
        // TODO: Implement classic tree drawing
        Ok(())
    }
    
    fn finish(&mut self, _writer: &mut dyn Write) -> Result<()> {
        // TODO: Implement summary
        Ok(())
    }
}