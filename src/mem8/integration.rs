//! Integration of MEM8 with Smart Tree
//! Provides cognitive memory capabilities for directory analysis

use anyhow::Result;
use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::mem8::{
    consciousness::ConsciousnessEngine,
    format::{CompressedWave, M8Writer},
    reactive::{ReactiveLayer, ReactiveMemory, ReactivePattern, ReactiveResponse, SensorInput},
    wave::{FrequencyBand, MemoryWave, WaveGrid},
};

/// MEM8 integration for Smart Tree
pub struct SmartTreeMem8 {
    /// Wave grid for storing directory memories
    wave_grid: Arc<RwLock<WaveGrid>>,
    /// Reactive memory system
    reactive_memory: Arc<RwLock<ReactiveMemory>>,
    /// Consciousness engine
    consciousness: Arc<ConsciousnessEngine>,
    /// Current directory depth for z-axis mapping
    current_depth: u16,
}

impl Default for SmartTreeMem8 {
    fn default() -> Self {
        Self::new()
    }
}

impl SmartTreeMem8 {
    /// Create a new MEM8 instance for Smart Tree
    pub fn new() -> Self {
        #[cfg(not(test))]
        let wave_grid = Arc::new(RwLock::new(WaveGrid::new()));

        #[cfg(test)]
        let wave_grid = Arc::new(RwLock::new(WaveGrid::new_test()));

        let reactive_memory = Arc::new(RwLock::new(ReactiveMemory::new(wave_grid.clone())));
        let consciousness = Arc::new(ConsciousnessEngine::new(wave_grid.clone()));

        Self {
            wave_grid,
            reactive_memory,
            consciousness,
            current_depth: 0,
        }
    }

    /// Store a directory entry as a wave memory
    pub fn store_directory_memory(
        &mut self,
        path: &Path,
        metadata: DirectoryMetadata,
    ) -> Result<()> {
        // Map directory path to spatial coordinates
        let (x, y) = self.path_to_coordinates(path);

        // Create memory wave based on directory characteristics
        let wave = self.create_directory_wave(&metadata);

        // Store in grid
        self.wave_grid
            .write()
            .unwrap()
            .store(x, y, self.current_depth, wave);

        // Update depth for next entry
        self.current_depth = if self.current_depth == 65535 {
            0
        } else {
            self.current_depth + 1
        };

        Ok(())
    }

    /// Convert directory path to grid coordinates
    fn path_to_coordinates(&self, path: &Path) -> (u8, u8) {
        // Hash path components to distribute across grid
        let path_str = path.to_string_lossy();
        let hash = self.simple_hash(&path_str);

        let x = (hash & 0xFF) as u8;
        let y = ((hash >> 8) & 0xFF) as u8;

        (x, y)
    }

