//! MEM8 Developer Personas Demo
//! Shows how each developer gets their own unique wave signature

use anyhow::Result;
use st::mem8::{SmartTreeMem8, PersonaAnalyzer, personas::{DeveloperPersona, StyleSignature, TemporalPattern, EmotionalProfile, ContributionMetrics}};
use std::cmp::Ordering;

// --- Constants for Analysis Thresholds ---
const REFACTOR_TENDENCY_THRESHOLD: f64 = 0.5;
const BUGFIX_RATIO_THRESHOLD: f64 = 0.5;
const FEATURE_RATIO_THRESHOLD: f64 = 0.5;
const NIGHT_OWL_THRESHOLD: f64 = -0.3;
const EARLY_BIRD_THRESHOLD: f64 = 0.3;
const WEEKEND_WARRIOR_THRESHOLD: f64 = 0.3;
const POSITIVITY_THRESHOLD: f64 = 0.5;
const FRUSTRATION_THRESHOLD: f64 = 0.5;
const HUMOR_THRESHOLD: f64 = 0.2;
const TOP_EXPERTISE_AREAS: usize = 5;

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
                        display_persona(developer, persona);
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

fn display_persona(developer: &str, persona: &DeveloperPersona) {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("👤 Developer: {}", developer);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    display_coding_style(&persona.style_signature);
    display_work_schedule(&persona.temporal_pattern);
    display_emotional_profile(&persona.emotional_profile);
    display_expertise(&persona.expertise_map);
    display_contribution_stats(&persona.metrics);

    println!();
}

fn display_coding_style(style: &StyleSignature) {
    println!("💻 Coding Style:");
    println!("   Average commit size: {:.0} lines", style.avg_commit_size);
    
    if style.refactor_tendency > REFACTOR_TENDENCY_THRESHOLD {
        println!("   🏗️  Architect - Loves big refactors!");
    } else if style.bugfix_ratio > BUGFIX_RATIO_THRESHOLD {
        println!("   🔧 Fixer - Squashes bugs like a pro!");
    } else if style.feature_ratio > FEATURE_RATIO_THRESHOLD {
        println!("   ✨ Builder - Creates new features!");
    } else {
        println!("   🎯 Generalist - Does a bit of everything!");
    }
    
    println!("   Feature work: {:.0}%", style.feature_ratio * 100.0);
    println!("   Bug fixes: {:.0}%", style.bugfix_ratio * 100.0);
    println!("   Documentation: {:.0}%", style.documentation_ratio * 100.0);
    println!("   Tests: {:.0}%", style.test_ratio * 100.0);
}

fn display_work_schedule(temporal: &TemporalPattern) {
    println!("\n⏰ Work Schedule:");
    let chronotype = temporal.chronotype;
    if chronotype < NIGHT_OWL_THRESHOLD {
        println!("   🦉 Night Owl (chronotype: {:.2})", chronotype);
        println!("   Most productive after dark!");
    } else if chronotype > EARLY_BIRD_THRESHOLD {
        println!("   🐦 Early Bird (chronotype: {:.2})", chronotype);
        println!("   Gets the worm with morning commits!");
    } else {
        println!("   ⏰ Flexible Schedule (chronotype: {:.2})", chronotype);
        println!("   Works throughout the day!");
    }
    
    if temporal.weekend_warrior > WEEKEND_WARRIOR_THRESHOLD {
        println!("   💪 Weekend Warrior - {:.0}% weekend commits!", 
                 temporal.weekend_warrior * 100.0);
    }
    
    println!("   Consistency score: {:.2}/1.0", temporal.consistency);
    
    let peak_hour = temporal.active_hours
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .map(|(hour, _)| hour)
        .unwrap_or(0);
    println!("   Peak coding hour: {}:00", peak_hour);
}

fn display_emotional_profile(emotional: &EmotionalProfile) {
    println!("\n😊 Emotional Profile:");
    
    let emoji = if emotional.positivity > POSITIVITY_THRESHOLD { "😊" } 
               else if emotional.frustration > FRUSTRATION_THRESHOLD { "😤" } 
               else { "😐" };
    
    println!("   Overall mood: {}", emoji);
    println!("   Positivity: {:.0}%", emotional.positivity * 100.0);
    println!("   Excitement: {:.0}%", emotional.excitement * 100.0);
    println!("   Frustration: {:.0}%", emotional.frustration * 100.0);
    
    if emotional.humor > HUMOR_THRESHOLD {
        println!("   🤣 Has a sense of humor!");
    }
}

fn display_expertise(expertise_map: &std::collections::HashMap<String, f64>) {
    println!("\n🎯 Top Expertise Areas:");
    let mut expertise: Vec<_> = expertise_map.iter().collect();
    expertise.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(Ordering::Equal));
    
    for (area, score) in expertise.iter().take(TOP_EXPERTISE_AREAS) {
        let bars = "█".repeat((score * 10.0) as usize);
        println!("   {} {} ({:.0}%)", area, bars, score * 100.0);
    }
}

fn display_contribution_stats(metrics: &ContributionMetrics) {
    println!("\n📈 Contribution Stats:");
    println!("   Total commits: {}", metrics.total_commits);
    println!("   Lines added: {:>8}", metrics.total_additions);
    println!("   Lines removed: {:>6}", metrics.total_deletions);
    println!("   Files touched: {}", metrics.files_touched);
    println!("   Active for {} days", metrics.active_days);
    
    let days_active = (metrics.last_commit - metrics.first_commit).num_days();
    if days_active > 0 {
        let commits_per_day = metrics.total_commits as f32 / days_active as f32;
        println!("   Average: {:.2} commits/day", commits_per_day);
    }
}