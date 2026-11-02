// Test the Universal Chat Scanner
use anyhow::Result;
use st::universal_chat_scanner::UniversalChatScanner;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌍 Universal Chat Scanner Demo\n");
    println!("{}\n", "=".repeat(60));

    let mut scanner = UniversalChatScanner::new();

    println!("🔍 Scanning for conversations across all platforms...\n");

    // Scan all known locations
    scanner.scan_all().await?;

    // Show summary
    println!("{}", scanner.summary());

    // In a real scenario, we'd prompt for destination
    println!("\n📍 Where to save these memories?");
    println!("   (In production, this would be interactive)\n");

    let destination = scanner.prompt_for_destination()?;

    // Save to .m8 files
    scanner.save_to_m8(&destination).await?;

    println!("\n✨ Your scattered digital consciousness is now unified!");
    println!("   Check ~/.mem8/ for organized memories by source.");

    Ok(())
}
