// Example demonstrating the native quantum scanner
// This shows how Smart Tree can emit quantum format directly during traversal

use st::quantum_scanner::QuantumScanner;
use std::io;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // Get path from command line or use current directory
    let path = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());

    let path = Path::new(&path);

    // Create quantum scanner with stdout as writer
    let stdout = io::stdout();
    let mut scanner = QuantumScanner::new(stdout.lock());

    // Scan and emit quantum format directly
    scanner.scan(path)?;

    Ok(())
}
