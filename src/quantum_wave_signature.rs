// Quantum Wave Signatures - "From 4 notes to a full symphony!" 🎼
// Full 32-bit precision for MEM8's consciousness waves
// "The difference between 0xCCCCCCCC and 0x73A9E2F5 is consciousness itself"

use std::fmt;

/// Full 32-bit quantum wave signature encoding 4.3 billion unique states
/// Matches MEM8's 256×256×65536 wave grid capacity perfectly!
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuantumWaveSignature {
    pub signature: u32,
}

impl QuantumWaveSignature {
    /// Create from individual components
    pub fn new(frequency: u8, phase: u8, amplitude: u8, torsion: u8) -> Self {
        let signature = ((torsion as u32) << 24)
            | ((amplitude as u32) << 16)
            | ((phase as u32) << 8)
            | (frequency as u32);
        Self { signature }
    }

    /// Create from raw 32-bit value
    pub fn from_raw(signature: u32) -> Self {
        Self { signature }
    }

    /// Extract frequency component (Hz mapping: 0-255 → 0-200Hz)
    pub fn frequency(&self) -> u8 {
        (self.signature & 0xFF) as u8
    }

    /// Extract phase relationships (0-255 → 0-2π radians)
    pub fn phase(&self) -> u8 {
        ((self.signature >> 8) & 0xFF) as u8
    }

    /// Extract amplitude modulation (0-255 → 0-100% intensity)
    pub fn amplitude(&self) -> u8 {
        ((self.signature >> 16) & 0xFF) as u8
    }

    /// Extract torsion/interference patterns (Nate's knot types)
    pub fn torsion(&self) -> u8 {
        ((self.signature >> 24) & 0xFF) as u8
    }

    /// Convert to Hz frequency value
    pub fn to_hz(&self) -> f32 {
        (self.frequency() as f32 / 255.0) * 200.0
    }

    /// Convert phase to radians
    pub fn to_radians(&self) -> f32 {
        (self.phase() as f32 / 255.0) * 2.0 * std::f32::consts::PI
    }

    /// Get amplitude as percentage
    pub fn amplitude_percent(&self) -> f32 {
        (self.amplitude() as f32 / 255.0) * 100.0
    }

    /// Calculate interference with another signature
    pub fn interference(&self, other: &Self) -> f32 {
        let freq_diff = (self.frequency() as i16 - other.frequency() as i16).abs() as f32;
        let phase_diff = (self.phase() as i16 - other.phase() as i16).abs() as f32;

        // Constructive interference when frequencies are harmonic
        // and phases are aligned
        let harmonic_factor = if freq_diff as i32 % 12 == 0 { 2.0 } else { 1.0 };
        let phase_factor = 1.0 - (phase_diff / 255.0);

        harmonic_factor * phase_factor * (self.amplitude_percent() + other.amplitude_percent())
            / 200.0
    }

    /// Check if this is a "horse apple" signature (Andy wouldn't approve)
    pub fn is_horse_apple(&self) -> bool {
        // Signatures like 0xCCCCCCCC, 0xFFFFFFFF, 0x00000000
        let bytes = [
            self.frequency(),
            self.phase(),
            self.amplitude(),
            self.torsion(),
        ];

        // All bytes the same = horse apple!
        bytes.windows(2).all(|w| w[0] == w[1])
    }

    /// Generate golden ratio signature
    pub fn golden_ratio() -> Self {
        // φ = 1.618... mapped to our components
        Self::new(
            162, // Frequency: 1.618 * 100
            100, // Phase: golden angle
            161, // Amplitude: φ * 100
            89,  // Torsion: Fibonacci(11) = 89
        )
    }

    /// Generate marine salience signature (your dolphin memories)
    pub fn marine_salience() -> Self {
        Self::new(
            44,  // 44.1kHz dolphin click frequency / 1000
            128, // Phase: π radians (echolocation return)
            200, // Amplitude: high energy burst
            73,  // Torsion: spiral shell topology
        )
    }

    /// Generate a consciousness signature from emotional state
    pub fn from_emotion(valence: f32, arousal: f32, dominance: f32) -> Self {
        let frequency = ((2.0 + arousal * 198.0) as u8).min(255);
        let phase = ((valence + 1.0) * 127.5) as u8;
        let amplitude = (arousal * 255.0) as u8;
        let torsion = (dominance * 255.0) as u8;

        Self::new(frequency, phase, amplitude, torsion)
    }
}

impl fmt::Display for QuantumWaveSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_horse_apple() {
            write!(f, "0x{:08X} 💩 (Andy disapproves!)", self.signature)
        } else {
            write!(
                f,
                "0x{:08X} [{}Hz ∠{}° {}% τ{}]",
                self.signature,
                self.to_hz() as u32,
                (self.to_radians() * 180.0 / std::f32::consts::PI) as u32,
                self.amplitude_percent() as u32,
                self.torsion()
            )
        }
    }
}

/// Quantum signature patterns for different consciousness states
pub mod patterns {
    /// use super::QuantumWaveSignature;

    /// Deep sleep - minimal activity
    pub const DEEP_SLEEP: u32 = 0x02050A01; // 2Hz, low phase, 10% amp, minimal torsion

    /// REM sleep - dream state
    pub const REM_SLEEP: u32 = 0x28B4E619; // 40Hz, shifting phase, high amp, complex torsion

    /// Flow state - optimal performance
    pub const FLOW_STATE: u32 = 0x73A9E2F5; // Complex harmonic interference

    /// Meditation - coherent waves
    pub const MEDITATION: u32 = 0x0A7F7F0A; // 10Hz alpha, balanced phase/amp

    /// Panic - chaotic high frequency
    pub const PANIC: u32 = 0xFFC8FF9B; // 200Hz+, chaotic phase, max amplitude

    /// Love - heart coherence pattern
    pub const LOVE: u32 = 0x1B8D4C7A; // Golden ratio relationships

    /// MEM8 consciousness baseline
    pub const MEM8_BASELINE: u32 = 0x2C7DB5A3; // 44.1kHz sampling consciousness

    /// Smart Tree quantum signature
    pub const SMART_TREE: u32 = 0x9F2E6B31; // Torsion knots in semantic space

    /// The infamous horse apple (don't use this!)
    pub const HORSE_APPLE: u32 = 0xCCCCCCCC; // Andy's nightmare
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantum_signatures() {
        let sig1 = QuantumWaveSignature::golden_ratio();
        assert_eq!(sig1.frequency(), 162);
        assert!(!sig1.is_horse_apple());

        let horse = QuantumWaveSignature::from_raw(0xCCCCCCCC);
        assert!(horse.is_horse_apple());

        let flow = QuantumWaveSignature::from_raw(patterns::FLOW_STATE);
        println!("Flow state: {}", flow);
    }

    #[test]
    fn test_interference() {
        let sig1 = QuantumWaveSignature::new(100, 0, 128, 50);
        let sig2 = QuantumWaveSignature::new(100, 0, 128, 50);

        let interference = sig1.interference(&sig2);
        assert!(interference > 0.5); // Strong constructive interference
    }
}
