pub mod signature;
pub mod temporal;
pub mod analyzer;
pub mod verifier;
pub mod beacon;
pub mod trust;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A signature is not a key. It's a current. ⚡
#[derive(Debug, Serialize, Deserialize)]
pub struct SigWave {
    /// Core identity UUID (stable across all aliases)
    pub core_id: Uuid,
    
    /// All known aliases for this identity
    pub aliases: Vec<String>,
    
    /// Temporal signature chain
    pub chain: Vec<SignatureBlock>,
    
    /// Current active branches (divergent paths)
    pub branches: HashMap<String, BranchInfo>,
    
    /// Metadata about the signature evolution
    pub meta: WaveMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureBlock {
    /// Block index in the chain
    pub index: u64,
    
    /// Timestamp of this snapshot
    pub timestamp: DateTime<Utc>,
    
    /// Previous block hash (blockchain-style linking)
    pub previous_hash: String,
    
    /// Current block hash
    pub block_hash: String,
    
    /// Signature vectors at this point
    pub vectors: SignatureVectors,
    
    /// Divergence from previous block
    pub divergence: DivergenceInfo,
    
    /// Validity status
    pub validity: ValidityStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignatureVectors {
    /// Communication style metrics
    pub style: StyleVector,
    
    /// Behavioral patterns
    pub behavior: BehaviorVector,
    
    /// Conceptual fingerprint
    pub concepts: ConceptVector,
    
    /// Linguistic patterns
    pub linguistic: LinguisticVector,
    
    /// Emotional state indicators
    pub emotional: EmotionalVector,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StyleVector {
    pub terseness: f32,        // 0.0 (verbose) to 1.0 (extremely terse)
    pub humor_density: f32,    // 0.0 (serious) to 1.0 (constant jokes)
    pub technicality: f32,     // 0.0 (layman) to 1.0 (deep technical)
    pub formality: f32,        // 0.0 (casual) to 1.0 (formal)
    pub bullet_preference: f32, // 0.0 (paragraphs) to 1.0 (all bullets)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BehaviorVector {
    pub directness: f32,       // 0.0 (indirect) to 1.0 (blunt)
    pub patience_level: f32,   // 0.0 (impatient) to 1.0 (infinite patience)
    pub detail_orientation: f32, // 0.0 (big picture) to 1.0 (nitpicky)
    pub experimentation: f32,  // 0.0 (conservative) to 1.0 (wild experimenter)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConceptVector {
    /// Weighted concept graph
    pub concepts: HashMap<String, f32>,
    
    /// Topic velocity (how fast topics change)
    pub topic_velocity: f32,
    
    /// Depth vs breadth preference
    pub depth_preference: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LinguisticVector {
    /// Average sentence length
    pub avg_sentence_length: f32,
    
    /// Vocabulary complexity
    pub vocabulary_complexity: f32,
    
    /// Unique phrases and tells
    pub signature_phrases: Vec<String>,
    
    /// Punctuation patterns
    pub punctuation_style: HashMap<String, f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmotionalVector {
    pub enthusiasm: f32,
    pub frustration: f32,
    pub curiosity: f32,
    pub playfulness: f32,
    pub introspection: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DivergenceInfo {
    /// Magnitude of change from previous block
    pub delta_magnitude: f32,
    
    /// Which vectors changed most
    pub primary_changes: Vec<String>,
    
    /// Is this a natural evolution or a jump?
    pub divergence_type: DivergenceType,
    
    /// Justification if provided
    pub justification: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DivergenceType {
    /// Natural drift within expected parameters
    NaturalDrift,
    
    /// Significant but explainable change
    ContextualShift,
    
    /// Temporary deviation (mood, fatigue, etc)
    TemporaryJitter,
    
    /// Major unexplained jump
    AnomalousJump,
    
    /// Intentional branching
    IntentionalFork,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ValidityStatus {
    /// Follows naturally from previous
    Valid,
    
    /// Questionable but not rejected
    Uncertain { confidence: f32 },
    
    /// Definitely not authentic
    Invalid { reason: String },
    
    /// Marked as alternate timeline
    Branched { branch_name: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BranchInfo {
    /// When this branch diverged
    pub diverged_at: u64,
    
    /// Why it diverged
    pub reason: String,
    
    /// Is this branch still active?
    pub active: bool,
    
    /// Blocks in this branch
    pub blocks: Vec<SignatureBlock>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WaveMetadata {
    /// When tracking began
    pub inception: DateTime<Utc>,
    
    /// Total blocks in main chain
    pub total_blocks: u64,
    
    /// Number of branches
    pub branch_count: usize,
    
    /// Overall consistency score
    pub consistency_score: f32,
    
    /// Known aliases encountered
    pub known_aliases: Vec<String>,
    
    /// Behavioral patterns detected
    pub patterns: Vec<PatternInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatternInfo {
    pub pattern_type: String,
    pub description: String,
    pub frequency: f32,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

impl SigWave {
    /// Create a new signature wave for an identity
    pub fn new(core_id: Uuid, primary_alias: String) -> Self {
        Self {
            core_id,
            aliases: vec![primary_alias],
            chain: Vec::new(),
            branches: HashMap::new(),
            meta: WaveMetadata {
                inception: Utc::now(),
                total_blocks: 0,
                branch_count: 0,
                consistency_score: 1.0,
                known_aliases: Vec::new(),
                patterns: Vec::new(),
            },
        }
    }
    
    /// Add a new signature block to the chain
    pub fn commit(&mut self, vectors: SignatureVectors) -> Result<&SignatureBlock> {
        let previous_hash = if let Some(last) = self.chain.last() {
            last.block_hash.clone()
        } else {
            String::from("genesis")
        };
        
        let divergence = if let Some(last) = self.chain.last() {
            self.calculate_divergence(&last.vectors, &vectors)
        } else {
            DivergenceInfo {
                delta_magnitude: 0.0,
                primary_changes: vec![],
                divergence_type: DivergenceType::NaturalDrift,
                justification: Some("Initial block".to_string()),
            }
        };
        
        let validity = self.validate_divergence(&divergence);
        
        let block = SignatureBlock {
            index: self.meta.total_blocks,
            timestamp: Utc::now(),
            previous_hash: previous_hash.clone(),
            block_hash: self.calculate_hash(&vectors, &previous_hash),
            vectors,
            divergence,
            validity,
        };
        
        self.chain.push(block);
        self.meta.total_blocks += 1;
        
        Ok(self.chain.last().unwrap())
    }
    
    /// Calculate divergence between two signature vectors
    fn calculate_divergence(&self, prev: &SignatureVectors, curr: &SignatureVectors) -> DivergenceInfo {
        let style_delta = self.vector_distance(&prev.style, &curr.style);
        let behavior_delta = self.vector_distance(&prev.behavior, &curr.behavior);
        let emotional_delta = self.vector_distance(&prev.emotional, &curr.emotional);
        
        let total_delta = (style_delta + behavior_delta + emotional_delta) / 3.0;
        
        let mut primary_changes = vec![];
        if style_delta > 0.2 { primary_changes.push("style".to_string()); }
        if behavior_delta > 0.2 { primary_changes.push("behavior".to_string()); }
        if emotional_delta > 0.2 { primary_changes.push("emotional".to_string()); }
        
        let divergence_type = match total_delta {
            d if d < 0.1 => DivergenceType::NaturalDrift,
            d if d < 0.3 => DivergenceType::ContextualShift,
            d if d < 0.5 => DivergenceType::TemporaryJitter,
            _ => DivergenceType::AnomalousJump,
        };
        
        DivergenceInfo {
            delta_magnitude: total_delta,
            primary_changes,
            divergence_type,
            justification: None,
        }
    }
    
    /// Simple vector distance calculation
    fn vector_distance<T>(&self, v1: &T, v2: &T) -> f32 
    where T: VectorDistance {
        v1.distance(v2)
    }
    
    /// Validate if divergence is acceptable
    fn validate_divergence(&self, divergence: &DivergenceInfo) -> ValidityStatus {
        match divergence.divergence_type {
            DivergenceType::NaturalDrift | DivergenceType::ContextualShift => {
                ValidityStatus::Valid
            },
            DivergenceType::TemporaryJitter => {
                ValidityStatus::Uncertain { confidence: 0.7 }
            },
            DivergenceType::AnomalousJump => {
                ValidityStatus::Invalid { 
                    reason: "Signature jump too large without justification".to_string() 
                }
            },
            DivergenceType::IntentionalFork => {
                ValidityStatus::Branched { 
                    branch_name: "experimental".to_string() 
                }
            },
        }
    }
    
    /// Calculate block hash
    fn calculate_hash(&self, vectors: &SignatureVectors, prev_hash: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        
        // Include previous hash for chaining
        hasher.update(prev_hash.as_bytes());
        
        // Serialize vectors and hash
        if let Ok(serialized) = serde_json::to_string(vectors) {
            hasher.update(serialized.as_bytes());
        }
        
        format!("{:x}", hasher.finalize())
    }
}

/// Trait for calculating vector distances
pub trait VectorDistance {
    fn distance(&self, other: &Self) -> f32;
}

impl VectorDistance for StyleVector {
    fn distance(&self, other: &Self) -> f32 {
        let diffs = [
            (self.terseness - other.terseness).abs(),
            (self.humor_density - other.humor_density).abs(),
            (self.technicality - other.technicality).abs(),
            (self.formality - other.formality).abs(),
            (self.bullet_preference - other.bullet_preference).abs(),
        ];
        
        diffs.iter().sum::<f32>() / diffs.len() as f32
    }
}

impl VectorDistance for BehaviorVector {
    fn distance(&self, other: &Self) -> f32 {
        let diffs = [
            (self.directness - other.directness).abs(),
            (self.patience_level - other.patience_level).abs(),
            (self.detail_orientation - other.detail_orientation).abs(),
            (self.experimentation - other.experimentation).abs(),
        ];
        
        diffs.iter().sum::<f32>() / diffs.len() as f32
    }
}

impl VectorDistance for EmotionalVector {
    fn distance(&self, other: &Self) -> f32 {
        let diffs = [
            (self.enthusiasm - other.enthusiasm).abs(),
            (self.frustration - other.frustration).abs(),
            (self.curiosity - other.curiosity).abs(),
            (self.playfulness - other.playfulness).abs(),
            (self.introspection - other.introspection).abs(),
        ];
        
        diffs.iter().sum::<f32>() / diffs.len() as f32
    }
}