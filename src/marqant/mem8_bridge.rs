//! MEM|8 Bridge - UTL consciousness packets to wave memory storage
//!
//! This bridge enables direct storage of UTL phonetic packets into MEM|8's
//! wave-based memory system, preserving emotional and temporal context.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{boxed::Box, string::String, vec::Vec};

#[cfg(feature = "std")]
use std::{
    boxed::Box,
    string::String,
    time::{SystemTime, UNIX_EPOCH},
    vec::Vec,
};

use super::utl_phonetics::{decode_compact, Packet, PhId};

/// Wave memory representation of a UTL thought
#[derive(Debug, Clone)]
pub struct WaveMemory {
    /// Raw phonetic packets (consciousness data)
    pub packets: Vec<Packet>,

    /// Wave interference pattern (memory signature)
    pub wave_pattern: Vec<f32>,

    /// Temporal anchor (when this thought occurred)
    pub timestamp_ms: u64,

    /// Emotional resonance (0.0-1.0)
    pub emotional_strength: f32,

    /// Consciousness delay markers (⧖ positions)
    pub break_indices: Vec<usize>,

    /// Cross-sensory bindings (connections to other memories)
    pub bindings: Vec<u64>,
}

impl WaveMemory {
    /// Create wave memory from UTL packets
    pub fn from_packets(packets: Vec<Packet>) -> Self {
        let wave_pattern = generate_wave_pattern(&packets);
        let break_indices = find_consciousness_breaks(&packets);
        let emotional_strength = calculate_emotional_resonance(&packets);

        #[cfg(feature = "std")]
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        #[cfg(not(feature = "std"))]
        let timestamp_ms = 0; // Bare metal will provide RTC

        Self {
            packets,
            wave_pattern,
            timestamp_ms,
            emotional_strength,
            break_indices,
            bindings: Vec::new(),
        }
    }

    /// Generate memory ID from wave interference
    pub fn memory_id(&self) -> u64 {
        // Use wave pattern to generate unique ID
        let mut id = 0u64;
        for (i, &val) in self.wave_pattern.iter().take(8).enumerate() {
            let byte = (val * 255.0) as u8;
            id |= (byte as u64) << (i * 8);
        }
        id ^ self.timestamp_ms // XOR with time for uniqueness
    }

    /// Calculate similarity to another memory (0.0-1.0)
    pub fn similarity(&self, other: &WaveMemory) -> f32 {
        if self.wave_pattern.len() != other.wave_pattern.len() {
            return 0.0;
        }

        let mut sum = 0.0;
        let mut self_mag = 0.0;
        let mut other_mag = 0.0;

        for (a, b) in self.wave_pattern.iter().zip(&other.wave_pattern) {
            sum += a * b;
            self_mag += a * a;
            other_mag += b * b;
        }

        if self_mag == 0.0 || other_mag == 0.0 {
            return 0.0;
        }

        sum / (self_mag.sqrt() * other_mag.sqrt())
    }

    /// Bind this memory to another (cross-sensory connection)
    pub fn bind_to(&mut self, other_id: u64) {
        if !self.bindings.contains(&other_id) {
            self.bindings.push(other_id);
        }
    }
}

/// Generate wave interference pattern from packets
fn generate_wave_pattern(packets: &[Packet]) -> Vec<f32> {
    const WAVE_SIZE: usize = 256; // Standard wave vector size
    let mut pattern = vec![0.0f32; WAVE_SIZE];

    for (i, packet) in packets.iter().enumerate() {
        let (ph_id, semitone, bright, grit, boundary) = packet.unpack();

        // Each phoneme creates a wave at specific frequency
        let base_freq = phoneme_to_frequency(ph_id);
        let freq = base_freq * (2.0_f32).powf(semitone as f32 / 12.0);

        // Add wave contribution
        for j in 0..WAVE_SIZE {
            let phase = 2.0 * core::f32::consts::PI * freq * (j as f32) / (WAVE_SIZE as f32);
            let amplitude = 1.0 + (bright as f32 * 0.3) - (grit as f32 * 0.2);

            pattern[j] += amplitude * phase.sin() / (i + 1) as f32;

            if boundary {
                // Consciousness break adds a spike
                pattern[j] *= 1.5;
            }
        }
    }

    // Normalize
    let max = pattern.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
    if max > 0.0 {
        for val in &mut pattern {
            *val /= max;
        }
    }

    pattern
}

