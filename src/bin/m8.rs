// M8 Format Tools - Validate, Inspect, and Debug M8 Files
// Implementing Omni's vision for forward-compatible, CRC-validated memory format
// "Unknown sections must be skipped and preserved" - The path to immortal data

use anyhow::{Result, Context};
use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;
use crc32fast::Hasher;
use base64::Engine;
use serde_json::json;

#[derive(Parser)]
#[command(
    name = "m8",
    about = "M8 Format Tools - Validate, inspect, and manage MEM8 files",
    long_about = "Tools for working with the MEM8 wave-based memory format.\n\
                  Provides validation, inspection, and debugging capabilities\n\
                  for ensuring format stability and forward compatibility."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a MEM8 file (magic, CRC32, section bounds)
    Validate {
        /// Path to the MEM8 file
        file: PathBuf,
        
        /// Verbose output with detailed checks
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Inspect a MEM8 file (show section table, sizes, hashes)
    Inspect {
        /// Path to the MEM8 file
        file: PathBuf,
        
        /// Show raw hex dump of headers
        #[arg(short = 'x', long)]
        hex: bool,
        
        /// Limit output to first N sections
        #[arg(short, long)]
        limit: Option<usize>,
    },
    
    /// Generate CRC32 for a file
    Crc {
        /// Path to the file
        file: PathBuf,
    },
    
    /// Create a golden test vector from a MEM8 file
    Golden {
        /// Input MEM8 file
        input: PathBuf,
        
        /// Output path for golden vector
        output: PathBuf,
    },
    
    /// Print section offsets and lengths (machine-readable index)
    Index {
        /// Path to the MEM8 file
        file: PathBuf,
        
        /// Output in JSON format
        #[arg(short, long)]
        json: bool,
    },
}

// MEM8 Format Constants
const MAGIC: &[u8; 4] = b"MEM8";
const VERSION: u8 = 1;

#[derive(Debug)]
struct M8Header {
    magic: [u8; 4],
    version: u8,
    flags: u8,
    section_count: u16,
    total_size: u32,
    crc32: u32,
}

#[derive(Debug)]
struct M8Section {
    id: [u8; 4],
    offset: u32,
    size: u32,
    flags: u8,
    _reserved: [u8; 3],
}

impl M8Header {
    fn read_from(file: &mut File) -> Result<Self> {
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        
        let mut buffer = [0u8; 12];
        file.read_exact(&mut buffer)?;
        
        Ok(Self {
            magic,
            version: buffer[0],
            flags: buffer[1],
            section_count: u16::from_le_bytes([buffer[2], buffer[3]]),
            total_size: u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
            crc32: u32::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]),
        })
    }
    
    fn validate(&self) -> Result<()> {
        if &self.magic != MAGIC {
            anyhow::bail!("Invalid magic: expected MEM8, got {:?}", 
                String::from_utf8_lossy(&self.magic));
        }
        
        if self.version != VERSION {
            eprintln!("Warning: Version mismatch. Expected {}, got {}", VERSION, self.version);
        }
        
        Ok(())
    }
}

impl M8Section {
    fn read_from(file: &mut File) -> Result<Self> {
        let mut buffer = [0u8; 16];
        file.read_exact(&mut buffer)?;
        
        Ok(Self {
            id: [buffer[0], buffer[1], buffer[2], buffer[3]],
            offset: u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
            size: u32::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]),
            flags: buffer[12],
            _reserved: [buffer[13], buffer[14], buffer[15]],
        })
    }
    
    fn id_str(&self) -> String {
        String::from_utf8_lossy(&self.id).to_string()
    }
}

fn validate_file(path: &PathBuf, verbose: bool) -> Result<()> {
    let mut file = File::open(path)
        .context("Failed to open MEM8 file")?;
    
    // Read and validate header
    let header = M8Header::read_from(&mut file)?;
    header.validate()?;
    
    if verbose {
        println!("✓ Magic: MEM8");
        println!("✓ Version: {}", header.version);
        println!("  Sections: {}", header.section_count);
        println!("  Total size: {} bytes", header.total_size);
    }
    
    // Read section table
    let mut sections = Vec::new();
    for _ in 0..header.section_count {
        sections.push(M8Section::read_from(&mut file)?);
    }
    
    // Validate section bounds
    let mut prev_end = 16 + (header.section_count as u32 * 16); // Header + section table size
    for (_i, section) in sections.iter().enumerate() {
        if section.offset < prev_end {
            anyhow::bail!("Section overlaps with previous data");
        }
        
        if section.offset + section.size > header.total_size {
            anyhow::bail!("Section extends beyond file bounds");
        }
        
        if verbose {
            println!("✓ Section: {} ({} bytes at 0x{:08x})", 
                section.id_str(), section.size, section.offset);
        }
        
        prev_end = section.offset + section.size;
    }
    
    // Calculate and verify CRC32
    file.seek(SeekFrom::Start(0))?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    
    // Zero out CRC field for calculation
    contents[12..16].fill(0);
    
    let mut hasher = Hasher::new();
    hasher.update(&contents);
    let calculated_crc = hasher.finalize();
    
    if calculated_crc != header.crc32 {
        anyhow::bail!("CRC32 mismatch: expected 0x{:08x}, got 0x{:08x}", 
            header.crc32, calculated_crc);
    }
    
    if verbose {
        println!("✓ CRC32: 0x{:08x}", header.crc32);
    }
    
    println!("✅ MEM8 file is valid!");
    Ok(())
}

