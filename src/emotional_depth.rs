//! Emotional Auto-Depth Mode - Smart Tree gets feelings about directories!
//! 
//! Like when you're exploring and suddenly think "Ok... this is getting boring now..."
//! The tree will dynamically adjust its depth based on how "interesting" directories are!

use crate::scanner::FileNode;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Emotional states for directory exploration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DirectoryEmotion {
    /// 🤩 "Ooh, what's in here?!" - Exciting new territory!
    Excited,
    /// 😊 "This looks interesting!" - Worth exploring
    Interested,
    /// 🤔 "Hmm, let me see..." - Mildly curious
    Curious,
    /// 😐 "More of the same..." - Getting repetitive
    Neutral,
    /// 😴 "Ugh, another node_modules..." - Boring!
    Bored,
    /// 😵 "STOP! Too much!" - Overwhelming
    Overwhelmed,
    /// 🙈 "I shouldn't be here..." - System/sensitive directories
    Anxious,
}

impl DirectoryEmotion {
    /// Get the depth modifier for this emotion
    /// Excited = go deeper, Bored = stop early
    pub fn depth_modifier(&self) -> i32 {
        match self {
            Self::Excited => 2,      // Go 2 levels deeper!
            Self::Interested => 1,   // One more level
            Self::Curious => 0,      // Normal depth
            Self::Neutral => 0,      // Normal depth
            Self::Bored => -2,       // Cut it short
            Self::Overwhelmed => -3, // STOP!
            Self::Anxious => -1,     // Back away slowly...
        }
    }
    
    /// Get the emoji for this emotion
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Excited => "🤩",
            Self::Interested => "😊",
            Self::Curious => "🤔",
            Self::Neutral => "😐",
            Self::Bored => "😴",
            Self::Overwhelmed => "😵",
            Self::Anxious => "🙈",
        }
    }
    
    /// Get a fun comment about this emotion
    pub fn comment(&self) -> &'static str {
        match self {
            Self::Excited => "Ooh, what treasures await?!",
            Self::Interested => "This looks promising!",
            Self::Curious => "Let's take a peek...",
            Self::Neutral => "Just another directory...",
            Self::Bored => "Zzz... seen it all before...",
            Self::Overwhelmed => "TOO. MANY. FILES!",
            Self::Anxious => "Should I even be here?",
        }
    }
}

/// Emotional intelligence for directory exploration
pub struct EmotionalDepthAnalyzer {
    /// Base depth setting (-2 to -5 for different aggression levels)
    base_aggression: i32,
    
    /// Patterns that trigger emotions
    emotion_triggers: HashMap<String, DirectoryEmotion>,
    
    /// Track how many similar dirs we've seen (for boredom)
    repetition_counter: HashMap<String, u32>,
    
    /// Current emotional state
    current_mood: DirectoryEmotion,
    
    /// Emotional memory - remember how we felt about paths
    emotional_memory: HashMap<PathBuf, DirectoryEmotion>,
}

impl EmotionalDepthAnalyzer {
    /// Create a new analyzer with given aggression level
    pub fn new(aggression: i32) -> Self {
        let mut emotion_triggers = HashMap::new();
        
        // Exciting patterns
        emotion_triggers.insert("src".to_string(), DirectoryEmotion::Excited);
        emotion_triggers.insert("lib".to_string(), DirectoryEmotion::Excited);
        emotion_triggers.insert("core".to_string(), DirectoryEmotion::Excited);
        emotion_triggers.insert("features".to_string(), DirectoryEmotion::Interested);
        
        // Interesting patterns
        emotion_triggers.insert("docs".to_string(), DirectoryEmotion::Interested);
        emotion_triggers.insert("tests".to_string(), DirectoryEmotion::Interested);
        emotion_triggers.insert("examples".to_string(), DirectoryEmotion::Interested);
        
        // Boring patterns
        emotion_triggers.insert("node_modules".to_string(), DirectoryEmotion::Bored);
        emotion_triggers.insert(".git".to_string(), DirectoryEmotion::Bored);
        emotion_triggers.insert("target".to_string(), DirectoryEmotion::Bored);
        emotion_triggers.insert("build".to_string(), DirectoryEmotion::Bored);
        emotion_triggers.insert("dist".to_string(), DirectoryEmotion::Bored);
        emotion_triggers.insert("cache".to_string(), DirectoryEmotion::Bored);
        emotion_triggers.insert("vendor".to_string(), DirectoryEmotion::Bored);
        
        // Anxious patterns
        emotion_triggers.insert("Windows".to_string(), DirectoryEmotion::Anxious);
        emotion_triggers.insert("System32".to_string(), DirectoryEmotion::Anxious);
        emotion_triggers.insert("private".to_string(), DirectoryEmotion::Anxious);
        emotion_triggers.insert("secret".to_string(), DirectoryEmotion::Anxious);
        
        Self {
            base_aggression: aggression.clamp(-5, -2),
            emotion_triggers,
            repetition_counter: HashMap::new(),
            current_mood: DirectoryEmotion::Curious,
            emotional_memory: HashMap::new(),
        }
    }
    
    /// Analyze a directory and determine emotional response
    pub fn analyze_directory(&mut self, path: &Path, children: &[FileNode]) -> DirectoryEmotion {
        // Check if we remember this path
        if let Some(&emotion) = self.emotional_memory.get(path) {
            return emotion;
        }
        
        let dir_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        // Check for trigger patterns
        if let Some(&emotion) = self.emotion_triggers.get(dir_name) {
            self.emotional_memory.insert(path.to_path_buf(), emotion);
            return emotion;
        }
        
        // Analyze based on content
        let emotion = self.analyze_content(dir_name, children);
        self.emotional_memory.insert(path.to_path_buf(), emotion);
        emotion
    }
    
