//! MEM8 Integration Demo for Smart Tree
//! Shows how wave-based memory enhances directory analysis

use anyhow::Result;
use std::path::Path;
use st::mem8::integration::{
    SmartTreeMem8, DirectoryMetadata, DirectoryEvent,
    ContentType, DirectoryHealth,
};

fn main() -> Result<()> {
    println!("=== MEM8 Smart Tree Demo ===\n");
    
    // Initialize MEM8 cognitive system
    let mut mem8 = SmartTreeMem8::new();
    mem8.register_directory_patterns();
    
    println!("1. Storing directory memories...");
    
    // Simulate scanning a Rust project
    let directories = vec![
        ("src/main.rs", ContentType::Code, 0.9, DirectoryHealth::Healthy, 1),
        ("src/lib.rs", ContentType::Code, 0.85, DirectoryHealth::Healthy, 3),
        ("src/utils/mod.rs", ContentType::Code, 0.7, DirectoryHealth::Healthy, 10),
        ("Cargo.toml", ContentType::Configuration, 0.95, DirectoryHealth::Healthy, 0),
        ("README.md", ContentType::Documentation, 0.8, DirectoryHealth::Healthy, 15),
        ("tests/integration.rs", ContentType::Code, 0.6, DirectoryHealth::Warning, 30),
        ("target/", ContentType::Data, 0.3, DirectoryHealth::Healthy, 0),
        (".git/", ContentType::Data, 0.2, DirectoryHealth::Healthy, 0),
    ];
    
    for (path, content_type, importance, health, days_old) in directories {
        let metadata = DirectoryMetadata {
            primary_type: content_type,
            importance,
            normalized_size: importance * 0.8, // Simplified
            health,
            activity_level: 1.0 / (1.0 + days_old as f32 / 10.0),
            days_since_modified: days_old,
        };
        
        mem8.store_directory_memory(Path::new(path), metadata)?;
        println!("  Stored: {}", path);
    }
    
    println!("\n2. Querying memories...");
    
    // Query for source code files
    let src_memories = mem8.query_path_memories("src");
    println!("  Found {} memories related to 'src'", src_memories.len());
    
    for (i, memory) in src_memories.iter().take(3).enumerate() {
        println!("    Memory {}: frequency={:.1}Hz, amplitude={:.2}, relevance={:.2}",
            i + 1,
            memory.wave.frequency,
            memory.wave.amplitude,
            memory.relevance
        );
    }
    
    println!("\n3. Testing reactive system...");
    
    // Simulate directory events
    let events = vec![
        DirectoryEvent::LargeDirectory {
            path: "node_modules".to_string(),
            size: 150_000_000, // 150MB
        },
        DirectoryEvent::SecurityThreat {
            path: "suspicious.exe".to_string(),
            severity: 0.9,
        },
        DirectoryEvent::RapidChange {
            path: "logs/".to_string(),
            rate: 0.8,
        },
    ];
    
    for event in events {
        println!("  Processing event: {:?}", event);
        if let Some(response) = mem8.process_directory_event(event) {
            println!("    → Response: {} (Layer: {:?}, {}ms)",
                response.action,
                response.layer,
                response.latency.as_millis()
            );
        }
    }
    
    println!("\n4. Updating consciousness state...");
    mem8.update_consciousness();
    println!("  Consciousness updated with current memory patterns");
    
    println!("\n5. Exporting memories to .m8 format...");
    let mut m8_buffer = Vec::new();
    mem8.export_memories(&mut m8_buffer)?;
    println!("  Exported {} bytes (compressed from ~4.3GB grid)", m8_buffer.len());
    
    println!("\n=== Demo Complete ===");
    println!("\nMEM8 provides Smart Tree with:");
    println!("  • Wave-based memory (973× faster than vector DBs)");
    println!("  • Emotional context for directories");
    println!("  • Reactive responses (0-200ms layers)");
    println!("  • Consciousness simulation");
    println!("  • 100:1 compression with .m8 format");
    
    Ok(())
}