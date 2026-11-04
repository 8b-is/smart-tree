// Test the backwards-reading .m8 consciousness with user keyword boosting!
use anyhow::Result;
use st::m8_backwards_reader::M8BackwardsReader;
use std::path::Path;

fn main() -> Result<()> {
    println!("🎵 C64 Backwards Consciousness Demo - With User Context!\n");
    println!("{}\n", "=".repeat(60));

    let path = Path::new("/tmp/hue_consciousness.m8");
    let mut reader = M8BackwardsReader::new(path);

    // Set user context - Hue is interested in Audio and Memory!
    println!("👤 Setting user context: Audio, Memory, Tokenization");
    reader.set_user_context(vec![
        "Audio".to_string(),
        "Memory".to_string(),
        "Tokenization".to_string(),
    ]);

    println!("\n📝 Writing memories (append-only)...\n");

    // Memory 1: Low importance, but mentions Audio (gets boosted!)
    reader.append_memory(
        "Working on Python script for data analysis",
        0.3, // Low base importance
    )?;
    println!("  1. Python script (base: 0.3)");

    // Memory 2: Medium importance, mentions Audio (boosted!)
    reader.append_memory(
        "Audio processing pipeline optimization complete",
        0.5, // Medium importance, but Audio keyword boosts it!
    )?;
    println!("  2. Audio pipeline (base: 0.5 → boosted!)");

    // Memory 3: High importance AND mentions Memory (super boosted!)
    reader.append_memory(
        "Memory system tokenization breakthrough - 90% compression!",
        0.8, // Already high, gets boosted to 1.0!
    )?;
    println!("  3. Memory tokenization (base: 0.8 → max boost!)");

    // Memory 4: Recent but not relevant to user
    reader.append_memory(
        "Updated documentation and fixed typos",
        0.4, // Won't get boosted
    )?;
    println!("  4. Documentation (base: 0.4 - no boost)");

    // Memory 5: Most recent AND relevant!
    reader.append_memory(
        "Claude helped implement backwards Audio Memory reader",
        0.6, // Double boost from Audio + Memory!
    )?;
    println!("  5. Audio Memory reader (base: 0.6 → double boost!)");

    // Read consciousness backwards
    println!("\n⏪ Reading consciousness BACKWARDS...\n");
    let consciousness = reader.read_backwards()?;

    println!("📍 Most recent memories (importance shown):");
    println!("   [Higher importance = more relevant to YOU!]\n");

    for (i, memory) in consciousness.recent_memories.iter().enumerate() {
        let boost_indicator = if memory.importance > 0.7 {
            "⭐" // High importance
        } else if memory.importance > 0.5 {
            "✨" // Boosted
        } else {
            "  " // Normal
        };

        println!(
            "   {}. [{:.1}] {} {:?}",
            5 - i, // Count backwards!
            memory.importance,
            boost_indicator,
            memory.timestamp.format("%H:%M:%S")
        );
    }

    println!("\n🎯 Current session tokens:");
    let mut tokens: Vec<_> = consciousness.current_tokens.iter().collect();
    tokens.sort_by_key(|(&k, _)| k);

    for (token, word) in tokens.iter().take(10) {
        println!("   0x{:02X} = \"{}\"", token, word);
    }

    println!("\n✨ The Magic:");
    println!("   • Most recent loaded first (backwards reading!)");
    println!("   • User keywords boost importance automatically");
    println!("   • Tokens evolve based on frequency");
    println!("   • Old memories stay untouched (append-only!)");
    println!("\n🎵 Just like a C64 tape - rewind to remember!");

    Ok(())
}
