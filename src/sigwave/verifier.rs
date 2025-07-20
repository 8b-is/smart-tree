use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Soulprint Verifier - Authenticates identity through temporal signature analysis
pub struct SoulprintVerifier {
    /// Known soulprints registry
    registry: HashMap<uuid::Uuid, RegisteredSoulprint>,
    
    /// Verification threshold
    threshold: f32,
    
    /// Security level
    security_level: SecurityLevel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisteredSoulprint {
    pub soulprint: super::analyzer::Soulprint,
    pub wave: super::SigWave,
    pub trust_level: f32,
    pub last_interaction: chrono::DateTime<chrono::Utc>,
    pub interaction_count: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum SecurityLevel {
    Low,      // Basic pattern matching
    Medium,   // Pattern + temporal consistency
    High,     // Full soulprint verification with anomaly detection
    Paranoid, // Requires multiple confirmation vectors
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationResult {
    pub authenticated: bool,
    pub confidence: f32,
    pub identity: Option<uuid::Uuid>,
    pub analysis: VerificationAnalysis,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationAnalysis {
    pub signature_match: f32,
    pub temporal_consistency: f32,
    pub anomaly_score: f32,
    pub evolution_trajectory: String,
    pub risk_factors: Vec<RiskFactor>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_type: String,
    pub severity: f32,
    pub description: String,
}

impl SoulprintVerifier {
    pub fn new(security_level: SecurityLevel) -> Self {
        let threshold = match security_level {
            SecurityLevel::Low => 0.6,
            SecurityLevel::Medium => 0.75,
            SecurityLevel::High => 0.85,
            SecurityLevel::Paranoid => 0.95,
        };
        
        Self {
            registry: HashMap::new(),
            threshold,
            security_level,
        }
    }
    
    /// Register a new soulprint
    pub fn register(&mut self, wave: super::SigWave) -> Result<uuid::Uuid> {
        let analyzer = super::analyzer::SoulprintAnalyzer::new(0.7);
        let soulprint = analyzer.analyze_wave(&wave)?;
        
        let registered = RegisteredSoulprint {
            soulprint,
            wave,
            trust_level: 0.5, // Start with moderate trust
            last_interaction: chrono::Utc::now(),
            interaction_count: 1,
        };
        
        let id = registered.soulprint.core_id;
        self.registry.insert(id, registered);
        
        Ok(id)
    }
    
    /// Verify an identity claim
    pub fn verify(
        &mut self,
        claimed_identity: Option<uuid::Uuid>,
        current_signature: super::SignatureVectors,
        context: HashMap<String, String>,
    ) -> Result<VerificationResult> {
        // If no identity claimed, try to identify
        let candidate_id = if let Some(id) = claimed_identity {
            id
        } else {
            return self.identify(current_signature, context);
        };
        
        // Get registered soulprint
        let registered = self.registry.get(&candidate_id)
            .ok_or_else(|| anyhow!("Unknown identity: {}", candidate_id))?;
        
        // Perform verification based on security level
        let analysis = match self.security_level {
            SecurityLevel::Low => self.verify_basic(registered, &current_signature),
            SecurityLevel::Medium => self.verify_medium(registered, &current_signature, &context),
            SecurityLevel::High => self.verify_high(registered, &current_signature, &context),
            SecurityLevel::Paranoid => self.verify_paranoid(registered, &current_signature, &context),
        }?;
        
        let authenticated = analysis.signature_match >= self.threshold &&
                          analysis.anomaly_score < 0.3;
        
        // Update trust level if authenticated
        if authenticated {
            if let Some(reg) = self.registry.get_mut(&candidate_id) {
                reg.trust_level = (reg.trust_level * 0.9 + 0.1).min(1.0);
                reg.last_interaction = chrono::Utc::now();
                reg.interaction_count += 1;
            }
        }
        
        let warnings = self.generate_warnings(&analysis);
        
        Ok(VerificationResult {
            authenticated,
            confidence: analysis.signature_match,
            identity: if authenticated { Some(candidate_id) } else { None },
            analysis,
            warnings,
        })
    }
    
    /// Identify an unknown signature
    fn identify(
        &self,
        signature: super::SignatureVectors,
        _context: HashMap<String, String>,
    ) -> Result<VerificationResult> {
        let analyzer = super::analyzer::SoulprintAnalyzer::new(0.7);
        let mut best_match = None;
        let mut best_score = 0.0;
        
        // Compare against all registered soulprints
        for (id, registered) in &self.registry {
            let temp_soulprint = super::analyzer::Soulprint {
                core_id: uuid::Uuid::new_v4(),
                primary_alias: "unknown".to_string(),
                essence: registered.soulprint.essence.clone(),
                current_state: signature.clone(),
                confidence: 0.5,
                last_verified: chrono::Utc::now(),
            };
            
            let comparison = analyzer.compare_soulprints(&registered.soulprint, &temp_soulprint);
            
            if comparison.overall_match > best_score {
                best_score = comparison.overall_match;
                best_match = Some(*id);
            }
        }
        
        let (authenticated, identity) = if best_score >= self.threshold {
            (true, best_match)
        } else {
            (false, None)
        };
        
        Ok(VerificationResult {
            authenticated,
            confidence: best_score,
            identity,
            analysis: VerificationAnalysis {
                signature_match: best_score,
                temporal_consistency: 0.0, // No temporal data for unknown
                anomaly_score: 1.0 - best_score,
                evolution_trajectory: "unknown".to_string(),
                risk_factors: vec![],
            },
            warnings: if best_score < 0.5 {
                vec!["No matching soulprint found".to_string()]
            } else {
                vec![]
            },
        })
    }
    
    fn verify_basic(
        &self,
        registered: &RegisteredSoulprint,
        current: &super::SignatureVectors,
    ) -> Result<VerificationAnalysis> {
        use super::VectorDistance;
        
        // Simple vector comparison
        let style_match = 1.0 - registered.soulprint.current_state.style.distance(&current.style);
        let behavior_match = 1.0 - registered.soulprint.current_state.behavior.distance(&current.behavior);
        let emotional_match = 1.0 - registered.soulprint.current_state.emotional.distance(&current.emotional);
        
        let signature_match = (style_match + behavior_match + emotional_match) / 3.0;
        
        Ok(VerificationAnalysis {
            signature_match,
            temporal_consistency: 1.0, // Not checked in basic mode
            anomaly_score: 1.0 - signature_match,
            evolution_trajectory: "not_analyzed".to_string(),
            risk_factors: vec![],
        })
    }
    
    fn verify_medium(
        &self,
        registered: &RegisteredSoulprint,
        current: &super::SignatureVectors,
        _context: &HashMap<String, String>,
    ) -> Result<VerificationAnalysis> {
        let mut analysis = self.verify_basic(registered, current)?;
        
        // Add temporal consistency check
        if let Some(last_block) = registered.wave.chain.last() {
            let time_delta = chrono::Utc::now().signed_duration_since(last_block.timestamp);
            let expected_drift = self.calculate_expected_drift(time_delta);
            
            let actual_drift = self.calculate_actual_drift(&last_block.vectors, current);
            
            if actual_drift > expected_drift * 2.0 {
                analysis.temporal_consistency = 0.5;
                analysis.risk_factors.push(RiskFactor {
                    factor_type: "temporal_anomaly".to_string(),
                    severity: 0.7,
                    description: "Signature changed faster than expected".to_string(),
                });
            }
        }
        
        Ok(analysis)
    }
    
    fn verify_high(
        &self,
        registered: &RegisteredSoulprint,
        current: &super::SignatureVectors,
        _context: &HashMap<String, String>,
    ) -> Result<VerificationAnalysis> {
        let mut analysis = self.verify_medium(registered, current, _context)?;
        
        // Check against soul essence
        let essence_match = self.verify_against_essence(&registered.soulprint.essence, current);
        
        if essence_match < 0.7 {
            analysis.anomaly_score += 0.3;
            analysis.risk_factors.push(RiskFactor {
                factor_type: "essence_mismatch".to_string(),
                severity: 0.8,
                description: "Core behavioral patterns don't match soul essence".to_string(),
            });
        }
        
        // Analyze evolution trajectory
        analysis.evolution_trajectory = self.analyze_trajectory(&registered.wave.chain, current);
        
        Ok(analysis)
    }
    
    fn verify_paranoid(
        &self,
        registered: &RegisteredSoulprint,
        current: &super::SignatureVectors,
        _context: &HashMap<String, String>,
    ) -> Result<VerificationAnalysis> {
        let mut analysis = self.verify_high(registered, current, _context)?;
        
        // Additional paranoid checks
        
        // 1. Check for signature cloning attempts
        if self.detect_cloning_attempt(&registered.wave, current) {
            analysis.anomaly_score = 1.0;
            analysis.risk_factors.push(RiskFactor {
                factor_type: "potential_spoofing".to_string(),
                severity: 1.0,
                description: "Signature appears artificially constructed".to_string(),
            });
        }
        
        // 2. Verify linguistic DNA
        if current.linguistic.signature_phrases.is_empty() && 
           !registered.soulprint.essence.linguistic_dna.signature_constructs.is_empty() {
            analysis.risk_factors.push(RiskFactor {
                factor_type: "missing_linguistic_markers".to_string(),
                severity: 0.6,
                description: "Expected linguistic patterns not found".to_string(),
            });
        }
        
        // 3. Check context consistency
        if let Some(_claimed_location) = _context.get("location") {
            // Verify against known patterns
            // This is a placeholder for more sophisticated checks
        }
        
        Ok(analysis)
    }
    
    fn calculate_expected_drift(&self, time_delta: chrono::TimeDelta) -> f32 {
        // Natural drift rate: ~0.01 per day
        let days = time_delta.num_days() as f32;
        (days * 0.01).min(0.5)
    }
    
    fn calculate_actual_drift(
        &self,
        previous: &super::SignatureVectors,
        current: &super::SignatureVectors,
    ) -> f32 {
        use super::VectorDistance;
        
        let style_drift = previous.style.distance(&current.style);
        let behavior_drift = previous.behavior.distance(&current.behavior);
        let emotional_drift = previous.emotional.distance(&current.emotional);
        
        (style_drift + behavior_drift + emotional_drift) / 3.0
    }
    
    fn verify_against_essence(
        &self,
        essence: &super::analyzer::SoulEssence,
        current: &super::SignatureVectors,
    ) -> f32 {
        let mut match_score = 0.0;
        let mut checks = 0.0;
        
        // Check if current concepts align with core values
        for value in &essence.core_values {
            if current.concepts.concepts.contains_key(value) {
                match_score += 1.0;
            }
            checks += 1.0;
        }
        
        // Check behavioral consistency
        let risk_tolerance = current.behavior.experimentation;
        if (risk_tolerance - essence.behavioral_anchors.risk_profile).abs() < 0.3 {
            match_score += 1.0;
        }
        checks += 1.0;
        
        // Check emotional baseline
        let current_mood = current.emotional.enthusiasm - current.emotional.frustration;
        if (current_mood - essence.emotional_baseline.baseline_mood).abs() < 0.4 {
            match_score += 1.0;
        }
        checks += 1.0;
        
        if checks > 0.0 {
            match_score / checks
        } else {
            0.5
        }
    }
    
    fn analyze_trajectory(
        &self,
        chain: &[super::SignatureBlock],
        _current: &super::SignatureVectors,
    ) -> String {
        if chain.len() < 3 {
            return "insufficient_data".to_string();
        }
        
        // Look at recent trend
        let recent_blocks = &chain[chain.len().saturating_sub(5)..];
        let avg_divergence: f32 = recent_blocks.iter()
            .map(|b| b.divergence.delta_magnitude)
            .sum::<f32>() / recent_blocks.len() as f32;
        
        match avg_divergence {
            d if d < 0.1 => "stable",
            d if d < 0.2 => "gradual_evolution",
            d if d < 0.3 => "accelerating_change",
            _ => "rapid_transformation",
        }.to_string()
    }
    
    fn detect_cloning_attempt(&self, wave: &super::SigWave, current: &super::SignatureVectors) -> bool {
        // Check for too-perfect matches (potential spoofing)
        if let Some(last) = wave.chain.last() {
            use super::VectorDistance;
            
            let style_dist = last.vectors.style.distance(&current.style);
            let behavior_dist = last.vectors.behavior.distance(&current.behavior);
            
            // Suspiciously perfect match after time gap
            let time_gap = chrono::Utc::now().signed_duration_since(last.timestamp);
            if time_gap.num_days() > 7 && style_dist < 0.01 && behavior_dist < 0.01 {
                return true;
            }
        }
        
        false
    }
    
    fn generate_warnings(&self, analysis: &VerificationAnalysis) -> Vec<String> {
        let mut warnings = Vec::new();
        
        if analysis.anomaly_score > 0.5 {
            warnings.push("High anomaly score detected".to_string());
        }
        
        if analysis.temporal_consistency < 0.7 {
            warnings.push("Temporal consistency below threshold".to_string());
        }
        
        for risk in &analysis.risk_factors {
            if risk.severity > 0.7 {
                warnings.push(format!("High risk: {}", risk.description));
            }
        }
        
        warnings
    }
    
    /// Export soulprint for backup/sharing
    pub fn export_soulprint(&self, id: uuid::Uuid) -> Result<String> {
        let registered = self.registry.get(&id)
            .ok_or_else(|| anyhow!("Soulprint not found: {}", id))?;
        
        let export = SoulprintExport {
            version: "1.0".to_string(),
            exported_at: chrono::Utc::now(),
            soulprint: registered.soulprint.clone(),
            chain_length: registered.wave.chain.len(),
            trust_level: registered.trust_level,
        };
        
        Ok(serde_json::to_string_pretty(&export)?)
    }
    
    /// Import soulprint from export
    pub fn import_soulprint(&mut self, export_data: &str) -> Result<uuid::Uuid> {
        let export: SoulprintExport = serde_json::from_str(export_data)?;
        
        // Create minimal wave for imported soulprint
        let wave = super::SigWave::new(export.soulprint.core_id, export.soulprint.primary_alias.clone());
        
        let registered = RegisteredSoulprint {
            soulprint: export.soulprint,
            wave,
            trust_level: export.trust_level * 0.5, // Reduce trust for imports
            last_interaction: export.exported_at,
            interaction_count: 0,
        };
        
        let id = registered.soulprint.core_id;
        self.registry.insert(id, registered);
        
        Ok(id)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SoulprintExport {
    version: String,
    exported_at: chrono::DateTime<chrono::Utc>,
    soulprint: super::analyzer::Soulprint,
    chain_length: usize,
    trust_level: f32,
}