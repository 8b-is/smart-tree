// Original memory index functionality
pub mod memindex;

// MEM8 wave-based cognitive architecture
pub mod consciousness;
pub mod conversation;
pub mod developer_personas;
pub mod format;
pub mod git_temporal;
pub mod integration;
pub mod reactive;
pub mod safety;
pub mod simd;
pub mod wave;

// Re-export original memindex types with namespace
pub mod index {
    pub use super::memindex::*;
}

// Re-export MEM8 types
pub use consciousness::{ConsciousnessEngine, ConsciousnessState, SensorArbitrator};
pub use conversation::{
    ConversationAnalyzer, ConversationMemory, ConversationSummary, ConversationType,
};
pub use developer_personas::{DeveloperPersona, PersonaAnalyzer};
pub use format::{CompressedWave, M8Writer, MarkqantEncoder};
pub use git_temporal::{create_temporal_grooves, GitCommit, GitFileHistory, GitTemporalAnalyzer};
pub use integration::{DirectoryEvent, DirectoryMetadata, SmartTreeMem8};
pub use reactive::{ReactiveLayer, ReactiveMemory, ReactiveResponse, SensorInput};
pub use safety::{
    CollectiveEmotionalIntelligence, Custodian, DivergenceTracker, EmotionalMemoryTherapy,
    RepetitionPrevention, SafetySystem, TemporalBlanketRecovery,
};
pub use simd::{PerformanceBenchmark, SimdGridOps, SimdWaveProcessor};
pub use wave::{FrequencyBand, MemoryWave, WaveGrid};
