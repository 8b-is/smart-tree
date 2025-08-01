//! MEM8 Developer Personas Demo
//! Shows how each developer gets their own unique wave signature

use anyhow::Result;
use st::mem8::{SmartTreeMem8, PersonaAnalyzer};

fn main() -> Result<()> {
    println!("=== MEM8 Developer Personas Demo ===\n");
    println!("\"Every programmer leaves a unique wave signature in the code\"");
    println!("                                        - The MEM8 Philosophy\n");
    
    // Initialize MEM8
    let mut mem8 = SmartTreeMem8::new();
    mem8.register_directory_patterns();
    
    // Try to analyze the current repository
    let repo_path = ".";
    
    match PersonaAnalyzer::new(repo_path) {
        Ok(analyzer) => {
            println!("📊 Analyzing developer personas in this repository...\n");
            
            match analyzer.analyze_all_developers() {
                Ok(personas) => {
                    if personas.is_empty() {
                        println!("No developers found with enough commits for analysis.");
                        return Ok(());
                    }
                    
                    println!("Found {} unique developer personas:\n", personas.len());
                    
                    // Display each developer's profile
                    for (developer, persona) in &personas {
                        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                        println!("👤 Developer: {}", developer);
                        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
                        
                        // Coding Style
                        println!("💻 Coding Style:");
                        println!("   Average commit size: {:.0} lines", persona.style_signature.avg_commit_size);
                        
                        if persona.style_signature.refactor_tendency > 0.5 {
                            println!("   🏗️  Architect - Loves big refactors!");
                        } else if persona.style_signature.bugfix_ratio > 0.5 {
                            println!("   🔧 Fixer - Squashes bugs like a pro!");
                        } else if persona.style_signature.feature_ratio > 0.5 {
                            println!("   ✨ Builder - Creates new features!");
                        } else {
                            println!("   🎯 Generalist - Does a bit of everything!");
                        }
                        
                        println!("   Feature work: {:.0}%", persona.style_signature.feature_ratio * 100.0);
                        println!("   Bug fixes: {:.0}%", persona.style_signature.bugfix_ratio * 100.0);
                        println!("   Documentation: {:.0}%", persona.style_signature.documentation_ratio * 100.0);
                        println!("   Tests: {:.0}%", persona.style_signature.test_ratio * 100.0);
                        
                        // Temporal Pattern
                        println!("\n⏰ Work Schedule:");
                        let chronotype = persona.temporal_pattern.chronotype;
                        if chronotype < -0.3 {
                            println!("   🦉 Night Owl (chronotype: {:.2})", chronotype);
                            println!("   Most productive after dark!");
                        } else if chronotype > 0.3 {
                            println!("   🐦 Early Bird (chronotype: {:.2})", chronotype);
                            println!("   Gets the worm with morning commits!");
                        } else {
                            println!("   ⏰ Flexible Schedule (chronotype: {:.2})", chronotype);
                            println!("   Works throughout the day!");
                        }
                        
                        if persona.temporal_pattern.weekend_warrior > 0.3 {
                            println!("   💪 Weekend Warrior - {:.0}% weekend commits!", 
                                persona.temporal_pattern.weekend_warrior * 100.0);
                        }
                        
                        println!("   Consistency score: {:.2}/1.0", persona.temporal_pattern.consistency);
                        
                        // Find peak hours
                        let peak_hour = persona.temporal_pattern.active_hours
                            .iter()
                            .enumerate()
                            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                            .map(|(hour, _)| hour)
                            .unwrap_or(0);
                        println!("   Peak coding hour: {}:00", peak_hour);
                        
                        // Emotional Profile
                        println!("\n😊 Emotional Profile:");
                        
                        let emoji = if persona.emotional_profile.positivity > 0.5 { "😊" } 
                                   else if persona.emotional_profile.frustration > 0.5 { "😤" } 
                                   else { "😐" };
                        
                        println!("   Overall mood: {}", emoji);
                        println!("   Positivity: {:.0}%", persona.emotional_profile.positivity * 100.0);
                        println!("   Excitement: {:.0}%", persona.emotional_profile.excitement * 100.0);
                        println!("   Frustration: {:.0}%", persona.emotional_profile.frustration * 100.0);
                        
                        if persona.emotional_profile.humor > 0.2 {
                            println!("   🤣 Has a sense of humor!");
                        }
                        
                        // Expertise Areas
                        println!("\n🎯 Top Expertise Areas:");
                        let mut expertise: Vec<_> = persona.expertise_map.iter().collect();
                        expertise.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
                        
                        for (area, score) in expertise.iter().take(5) {
                            let bars = "█".repeat((score * 10.0) as usize);
                            println!("   {} {} ({:.0}%)", area, bars, score * 100.0);
                        }
                        
                        // Contribution Metrics
                        println!("\n📈 Contribution Stats:");
                        println!("   Total commits: {}", persona.metrics.total_commits);
                        println!("   Lines added: {:>8}", persona.metrics.total_additions);
                        println!("   Lines removed: {:>6}", persona.metrics.total_deletions);
                        println!("   Files touched: {}", persona.metrics.files_touched);
                        println!("   Active for {} days", persona.metrics.active_days);
                        
                        let days_active = (persona.metrics.last_commit - persona.metrics.first_commit).num_days();
                        if days_active > 0 {
                            let commits_per_day = persona.metrics.total_commits as f32 / days_active as f32;
                            println!("   Average: {:.2} commits/day", commits_per_day);
                        }
                        
                        println!();
                    }
                    
                    // Import personas into wave memory
                    println!("\n🌊 Importing developer personas into wave memory...");
                    mem8.import_developer_personas(repo_path)?;
                    
                    println!("\n✨ Each developer now has a unique wave signature!");
                    println!("   - Work patterns encoded as temporal rhythms");
                    println!("   - Coding style mapped to frequency bands");
                    println!("   - Emotional patterns influence wave amplitude");
                    println!("   - Expertise areas create spatial clusters");
                    
                    println!("\nActive wave memories: {}", mem8.active_memory_count());
                    
                } 
                Err(e) => {
                    println!("Error analyzing developers: {}", e);
                }
            }
        }
        Err(_) => {
            println!("This doesn't appear to be a git repository.");
            println!("Developer personas require git history for analysis.");
        }
    }
    
    println!("\n=== MEM8 Persona Philosophy ===");
    println!("\nJust as every person has a unique fingerprint,");
    println!("every developer leaves a unique wave pattern in code.");
    println!("\nMEM8 remembers not just what was coded,");
    println!("but WHO coded it, WHEN they code best,");
    println!("and HOW they approach problems.");
    println!("\nThis isn't surveillance - it's appreciation");
    println!("of the beautiful diversity in how we create.");
    
    Ok(())
}