fn inspect_file(path: &PathBuf, hex: bool, limit: Option<usize>) -> Result<()> {
    let mut file = File::open(path)
        .context("Failed to open MEM8 file")?;
    
    // Read header
    let header = M8Header::read_from(&mut file)?;
    
    println!("MEM8 File Inspector");
    println!("===================");
    println!("Magic:    {}", String::from_utf8_lossy(&header.magic));
    println!("Version:  {}", header.version);
    println!("Flags:    0x{:02x}", header.flags);
    println!("Sections: {}", header.section_count);
    println!("Size:     {} bytes", header.total_size);
    println!("CRC32:    0x{:08x}", header.crc32);
    println!();
    
    if hex {
        // Show hex dump of header
        file.seek(SeekFrom::Start(0))?;
        let mut header_bytes = [0u8; 16];
        file.read_exact(&mut header_bytes)?;
        
        println!("Header (hex):");
        for (i, chunk) in header_bytes.chunks(8).enumerate() {
            print!("  {:04x}: ", i * 8);
            for byte in chunk {
                print!("{:02x} ", byte);
            }
            println!();
        }
        println!();
    }
    
    // Read and display section table
    println!("Section Table:");
    println!("ID    Offset      Size        Flags");
    println!("----  ----------  ----------  -----");
    
    let max_sections = limit.unwrap_or(header.section_count as usize);
    for _i in 0..header.section_count.min(max_sections as u16) {
        let section = M8Section::read_from(&mut file)?;
        println!("{:4}  0x{:08x}  {:10}  0x{:02x}", 
            section.id_str(), section.offset, section.size, section.flags);
    }
    
    if limit.is_some() && max_sections < header.section_count as usize {
        println!("... {} more sections", header.section_count as usize - max_sections);
    }
    
    Ok(())
}

fn calculate_crc(path: &PathBuf) -> Result<()> {
    let mut file = File::open(path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    
    let mut hasher = Hasher::new();
    hasher.update(&contents);
    let crc = hasher.finalize();
    
    println!("CRC32: 0x{:08x}", crc);
    Ok(())
}

fn create_golden(input: &PathBuf, output: &PathBuf) -> Result<()> {
    // Read input file
    let mut file = File::open(input)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    
    // Create golden vector with metadata
    let mut hasher = Hasher::new();
    hasher.update(&contents);
    let crc32_hex = format!("0x{:08x}", hasher.finalize());
    
    let preview_size = contents.len().min(256);
    let hex_dump = hex::encode(&contents[..preview_size]);
    
    let base64_str = base64::engine::general_purpose::STANDARD.encode(&contents);
    
    let golden = serde_json::json!({
        "source": input.display().to_string(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "size": contents.len(),
        "crc32": crc32_hex,
        "hex_dump": hex_dump,
        "base64": base64_str
    });
    
    // Write golden vector
    let mut output_file = File::create(output)?;
    output_file.write_all(serde_json::to_string_pretty(&golden)?.as_bytes())?;
    
    println!("✅ Golden vector created: {}", output.display());
    Ok(())
}

fn print_index(path: &PathBuf, json_format: bool) -> Result<()> {
    let mut file = File::open(path)
        .context("Failed to open MEM8 file")?;
    
    // Read header
    let header = M8Header::read_from(&mut file)?;
    header.validate()?;
    
    // Read section table
    let mut sections = Vec::new();
    for i in 0..header.section_count {
        let section = M8Section::read_from(&mut file)?;
        sections.push((i, section));
    }
    
    if json_format {
        // JSON output for machine consumption
        let index = json!({
            "header": {
                "version": header.version,
                "sections": header.section_count,
                "total_size": header.total_size,
                "crc32": format!("0x{:08x}", header.crc32),
            },
            "sections": sections.iter().map(|(i, s)| json!({
                "index": i,
                "id": s.id_str(),
                "offset": s.offset,
                "size": s.size,
                "flags": format!("0x{:02x}", s.flags),
            })).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&index)?);
    } else {
        // Human-readable output
        println!("MEM8 Index");
        println!("==========");
        println!("Sections: {}", header.section_count);
        println!();
        println!("Index  ID    Offset       Size");
        println!("-----  ----  -----------  -----------");
        for (i, section) in sections {
            println!("{:5}  {:4}  0x{:08x}   {:10}",
                i, section.id_str(), section.offset, section.size);
        }
    }
    
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Validate { file, verbose } => {
            validate_file(&file, verbose)?;
        }
        Commands::Inspect { file, hex, limit } => {
            inspect_file(&file, hex, limit)?;
        }
        Commands::Crc { file } => {
            calculate_crc(&file)?;
        }
        Commands::Golden { input, output } => {
            create_golden(&input, &output)?;
        }
        Commands::Index { file, json } => {
            print_index(&file, json)?;
        }
    }
    
    Ok(())
}

// Trish says: "This validator sparkles with reliability! Every byte accounted for!" 💜