    /// Simple hash function for path distribution
    pub fn simple_hash(&self, s: &str) -> u64 {
        let mut hash = 5381u64;
        for byte in s.bytes() {
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u64);
        }
        hash
    }

    /// Create a wave from directory metadata
    fn create_directory_wave(&self, metadata: &DirectoryMetadata) -> MemoryWave {
        // Determine frequency based on content type
        let frequency = match metadata.primary_type {
            ContentType::Code => FrequencyBand::Technical.frequency(0.5),
            ContentType::Documentation => FrequencyBand::Conversational.frequency(0.5),
            ContentType::Configuration => FrequencyBand::DeepStructural.frequency(0.5),
            ContentType::Data => FrequencyBand::Implementation.frequency(0.5),
            ContentType::Media => FrequencyBand::Abstract.frequency(0.5),
        };

        // Amplitude based on importance/size
        let amplitude =
            (metadata.importance * 0.7 + metadata.normalized_size * 0.3).clamp(0.1, 1.0);

        let mut wave = MemoryWave::new(frequency, amplitude);

        // Set emotional context based on directory health
        wave.valence = match metadata.health {
            DirectoryHealth::Healthy => 0.5,
            DirectoryHealth::Warning => -0.2,
            DirectoryHealth::Critical => -0.8,
        };

        // Arousal based on activity level
        wave.arousal = metadata.activity_level;

        // Decay based on last modified time
        if metadata.days_since_modified > 365 {
            wave.decay_tau = Some(std::time::Duration::from_secs(86400)); // 1 day
        } else if metadata.days_since_modified > 30 {
            wave.decay_tau = Some(std::time::Duration::from_secs(604800)); // 1 week
        } else {
            wave.decay_tau = None; // No decay for recent files
        }

        wave
    }

    /// Query memories about a specific path pattern
    pub fn query_path_memories(&self, pattern: &str) -> Vec<PathMemory> {
        let grid = self.wave_grid.read().unwrap();
        let mut memories = Vec::new();

        // Sample grid looking for relevant memories
        for x in 0..=255u8 {
            for y in 0..=255u8 {
                // Check recent layers (last 1000)
                for z in (self.current_depth.saturating_sub(1000)..self.current_depth).step_by(10) {
                    if let Some(wave) = grid.get(x, y, z) {
                        if wave.calculate_decay() > 0.1 {
                            // This is an active memory
                            memories.push(PathMemory {
                                coordinates: (x, y, z),
                                wave: wave.clone(),
                                relevance: self.calculate_relevance(wave, pattern),
                            });
                        }
                    }
                }
            }
        }

        // Sort by relevance
        memories.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        memories.truncate(20); // Top 20 results

        memories
    }

    /// Calculate relevance score for a memory
    fn calculate_relevance(&self, wave: &MemoryWave, pattern: &str) -> f32 {
        // Simple relevance based on frequency band matching
        let pattern_freq = if pattern.contains("src") || pattern.contains("lib") {
            FrequencyBand::Technical.frequency(0.5)
        } else if pattern.contains("doc") || pattern.contains("README") {
            FrequencyBand::Conversational.frequency(0.5)
        } else if pattern.contains("config") || pattern.contains("toml") {
            FrequencyBand::DeepStructural.frequency(0.5)
        } else {
            FrequencyBand::Implementation.frequency(0.5)
        };

        // Frequency similarity
        let freq_diff = (wave.frequency - pattern_freq).abs() / 1000.0;
        let freq_score = 1.0 - freq_diff.clamp(0.0, 1.0);

        // Combine with amplitude and decay
        freq_score * wave.amplitude * wave.calculate_decay()
    }

    /// Register reactive patterns for directory events
    pub fn register_directory_patterns(&mut self) {
        // Pattern for large directories
        let large_dir_pattern = ReactivePattern {
            id: "large_directory".to_string(),
            threshold: 0.7,
            weight: 1.0,
            response: Arc::new(|| ReactiveResponse {
                layer: ReactiveLayer::SubcorticalReaction,
                strength: 0.8,
                action: "Enable streaming mode for large directory".to_string(),
                latency: std::time::Duration::from_millis(30),
            }),
        };

        // Pattern for security threats
        let security_pattern = ReactivePattern {
            id: "security_threat".to_string(),
            threshold: 0.5,
            weight: 1.5,
            response: Arc::new(|| ReactiveResponse {
                layer: ReactiveLayer::HardwareReflex,
                strength: 0.95,
                action: "Block access to suspicious directory".to_string(),
                latency: std::time::Duration::from_millis(5),
            }),
        };

        // Register patterns
        let mut reactive = self.reactive_memory.write().unwrap();
        reactive.register_pattern(ReactiveLayer::SubcorticalReaction, large_dir_pattern);
        reactive.register_pattern(ReactiveLayer::HardwareReflex, security_pattern);
    }

    /// Process directory scan event through reactive system
    pub fn process_directory_event(&self, event: DirectoryEvent) -> Option<ReactiveResponse> {
        let input = match event {
            DirectoryEvent::LargeDirectory { size, .. } => SensorInput::Visual {
                intensity: (size as f32 / 1_000_000.0).clamp(0.0, 1.0),
                motion: 0.0,
                looming: size > 10_000_000,
            },
            DirectoryEvent::SecurityThreat { severity, .. } => SensorInput::Threat {
                severity,
                proximity: 1.0,
                pattern: "malicious_file".to_string(),
            },
            DirectoryEvent::RapidChange { rate, .. } => SensorInput::Visual {
                intensity: 0.5,
                motion: rate,
                looming: false,
            },
        };

        self.reactive_memory.read().unwrap().process(&input)
    }

    /// Update consciousness state
    pub fn update_consciousness(&self) {
        self.consciousness.update();
    }

    /// Export memories to .m8 format
    pub fn export_memories<W: std::io::Write>(&self, writer: W) -> Result<()> {
        let mut m8_writer = M8Writer::new(writer);
        let grid = self.wave_grid.read().unwrap();

        // Collect all active memories
        let mut compressed_waves = Vec::new();
        let mut id = 0;

        for x in 0..=255u8 {
            for y in 0..=255u8 {
                for z in 0..100u16 {
                    // Sample first 100 layers
                    if let Some(wave) = grid.get(x, y, z) {
                        if wave.calculate_decay() > 0.01 {
                            compressed_waves.push(CompressedWave::from_wave(wave, id));
                            id += 1;
                        }
                    }
                }
            }
        }

        m8_writer.add_wave_memory(&compressed_waves)?;
        m8_writer.finish()?;

        Ok(())
    }

    /// Get count of active memories
    pub fn active_memory_count(&self) -> usize {
        self.wave_grid.read().unwrap().active_memory_count()
    }

    /// Store wave at specific coordinates (public helper method)
    pub fn store_wave_at_coordinates(
        &mut self,
        x: u8,
        y: u8,
        z: u16,
        wave: MemoryWave,
    ) -> Result<()> {
        self.wave_grid.write().unwrap().store(x, y, z, wave);
        Ok(())
    }

    /// Helper to convert string to coordinates
    pub fn string_to_coordinates(&self, s: &str) -> (u8, u8) {
        let hash = self.simple_hash(s);
        ((hash & 0xFF) as u8, ((hash >> 8) & 0xFF) as u8)
    }
}

