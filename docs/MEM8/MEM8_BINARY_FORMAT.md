# .mem8 Binary Format Specification

## Overview

The .mem8 binary format provides ultra-compact semantic context storage with typical 85-95% size reduction compared to YAML/JSON while maintaining instant parsing and CRC validation.

## Binary Structure

### File Header (16 bytes)
```
[0-3]   Magic: 0x4D454D38 ("MEM8")
[4-5]   Version: u16 (0x0100 = v1.0)
[6-7]   Flags: u16
[8-11]  CRC32: u32 (of entire file after this field)
[12-15] Offset to index: u32
```

### Flags (16 bits)
```
Bit 0: Compressed (1=zstd compressed)
Bit 1: Encrypted (1=encrypted)
Bit 2: Has parent (1=inherits from parent)
Bit 3: Has snapshots (1=includes code snapshots)
Bit 4: Has metrics (1=includes compilation metrics)
Bit 5-15: Reserved
```

### String Table
All strings stored once, referenced by u16 index:
```
[0-1]   Count: u16
[2-3]   Total size: u16
Then for each string:
  [0-1] Length: u16
  [2-N] UTF-8 bytes
```

### Core Sections

#### 1. Identity Section (Type 0x01)
```
[0]     Section type: 0x01
[1-2]   Section length: u16
[3-4]   Path string index: u16
[5]     Project type: u8 (enum)
[6-7]   Purpose string index: u16
[8-11]  Version: u32 (semantic version packed)
[12-15] Created timestamp: u32 (unix time)
[16-19] Modified timestamp: u32
```

#### 2. Context Section (Type 0x02)
```
[0]     Section type: 0x02
[1-2]   Section length: u16
[3]     Concept count: u8
Then for each concept:
  [0-1] Name index: u16
  [2-3] Description index: u16
  [4]   Importance: u8 (0-255)
```

#### 3. Structure Section (Type 0x03)
```
[0]     Section type: 0x03
[1-2]   Section length: u16
[3]     Entry count: u8
Then for each entry:
  [0-1] Path index: u16
  [2-3] Purpose index: u16
  [4]   Flags: u8
    Bit 0: Is directory
    Bit 1: Has .mem8
    Bit 2: Is key file
    Bit 3: Is historic
```

#### 4. Compilation Section (Type 0x04)
```
[0]     Section type: 0x04
[1-2]   Section length: u16
[3]     Status: u8 (0=failed, 1=partial, 2=success)
[4-7]   Last build: u32 (unix timestamp)
[8]     Error count: u8
[9]     Warning count: u8
```

#### 5. Cache Section (Type 0x05)
```
[0]     Section type: 0x05
[1-2]   Section length: u16
[3-6]   Directory CRC: u32
[7-38]  Content SHA256: [u8; 32]
[39-42] Expires: u32 (unix timestamp)
```

#### 6. AI Context Section (Type 0x06)
```
[0]     Section type: 0x06
[1-2]   Section length: u16
[3]     Understanding: u8 (0=surface, 1=moderate, 2=deep)
[4]     Hint count: u8
[5]     Historic count: u8
Then hints and historic events as string indices
```

#### 7. Relationships Section (Type 0x07)
```
[0]     Section type: 0x07
[1-2]   Section length: u16
[3]     Upstream count: u8
[4]     Downstream count: u8
Then string indices for each
```

#### 8. Sensor Arbitration Section (Type 0x08) - Special for MEM8
```
[0]     Section type: 0x08
[1-2]   Section length: u16
[3-6]   Subconscious weight: f32 (typically 0.3)
[7-10]  LLM weight: f32 (typically 0.7)
[11]    Override threshold: u8 (scaled 0-255, typically 204 for 0.8)
[12-15] Independence date: u32 (unix date of implementation)
```

### Compact Archive Format (.m8a)

For multiple .mem8 files in one archive:
```
Header (32 bytes):
[0-3]   Magic: 0x4D384152 ("M8AR")  
[4-5]   Version: u16
[6-7]   File count: u16
[8-11]  Total size: u32
[12-15] Creation time: u32
[16-31] Root checksum: [u8; 16] (MD5 for speed)

Then for each file:
[0-1]   Path length: u16
[2-N]   Path UTF-8
[N+1-N+4] File offset: u32
[N+5-N+8] File size: u32

Followed by concatenated .mem8 binary files
```

## Encoding Examples

### Concept Encoding
Instead of:
```yaml
key_concepts:
  - wave_patterns: "Fundamental data structure"
```

