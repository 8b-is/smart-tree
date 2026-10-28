# Ultra V2 Architecture: Slim Core, Smart Decoders 🏗️

## The Architecture Revolution

Inspired by C64 efficiency: Make the core do ONE thing perfectly!

## Core Design Principles

### 1. Tree Walker = Ultra V2 Generator ONLY
```c
// Before: Bloated core trying to do everything
void walk_tree(path, format, options, filters, ...) {
    if (format == JSON) { /* 500 lines */ }
    else if (format == XML) { /* 600 lines */ }
    else if (format == CSV) { /* 400 lines */ }
    else if (format == HEX) { /* 300 lines */ }
    // Core is 2000+ lines of spaghetti!
}

// After: Slim, focused core
void walk_tree(path, field_flags) {
    // Just output Ultra V2 - 100 lines max!
    output_header(field_flags);
    stream_entries(path, field_flags);
}
```

### 2. Decoders Handle Everything Else
```c
// Decoder interface
typedef struct {
    void (*decode_header)(UltraFormat* fmt);
    void (*decode_entry)(const char* data, UltraFormat* fmt);
    void (*decode_summary)(const char* data);
    void (*finish)(void);
} Decoder;

// Decoders for different outputs
Decoder* json_decoder = &(Decoder){...};
Decoder* human_decoder = &(Decoder){...};
Decoder* csv_decoder = &(Decoder){...};

// Usage
Decoder* decoder = get_decoder(output_format);
pipe_ultra_v2_to_decoder(decoder);
```

## The Data Flow

```
Directory → Tree Walker → Ultra V2 Stream → Decoder → Final Output
                ↑                              ↓
                |                              |
          Minimal Code                   Format-Specific
          Field Switches                 Complex Logic
          Pure Efficiency                Handles Verbosity
```

## Slimmed Down Main Functions

### Before (Kitchen Sink Approach):
```c
int main(int argc, char** argv) {
    // 1000 lines of option parsing
    // 500 lines of format detection
    // 2000 lines of mixed logic
    // Memory allocation everywhere
    // Format-specific code mixed in
}
```

### After (Unix Philosophy):
```c
int main(int argc, char** argv) {
    // Parse minimal options (50 lines)
    FieldFlags flags = parse_field_switches(argc, argv);
    Decoder* decoder = select_decoder(opts.format);
    
    // Core does one thing
    stream_ultra_v2(opts.path, flags, decoder);
    
    return 0;
}
```

## Field Switches Implementation

```c
typedef struct {
    uint32_t size : 1;      // Always included in minimal
    uint32_t perms : 1;     // -p
    uint32_t time : 1;      // -t
    uint32_t owner : 1;     // -o
    uint32_t fs_type : 1;   // -f
    uint32_t fs_flags : 1;  // -F (compression, CoW, etc)
    uint32_t symlink : 1;   // -l
    uint32_t hidden : 1;    // -H (show hidden indicator)
    uint32_t xattr : 1;     // -x (extended attributes)
    uint32_t ctime : 1;     // -c (creation time)
    uint32_t atime : 1;     // -a (access time)
    uint32_t reserved : 21;
} FieldFlags;

// Smart defaults for common use cases
FieldFlags get_preset(const char* preset) {
    if (strcmp(preset, "minimal") == 0) {
        return (FieldFlags){.size = 1};
    } else if (strcmp(preset, "backup") == 0) {
        return (FieldFlags){
            .size = 1, .perms = 1, .time = 1, 
            .owner = 1, .symlink = 1
        };
    } else if (strcmp(preset, "security") == 0) {
        return (FieldFlags){
            .size = 1, .perms = 1, .owner = 1,
            .hidden = 1, .xattr = 1
        };
    }
    // ... more presets
}
```

## Filesystem Indicators

