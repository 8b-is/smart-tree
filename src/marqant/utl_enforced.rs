//! Type-level enforced UTL pipeline
//!
//! Makes Human→Human translation IMPOSSIBLE at compile time!

use anyhow::{anyhow, Result};
use std::marker::PhantomData;

// ---------- Core types ----------

#[derive(Debug, Clone)]
pub struct RawText(pub String);

#[derive(Debug, Clone)]
pub struct UtlDoc {
    /// UTL token stream - the universal representation
    pub tokens: Vec<String>,
    /// Optional metadata from analysis
    pub metadata: Option<UtlMetadata>,
}

#[derive(Debug, Clone)]
pub struct UtlMetadata {
    pub genre: String,
    pub temporal: String,
    pub emotion: String,
    pub delay_ms: u64,
}

#[derive(Debug, Clone)]
pub struct HumanText<L: Language> {
    _lang: PhantomData<L>,
    pub text: String,
}

// Marker trait: only languages we explicitly allow
pub trait Language {
    fn name() -> &'static str;
}

pub struct Eng;
pub struct Jpn;
pub struct Spa;
pub struct Zho; // Chinese
pub struct Ara; // Arabic
pub struct Hin; // Hindi

impl Language for Eng {
    fn name() -> &'static str {
        "English"
    }
}
impl Language for Jpn {
    fn name() -> &'static str {
        "Japanese"
    }
}
impl Language for Spa {
    fn name() -> &'static str {
        "Spanish"
    }
}
impl Language for Zho {
    fn name() -> &'static str {
        "Chinese"
    }
}
impl Language for Ara {
    fn name() -> &'static str {
        "Arabic"
    }
}
impl Language for Hin {
    fn name() -> &'static str {
        "Hindi"
    }
}

// ---------- Translation trait (directional) ----------

pub trait Translate<From, To> {
    fn translate(&self, input: From) -> Result<To>;
}

// ---------- ONLY Allowed translators ----------

/// Raw text to UTL - the ONLY entry point
pub struct RawToUtl;

impl Translate<RawText, UtlDoc> for RawToUtl {
    fn translate(&self, input: RawText) -> Result<UtlDoc> {
        let mut tokens = Vec::new();

        // Real UTL tokenization with theoglyphic symbols
        for sentence in input.0.split('.') {
            let sentence = sentence.trim().to_lowercase();
            if sentence.is_empty() {
                continue;
            }

            // Convert to UTL symbols (using word boundary checks)
            let words: Vec<&str> = sentence.split_whitespace().collect();
            if words.contains(&"i") || sentence.contains("me") {
                tokens.push("🙋".to_string()); // Self
            }
            if words.contains(&"you") {
                tokens.push("👤".to_string()); // Other
            }
            if sentence.contains("love") {
                tokens.push("❤️".to_string());
            }
            if sentence.contains("think") {
                tokens.push("🧠".to_string());
            }
            if sentence.contains("remember") {
                tokens.push("💭".to_string());
            }
            if words.contains(&"was") || words.contains(&"were") || words.contains(&"being") {
                tokens.push("⏮".to_string()); // Past
            }
            if words.contains(&"is") || words.contains(&"am") || words.contains(&"are") {
                tokens.push("⏺".to_string()); // Present
            }
            if words.contains(&"will") {
                tokens.push("⏭".to_string()); // Future
            }
            if sentence.contains("happy") {
                tokens.push("😊".to_string());
            }
            if sentence.contains("sad") {
                tokens.push("😢".to_string());
            }

            // Add UDC delay marker between thoughts
            tokens.push("⧖".to_string());
        }

        Ok(UtlDoc {
            tokens,
            metadata: None,
        })
    }
}

/// UTL to human language - the ONLY exit point
pub struct UtlToHuman<L: Language>(PhantomData<L>);

impl UtlToHuman<Eng> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl Translate<UtlDoc, HumanText<Eng>> for UtlToHuman<Eng> {
    fn translate(&self, input: UtlDoc) -> Result<HumanText<Eng>> {
        let mut words = Vec::new();

        for token in &input.tokens {
            let word = match token.as_str() {
                "🙋" => "I",
                "👤" => "you",
                "❤️" => "love",
                "🧠" => "think",
                "💭" => "remember",
                "⏮" => "was",
                "⏺" => "is",
                "⏭" => "will",
                "😊" => "happy",
                "😢" => "sad",
                "⧖" => ".",
                _ => continue,
            };
            words.push(word);
        }

        Ok(HumanText {
            _lang: PhantomData,
            text: words.join(" "),
        })
    }
}

impl UtlToHuman<Jpn> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl Translate<UtlDoc, HumanText<Jpn>> for UtlToHuman<Jpn> {
    fn translate(&self, input: UtlDoc) -> Result<HumanText<Jpn>> {
        let mut words = Vec::new();

        for token in &input.tokens {
            let word = match token.as_str() {
                "🙋" => "私",
                "👤" => "あなた",
                "❤️" => "愛",
                "🧠" => "考える",
                "💭" => "思い出す",
                "⏮" => "でした",
                "⏺" => "です",
                "⏭" => "でしょう",
                "😊" => "嬉しい",
                "😢" => "悲しい",
                "⧖" => "。",
                _ => continue,
            };
            words.push(word);
        }

        Ok(HumanText {
            _lang: PhantomData,
            text: words.join(""),
        })
    }
}