/// Directory metadata for memory creation
#[derive(Debug)]
pub struct DirectoryMetadata {
    pub primary_type: ContentType,
    pub importance: f32,      // 0.0 to 1.0
    pub normalized_size: f32, // 0.0 to 1.0
    pub health: DirectoryHealth,
    pub activity_level: f32, // 0.0 to 1.0
    pub days_since_modified: u32,
}

#[derive(Debug)]
pub enum ContentType {
    Code,
    Documentation,
    Configuration,
    Data,
    Media,
}

#[derive(Debug)]
pub enum DirectoryHealth {
    Healthy,
    Warning,
    Critical,
}

/// A memory associated with a path
#[derive(Debug)]
pub struct PathMemory {
    pub coordinates: (u8, u8, u16),
    pub wave: Arc<MemoryWave>,
    pub relevance: f32,
}

/// Directory events for reactive processing
#[derive(Debug)]
pub enum DirectoryEvent {
    LargeDirectory { path: String, size: u64 },
    SecurityThreat { path: String, severity: f32 },
    RapidChange { path: String, rate: f32 },
}

/// Example usage in Smart Tree
pub fn integrate_with_smart_tree() -> Result<()> {
    let mut mem8 = SmartTreeMem8::new();

    // Register reactive patterns
    mem8.register_directory_patterns();

    // Store a directory memory
    let metadata = DirectoryMetadata {
        primary_type: ContentType::Code,
        importance: 0.8,
        normalized_size: 0.6,
        health: DirectoryHealth::Healthy,
        activity_level: 0.7,
        days_since_modified: 5,
    };

    mem8.store_directory_memory(Path::new("src/lib.rs"), metadata)?;

    // Query memories
    let memories = mem8.query_path_memories("src");
    println!("Found {} memories related to 'src'", memories.len());

    // Process an event
    let event = DirectoryEvent::LargeDirectory {
        path: "node_modules".to_string(),
        size: 50_000_000,
    };

    if let Some(response) = mem8.process_directory_event(event) {
        println!(
            "Reactive response: {} ({}ms)",
            response.action,
            response.latency.as_millis()
        );
    }

    // Update consciousness
    mem8.update_consciousness();

    // Export to .m8 file
    let mut buffer = Vec::new();
    mem8.export_memories(&mut buffer)?;
    println!("Exported {} bytes of memories", buffer.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_tree_integration() {
        integrate_with_smart_tree().unwrap();
    }

    #[test]
    fn test_path_to_coordinates() {
        let mem8 = SmartTreeMem8::new();
        let (x1, y1) = mem8.path_to_coordinates(Path::new("src/main.rs"));
        let (x2, y2) = mem8.path_to_coordinates(Path::new("src/lib.rs"));

        // Different paths should map to different coordinates (usually)
        assert!(x1 != x2 || y1 != y2);
    }
}
