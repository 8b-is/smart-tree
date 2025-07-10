use crate::{Result, ToolError};

pub fn quantize_8bit(data: &[u8]) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    
    // Simple 8-bit quantization with pattern detection
    let mut quantized = Vec::with_capacity(data.len() / 8);
    let mut pattern_map = std::collections::HashMap::new();
    
    for chunk in data.chunks(8) {
        let hash = blake3::hash(chunk);
        let byte = hash.as_bytes()[0];
        
        pattern_map.entry(byte).and_modify(|e| *e += 1).or_insert(1);
        quantized.push(byte);
    }
    
    // Add header with pattern info
    let mut output = vec![0xE8]; // Magic byte for "E8" (Eight)
    output.push(quantized.len() as u8);
    output.extend_from_slice(&quantized);
    
    Ok(output)
}

pub fn dequantize_8bit(data: &[u8]) -> Result<Vec<u8>> {
    if data.is_empty() || data[0] != 0xE8 {
        return Err(ToolError::QuantizationError("Invalid quantized data".to_string()));
    }
    
    // For now, just return the quantized data
    // Full implementation would reconstruct based on patterns
    Ok(data[2..].to_vec())
}

// uLaw/aLaw style companding
pub fn compand_ulaw(sample: i16) -> u8 {
    const BIAS: i16 = 0x84;
    const CLIP: i16 = 32635;
    
    let sign = if sample < 0 { 0x80 } else { 0 };
    let mut sample = sample.abs();
    
    if sample > CLIP {
        sample = CLIP;
    }
    
    sample += BIAS;
    
    let exponent = (sample >> 7).leading_zeros() as u8;
    let mantissa = ((sample >> (exponent + 3)) & 0x0F) as u8;
    
    sign | ((7 - exponent) << 4) | mantissa
}

pub fn expand_ulaw(ulaw: u8) -> i16 {
    const BIAS: i16 = 0x84;
    
    let sign = if ulaw & 0x80 != 0 { -1 } else { 1 };
    let exponent = (ulaw >> 4) & 0x07;
    let mantissa = ulaw & 0x0F;
    
    let mut sample = ((mantissa as i16) << 3) + BIAS;
    sample <<= exponent;
    
    sign * (sample - BIAS)
}