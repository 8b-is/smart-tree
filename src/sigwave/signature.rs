use anyhow::Result;
use std::collections::HashMap;

/// Generate a signature from recent interactions
pub fn generate_signature(
    messages: &[String],
    metadata: HashMap<String, String>,
) -> Result<super::SignatureVectors> {
    let style = analyze_style(messages);
    let behavior = analyze_behavior(messages, &metadata);
    let concepts = analyze_concepts(messages);
    let linguistic = analyze_linguistic(messages);
    let emotional = analyze_emotional(messages);

    Ok(super::SignatureVectors {
        style,
        behavior,
        concepts,
        linguistic,
        emotional,
    })
}

fn analyze_style(messages: &[String]) -> super::StyleVector {
    let total_chars: usize = messages.iter().map(|m| m.len()).sum();
    let total_messages = messages.len() as f32;
    
    // Calculate average message length
    let avg_length = if total_messages > 0.0 {
        total_chars as f32 / total_messages
    } else {
        100.0
    };
    
    // Terseness based on average message length
    let terseness = 1.0 - (avg_length / 500.0).min(1.0);
    
    // Count humor indicators
    let humor_count = messages.iter()
        .filter(|m| {
            m.contains("😄") || m.contains("🎸") || m.contains("lol") || 
            m.contains("haha") || m.contains("joke")
        })
        .count() as f32;
    let humor_density = (humor_count / total_messages.max(1.0)).min(1.0);
    
    // Technical indicators
    let tech_count = messages.iter()
        .filter(|m| {
            m.contains("fn ") || m.contains("impl ") || m.contains("struct ") ||
            m.contains("async") || m.contains("Result<") || m.contains("Vec<")
        })
        .count() as f32;
    let technicality = (tech_count / total_messages.max(1.0)).min(1.0);
    
    // Bullet point usage
    let bullet_count = messages.iter()
        .filter(|m| m.contains("•") || m.contains("-") || m.starts_with("*"))
        .count() as f32;
    let bullet_preference = (bullet_count / total_messages.max(1.0)).min(1.0);
    
    super::StyleVector {
        terseness,
        humor_density,
        technicality,
        formality: 0.3, // Default moderate formality
        bullet_preference,
    }
}

fn analyze_behavior(messages: &[String], metadata: &HashMap<String, String>) -> super::BehaviorVector {
    let total_messages = messages.len() as f32;
    
    // Directness - look for hedging language
    let hedge_count = messages.iter()
        .filter(|m| {
            m.contains("maybe") || m.contains("perhaps") || m.contains("might") ||
            m.contains("could be") || m.contains("possibly")
        })
        .count() as f32;
    let directness = 1.0 - (hedge_count / total_messages.max(1.0)).min(0.5);
    
    // Patience level - based on message frequency
    let patience_level = metadata.get("avg_response_time")
        .and_then(|t| t.parse::<f32>().ok())
        .map(|t| (t / 60.0).min(1.0)) // Normalize to 0-1 based on minutes
        .unwrap_or(0.7);
    
    // Detail orientation
    let detail_words = messages.iter()
        .filter(|m| {
            m.contains("specifically") || m.contains("exactly") || 
            m.contains("precisely") || m.contains("detail")
        })
        .count() as f32;
    let detail_orientation = (detail_words / total_messages.max(1.0) * 2.0).min(1.0);
    
    super::BehaviorVector {
        directness,
        patience_level,
        detail_orientation,
        experimentation: 0.8, // Default high for developers
    }
}

fn analyze_concepts(messages: &[String]) -> super::ConceptVector {
    let mut concepts = HashMap::new();
    
    // Extract key technical concepts
    let tech_concepts = [
        ("rust", "Rust"),
        ("python", "Python"),
        ("javascript", "JavaScript"),
        ("mcp", "MCP"),
        ("memory", "Memory"),
        ("compression", "Compression"),
        ("ai", "AI"),
        ("quantum", "Quantum"),
        ("vector", "Vector"),
        ("signature", "Signature"),
    ];
    
    for (pattern, concept) in tech_concepts {
        let count = messages.iter()
            .filter(|m| m.to_lowercase().contains(pattern))
            .count() as f32;
        
        if count > 0.0 {
            concepts.insert(concept.to_string(), count);
        }
    }
    
    // Normalize weights
    let total: f32 = concepts.values().sum();
    if total > 0.0 {
        for weight in concepts.values_mut() {
            *weight /= total;
        }
    }
    
    // Topic velocity - how many unique concepts per message
    let unique_concepts = concepts.len() as f32;
    let topic_velocity = (unique_concepts / messages.len() as f32).min(1.0);
    
    super::ConceptVector {
        concepts,
        topic_velocity,
        depth_preference: 0.7, // Default to preferring depth
    }
}

