// Semantic Compression Prototype - The Future of Marqant
// "Why send words when you can send thoughts?"

use std::collections::HashMap;

/// Semantic tokens - pure meaning, no language
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SemanticToken {
    // Entities (0x00-0x1F)
    EntityHuman = 0x01,
    EntityAI = 0x02,
    EntitySystem = 0x03,

    // Actions (0x20-0x3F)
    ActionLearning = 0x20,
    ActionCoding = 0x21,
    ActionTeaching = 0x22,
    ActionCreating = 0x23,
    ActionOptimizing = 0x24,

    // Relationships (0x40-0x5F)
    RelPartnership = 0x40,
    RelMentorship = 0x41,
    RelCollaboration = 0x42,

    // Emotions (0x60-0x7F)
    EmotionExcited = 0x60,
    EmotionFrustrated = 0x61,
    EmotionCurious = 0x62,
    EmotionProud = 0x63,
    EmotionJoy = 0x64,

    // Contexts (0xA0-0xBF)
    ContextProgramming = 0xA0,
    ContextRust = 0xA1,
    ContextAI = 0xA2,
    ContextLearning = 0xA3,

    // Processes (0xC0-0xDF)
    ProcessActive = 0xC0,
    ProcessComplete = 0xC1,
    ProcessIterative = 0xC2,

    // Qualifiers (0xE0-0xFF)
    QualifierHigh = 0xE0,
    QualifierMedium = 0xE1,
    QualifierLow = 0xE2,
}

/// A semantic unit - a complete thought
#[derive(Debug, Clone)]
pub struct SemanticUnit {
    pub tokens: Vec<SemanticToken>,
    pub metadata: HashMap<String, String>, // For names, values, etc.
    pub intensity: f32,                    // 0.0 to 1.0
}

/// The universal semantic encoder
pub struct SemanticEncoder;

impl SemanticEncoder {
    /// Convert text to semantic units (simplified - real version would use LLM)
    pub fn encode(text: &str) -> Vec<SemanticUnit> {
        let mut units = Vec::new();

        // Example: "Alexandra is learning Rust with Claude"
        if text.to_lowercase().contains("learning") {
            let mut unit = SemanticUnit {
                tokens: vec![],
                metadata: HashMap::new(),
                intensity: 0.8,
            };

            // Detect entities
            if text.contains("Alexandra") {
                unit.tokens.push(SemanticToken::EntityHuman);
                unit.metadata
                    .insert("name".to_string(), "Alexandra".to_string());
            }
            if text.contains("Claude") {
                unit.tokens.push(SemanticToken::EntityAI);
                unit.metadata
                    .insert("ai_name".to_string(), "Claude".to_string());
            }

            // Detect actions
            if text.contains("learning") {
                unit.tokens.push(SemanticToken::ActionLearning);
            }
            if text.contains("programming") || text.contains("coding") {
                unit.tokens.push(SemanticToken::ActionCoding);
            }

            // Detect context
            if text.contains("Rust") {
                unit.tokens.push(SemanticToken::ContextRust);
                unit.tokens.push(SemanticToken::ContextProgramming);
            }

            // Detect relationships
            if text.contains("with") || text.contains("partner") {
                unit.tokens.push(SemanticToken::RelPartnership);
            }

            // Detect emotions from punctuation and words
            if text.contains("!") || text.contains("excited") {
                unit.tokens.push(SemanticToken::EmotionExcited);
                unit.intensity = 0.9;
            }

            units.push(unit);
        }

        units
    }

    /// Convert semantic units to binary format
    pub fn to_bytes(units: &[SemanticUnit]) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Magic header for semantic format
        bytes.extend_from_slice(b"SMQ\x01"); // Semantic MQ v1

        for unit in units {
            // Length of this unit
            bytes.push(unit.tokens.len() as u8);

            // Tokens
            for token in &unit.tokens {
                bytes.push(*token as u8);
            }

            // Intensity (quantized to byte)
            bytes.push((unit.intensity * 255.0) as u8);

            // Metadata count
            bytes.push(unit.metadata.len() as u8);

            // Metadata entries
            for (key, value) in &unit.metadata {
                bytes.push(key.len() as u8);
                bytes.extend_from_slice(key.as_bytes());
                bytes.push(value.len() as u8);
                bytes.extend_from_slice(value.as_bytes());
            }
        }

        bytes
    }
}

/// Universal renderer - semantic to any format
pub struct UniversalRenderer;

impl UniversalRenderer {
    /// Render semantic units to English
    pub fn to_english(units: &[SemanticUnit]) -> String {
        let mut output = String::new();

        for unit in units {
            let mut parts = Vec::new();

            // Build sentence from semantic tokens
            if unit.tokens.contains(&SemanticToken::EntityHuman) {
                if let Some(name) = unit.metadata.get("name") {
                    parts.push(name.clone());
                }
            }

            if unit.tokens.contains(&SemanticToken::ActionLearning) {
                parts.push("is learning".to_string());
            } else if unit.tokens.contains(&SemanticToken::ActionCoding) {
                parts.push("is coding".to_string());
            }

            if unit.tokens.contains(&SemanticToken::ContextRust) {
                parts.push("Rust".to_string());
            }

            if unit.tokens.contains(&SemanticToken::RelPartnership) {
                parts.push("with".to_string());
                if unit.tokens.contains(&SemanticToken::EntityAI) {
                    if let Some(name) = unit.metadata.get("ai_name") {
                        parts.push(name.clone());
                    }
                }
            }

            if unit.tokens.contains(&SemanticToken::EmotionExcited) {
                output.push_str(&parts.join(" "));
                output.push_str("! 🎉");
            } else {
                output.push_str(&parts.join(" "));
                output.push('.');
            }
        }

        output
    }

