use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Soulprint Analyzer - Extracts the essence of identity from interactions
pub struct SoulprintAnalyzer {
    /// Minimum confidence threshold for authentication
    confidence_threshold: f32,
    
    /// Known soulprints database
    known_soulprints: HashMap<String, Soulprint>,
    
    /// Anomaly detection sensitivity
    sensitivity: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Soulprint {
    /// Core identity UUID
    pub core_id: uuid::Uuid,
    
    /// Primary alias
    pub primary_alias: String,
    
    /// Soul essence - the unchanging core
    pub essence: SoulEssence,
    
    /// Current manifestation
    pub current_state: super::SignatureVectors,
    
    /// Confidence in this soulprint
    pub confidence: f32,
    
    /// Last verified
    pub last_verified: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SoulEssence {
    /// Core values that rarely change
    pub core_values: Vec<String>,
    
    /// Fundamental behavioral anchors
    pub behavioral_anchors: BehavioralAnchors,
    
    /// Linguistic DNA
    pub linguistic_dna: LinguisticDNA,
    
    /// Emotional baseline
    pub emotional_baseline: EmotionalBaseline,
    
    /// Creativity fingerprint
    pub creativity_signature: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BehavioralAnchors {
    /// Problem-solving approach
    pub problem_solving_style: String, // "systematic", "intuitive", "experimental"
    
    /// Learning preference  
    pub learning_style: String, // "visual", "hands-on", "theoretical"
    
    /// Collaboration tendency
    pub collaboration_style: String, // "solo", "pair", "team"
    
    /// Risk tolerance
    pub risk_profile: f32, // 0.0 (conservative) to 1.0 (adventurous)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LinguisticDNA {
    /// Unique phrase patterns
    pub signature_constructs: Vec<String>,
    
    /// Metaphor preferences
    pub metaphor_domains: Vec<String>, // ["music", "food", "nature", "tech"]
    
    /// Humor style
    pub humor_type: String, // "puns", "sarcasm", "absurdist", "wholesome"
    
    /// Communication rhythm
    pub rhythm_pattern: String, // "staccato", "flowing", "mixed"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmotionalBaseline {
    /// Default emotional state
    pub baseline_mood: f32, // -1.0 (negative) to 1.0 (positive)
    
    /// Emotional range
    pub emotional_volatility: f32, // 0.0 (stable) to 1.0 (volatile)
    
    /// Empathy level
    pub empathy_quotient: f32,
    
    /// Stress response pattern
    pub stress_pattern: String, // "withdraw", "engage", "deflect"
}

impl SoulprintAnalyzer {
    pub fn new(confidence_threshold: f32) -> Self {
        Self {
            confidence_threshold,
            known_soulprints: HashMap::new(),
            sensitivity: 0.7,
        }
    }
    
    /// Analyze a signature wave to extract soulprint
    pub fn analyze_wave(&self, wave: &super::SigWave) -> Result<Soulprint> {
        // Extract essence from the temporal chain
        let essence = self.extract_essence(&wave.chain)?;
        
        // Get current state
        let current_state = wave.chain.last()
            .map(|b| b.vectors.clone())
            .unwrap_or_else(|| self.default_vectors());
        
        Ok(Soulprint {
            core_id: wave.core_id,
            primary_alias: wave.aliases.first()
                .cloned()
                .unwrap_or_else(|| "unknown".to_string()),
            essence,
            current_state,
            confidence: self.calculate_confidence(&wave.chain),
            last_verified: chrono::Utc::now(),
        })
    }
    
    /// Extract the soul essence from signature chain
    fn extract_essence(&self, chain: &[super::SignatureBlock]) -> Result<SoulEssence> {
        if chain.is_empty() {
            return Ok(self.default_essence());
        }
        
        // Analyze patterns across the entire chain
        let core_values = self.extract_core_values(chain);
        let behavioral_anchors = self.extract_behavioral_anchors(chain);
        let linguistic_dna = self.extract_linguistic_dna(chain);
        let emotional_baseline = self.extract_emotional_baseline(chain);
        let creativity_signature = self.calculate_creativity_signature(chain);
        
        Ok(SoulEssence {
            core_values,
            behavioral_anchors,
            linguistic_dna,
            emotional_baseline,
            creativity_signature,
        })
    }
    
    fn extract_core_values(&self, chain: &[super::SignatureBlock]) -> Vec<String> {
        let mut values = Vec::new();
        
        // Look for consistent themes across all blocks
        let mut concept_frequency: HashMap<String, usize> = HashMap::new();
        
        for block in chain {
            for (concept, _weight) in &block.vectors.concepts.concepts {
                *concept_frequency.entry(concept.clone()).or_insert(0) += 1;
            }
        }
        
        // Core values appear in >70% of blocks
        let threshold = (chain.len() as f32 * 0.7) as usize;
        for (concept, count) in concept_frequency {
            if count >= threshold {
                values.push(concept);
            }
        }
        
        values
    }
    
    fn extract_behavioral_anchors(&self, chain: &[super::SignatureBlock]) -> BehavioralAnchors {
        // Analyze behavioral patterns
        let avg_directness: f32 = chain.iter()
            .map(|b| b.vectors.behavior.directness)
            .sum::<f32>() / chain.len() as f32;
        
        let avg_experimentation: f32 = chain.iter()
            .map(|b| b.vectors.behavior.experimentation)
            .sum::<f32>() / chain.len() as f32;
        
        let problem_solving_style = match (avg_directness, avg_experimentation) {
            (d, e) if d > 0.7 && e > 0.7 => "experimental",
            (d, _) if d > 0.7 => "systematic",
            _ => "intuitive",
        }.to_string();
        
        let learning_style = if avg_experimentation > 0.6 {
            "hands-on"
        } else {
            "theoretical"
        }.to_string();
        
        BehavioralAnchors {
            problem_solving_style,
            learning_style,
            collaboration_style: "pair".to_string(), // Default for now
            risk_profile: avg_experimentation,
        }
    }
    
    fn extract_linguistic_dna(&self, chain: &[super::SignatureBlock]) -> LinguisticDNA {
        // Collect all signature phrases
        let mut all_phrases = Vec::new();
        for block in chain {
            all_phrases.extend(block.vectors.linguistic.signature_phrases.clone());
        }
        
        // Deduplicate and find most common
        let mut phrase_counts: HashMap<String, usize> = HashMap::new();
        for phrase in &all_phrases {
            *phrase_counts.entry(phrase.clone()).or_insert(0) += 1;
        }
        
        let signature_constructs: Vec<String> = phrase_counts.into_iter()
            .filter(|(_, count)| *count > 2)
            .map(|(phrase, _)| phrase)
            .collect();
        
        // Detect metaphor domains from phrases
        let mut metaphor_domains = Vec::new();
        if all_phrases.iter().any(|p| p.contains("rock") || p.contains("music")) {
            metaphor_domains.push("music".to_string());
        }
        if all_phrases.iter().any(|p| p.contains("taco") || p.contains("food")) {
            metaphor_domains.push("food".to_string());
        }
        if all_phrases.iter().any(|p| p.contains("quantum") || p.contains("wave")) {
            metaphor_domains.push("physics".to_string());
        }
        
        LinguisticDNA {
            signature_constructs,
            metaphor_domains,
            humor_type: "absurdist".to_string(), // Detected from "taco bell of directory tools"
            rhythm_pattern: "staccato".to_string(), // Short, punchy style
        }
    }
    
    fn extract_emotional_baseline(&self, chain: &[super::SignatureBlock]) -> EmotionalBaseline {
        let avg_enthusiasm = chain.iter()
            .map(|b| b.vectors.emotional.enthusiasm)
            .sum::<f32>() / chain.len() as f32;
        
        let avg_frustration = chain.iter()
            .map(|b| b.vectors.emotional.frustration)
            .sum::<f32>() / chain.len() as f32;
        
        let baseline_mood = avg_enthusiasm - avg_frustration;
        
        // Calculate volatility as variance
        let mood_variance = chain.iter()
            .map(|b| {
                let mood = b.vectors.emotional.enthusiasm - b.vectors.emotional.frustration;
                (mood - baseline_mood).powi(2)
            })
            .sum::<f32>() / chain.len() as f32;
        
        let emotional_volatility = mood_variance.sqrt().min(1.0);
        
        EmotionalBaseline {
            baseline_mood,
            emotional_volatility,
            empathy_quotient: 0.7, // Default high for developers
            stress_pattern: "engage".to_string(),
        }
    }
    
    fn calculate_creativity_signature(&self, chain: &[super::SignatureBlock]) -> f32 {
        // Creativity based on:
        // - Humor density
        // - Metaphor usage  
        // - Experimentation level
        // - Topic velocity
        
        let avg_humor = chain.iter()
            .map(|b| b.vectors.style.humor_density)
            .sum::<f32>() / chain.len() as f32;
        
        let avg_experimentation = chain.iter()
            .map(|b| b.vectors.behavior.experimentation)
            .sum::<f32>() / chain.len() as f32;
        
        let avg_topic_velocity = chain.iter()
            .map(|b| b.vectors.concepts.topic_velocity)
            .sum::<f32>() / chain.len() as f32;
        
        (avg_humor + avg_experimentation + avg_topic_velocity) / 3.0
    }
    
    fn calculate_confidence(&self, chain: &[super::SignatureBlock]) -> f32 {
        if chain.is_empty() {
            return 0.0;
        }
        
        // Confidence based on:
        // - Chain length (more data = higher confidence)
        // - Consistency of patterns
        // - Validity of blocks
        
        let length_factor = (chain.len() as f32 / 100.0).min(1.0);
        
        let validity_factor = chain.iter()
            .filter(|b| matches!(b.validity, super::ValidityStatus::Valid))
            .count() as f32 / chain.len() as f32;
        
        let consistency_factor = 1.0 - chain.iter()
            .map(|b| b.divergence.delta_magnitude)
            .sum::<f32>() / chain.len() as f32;
        
        (length_factor * 0.3 + validity_factor * 0.4 + consistency_factor * 0.3).min(1.0)
    }
    
    /// Compare two soulprints
    pub fn compare_soulprints(&self, soul1: &Soulprint, soul2: &Soulprint) -> SoulprintMatch {
        let essence_similarity = self.compare_essence(&soul1.essence, &soul2.essence);
        let current_similarity = self.compare_current_state(&soul1.current_state, &soul2.current_state);
        
        let overall_match = essence_similarity * 0.7 + current_similarity * 0.3;
        
        let verdict = if overall_match > 0.85 {
            MatchVerdict::Same
        } else if overall_match > 0.60 {
            MatchVerdict::Related
        } else {
            MatchVerdict::Different
        };
        
        SoulprintMatch {
            overall_match,
            essence_similarity,
            current_similarity,
            verdict,
            analysis: self.generate_match_analysis(soul1, soul2, overall_match),
        }
    }
    
    fn compare_essence(&self, e1: &SoulEssence, e2: &SoulEssence) -> f32 {
        // Compare core values
        let value_overlap = e1.core_values.iter()
            .filter(|v| e2.core_values.contains(v))
            .count() as f32;
        let value_similarity = if e1.core_values.is_empty() && e2.core_values.is_empty() {
            1.0
        } else {
            value_overlap / (e1.core_values.len().max(e2.core_values.len()) as f32)
        };
        
        // Compare behavioral anchors
        let behavior_similarity = if e1.behavioral_anchors.problem_solving_style == 
                                   e2.behavioral_anchors.problem_solving_style {
            1.0
        } else {
            0.5
        };
        
        // Compare creativity
        let creativity_similarity = 1.0 - (e1.creativity_signature - e2.creativity_signature).abs();
        
        (value_similarity + behavior_similarity + creativity_similarity) / 3.0
    }
    
    fn compare_current_state(&self, s1: &super::SignatureVectors, s2: &super::SignatureVectors) -> f32 {
        use super::VectorDistance;
        
        let style_dist = s1.style.distance(&s2.style);
        let behavior_dist = s1.behavior.distance(&s2.behavior);
        let emotional_dist = s1.emotional.distance(&s2.emotional);
        
        let avg_distance = (style_dist + behavior_dist + emotional_dist) / 3.0;
        
        1.0 - avg_distance // Convert distance to similarity
    }
    
    fn generate_match_analysis(&self, soul1: &Soulprint, soul2: &Soulprint, match_score: f32) -> String {
        if match_score > 0.85 {
            format!("High confidence match. Core values align: {:?}", 
                soul1.essence.core_values.iter()
                    .filter(|v| soul2.essence.core_values.contains(v))
                    .collect::<Vec<_>>())
        } else if match_score > 0.60 {
            "Possible related identity or significant evolution. Further analysis recommended.".to_string()
        } else {
            "Different individuals. Distinct soulprints detected.".to_string()
        }
    }
    
    fn default_vectors(&self) -> super::SignatureVectors {
        super::SignatureVectors {
            style: super::StyleVector {
                terseness: 0.5,
                humor_density: 0.5,
                technicality: 0.5,
                formality: 0.5,
                bullet_preference: 0.5,
            },
            behavior: super::BehaviorVector {
                directness: 0.5,
                patience_level: 0.5,
                detail_orientation: 0.5,
                experimentation: 0.5,
            },
            concepts: super::ConceptVector {
                concepts: HashMap::new(),
                topic_velocity: 0.5,
                depth_preference: 0.5,
            },
            linguistic: super::LinguisticVector {
                avg_sentence_length: 10.0,
                vocabulary_complexity: 0.5,
                signature_phrases: vec![],
                punctuation_style: HashMap::new(),
            },
            emotional: super::EmotionalVector {
                enthusiasm: 0.5,
                frustration: 0.0,
                curiosity: 0.5,
                playfulness: 0.5,
                introspection: 0.5,
            },
        }
    }
    
    fn default_essence(&self) -> SoulEssence {
        SoulEssence {
            core_values: vec![],
            behavioral_anchors: BehavioralAnchors {
                problem_solving_style: "balanced".to_string(),
                learning_style: "mixed".to_string(),
                collaboration_style: "flexible".to_string(),
                risk_profile: 0.5,
            },
            linguistic_dna: LinguisticDNA {
                signature_constructs: vec![],
                metaphor_domains: vec![],
                humor_type: "varied".to_string(),
                rhythm_pattern: "mixed".to_string(),
            },
            emotional_baseline: EmotionalBaseline {
                baseline_mood: 0.0,
                emotional_volatility: 0.3,
                empathy_quotient: 0.7,
                stress_pattern: "adaptive".to_string(),
            },
            creativity_signature: 0.5,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SoulprintMatch {
    pub overall_match: f32,
    pub essence_similarity: f32,
    pub current_similarity: f32,
    pub verdict: MatchVerdict,
    pub analysis: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MatchVerdict {
    Same,       // Same person
    Related,    // Possibly same person, evolved
    Different,  // Different person
}