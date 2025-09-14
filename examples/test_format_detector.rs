// Test the Universal Format Detector
use anyhow::Result;
use st::universal_format_detector::{UniversalFormatDetector, demo_format_detection};

fn main() -> Result<()> {
    println!("🔍 Universal Format Detector - Structural Analysis Demo\n");
    println!("{}\n", "=".repeat(60));
    
    // Run the built-in demo
    demo_format_detection()?;
    
    // Test with a complex mixed format
    println!("Testing Complex ChatGPT Export Format:\n");
    
    let chatgpt_export = r#"{
  "conversations": [
    {
      "id": "abc-123",
      "create_time": 1234567890.123,
      "messages": [
        {
          "author": {
            "role": "user"
          },
          "content": {
            "parts": ["Can you help me understand tokenization?"]
          }
        },
        {
          "author": {
            "role": "assistant"
          },
          "content": {
            "parts": ["Tokenization is the process of breaking down text into smaller units called tokens. These tokens can be words, subwords, or even characters, depending on the tokenization strategy used. In natural language processing, tokenization is a fundamental preprocessing step that converts raw text into a format that machine learning models can understand and process.\n\nHere's how it works:\n\n1. **Word-level tokenization**: Splits text by spaces and punctuation\n2. **Subword tokenization**: Breaks words into smaller meaningful units\n3. **Character-level tokenization**: Uses individual characters as tokens\n\nModern language models like GPT use sophisticated tokenization methods like Byte-Pair Encoding (BPE) or SentencePiece to handle diverse languages and reduce vocabulary size while maintaining semantic meaning."]
          }
        },
        {
          "author": {
            "role": "user"
          },
          "content": {
            "parts": ["That's helpful! Can you show me a code example?"]
          }
        }
      ]
    }
  ]
}"#;
    
    let mut detector = UniversalFormatDetector::new();
    let format = detector.detect_format(chatgpt_export);
    detector.analyze_structure(chatgpt_export)?;
    
    println!("Detected format: {:?}\n", format);
    println!("{}", detector.get_conversation_summary());
    
    if let Some((speaker, bytes)) = detector.get_dominant_speaker() {
        println!("\nDominant speaker: {} ({} bytes total)", speaker, bytes);
    }
    
    // Show structural tokens
    let tokens = detector.tokenize_structure();
    if !tokens.is_empty() {
        println!("\n🎯 Structural tokens (most frequent patterns):");
        let mut sorted_tokens: Vec<_> = tokens.iter().collect();
        sorted_tokens.sort_by_key(|(_, &token)| token);
        
        for (pattern, token) in sorted_tokens.iter().take(10) {
            println!("  0x{:02X} = {}", token, pattern);
        }
    }
    
    println!("\n✨ The Magic:");
    println!("  • Format detected by STRUCTURE, not keywords");
    println!("  • Depth tracking shows nesting (XML/JSON)");
    println!("  • Block analysis finds conversations");
    println!("  • Pattern tokenization for compression");
    println!("  • Works with ANY format!");
    
    Ok(())
}