    /// Analyze directory content to determine emotion
    fn analyze_content(&mut self, dir_name: &str, children: &[FileNode]) -> DirectoryEmotion {
        let file_count = children.len();
        let dir_count = children.iter().filter(|n| n.is_directory).count();
        
        // Track repetition
        let pattern = format!("f{}_d{}", file_count / 10, dir_count / 5);
        let repetitions = self.repetition_counter.entry(pattern.clone()).or_insert(0);
        *repetitions += 1;
        
        // Overwhelming check
        if file_count > 1000 {
            return DirectoryEmotion::Overwhelmed;
        }
        
        // Boredom check - seen this pattern too many times
        if *repetitions > 5 {
            return DirectoryEmotion::Bored;
        }
        
        // Interest based on variety
        let unique_extensions: std::collections::HashSet<_> = children.iter()
            .filter_map(|n| {
                if !n.is_directory {
                    Path::new(&n.name).extension()?.to_str()
                } else {
                    None
                }
            })
            .collect();
        
        match unique_extensions.len() {
            0 if dir_count == 0 => DirectoryEmotion::Bored,     // Empty dir
            0 => DirectoryEmotion::Neutral,                     // Only subdirs
            1..=2 => DirectoryEmotion::Neutral,                 // Low variety
            3..=5 => DirectoryEmotion::Curious,                 // Some variety
            6..=10 => DirectoryEmotion::Interested,             // Good variety!
            _ => DirectoryEmotion::Excited,                     // Lots of variety!
        }
    }
    
    /// Calculate effective depth for a path based on emotions
    pub fn calculate_depth(&mut self, path: &Path, children: &[FileNode], current_depth: usize) -> usize {
        let emotion = self.analyze_directory(path, children);
        self.current_mood = emotion;
        
        // Base depth from aggression level (negative means "from current")
        let base_depth = if self.base_aggression < 0 {
            (current_depth as i32) + self.base_aggression.abs()
        } else {
            self.base_aggression
        };
        
        // Apply emotional modifier
        let emotional_depth = base_depth + emotion.depth_modifier();
        
        // Clamp to reasonable bounds
        emotional_depth.max(0) as usize
    }
    
    /// Get a status message about current emotional state
    pub fn get_emotional_status(&self) -> String {
        format!(
            "{} {} (aggression: {})",
            self.current_mood.emoji(),
            self.current_mood.comment(),
            self.base_aggression
        )
    }
    
    /// Generate an emotional summary of the exploration
    pub fn emotional_journey_summary(&self) -> String {
        let mut summary = String::from("🎭 Emotional Journey Through The File System:\n\n");
        
        // Count emotions
        let mut emotion_counts: HashMap<DirectoryEmotion, usize> = HashMap::new();
        for emotion in self.emotional_memory.values() {
            *emotion_counts.entry(*emotion).or_insert(0) += 1;
        }
        
        // Most common emotion
        if let Some((dominant_emotion, _)) = emotion_counts.iter().max_by_key(|(_, count)| *count) {
            summary.push_str(&format!(
                "Dominant feeling: {} {}\n", 
                dominant_emotion.emoji(), 
                dominant_emotion.comment()
            ));
        }
        
        // Emotional breakdown
        summary.push_str("\nEmotional breakdown:\n");
        for (emotion, count) in emotion_counts.iter() {
            summary.push_str(&format!(
                "  {} × {} ({})\n", 
                count, 
                emotion.emoji(),
                match emotion {
                    DirectoryEmotion::Excited => "exciting discoveries",
                    DirectoryEmotion::Interested => "interesting finds",
                    DirectoryEmotion::Curious => "curiosity sparked",
                    DirectoryEmotion::Neutral => "meh moments",
                    DirectoryEmotion::Bored => "boring directories",
                    DirectoryEmotion::Overwhelmed => "overwhelming chaos",
                    DirectoryEmotion::Anxious => "anxious encounters",
                }
            ));
        }
        
        // Fun insights
        if emotion_counts.get(&DirectoryEmotion::Bored).unwrap_or(&0) > &5 {
            summary.push_str("\n💭 Note: Maybe skip node_modules next time? 😴\n");
        }
        
        if emotion_counts.get(&DirectoryEmotion::Excited).unwrap_or(&0) > &3 {
            summary.push_str("\n✨ What an adventure! So many exciting discoveries!\n");
        }
        
        summary
    }
}

/// Auto-depth mode settings
#[derive(Debug, Clone, Copy)]
pub enum AutoDepthMode {
    /// -2: Gentle exploration (easily satisfied)
    Gentle,
    /// -3: Normal exploration (balanced)
    Normal,
    /// -4: Thorough exploration (hard to bore)
    Thorough,
    /// -5: Exhaustive exploration (never gives up!)
    Exhaustive,
}

impl AutoDepthMode {
    pub fn aggression_level(&self) -> i32 {
        match self {
            Self::Gentle => -2,
            Self::Normal => -3,
            Self::Thorough => -4,
            Self::Exhaustive => -5,
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            Self::Gentle => "Gentle exploration - easily satisfied",
            Self::Normal => "Normal exploration - balanced curiosity",
            Self::Thorough => "Thorough exploration - hard to bore",
            Self::Exhaustive => "Exhaustive exploration - never gives up!",
        }
    }
}

// Trisha says: "This is like watching someone shop! They get excited about 
// new stores, bored in the same old aisles, and anxious near the 
// 'Employees Only' signs! Pure psychology!" 🛍️💭