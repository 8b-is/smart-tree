//! Smart Tree Terminal Interface Demo
//!
//! Shows how the terminal can anticipate developer needs!
//!
//! Requires the `tui` feature:
//! ```bash
//! cargo run --example terminal_demo --features tui
//! ```

#[cfg(feature = "tui")]
use anyhow::Result;
#[cfg(feature = "tui")]
use st::terminal::SmartTreeTerminal;

#[cfg(feature = "tui")]
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

#[cfg(not(feature = "tui"))]
fn main() {
    eprintln!("This example requires the 'tui' feature.");
    eprintln!("Run with: cargo run --example terminal_demo --features tui");
}

// Trisha says: "It's like having an assistant who files your taxes while you sleep!" 💤
