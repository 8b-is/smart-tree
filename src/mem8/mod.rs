// Original memory index functionality  
pub mod memindex;

// MEM8 wave-based cognitive architecture
pub mod wave;
pub mod reactive;
pub mod consciousness;
pub mod format;
pub mod integration;

// Re-export original memindex types with namespace
pub mod index {
    pub use super::memindex::*;
}

// Re-export MEM8 types
pub use wave::{MemoryWave, WaveGrid, FrequencyBand};
pub use reactive::{ReactiveLayer, ReactiveMemory, ReactiveResponse, SensorInput};
pub use consciousness::{ConsciousnessEngine, ConsciousnessState, SensorArbitrator};
pub use format::{M8Writer, CompressedWave, MarkqantEncoder};
pub use integration::{SmartTreeMem8, DirectoryMetadata, DirectoryEvent};