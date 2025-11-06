//! Universal Document Ingestion with Theoglyphic Context
//!
//! Transforms any document into UTL format with full context preservation

/// Document types that affect interpretation
#[derive(Debug, Clone)]
pub enum DocumentGenre {
    Fiction,    // Made-up stories
    NonFiction, // Factual accounts
    Memoir,     // Personal memories
    Letter,     // Correspondence
    Diary,      // Personal journal
    Poem,       // Poetry
    Essay,      // Thoughtful writing
    Musing,     // Random thoughts
    Recipe,     // Instructions
    List,       // Enumeration
    Unknown,    // Can't determine
}

/// Temporal context for the document
#[derive(Debug, Clone)]
pub struct TemporalContext {
    /// When was this written?
    pub creation_date: Option<String>,
    /// What time period does it describe?
    pub describes_period: Option<(String, String)>,
    /// How certain are we about these dates?
    pub temporal_confidence: f32,
    /// Delay between event and recording (UDC principle)
    pub memory_delay_ms: Option<u64>,
}

/// Emotional context using theoglyphic emotion symbols
#[derive(Debug, Clone)]
pub struct EmotionalContext {
    /// Primary emotion glyph: 😊(joy) 😢(sad) 😡(anger) 😨(fear) 😮(surprise)
    pub primary_emotion: String,
    /// Intensity 0.0-1.0
    pub intensity: f32,
    /// Mixed emotions present
    pub emotion_blend: Vec<(String, f32)>,
}

/// Relationships and actors in the document
#[derive(Debug, Clone)]
pub struct RelationalContext {
    /// Who wrote this?
    pub author: Option<String>,
    /// Who is it about?
    pub subjects: Vec<String>,
    /// Who was it written for?
    pub audience: Option<String>,
    /// Relationships mentioned
    pub relationships: Vec<(String, String, String)>, // (person1, relation, person2)
}

/// Universal Document Container with full context
#[derive(Debug, Clone)]
pub struct UniversalDocument {
    /// Raw text content
    pub raw_text: String,

    /// Genre classification
    pub genre: DocumentGenre,
    pub genre_confidence: f32,

    /// Temporal information
    pub temporal: TemporalContext,

    /// Emotional coloring
    pub emotional: EmotionalContext,

    /// People and relationships
    pub relational: RelationalContext,

    /// Theoglyphic representation
    pub utl_encoding: String,

    /// MEM|8 wave signature (for memory storage)
    pub wave_signature: Option<Vec<f32>>,
}

impl UniversalDocument {
    /// Analyze text to determine genre
    pub fn classify_genre(text: &str) -> (DocumentGenre, f32) {
        let lower = text.to_lowercase();

        // Look for genre markers
        if lower.contains("once upon a time") || lower.contains("the end") {
            return (DocumentGenre::Fiction, 0.8);
        }

        if lower.contains("dear ") && (lower.contains("sincerely") || lower.contains("love,")) {
            return (DocumentGenre::Letter, 0.9);
        }

        if lower.contains("ingredients:") || lower.contains("instructions:") {
            return (DocumentGenre::Recipe, 0.95);
        }

        // Look for temporal markers
        let date_patterns = [
            "january",
            "february",
            "march",
            "april",
            "may",
            "june",
            "july",
            "august",
            "september",
            "october",
            "november",
            "december",
            "1900",
            "1901",
            "1902",
            "1903",
            "1904",
            "1905",
            "1906",
            "19",
            "20", // century markers
        ];

        let date_count = date_patterns.iter().filter(|p| lower.contains(*p)).count();

        if date_count > 3 {
            if lower.contains("diary") || lower.contains("journal") {
                return (DocumentGenre::Diary, 0.7);
            }
            return (DocumentGenre::Memoir, 0.6);
        }

        // Check for poetry patterns (short lines, rhyme potential)
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() > 4 {
            let avg_line_length = lines.iter().map(|l| l.len()).sum::<usize>() / lines.len();
            if avg_line_length < 50 && lines.len() > 8 {
                return (DocumentGenre::Poem, 0.5);
            }
        }

        (DocumentGenre::Unknown, 0.1)
    }

    /// Extract temporal context from text
    pub fn extract_temporal(text: &str) -> TemporalContext {
        // Look for date patterns
        let mut dates = Vec::new();

        // Simple year extraction (1900-2099)
        for word in text.split_whitespace() {
            if let Ok(year) = word.trim_matches(|c: char| !c.is_numeric()).parse::<i32>() {
                if (1900..=2099).contains(&year) {
                    dates.push(year);
                }
            }
        }

        // If we found dates, use them
        let (_start_year, _end_year) = if !dates.is_empty() {
            dates.sort();
            (dates[0], dates[dates.len() - 1])
        } else {
            (1900, 2024) // Default range
        };

        TemporalContext {
            creation_date: None,    // Would need more context
            describes_period: None, // Simplified for now
            temporal_confidence: if dates.is_empty() { 0.1 } else { 0.5 },
            memory_delay_ms: Some(250), // UDC default delay
        }
    }