fn analyze_linguistic(messages: &[String]) -> super::LinguisticVector {
    let sentences: Vec<&str> = messages.iter()
        .flat_map(|m| m.split('.').filter(|s| !s.trim().is_empty()))
        .collect();
    
    let total_sentences = sentences.len() as f32;
    
    // Average sentence length
    let avg_sentence_length = if total_sentences > 0.0 {
        sentences.iter()
            .map(|s| s.split_whitespace().count())
            .sum::<usize>() as f32 / total_sentences
    } else {
        10.0
    };
    
    // Vocabulary complexity (unique words / total words)
    let all_words: Vec<String> = messages.iter()
        .flat_map(|m| m.split_whitespace().map(|w| w.to_lowercase()))
        .collect();
    let unique_words: std::collections::HashSet<_> = all_words.iter().cloned().collect();
    let vocabulary_complexity = if !all_words.is_empty() {
        unique_words.len() as f32 / all_words.len() as f32
    } else {
        0.5
    };
    
    // Signature phrases
    let mut signature_phrases = Vec::new();
    let common_phrases = [
        "let's", "shall we", "rock on", "the cheet", "blazingly fast",
        "taco bell", "franchise wars", "quantum", "wave", "temporal"
    ];
    
    for phrase in common_phrases {
        if messages.iter().any(|m| m.to_lowercase().contains(phrase)) {
            signature_phrases.push(phrase.to_string());
        }
    }
    
    // Punctuation style
    let mut punctuation_style = HashMap::new();
    let exclamation_count = messages.iter()
        .map(|m| m.matches('!').count())
        .sum::<usize>() as f32;
    let question_count = messages.iter()
        .map(|m| m.matches('?').count())
        .sum::<usize>() as f32;
    
    punctuation_style.insert("exclamation".to_string(), 
        exclamation_count / messages.len() as f32);
    punctuation_style.insert("question".to_string(), 
        question_count / messages.len() as f32);
    
    super::LinguisticVector {
        avg_sentence_length,
        vocabulary_complexity,
        signature_phrases,
        punctuation_style,
    }
}

fn analyze_emotional(messages: &[String]) -> super::EmotionalVector {
    let total_messages = messages.len() as f32;
    
    // Enthusiasm indicators
    let enthusiasm_count = messages.iter()
        .filter(|m| {
            m.contains('!') || m.contains("awesome") || m.contains("great") ||
            m.contains("amazing") || m.contains("excellent") || m.contains("🎸")
        })
        .count() as f32;
    let enthusiasm = (enthusiasm_count / total_messages * 0.5).min(1.0);
    
    // Frustration indicators
    let frustration_count = messages.iter()
        .filter(|m| {
            m.contains("ugh") || m.contains("damn") || m.contains("error") ||
            m.contains("failed") || m.contains("broken")
        })
        .count() as f32;
    let frustration = (frustration_count / total_messages * 0.5).min(1.0);
    
    // Curiosity indicators
    let curiosity_count = messages.iter()
        .filter(|m| {
            m.contains('?') || m.contains("how") || m.contains("why") ||
            m.contains("what if") || m.contains("wonder")
        })
        .count() as f32;
    let curiosity = (curiosity_count / total_messages * 0.5).min(1.0);
    
    // Playfulness
    let playful_count = messages.iter()
        .filter(|m| {
            m.contains("😄") || m.contains("🎸") || m.contains("rock") ||
            m.contains("cheet") || m.contains("taco bell")
        })
        .count() as f32;
    let playfulness = (playful_count / total_messages * 0.5).min(1.0);
    
    super::EmotionalVector {
        enthusiasm,
        frustration,
        curiosity,
        playfulness,
        introspection: 0.5, // Default moderate
    }
}

/// Extract signature from a file path
pub fn extract_from_file(_path: &std::path::Path) -> Result<super::SignatureVectors> {
    // This would analyze code style, comments, patterns
    // For now, return a placeholder
    Ok(super::SignatureVectors {
        style: super::StyleVector {
            terseness: 0.7,
            humor_density: 0.3,
            technicality: 0.8,
            formality: 0.4,
            bullet_preference: 0.6,
        },
        behavior: super::BehaviorVector {
            directness: 0.8,
            patience_level: 0.6,
            detail_orientation: 0.7,
            experimentation: 0.8,
        },
        concepts: super::ConceptVector {
            concepts: HashMap::new(),
            topic_velocity: 0.5,
            depth_preference: 0.7,
        },
        linguistic: super::LinguisticVector {
            avg_sentence_length: 12.0,
            vocabulary_complexity: 0.6,
            signature_phrases: vec![],
            punctuation_style: HashMap::new(),
        },
        emotional: super::EmotionalVector {
            enthusiasm: 0.6,
            frustration: 0.2,
            curiosity: 0.7,
            playfulness: 0.4,
            introspection: 0.5,
        },
    })
}