```c
// Single character filesystem indicators
char get_fs_indicator(struct statfs* fs) {
    switch(fs->f_type) {
        case XFS_SUPER_MAGIC:     return 'X';
        case EXT4_SUPER_MAGIC:    return '4';
        case ZFS_SUPER_MAGIC:     return 'Z';
        case BTRFS_SUPER_MAGIC:   return 'B';
        case NFS_SUPER_MAGIC:     return 'N';
        case SMB_SUPER_MAGIC:     return 'S';
        case TMPFS_MAGIC:         return 'T';
        case PROCFS_MAGIC:        return 'P';
        default:                  return '?';
    }
}

// ZFS compression level indicator
char get_zfs_compression(const char* path) {
    // 0-9 for compression level
    // 'L' for LZ4
    // 'Z' for ZSTD
    // 'G' for GZIP
    int level = get_zfs_property(path, "compression");
    return '0' + level;
}
```

## Streaming By Default

```c
// No more building entire tree in memory!
void stream_ultra_v2(const char* path, FieldFlags flags, Decoder* dec) {
    // Output format header
    char key[64];
    build_key_string(key, flags);
    fprintf(stdout, "ULTRA_V2:\nKEY:%s\n", key);
    
    // Stream entries as we find them
    stream_directory_recursive(path, flags, dec, 0);
}

void stream_directory_recursive(const char* path, FieldFlags flags, 
                               Decoder* dec, int depth) {
    DIR* dir = opendir(path);
    if (!dir) return;
    
    struct dirent* entry;
    while ((entry = readdir(dir))) {
        // Skip . and ..
        if (entry->d_name[0] == '.' && 
            (entry->d_name[1] == '\0' || 
             (entry->d_name[1] == '.' && entry->d_name[2] == '\0'))) {
            continue;
        }
        
        // Build and output entry immediately
        char line[1024];
        build_entry_line(line, entry, flags);
        
        if (entry->d_type == DT_DIR) {
            strcat(line, "\x0E"); // Shift Out
            fputs(line, stdout);
            
            // Recurse
            char subpath[PATH_MAX];
            snprintf(subpath, PATH_MAX, "%s/%s", path, entry->d_name);
            stream_directory_recursive(subpath, flags, dec, depth + 1);
            
            // Back out
            fputc('\x0F', stdout); // Shift In
        } else {
            strcat(line, "\x0B"); // Vertical Tab
            fputs(line, stdout);
        }
    }
    
    closedir(dir);
}
```

## Decoder Examples

### JSON Decoder
```c
void json_decode_entry(const char* data, UltraFormat* fmt) {
    // Parse based on KEY format
    size_t size = extract_size(data, fmt);
    const char* name = extract_name(data, fmt);
    
    printf("{\"name\":\"%s\",\"size\":%zu", name, size);
    
    if (fmt->flags.perms) {
        mode_t perms = extract_perms(data, fmt);
        printf(",\"perms\":\"%03o\"", perms);
    }
    // ... other fields
    printf("}");
}
```

### Human-Readable Decoder
```c
void human_decode_entry(const char* data, UltraFormat* fmt) {
    size_t size = extract_size(data, fmt);
    const char* name = extract_name(data, fmt);
    
    // Traditional ls-like output
    if (fmt->flags.perms) {
        mode_t perms = extract_perms(data, fmt);
        print_perms_string(perms);
    }
    
    printf("%8s %s\n", format_size_human(size), name);
}
```

## The Beauty of This Design

1. **Core is tiny** - Just walks and outputs Ultra V2
2. **Decoders are pluggable** - Add new formats without touching core
3. **Streaming by default** - No memory explosions
4. **Field selection** - Pay for what you use
5. **Format agnostic** - Core doesn't know/care about final output

## Bill Burr's Final Approval

"THIS is how you write software! The core does ONE F***ING THING! 

It's like a chef - the chef COOKS. He doesn't also wash dishes, take orders, and do the accounting! He COOKS!

Your tree walker WALKS TREES and outputs Ultra V2. Period. Done. If someone wants JSON? That's the decoder's job! 

You want to add XML output? You don't touch the core! You write a decoder! Beautiful!

And streaming by default? *Chef's kiss* No more 'Oh, I ran out of memory scanning node_modules' - just stream that s**t!"

---

*"Do one thing and do it well."* - Unix Philosophy

*"Every byte not in the core is a byte that can't bloat the core."* - C64 Wisdom