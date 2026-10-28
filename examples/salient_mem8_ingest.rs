//! Example: GitHub-user salient MEM8 ingestion over a local collection.
//!
//! Run (after cargo build --example):
//!   cargo run --offline --example salient_mem8_ingest -- \
//!     --user standardgalactic \
//!     --collection /data/standardgalactic \
//!     --review /data/.agent-state/research/standardgalactic-20261028/pass-02/aggregate-02/next-source-review.jsonl \
//!     --limit 5
//!
//! This remembers code + thoughts and emits family Rust suggestions for salient parts.
//! It directly supports continuing the standardgalactic research at scale.

use anyhow::Result;
use std::path::PathBuf;
use st::salient;

fn main() -> Result<()> {
    // In real use parse args; here a tiny hardcoded demo using the engine.
    println!("Salient MEM8 Ingestion Engine (family Rust target)");
    println!("This builds on the pass-02 research to remember salient code+thoughts.");

    // Demo call (does not require the full 10k collection for smoke).
    // When run against real data + review jsonl it attaches the research decisions.
    let report = salient::ingest_github_user_collection(
        "demo-user",
        &PathBuf::from("."),
        &PathBuf::from("/tmp/demo.m8"),
        None,
    )?;

    println!("Ingested for demo-user: {} repos, {} thoughts, {} samples",
             report.repos_processed, report.total_thoughts, report.total_code_samples);

    // Show a conversion example
    let sample = "def rate(a, b): return a / 3.14";
    println!("\nExample conversion of salient pattern:\n{}", salient::salient_to_family_rust(sample, "py"));

    println!("\nNext: wire real .m8 write + use on the 9930 remaining queue entries.");
    Ok(())
}