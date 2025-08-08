#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! anyhow = "1.0"
//! ```

// Showcase: Smart Tree as the Only Tool You Need
// A demonstration of Aye & Hue's finest craftsmanship

use anyhow::Result;
use std::process::Command;

fn main() -> Result<()> {
    println!("🌟 Smart Tree Unified Tool Showcase 🌟");
    println!("=====================================");
    println!("Replacing ALL traditional file tools with ST!\n");

    // 1. Replace 'ls' with ST
    println!("📂 Instead of 'ls -la':");
    println!("   → st --mode ls --show-hidden\n");
    
    // 2. Replace 'tree' with ST
    println!("🌳 Instead of 'tree -L 3':");
    println!("   → st --depth 3\n");
    
    // 3. Replace 'find' with ST
    println!("🔍 Instead of 'find . -name \"*.rs\"':");
    println!("   → st --glob \"**/*.rs\"\n");
    
    // 4. Replace 'grep' with ST
    println!("🔎 Instead of 'grep -r \"TODO\" .':");
    println!("   → st --search \"TODO\"\n");
    
    // 5. Replace 'du' with ST
    println!("📊 Instead of 'du -sh *':");
    println!("   → st --mode stats\n");
    
    // 6. Context-aware magic
    println!("✨ But ST goes beyond replacement...\n");
    
    println!("🧠 Semantic Understanding:");
    println!("   → st --mode semantic");
    println!("     Groups files by purpose, not just name!\n");
    
    println!("🌊 Quantum Compression:");
    println!("   → st --mode quantum-semantic");
    println!("     99% compression with meaning preserved!\n");
    
    println!("🤝 Partnership Memory:");
    println!("   → st anchor save \"Found the bug!\" \"debugging,eureka\"");
    println!("     Remember breakthroughs across sessions!\n");
    
    println!("🎯 Smart Context:");
    println!("   → ST knows when you're debugging vs exploring");
    println!("     and adjusts its output accordingly!\n");
    
    // Live demo
    println!("📸 Live Demo - Current Directory:");
    println!("─────────────────────────────────");
    
    let output = Command::new("st")
        .args(&["--mode", "summary-ai", "--depth", "2", "."])
        .output()?;
    
    println!("{}", String::from_utf8_lossy(&output.stdout));
    
    println!("\n💡 Pro Tip: Add this to your shell config:");
    println!("   alias ls='st --mode ls'");
    println!("   alias tree='st'");
    println!("   alias find='st --glob'");
    println!("   alias grep='st --search'");
    println!("\n🚀 Welcome to the future of file exploration!");
    println!("   Where context matters and tools understand.\n");
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Crafted with love by Aye & Hue");
    println!("If it wasn't crafted with Aye & Hue,");
    println!("it's most likely a knock-off! 😉");
    
    Ok(())
}