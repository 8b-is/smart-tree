//! Temporal Context Analysis Demo
//! 
//! Shows how Smart Tree can analyze context over time to find patterns

use std::path::PathBuf;
use anyhow::Result;
use st::context_gatherer::{ContextGatherer, GatherConfig};
use st::context_gatherer::temporal::TemporalResolution;

fn main() -> Result<()> {
    println!("=== Smart Tree Temporal Context Analysis Demo ===\n");
    
    // Get current directory as the project
    let project_path = std::env::current_dir()?;
    println!("Project: {}\n", project_path.display());
    
    // Configure gatherer
    let mut config = GatherConfig::default();
    config.project_identifiers = vec![
        "smart-tree".to_string(),
        project_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string(),
    ];
    
    // Gather contexts
    println!("📊 Gathering context from AI tools...");
    let mut gatherer = ContextGatherer::new(project_path.clone(), config);
    gatherer.gather_all()?;
    
    println!("Found {} contexts\n", gatherer.contexts().len());
    
    // Test different temporal resolutions
    let resolutions = vec![
        (TemporalResolution::Hour, "Hourly"),
        (TemporalResolution::Day, "Daily"),
        (TemporalResolution::Week, "Weekly"),
        (TemporalResolution::Month, "Monthly"),
    ];
    
    for (resolution, name) in resolutions {
        println!("📅 {} Analysis:", name);
        println!("=" .repeat(50));
        
        let patterns = gatherer.analyze_temporal(resolution);
        
        println!("Work Sessions: {}", patterns.work_sessions.len());
        if let Some(first_session) = patterns.work_sessions.first() {
            println!("  First: {} - {} ({} activities)", 
                first_session.start_time.format("%Y-%m-%d %H:%M"),
                first_session.end_time.format("%Y-%m-%d %H:%M"),
                first_session.total_activities
            );
        }
        if let Some(last_session) = patterns.work_sessions.last() {
            println!("  Last: {} - {} ({} activities)", 
                last_session.start_time.format("%Y-%m-%d %H:%M"),
                last_session.end_time.format("%Y-%m-%d %H:%M"),
                last_session.total_activities
            );
        }
        
        println!("\nPeak Activity Times: {}", patterns.peak_times.len());
        for (i, peak) in patterns.peak_times.iter().take(3).enumerate() {
            println!("  {}. {} (intensity: {:.2})", 
                i + 1,
                peak.timestamp.format("%Y-%m-%d %H:%M"),
                peak.intensity
            );
        }
        
        println!("\nMomentum: {:.3} ({})", 
            patterns.momentum,
            if patterns.momentum > 0.1 { "📈 Increasing" }
            else if patterns.momentum < -0.1 { "📉 Decreasing" }
            else { "➡️ Stable" }
        );
        
        println!("Total Duration: {} days", patterns.total_duration.num_days());
        println!("Active Days: {}", patterns.active_days);
        
        if !patterns.periodic_patterns.is_empty() {
            println!("\nPeriodic Patterns:");
            for pattern in &patterns.periodic_patterns {
                println!("  {}: {}", pattern.period_type, pattern.peak_periods.join(", "));
            }
        }
        
        println!();
    }
    
    // Test temporal decay
    println!("🕰️ Temporal Decay Analysis:");
    println!("=" .repeat(50));
    
    let original_top = gatherer.contexts().iter()
        .take(5)
        .map(|c| (c.ai_tool.clone(), c.relevance_score))
        .collect::<Vec<_>>();
    
    println!("Original top contexts:");
    for (tool, score) in &original_top {
        println!("  {} - score: {:.3}", tool, score);
    }
    
    // Apply decay (30 day half-life)
    gatherer.apply_temporal_decay(30.0);
    
    println!("\nAfter 30-day half-life decay:");
    for (i, context) in gatherer.contexts().iter().take(5).enumerate() {
        let original_score = original_top.get(i).map(|(_, s)| *s).unwrap_or(0.0);
        println!("  {} - score: {:.3} (was {:.3})", 
            context.ai_tool, 
            context.relevance_score,
            original_score
        );
    }
    
    // Create temporal waves
    println!("\n🌊 Temporal Wave Analysis:");
    println!("=" .repeat(50));
    
    let wave_grid = gatherer.create_temporal_waves(TemporalResolution::Day);
    let resonance_peaks = wave_grid.find_resonance_peaks();
    
    println!("Found {} resonance peaks (high activity convergence)", resonance_peaks.len());
    for (i, peak) in resonance_peaks.iter().take(5).enumerate() {
        println!("  {}. {}", 
            i + 1,
            peak.format("%Y-%m-%d")
        );
    }
    
    println!("\n✅ Temporal analysis complete!");
    println!("\nInsights:");
    println!("- Your work patterns show when you're most active with this project");
    println!("- Momentum indicates if your engagement is increasing or decreasing");
    println!("- Resonance peaks show days with high cross-tool activity");
    println!("- Temporal decay helps prioritize recent context over old");
    
    Ok(())
}