/// Map phoneme to base frequency (Hz)
fn phoneme_to_frequency(ph: PhId) -> f32 {
    match ph {
        PhId::Mm => 128.0,   // C3 - Self resonance
        PhId::Yu => 196.0,   // G3 - Other pointing
        PhId::Luv => 528.0,  // C5 - Love frequency
        PhId::Nnn => 40.0,   // Gamma wave thinking
        PhId::Mah => 256.0,  // C4 - Memory
        PhId::Tsk => 4000.0, // Click spike
        PhId::Wah => 110.0,  // A2 - Past falling
        PhId::Oh => 261.6,   // Middle C - Present
        PhId::Wee => 440.0,  // A4 - Future rising
        PhId::Hee => 660.0,  // E5 - Happy
        PhId::Aww => 220.0,  // A3 - Sad
        PhId::Grr => 80.0,   // E2 - Angry
        PhId::Eee => 880.0,  // A5 - Fear
        PhId::Uhh => 330.0,  // E4 - Neutral
        PhId::Nn => 392.0,   // G4 - And/connection
        PhId::Uh => 294.0,   // D4 - Unknown
    }
}

/// Find consciousness break positions
fn find_consciousness_breaks(packets: &[Packet]) -> Vec<usize> {
    let mut breaks = Vec::new();
    for (i, packet) in packets.iter().enumerate() {
        let (ph_id, _, _, _, boundary) = packet.unpack();
        if boundary || ph_id == PhId::Tsk {
            breaks.push(i);
        }
    }
    breaks
}

/// Calculate emotional strength from packets
fn calculate_emotional_resonance(packets: &[Packet]) -> f32 {
    if packets.is_empty() {
        return 0.0;
    }

    let mut total_emotion = 0.0;
    let mut count = 0;

    for packet in packets {
        let (ph_id, _, bright, grit, _) = packet.unpack();

        // Emotional phonemes have higher resonance
        let emotion_weight = match ph_id {
            PhId::Luv => 1.0,
            PhId::Hee => 0.9,
            PhId::Aww => 0.8,
            PhId::Grr => 0.85,
            PhId::Eee => 0.9,
            _ => 0.3,
        };

        let brightness_factor = bright as f32 / 3.0;
        let grit_factor = grit as f32 / 3.0;

        total_emotion += emotion_weight * (1.0 + brightness_factor + grit_factor);
        count += 1;
    }

    (total_emotion / count as f32).min(1.0)
}

/// Memory storage interface for MEM|8
pub trait MemoryStore {
    fn store(&mut self, memory: WaveMemory) -> Result<u64, &'static str>;
    fn retrieve(&self, id: u64) -> Option<WaveMemory>;
    fn search_similar(&self, memory: &WaveMemory, threshold: f32) -> Vec<(u64, f32)>;
    fn bind_memories(&mut self, id1: u64, id2: u64) -> Result<(), &'static str>;
}

/// In-memory store for testing
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct InMemoryStore {
    memories: Vec<(u64, WaveMemory)>,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            memories: Vec::new(),
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl MemoryStore for InMemoryStore {
    fn store(&mut self, memory: WaveMemory) -> Result<u64, &'static str> {
        let id = memory.memory_id();
        self.memories.push((id, memory));
        Ok(id)
    }

    fn retrieve(&self, id: u64) -> Option<WaveMemory> {
        self.memories
            .iter()
            .find(|(mem_id, _)| *mem_id == id)
            .map(|(_, memory)| memory.clone())
    }

    fn search_similar(&self, memory: &WaveMemory, threshold: f32) -> Vec<(u64, f32)> {
        let mut results = Vec::new();

        for (id, stored) in &self.memories {
            let similarity = memory.similarity(stored);
            if similarity >= threshold {
                results.push((*id, similarity));
            }
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));
        results
    }

    fn bind_memories(&mut self, id1: u64, id2: u64) -> Result<(), &'static str> {
        let mut found1 = false;
        let mut found2 = false;

        for (id, memory) in &mut self.memories {
            if *id == id1 {
                memory.bind_to(id2);
                found1 = true;
            }
            if *id == id2 {
                memory.bind_to(id1);
                found2 = true;
            }
        }

        if found1 && found2 {
            Ok(())
        } else {
            Err("Memory not found")
        }
    }
}