Binary (11 bytes vs 54 bytes YAML):
```
[02]    Concept section
[00 09] Length: 9 bytes
[01]    Count: 1
[00 0A] Name index: 10 ("wave_patterns")
[00 1B] Desc index: 27 ("Fundamental data structure")
[FF]    Importance: 255 (maximum)
```

### Compilation Status
Instead of:
```yaml
compilation:
  status: success
  last_build: 2025-01-05T11:00:00Z
  errors: []
  warnings: ["unused import"]
```

Binary (10 bytes vs 108 bytes YAML):
```
[04]        Section type
[00 08]     Length: 8 bytes
[02]        Status: success
[65 9A BC 10] Timestamp
[00]        0 errors
[01]        1 warning
```

## Compression Strategy

### zstd Dictionary Training
Train compression dictionary on common patterns:
- Common strings: "rust_library", "purpose", "src/"
- Common paths: "Cargo.toml", "lib.rs"
- Common concepts: "wave_patterns", "memory", "sensor"

Expected compression: 60-80% additional size reduction

### Integer Packing
- Use varint encoding for larger integers
- Pack booleans into bit flags
- Use u8/u16 instead of u32/u64 where possible

## Performance Characteristics

### Read Performance
```rust
// Instant CRC validation (16 bytes read)
let header = read_header(file)?;
if !validate_crc(header.crc32) {
    return cache_hit(); // Use cached version
}

// Section jump table for random access
let index_offset = header.index_offset;
let section = read_section_at(index_offset + section_id * 8)?;
```

### Write Performance
- Build string table first (deduplication)
- Write sections sequentially
- Calculate CRC32 while writing
- Compress if >1KB

## Size Comparisons

### Single .mem8 File
```
YAML:    ~4KB (typical)
JSON:    ~3.5KB (minified)
Binary:  ~400 bytes (90% reduction)
Compressed: ~200 bytes (95% reduction)
```

### Full Project Archive
```
YAML archive:     ~150KB (16 directories)
JSON archive:     ~120KB (minified)
Binary archive:   ~12KB (90% reduction)
Compressed:       ~4KB (97% reduction)
```

## Implementation

### Rust Structure
```rust
#[repr(C, packed)]
struct Mem8Header {
    magic: [u8; 4],
    version: u16,
    flags: u16,
    crc32: u32,
    index_offset: u32,
}

#[repr(u8)]
enum SectionType {
    Identity = 0x01,
    Context = 0x02,
    Structure = 0x03,
    Compilation = 0x04,
    Cache = 0x05,
    AiContext = 0x06,
    Relationships = 0x07,
    SensorArbitration = 0x08,
}

struct StringTable {
    strings: Vec<String>,
    index_map: HashMap<String, u16>,
}

impl StringTable {
    fn get_or_insert(&mut self, s: &str) -> u16 {
        if let Some(&idx) = self.index_map.get(s) {
            return idx;
        }
        let idx = self.strings.len() as u16;
        self.strings.push(s.to_string());
        self.index_map.insert(s.to_string(), idx);
        idx
    }
}
```

### Binary Reader
```rust
pub struct Mem8Reader {
    data: Vec<u8>,
    string_table: StringTable,
    sections: HashMap<SectionType, (u32, u16)>, // offset, length
}

impl Mem8Reader {
    pub fn parse(data: &[u8]) -> Result<Self> {
        // Validate header
        let header = Self::read_header(data)?;
        
        // Read string table
        let string_table = Self::read_strings(data, 16)?;
        
        // Build section index
        let sections = Self::read_index(data, header.index_offset)?;
        
        Ok(Self { data: data.to_vec(), string_table, sections })
    }
    
    pub fn get_purpose(&self) -> Result<&str> {
        let identity = self.read_section(SectionType::Identity)?;
        let purpose_idx = u16::from_le_bytes([identity[6], identity[7]]);
        Ok(&self.string_table.strings[purpose_idx as usize])
    }
}
```

## Advantages

1. **Size**: 90-97% smaller than text formats
2. **Speed**: Instant CRC validation, O(1) section access
3. **Integrity**: Built-in checksums
4. **Portability**: Fixed byte order (little endian)
5. **Extensibility**: New section types without breaking compatibility
6. **Compression**: Optional zstd for further reduction

## Usage

```bash
# Convert YAML to binary
mem8 compile config.yaml -o config.mem8

# Create compressed archive
mem8 archive /project -o project.m8a --compress

# Validate binary file
mem8 verify config.mem8

# Extract single field (no full parse needed)
mem8 get config.mem8 --field purpose
```

This binary format achieves the efficiency goals while maintaining all the semantic richness needed for AI understanding and project navigation.

*"From 4KB YAML to 400 bytes binary - because every byte counts at 973x speed!"*