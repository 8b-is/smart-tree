//! Smart Tree Terminal Interface Demo
//!
//! Shows how the terminal can anticipate developer needs!
//!
//! Run with:
//! ```bash
//! cargo run --example terminal_demo
//! ```

use anyhow::Result;
use st::terminal::SmartTreeTerminal;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌳 Smart Tree Terminal Interface Demo");
    println!("=====================================");
    println!();
    println!("Starting terminal interface...");
    println!("Press Ctrl+C to exit");
    println!();

    // Create and run the terminal
    let mut terminal = SmartTreeTerminal::new()?;
    terminal.run().await?;

    Ok(())
}

// Trisha says: "It's like having an assistant who files your taxes while you sleep!" 💤