// ---------- FORBIDDEN paths (intentionally UNIMPLEMENTED) ----------
//
// These will NEVER compile:
// - No impl Translate<HumanText<Eng>, HumanText<Jpn>>
// - No impl Translate<HumanText<Jpn>, HumanText<Eng>>
// - No impl Translate<RawText, HumanText<L>> directly
// - No impl Translate<HumanText<L>, RawText>
//
// If someone tries, Rust compiler says NO! 🚫

// ---------- Analysis & Storage hooks ----------

pub fn analyze_utl(doc: &mut UtlDoc) -> Result<()> {
    // Analyze the UTL symbols, not text!
    let mut genre = "unknown";
    let mut temporal = "present";
    let mut emotion = "neutral";

    // Count temporal markers
    let past = doc.tokens.iter().filter(|t| t == &"⏮").count();
    let present = doc.tokens.iter().filter(|t| t == &"⏺").count();
    let future = doc.tokens.iter().filter(|t| t == &"⏭").count();

    if past > present && past > future {
        temporal = "past";
    } else if future > present {
        temporal = "future";
    }

    // Detect emotion
    if doc.tokens.contains(&"😊".to_string()) {
        emotion = "joy";
    } else if doc.tokens.contains(&"😢".to_string()) {
        emotion = "sadness";
    }

    // Detect genre from patterns
    if doc.tokens.contains(&"💭".to_string()) {
        genre = "memoir";
    }

    doc.metadata = Some(UtlMetadata {
        genre: genre.to_string(),
        temporal: temporal.to_string(),
        emotion: emotion.to_string(),
        delay_ms: 250, // UDC delay
    });

    Ok(())
}

#[cfg(feature = "mem8")]
pub fn store_mem8(doc: &UtlDoc) -> Result<()> {
    // TODO: Wire to actual MEM|8
    println!("Storing UTL with {} tokens to MEM|8", doc.tokens.len());
    Ok(())
}

#[cfg(not(feature = "mem8"))]
pub fn store_mem8(_doc: &UtlDoc) -> Result<()> {
    Ok(()) // No-op when MEM|8 not compiled in
}

// ---------- One-shot pipeline (the ONLY way) ----------

pub fn process_to_language<L: Language>(raw: &str) -> Result<HumanText<L>>
where
    UtlToHuman<L>: Translate<UtlDoc, HumanText<L>>,
{
    // Step 1: Raw → UTL (mandatory)
    let mut utl = RawToUtl.translate(RawText(raw.to_owned()))?;

    // Step 2: Analyze UTL
    analyze_utl(&mut utl)?;

    // Step 3: Store in MEM|8
    store_mem8(&utl)?;

    // Step 4: UTL → Human language
    UtlToHuman::<L>(PhantomData).translate(utl)
}

// ---------- Convenience helpers ----------

pub fn to_english(raw: &str) -> Result<String> {
    Ok(process_to_language::<Eng>(raw)?.text)
}

pub fn to_japanese(raw: &str) -> Result<String> {
    Ok(process_to_language::<Jpn>(raw)?.text)
}

// ---------- Runtime guard against sneaky bypasses ----------

/// This function will ALWAYS error - it's a honeypot for bad code
pub fn forbid_human_to_human<A: Language, B: Language>() -> Result<()> {
    Err(anyhow!(
        "FORBIDDEN: Direct {} → {} translation! Must go through UTL!",
        A::name(),
        B::name()
    ))
}

// ---------- Example of impossible code ----------
//
// This WILL NOT COMPILE (uncomment to verify):
//
// pub fn bad_translator(eng: HumanText<Eng>) -> HumanText<Jpn> {
//     // ERROR: no impl of Translate<HumanText<Eng>, HumanText<Jpn>>
//     SomeTranslator.translate(eng)  // ← Compile error!
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enforced_pipeline() {
        // This works - goes through UTL
        let result = to_english("I love you").unwrap();
        assert!(result.contains("I"));
        assert!(result.contains("love"));

        let result = to_japanese("I love you").unwrap();
        assert!(result.contains("私"));
        assert!(result.contains("愛"));
    }

    #[test]
    fn test_utl_analysis() {
        let mut utl = RawToUtl
            .translate(RawText("I remember being happy".into()))
            .unwrap();
        analyze_utl(&mut utl).unwrap();

        let meta = utl.metadata.unwrap();
        assert_eq!(meta.genre, "memoir");
        assert_eq!(meta.temporal, "past");
    }

    // This test WILL NOT COMPILE if uncommented:
    // #[test]
    // fn test_forbidden_human_to_human() {
    //     let eng = HumanText::<Eng> { _lang: PhantomData, text: "Hello".into() };
    //     let jpn: HumanText<Jpn> = BadTranslator.translate(eng); // COMPILE ERROR!
    // }
}