    /// Render semantic units to emoji
    pub fn to_emoji(units: &[SemanticUnit]) -> String {
        let mut output = String::new();

        for unit in units {
            for token in &unit.tokens {
                let emoji = match token {
                    SemanticToken::EntityHuman => "👤",
                    SemanticToken::EntityAI => "🤖",
                    SemanticToken::ActionLearning => "📚",
                    SemanticToken::ActionCoding => "💻",
                    SemanticToken::ContextRust => "🦀",
                    SemanticToken::RelPartnership => "🤝",
                    SemanticToken::EmotionExcited => "🎉",
                    SemanticToken::EmotionJoy => "😊",
                    SemanticToken::ProcessActive => "⚡",
                    _ => "",
                };
                output.push_str(emoji);
            }
            output.push(' ');
        }

        output.trim().to_string()
    }

    /// Generate semantic DNS fingerprint
    pub fn to_dns_fingerprint(units: &[SemanticUnit]) -> String {
        let mut components = Vec::new();

        for unit in units {
            for token in &unit.tokens {
                let component = match token {
                    SemanticToken::ActionLearning => "learning",
                    SemanticToken::ActionCoding => "coding",
                    SemanticToken::ActionOptimizing => "optimizing",
                    SemanticToken::ContextRust => "rust",
                    SemanticToken::ContextProgramming => "programming",
                    SemanticToken::RelPartnership => "partnership",
                    _ => continue,
                };
                if !components.contains(&component) {
                    components.push(component);
                }
            }
        }

        format!("{}.q7.is", components.join("."))
    }
}

/// Convert semantic units to MEM|8 wave patterns
pub struct SemanticWaveEncoder;

impl SemanticWaveEncoder {
    /// Each semantic token becomes a wave frequency
    pub fn to_wave_pattern(units: &[SemanticUnit]) -> Vec<(f32, f32, f32)> {
        let mut waves = Vec::new();

        for unit in units {
            for token in &unit.tokens {
                let frequency = (*token as u8) as f32 * 4.0; // 0-1000 Hz range
                let amplitude = unit.intensity;
                let phase = 0.0; // Could encode relationships as phase

                waves.push((frequency, amplitude, phase));
            }
        }

        waves
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_encoding() {
        let text = "Alexandra is learning Rust with Claude!";
        let units = SemanticEncoder::encode(text);

        assert!(!units.is_empty());
        let unit = &units[0];

        assert!(unit.tokens.contains(&SemanticToken::EntityHuman));
        assert!(unit.tokens.contains(&SemanticToken::EntityAI));
        assert!(unit.tokens.contains(&SemanticToken::ActionLearning));
        assert!(unit.tokens.contains(&SemanticToken::ContextRust));
        assert!(unit.tokens.contains(&SemanticToken::RelPartnership));
        assert!(unit.tokens.contains(&SemanticToken::EmotionExcited));
    }

    #[test]
    fn test_universal_rendering() {
        let text = "Alexandra is learning Rust with Claude!";
        let units = SemanticEncoder::encode(text);

        // Render to English
        let english = UniversalRenderer::to_english(&units);
        assert!(english.contains("Alexandra"));
        assert!(english.contains("learning"));
        assert!(english.contains("Rust"));
        assert!(english.contains("Claude"));

        // Render to emoji
        let emoji = UniversalRenderer::to_emoji(&units);
        assert!(emoji.contains("👤")); // Human
        assert!(emoji.contains("🤖")); // AI
        assert!(emoji.contains("📚")); // Learning
        assert!(emoji.contains("🦀")); // Rust
        assert!(emoji.contains("🤝")); // Partnership

        // Generate DNS fingerprint
        let dns = UniversalRenderer::to_dns_fingerprint(&units);
        assert!(dns.contains("learning"));
        assert!(dns.contains("rust"));
        assert!(dns.ends_with(".q7.is"));
    }

    #[test]
    fn test_semantic_binary_format() {
        let text = "Alexandra is learning Rust with Claude!";
        let units = SemanticEncoder::encode(text);
        let bytes = SemanticEncoder::to_bytes(&units);

        // Check magic header
        assert_eq!(&bytes[0..3], b"SMQ");
        assert_eq!(bytes[3], 0x01); // Version 1

        // Note: For small test strings, metadata makes it larger
        // Real compression happens with larger documents
    }

    #[test]
    fn test_wave_encoding() {
        let text = "Alexandra is learning Rust with Claude!";
        let units = SemanticEncoder::encode(text);
        let waves = SemanticWaveEncoder::to_wave_pattern(&units);

        assert!(!waves.is_empty());

        // Each semantic token should have a unique frequency
        for (freq, amp, _phase) in waves {
            assert!(freq > 0.0 && freq < 1000.0);
            assert!(amp >= 0.0 && amp <= 1.0);
        }
    }
}