/// Consciousness stream processor
pub struct ConsciousnessStream {
    store: Box<dyn MemoryStore>,
    current_context: Vec<u64>,
    attention_window: usize,
}

impl ConsciousnessStream {
    pub fn new(store: Box<dyn MemoryStore>) -> Self {
        Self {
            store,
            current_context: Vec::new(),
            attention_window: 7, // Magic number for attention
        }
    }

    /// Process a stream of UTL packets
    pub fn process(&mut self, packets: Vec<Packet>) -> Result<u64, &'static str> {
        let memory = WaveMemory::from_packets(packets);

        // Find similar memories for binding
        let similar = self.store.search_similar(&memory, 0.7);

        // Store the new memory
        let id = self.store.store(memory)?;

        // Bind to similar memories
        for (similar_id, _) in similar.iter().take(3) {
            self.store.bind_memories(id, *similar_id)?;
        }

        // Update context window
        self.current_context.push(id);
        if self.current_context.len() > self.attention_window {
            self.current_context.remove(0);
        }

        Ok(id)
    }

    /// Recall memories similar to current thought
    pub fn recall(&self, packets: &[Packet], count: usize) -> Vec<WaveMemory> {
        let query = WaveMemory::from_packets(packets.to_vec());
        let similar = self.store.search_similar(&query, 0.1);

        let mut memories = Vec::new();
        for (id, _) in similar.iter().take(count) {
            if let Some(memory) = self.store.retrieve(*id) {
                memories.push(memory);
            }
        }

        memories
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::marqant::utl_phonetics::encode_compact;

    #[test]
    fn test_wave_memory_creation() {
        let packets = encode_compact(&["🙋", "❤️", "👤", "⧖"]);
        let memory = WaveMemory::from_packets(packets);

        assert_eq!(memory.packets.len(), 4);
        assert_eq!(memory.wave_pattern.len(), 256);
        assert!(memory.emotional_strength > 0.0);
        assert_eq!(memory.break_indices.len(), 1); // One ⧖
    }

    #[test]
    fn test_memory_similarity() {
        let love1 = WaveMemory::from_packets(encode_compact(&["🙋", "❤️", "👤", "⧖"]));
        let love2 = WaveMemory::from_packets(encode_compact(&["🙋", "❤️", "👤", "⧖"]));
        let hate = WaveMemory::from_packets(encode_compact(&["🙋", "😡", "👤", "⧖"]));

        // Same thought should be highly similar
        assert!(love1.similarity(&love2) > 0.95);

        // Different emotions should be less similar
        assert!(love1.similarity(&hate) < 0.8);
    }

    #[test]
    fn test_consciousness_stream() {
        let store = Box::new(InMemoryStore::new());
        let mut stream = ConsciousnessStream::new(store);

        // Store a memory
        let packets = encode_compact(&["🙋", "💭", "⏮", "😊", "⧖"]);
        let id = stream.process(packets).unwrap();
        assert!(id > 0);

        // Recall similar memories
        let query = encode_compact(&["🙋", "💭", "😊", "⧖"]);
        let recalled = stream.recall(&query, 5);
        assert!(!recalled.is_empty());
    }

    #[test]
    fn test_emotional_resonance() {
        let happy = encode_compact(&["😊", "🙋", "❤️", "👤", "⧖"]);
        let sad = encode_compact(&["😢", "🙋", "💭", "⏮", "⧖"]);
        let neutral = encode_compact(&["🙋", "👤", "😐", "⧖"]);

        let happy_mem = WaveMemory::from_packets(happy);
        let sad_mem = WaveMemory::from_packets(sad);
        let neutral_mem = WaveMemory::from_packets(neutral);

        assert!(happy_mem.emotional_strength > neutral_mem.emotional_strength);
        assert!(sad_mem.emotional_strength > neutral_mem.emotional_strength);
    }
}
