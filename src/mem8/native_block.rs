//! Safe, little-endian encoding of Aye's MEM8 v1.1.1 even/odd blocks.
//!
//! Layouts match `mem8-core/src/wave_atom.rs` and
//! `mem8-storage/src/block_v1.rs` in Aye. Application records occupy the binary
//! RAW8 lane; adjacent pairs continue records larger than one raw payload.

use crate::mem8_lite::Wave;
use anyhow::{ensure, Result};

pub const BLOCK_SIZE: usize = 4096;
pub const PAIR_SIZE: usize = BLOCK_SIZE * 2;
pub const PAYLOAD_SIZE: usize = 4074;
const MEM8: u32 = 0x4d454d38;
const RAW8: u32 = 0x52415738;

pub fn encode(payload: &[u8], location: u64, wave: &Wave, tombstoned: bool) -> Result<Vec<u8>> {
    validate_wave(wave)?;
    let count = payload.len().div_ceil(PAYLOAD_SIZE);
    let mut output = Vec::with_capacity(count * PAIR_SIZE);
    for chunk in payload.chunks(PAYLOAD_SIZE) {
        let mut even = [0u8; BLOCK_SIZE];
        even[..8].copy_from_slice(&location.to_le_bytes());
        even[8..12].copy_from_slice(&MEM8.to_le_bytes());
        even[17] = 1; // v1.1
        even[18] = 1; // One active WaveAtom, remaining slots reserved.
        even[19] = 2; // Sealed.
                      // Aye WaveAtom: rational frequency, Q0.16 arousal, Q1.15 valence.
        let denominator = (u16::MAX as f64 / wave.frequency.max(1.0))
            .floor()
            .clamp(1.0, 1000.0) as u16;
        let numerator = (wave.frequency * f64::from(denominator)).round() as u16;
        even[20..22].copy_from_slice(&numerator.to_le_bytes());
        even[22..24].copy_from_slice(&denominator.to_le_bytes());
        even[26..28].copy_from_slice(&u16::MAX.to_le_bytes());
        even[28..30].copy_from_slice(&((wave.arousal * 65535.0).round() as u16).to_le_bytes());
        even[30..32]
            .copy_from_slice(&((wave.emotional_valence * 32767.0).round() as i16).to_le_bytes());
        if count > 1 {
            even[43] = 2;
        } // FLAG_HAS_CHAINED_RAW
        if tombstoned {
            even[26..28].fill(0);
            even[43] |= 1; // WaveAtom::FLAG_TOMBSTONED
        }
        seal(&mut even);

        let mut odd = [0u8; BLOCK_SIZE];
        odd[..8].copy_from_slice(&location.to_le_bytes());
        odd[8..12].copy_from_slice(&RAW8.to_le_bytes());
        odd[16] = 1;
        odd[17] = 1;
        odd[18] = 3; // Binary application payload.
        odd[20..22].copy_from_slice(&(chunk.len() as u16).to_le_bytes());
        odd[22..22 + chunk.len()].copy_from_slice(chunk);
        seal(&mut odd);
        output.extend_from_slice(&even);
        output.extend_from_slice(&odd);
    }
    Ok(output)
}

pub fn decode_pair(pair: &[u8; PAIR_SIZE]) -> Result<(u64, Wave, &[u8])> {
    let (even, odd) = pair.split_at(BLOCK_SIZE);
    verify(even, MEM8, 0)?;
    verify(odd, RAW8, 1)?;
    ensure!(even[..8] == odd[..8], "Mismatched MEM8/RAW8 pair");
    ensure!(
        even[18] == 1 && odd[18] == 3,
        "Unsupported MEM8 application block"
    );
    let denominator = u16::from_le_bytes(even[22..24].try_into()?);
    ensure!(denominator != 0, "Invalid MEM8 frequency denominator");
    let wave = Wave::new(
        f64::from(u16::from_le_bytes(even[20..22].try_into()?)) / f64::from(denominator),
        f64::from(i16::from_le_bytes(even[30..32].try_into()?)) / 32767.0,
        f64::from(u16::from_le_bytes(even[28..30].try_into()?)) / 65535.0,
    );
    let length = usize::from(u16::from_le_bytes(odd[20..22].try_into()?));
    ensure!(
        length > 0 && length <= PAYLOAD_SIZE,
        "Invalid RAW8 payload size"
    );
    Ok((
        u64::from_le_bytes(even[..8].try_into()?),
        wave,
        &odd[22..22 + length],
    ))
}

fn validate_wave(wave: &Wave) -> Result<()> {
    ensure!(
        wave.frequency.is_finite()
            && (0.0..=65535.0).contains(&wave.frequency)
            && wave.arousal.is_finite()
            && (0.0..=1.0).contains(&wave.arousal)
            && wave.emotional_valence.is_finite()
            && (-1.0..=1.0).contains(&wave.emotional_valence),
        "Wave is outside MEM8 WaveAtom range"
    );
    Ok(())
}

fn seal(block: &mut [u8; BLOCK_SIZE]) {
    block[12..16].fill(0);
    let checksum = crc32fast::hash(block);
    block[12..16].copy_from_slice(&checksum.to_le_bytes());
}

fn verify(block: &[u8], magic: u32, kind: u8) -> Result<()> {
    ensure!(
        u32::from_le_bytes(block[8..12].try_into()?) == magic
            && block[16] == kind
            && block[17] == 1,
        "Unsupported MEM8 block format"
    );
    let expected = u32::from_le_bytes(block[12..16].try_into()?);
    let mut hash = crc32fast::Hasher::new();
    hash.update(&block[..12]);
    hash.update(&[0; 4]);
    hash.update(&block[16..]);
    ensure!(hash.finalize() == expected, "MEM8 block checksum mismatch");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_layout_roundtrip_and_corruption_detection() {
        let wave = Wave::new(73.0 / 100.0, -0.5, 0.8);
        let bytes = encode(b"exact source text", 42, &wave, false).unwrap();
        // Generated independently using Aye's actual block_v1/spool codecs at
        // 9d052392a673d6aeebc69f1846f4ed1fb728e3f4, not this encoder.
        use sha2::{Digest, Sha256};
        assert_eq!(
            hex::encode(Sha256::digest(&bytes)),
            "416d597ba8d14519354c388105840a9f64dc6c168b9923f8b46be580a78b83bc"
        );
        assert_eq!(bytes.len(), PAIR_SIZE);
        assert_eq!(&bytes[8..12], &MEM8.to_le_bytes());
        assert_eq!(&bytes[BLOCK_SIZE + 8..BLOCK_SIZE + 12], &RAW8.to_le_bytes());
        let mut pair: [u8; PAIR_SIZE] = bytes.try_into().unwrap();
        let (location, restored, payload) = decode_pair(&pair).unwrap();
        assert_eq!(location, 42);
        assert_eq!(payload, b"exact source text");
        assert!((restored.frequency - wave.frequency).abs() < 0.001);
        assert!((restored.arousal - wave.arousal).abs() < 0.0001);
        assert!((restored.emotional_valence - wave.emotional_valence).abs() < 0.0001);
        pair[BLOCK_SIZE + 22] ^= 1;
        assert!(decode_pair(&pair).is_err());
        let deleted = encode(b"deleted", 42, &wave, true).unwrap();
        assert_eq!(&deleted[26..28], &[0, 0]);
        assert_eq!(deleted[43] & 1, 1);
    }
}