    /// Detect emotional content
    pub fn extract_emotional(text: &str) -> EmotionalContext {
        let lower = text.to_lowercase();

        // Simple emotion detection
        let joy_words = ["happy", "joy", "love", "wonderful", "beautiful", "excited"];
        let sad_words = ["sad", "died", "loss", "grief", "mourn", "tears"];
        let anger_words = ["angry", "furious", "mad", "hate", "rage"];
        let fear_words = ["afraid", "scared", "fear", "terror", "worried"];

        let mut scores = vec![
            (
                "😊",
                joy_words.iter().filter(|w| lower.contains(**w)).count() as f32,
            ),
            (
                "😢",
                sad_words.iter().filter(|w| lower.contains(**w)).count() as f32,
            ),
            (
                "😡",
                anger_words.iter().filter(|w| lower.contains(**w)).count() as f32,
            ),
            (
                "😨",
                fear_words.iter().filter(|w| lower.contains(**w)).count() as f32,
            ),
        ];

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let total: f32 = scores.iter().map(|(_, s)| s).sum();
        let primary = if total > 0.0 {
            (scores[0].0.to_string(), scores[0].1 / total.max(1.0))
        } else {
            ("😐".to_string(), 0.5) // Neutral
        };

        EmotionalContext {
            primary_emotion: primary.0,
            intensity: primary.1,
            emotion_blend: scores
                .into_iter()
                .filter(|(_, s)| *s > 0.0)
                .map(|(e, s)| (e.to_string(), s / total.max(1.0)))
                .collect(),
        }
    }

    /// Extract people and relationships
    pub fn extract_relational(text: &str) -> RelationalContext {
        let mut subjects = Vec::new();

        // Look for capitalized names (simple heuristic)
        for word in text.split_whitespace() {
            let clean = word.trim_matches(|c: char| !c.is_alphabetic());
            if clean.len() > 2 && clean.chars().next().unwrap().is_uppercase() {
                // Common names to look for
                let common_names = ["Mike", "Alice", "Dad", "Mom", "Jody", "Bill", "Maude"];
                if common_names.contains(&clean) && !subjects.contains(&clean.to_string()) {
                    subjects.push(clean.to_string());
                }
            }
        }

        RelationalContext {
            author: None, // Would need metadata
            subjects,
            audience: None,
            relationships: Vec::new(), // Would need NLP
        }
    }

    /// Convert document to Theoglyphic representation
    pub fn to_theoglyphic(&self) -> String {
        // Build UTL encoding using theoglyphic symbols
        let mut utl = String::new();

        // Genre marker
        utl.push_str(match self.genre {
            DocumentGenre::Fiction => "📖",  // Book for fiction
            DocumentGenre::Memoir => "🧠💭", // Brain-thought for memory
            DocumentGenre::Letter => "✉️",   // Letter
            DocumentGenre::Diary => "📔",    // Notebook
            DocumentGenre::Poem => "🎵",     // Musical note for poetry
            _ => "📄",                       // Generic document
        });

        // Add temporal marker
        if self.temporal.temporal_confidence > 0.3 {
            utl.push('⏰'); // Time marker
        }

        // Add primary emotion
        utl.push_str(&self.emotional.primary_emotion);

        // Add relationship marker if people mentioned
        if !self.relational.subjects.is_empty() {
            utl.push('👥'); // People marker
        }

        // Add delay marker (UDC principle)
        if let Some(delay) = self.temporal.memory_delay_ms {
            if delay > 500 {
                utl.push('⧖'); // The UDC delay symbol!
            }
        }

        utl
    }

    /// Create a universal document from raw text
    pub fn from_text(text: String) -> Self {
        let (genre, genre_confidence) = Self::classify_genre(&text);
        let temporal = Self::extract_temporal(&text);
        let emotional = Self::extract_emotional(&text);
        let relational = Self::extract_relational(&text);

        let mut doc = UniversalDocument {
            raw_text: text,
            genre,
            genre_confidence,
            temporal,
            emotional,
            relational,
            utl_encoding: String::new(),
            wave_signature: None,
        };

        doc.utl_encoding = doc.to_theoglyphic();
        doc
    }
}

/// Convert Publisher document to Universal format
pub fn publisher_to_universal(pub_text: &str) -> UniversalDocument {
    // Clean up Publisher artifacts
    let cleaned = pub_text
        .replace("CHNKINK", "")
        .replace("TEXTTEXT", "")
        .replace("FDPP", "")
        .replace("STSH", "")
        .lines()
        .filter(|line| {
            // Filter out binary garbage
            !line.contains("")
                && line
                    .chars()
                    .filter(|c| c.is_ascii_graphic() || c.is_whitespace())
                    .count()
                    > line.len() / 2
        })
        .collect::<Vec<_>>()
        .join("\n");

    UniversalDocument::from_text(cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genre_classification() {
        let fiction = "Once upon a time there was a princess. The end.";
        let (genre, conf) = UniversalDocument::classify_genre(fiction);
        assert!(matches!(genre, DocumentGenre::Fiction));
        assert!(conf > 0.7);

        let letter = "Dear Mom,\nI hope you are well.\nLove, Jody";
        let (genre, conf) = UniversalDocument::classify_genre(letter);
        assert!(matches!(genre, DocumentGenre::Letter));
        assert!(conf > 0.8);
    }

    #[test]
    fn test_emotional_extraction() {
        let happy = "I'm so happy and excited about this wonderful day!";
        let emotional = UniversalDocument::extract_emotional(happy);
        assert_eq!(emotional.primary_emotion, "😊");

        let sad = "She died and we all mourned her loss with tears.";
        let emotional = UniversalDocument::extract_emotional(sad);
        assert_eq!(emotional.primary_emotion, "😢");
    }

    #[test]
    fn test_theoglyphic_encoding() {
        let text = "Dear Alice, I remember our happy times in 1927. Love, Mike";
        let doc = UniversalDocument::from_text(text.to_string());

        // Should have letter marker, emotion, people marker
        assert!(doc.utl_encoding.contains("✉️"));
        assert!(doc.utl_encoding.contains("😊"));
        assert!(doc.utl_encoding.contains("👥"));
    }
}
