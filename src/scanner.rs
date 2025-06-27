//
// -----------------------------------------------------------------------------
//  WELCOME TO THE JUNGLE! ...The filesystem jungle, that is. 🌴
//
//  You've found scanner.rs, the intrepid explorer and engine room of st.
//  This module is the Indiana Jones of our codebase. It bravely dives into
//  the deepest, darkest directories, dodges `.gitignore` traps, inspects
//  every file for treasure (metadata), and reports back its findings.
//
//  So grab your hat, and let's go on an adventure!
//
//  Brought to you by The Cheet - making filesystem traversal a rock concert! 🥁🧻
// -----------------------------------------------------------------------------
//

use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder}; // For powerful gitignore-style pattern matching.
use regex::Regex; // For user-defined find patterns.
use std::collections::{HashMap, HashSet}; // Our trusty hash-based collections.
use std::fs; // Filesystem operations, the bread and butter here.
use std::io::{BufRead, BufReader}; // For efficient reading, especially for content search.
use std::path::{Path, PathBuf}; // Path manipulation is key.
use std::sync::mpsc; // For streaming results from a worker thread.
use std::time::SystemTime; // To know when files were last touched.
use walkdir::{DirEntry, WalkDir}; // The excellent `walkdir` crate does the actual directory walking.

// Unix-specific imports for richer metadata like permissions, UID, GID.
// On other platforms, we'll use sensible defaults.
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

/// # FileNode: The Ultimate Backstage Pass
///
/// Every file and directory we meet gets one of these. It's a VIP pass that
/// holds all the juicy details: its name, size, when it was last cool (modified),
/// and whether it's on the super-secret "ignored" list. It's the atom of our
/// `st` universe.
#[derive(Debug, Clone)]
pub struct FileNode {
    /// The full path to the file or directory. The source of truth for location!
    pub path: PathBuf,
    /// Is it a directory? `true` if yes, `false` if it's a file or symlink.
    pub is_dir: bool,
    /// Size of the file in bytes. For directories, this is often 0 or metadata-dependent.
    pub size: u64,
    /// File permissions (e.g., `rwxr-xr-x`). Stored as a u32, typically from Unix mode.
    pub permissions: u32,
    /// User ID of the owner (Unix-specific).
    pub uid: u32,
    /// Group ID of the owner (Unix-specific).
    pub gid: u32,
    /// Timestamp of the last modification. Tells us how fresh or ancient a file is.
    pub modified: SystemTime,
    /// Is it a symbolic link? `true` if yes. We handle these with care.
    pub is_symlink: bool,
    /// Is it a hidden file (e.g., starts with a `.` on Unix)?
    pub is_hidden: bool,
    /// Did we encounter a "Permission Denied" error when trying to access this?
    /// Important for gracefully handling parts of the filesystem we can't read.
    pub permission_denied: bool,
    /// Is this file or directory ignored based on `.gitignore` or default ignore rules?
    pub is_ignored: bool,
    /// The depth of this entry relative to the scan root (root is depth 0).
    pub depth: usize,
    /// The specific type of the file (e.g., RegularFile, Symlink, Executable).
    pub file_type: FileType,
    /// A category assigned based on extension or name, used for coloring and context.
    /// (e.g., Rust, Python, Image, Archive).
    pub category: FileCategory,
    /// For content search: A list of byte offsets (hex positions) where the search keyword was found.
    /// `None` if no search was performed or no matches.
    pub search_matches: Option<Vec<usize>>,
    /// The filesystem type this file resides on
    pub filesystem_type: FilesystemType,
}

/// # FileType: Distinguishing Different Kinds of Filesystem Objects
///
/// This enum helps us categorize entries beyond just "file" or "directory".
/// It's especially useful on Unix-like systems where you have sockets, pipes, etc.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    Directory,   // A folder, a container of other things.
    RegularFile, // Your everyday, garden-variety file.
    Symlink,     // A pointer to another file or directory.
    Executable,  // A file that can be run (has execute permissions).
    Socket,      // A Unix domain socket.
    Pipe,        // A named pipe (FIFO).
    BlockDevice, // A block special file (e.g., /dev/sda).
    CharDevice,  // A character special file (e.g., /dev/tty).
}

/// # FilesystemType: Identifying the underlying filesystem
///
/// This enum represents different filesystem types with single-character codes
/// for compact display. The mapping is designed to be memorable and intuitive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilesystemType {
    Ext4,    // '4' - The most common Linux filesystem
    Ext3,    // '3' - Older ext filesystem
    Ext2,    // '2' - Even older ext filesystem
    Xfs,     // 'X' - XFS filesystem
    Btrfs,   // 'B' - Btrfs (B-tree filesystem)
    Zfs,     // 'Z' - ZFS filesystem
    Ntfs,    // 'N' - Windows NTFS
    Fat32,   // 'F' - FAT32
    ExFat,   // 'E' - exFAT
    Apfs,    // 'A' - Apple File System
    Hfs,     // 'H' - HFS+ (older Mac)
    Nfs,     // 'R' - Remote NFS mount
    Smb,     // 'S' - SMB/CIFS network filesystem
    Tmpfs,   // 'T' - Temporary filesystem (RAM)
    Procfs,  // 'P' - /proc virtual filesystem
    Sysfs,   // 'Y' - /sys virtual filesystem
    Devfs,   // 'D' - /dev virtual filesystem
    Mem8,    // 'M' - Mem8 filesystem (custom!)
    Unknown, // '?' - Unknown filesystem
}

impl FilesystemType {
    /// Get the single-character code for this filesystem type
    pub fn to_char(&self) -> char {
        match self {
            FilesystemType::Ext4 => '4',
            FilesystemType::Ext3 => '3',
            FilesystemType::Ext2 => '2',
            FilesystemType::Xfs => 'X',
            FilesystemType::Btrfs => 'B',
            FilesystemType::Zfs => 'Z',
            FilesystemType::Ntfs => 'N',
            FilesystemType::Fat32 => 'F',
            FilesystemType::ExFat => 'E',
            FilesystemType::Apfs => 'A',
            FilesystemType::Hfs => 'H',
            FilesystemType::Nfs => 'R',
            FilesystemType::Smb => 'S',
            FilesystemType::Tmpfs => 'T',
            FilesystemType::Procfs => 'P',
            FilesystemType::Sysfs => 'Y',
            FilesystemType::Devfs => 'D',
            FilesystemType::Mem8 => 'M',
            FilesystemType::Unknown => '?',
        }
    }

    /// Check if this is a virtual filesystem that should be skipped
    pub fn is_virtual(&self) -> bool {
        matches!(
            self,
            FilesystemType::Procfs
                | FilesystemType::Sysfs
                | FilesystemType::Devfs
                | FilesystemType::Tmpfs
        )
    }

    /// Check if this filesystem type should be shown by default
    /// (only "interesting" filesystems based on platform)
    pub fn should_show_by_default(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            matches!(
                self,
                FilesystemType::Ext4
                    | FilesystemType::Ext3
                    | FilesystemType::Xfs
                    | FilesystemType::Btrfs
                    | FilesystemType::Zfs
                    | FilesystemType::Nfs
                    | FilesystemType::Smb
                    | FilesystemType::Mem8
            )
        }
        #[cfg(target_os = "macos")]
        {
            matches!(
                self,
                FilesystemType::Apfs
                    | FilesystemType::Hfs
                    | FilesystemType::Nfs
                    | FilesystemType::Smb
                    | FilesystemType::Mem8
            )
        }
        #[cfg(target_os = "windows")]
        {
            matches!(
                self,
                FilesystemType::Ntfs
                    | FilesystemType::Fat32
                    | FilesystemType::ExFat
                    | FilesystemType::Mem8
            )
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            // Show all non-virtual filesystems on other platforms
            !self.is_virtual()
        }
    }
}

/// # FileCategory: Adding Semantic Flavor to Files
///
/// This enum provides a higher-level categorization based on common file extensions
/// or names. It's primarily used for display purposes, like coloring output,
/// and can also help in understanding the nature of a directory's contents.
/// Trish loves how this makes the tree output more intuitive!
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileCategory {
    // --- Programming Languages ---
    Rust,       // .rs
    Python,     // .py, .pyw, .pyx, .pyi
    JavaScript, // .js, .mjs, .cjs
    TypeScript, // .ts, .tsx
    Java,       // .java, .class, .jar
    C,          // .c, .h
    Cpp,        // .cpp, .cc, .cxx, .hpp, .hxx
    Go,         // .go
    Ruby,       // .rb
    PHP,        // .php
    Shell,      // .sh, .bash, .zsh, .fish

    // --- Markup & Data Formats ---
    Markdown, // .md, .markdown
    Html,     // .html, .htm
    Css,      // .css, .scss, .sass, .less
    Json,     // .json, .jsonc
    Yaml,     // .yaml, .yml
    Xml,      // .xml, .svg (SVG is XML-based)
    Toml,     // .toml

    // --- Build Systems & Configuration ---
    Makefile,   // Makefile, makefile, GNUmakefile
    Dockerfile, // Dockerfile, .dockerfile
    GitConfig,  // .gitignore, .gitconfig, .gitmodules

    // --- Archives & Compressed Files ---
    Archive, // .zip, .tar, .gz, .bz2, .xz, .7z, .rar

    // --- Media Files ---
    Image, // .jpg, .jpeg, .png, .gif, .bmp, .ico, .webp
    Video, // .mp4, .avi, .mkv, .mov, .wmv, .flv, .webm
    Audio, // .mp3, .wav, .flac, .aac, .ogg, .wma

    // --- System & Binary Files ---
    SystemFile, // Special system files like swap.img, vmlinuz
    Binary,     // Executables, shared libraries (.exe, .dll, .so, .dylib, .o, .a)

    // --- Default/Fallback ---
    Unknown, // If we can't categorize it, it's a mysterious Unknown!
}

/// # TreeStats: The Final Scoreboard
///
/// After the concert is over, this is where we see how we did. It's the
/// scoreboard that tracks total files, total directories, the biggest hits
/// (largest files), and more. It's the answer to "So, how was the show?"
#[derive(Debug, Default)]
pub struct TreeStats {
    /// Total number of files encountered (excluding directories).
    pub total_files: u64,
    /// Total number of directories encountered.
    pub total_dirs: u64,
    /// Total size of all files (in bytes).
    pub total_size: u64,
    /// A map of file extensions to their counts (e.g., {"rs": 10, "toml": 2}).
    pub file_types: HashMap<String, u64>,
    /// Top N largest files found (path and size). N is usually 10.
    pub largest_files: Vec<(u64, PathBuf)>,
    /// Top N newest files found (path and modification time).
    pub newest_files: Vec<(SystemTime, PathBuf)>,
    /// Top N oldest files found (path and modification time).
    pub oldest_files: Vec<(SystemTime, PathBuf)>,
}

impl TreeStats {
    /// Updates the statistics based on a newly processed `FileNode`.
    /// This method is called for each non-permission-denied node.
    pub fn update_file(&mut self, node: &FileNode) {
        if node.is_dir {
            self.total_dirs += 1;
        } else {
            // It's a file!
            self.total_files += 1;
            self.total_size += node.size;

            // Track file extensions for type distribution.
            if let Some(ext) = node.path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    *self.file_types.entry(ext_str.to_string()).or_insert(0) += 1;
                }
            }

            // --- Update Top N Lists ---
            // These lists are kept sorted and truncated to maintain a fixed size (e.g., top 10).

            // Update largest files: Add, sort by size (desc), truncate.
            self.largest_files.push((node.size, node.path.clone()));
            self.largest_files.sort_by(|a, b| b.0.cmp(&a.0)); // Largest first
            self.largest_files.truncate(10); // Keep only the top 10

            // Update newest files: Add, sort by modification time (desc), truncate.
            self.newest_files.push((node.modified, node.path.clone()));
            self.newest_files.sort_by(|a, b| b.0.cmp(&a.0)); // Newest first
            self.newest_files.truncate(10);

            // Update oldest files: Add, sort by modification time (asc), truncate.
            self.oldest_files.push((node.modified, node.path.clone()));
            self.oldest_files.sort_by(|a, b| a.0.cmp(&b.0)); // Oldest first
            self.oldest_files.truncate(10);
        }
    }
}

/// # ScannerConfig: The Rider for our Rock Star Scanner
///
/// This is the list of demands for our scanner. "Don't show me hidden files,"
/// "I only want to see files bigger than a tour bus," "Ignore the messy backstage
/// area (`.gitignore`)." We build this from the user's command-line arguments
/// to make sure the scanner puts on the exact show the user wants to see.
pub struct ScannerConfig {
    /// Maximum depth to traverse into subdirectories.
    pub max_depth: usize,
    /// Should symbolic links be followed? (Currently always `false`).
    pub follow_symlinks: bool,
    /// Should `.gitignore` files be respected?
    pub respect_gitignore: bool,
    /// Should hidden files (starting with `.`) be shown?
    pub show_hidden: bool,
    /// Should ignored files/directories be shown (usually in brackets)?
    pub show_ignored: bool,
    /// An optional regex pattern to filter files/directories by name.
    pub find_pattern: Option<Regex>,
    /// An optional file extension to filter by (e.g., "rs").
    pub file_type_filter: Option<String>,
    /// Optional minimum file size filter.
    pub min_size: Option<u64>,
    /// Optional maximum file size filter.
    pub max_size: Option<u64>,
    /// Optional filter for files newer than a specific date.
    pub newer_than: Option<SystemTime>,
    /// Optional filter for files older than a specific date.
    pub older_than: Option<SystemTime>,
    /// Should the scanner use its built-in list of default ignore patterns
    /// (like `node_modules`, `__pycache__`, `target/`)?
    pub use_default_ignores: bool,
    /// An optional keyword to search for within file contents.
    pub search_keyword: Option<String>,
    /// Should filesystem type indicators be shown?
    pub show_filesystems: bool,
}

// --- Default Ignore Patterns: The "Please Don't Play These Songs" List ---
// Every band has songs they'd rather not play. This is our list of files and
// directories (`node_modules`, `target/`, etc.) that we usually skip to keep
// the show clean and focused on the hits. A tidy tree is a happy tree!
const DEFAULT_IGNORE_PATTERNS: &[&str] = &[
    // Version control systems (but not all hidden dirs like .ssh)
    ".git",
    ".svn",
    ".hg",
    ".bzr",
    "_darcs",
    // Python artifacts
    "__pycache__",
    "*.pyc",
    "*.pyo",
    "*.pyd",
    ".Python",
    ".pytest_cache",
    ".tox",
    ".coverage",
    "*.egg-info",
    ".eggs",
    // Node.js / JavaScript artifacts
    "node_modules",
    ".npm",
    ".yarn",
    ".pnpm-store",
    "bower_components",
    ".next",
    ".nuxt",
    // General cache directories often found in projects
    ".cache", // Common cache dir name
    // Virtual environments
    "venv",
    "env",
    "ENV",
    "virtualenv",
    ".venv",
    ".env",
    "conda-meta",
    // Build/compilation artifacts from various languages/systems
    "target", // Rust
    "build",
    "dist",
    "out",
    "bin",
    "obj", // Common build output dirs
    "*.o",
    "*.a",
    "*.so",
    "*.dll",
    "*.dylib", // Object files, libraries
    // Package manager caches/data
    ".cargo",
    ".rustup", // Rust
    ".gem",
    ".bundle", // Ruby
    // IDEs and editor-specific files/directories
    ".idea",
    ".vscode",
    ".vs", // Common IDE metadata
    "*.swp",
    "*.swo",
    "*~", // Vim/editor backup/swap files
    ".project",
    ".classpath",
    ".settings", // Eclipse/Java
    // Development tool caches
    ".mypy_cache",
    ".ruff_cache",
    ".hypothesis",
    ".pytest_cache",
    ".tox",
    ".coverage",
    ".sass-cache",
    // OS-specific junk files
    ".DS_Store",    // macOS
    "Thumbs.db",    // Windows
    "desktop.ini",  // Windows
    "$RECYCLE.BIN", // Windows recycle bin
    // Common temporary file/directory names and patterns
    "tmp",
    "temp",
    ".tmp",
    ".temp",
    "*.tmp",
    "*.temp",
    "*.log", // Often temporary or verbose
    // More cache directories
    ".sass-cache", // Sass CSS preprocessor
    "__MACOSX",    // macOS archive metadata
    // System directories that are almost never useful to traverse deeply from a user's project root.
    // These are more aggressively ignored if `st` is run on `/`.
    // "proc", "sys", "dev", "lost+found", "mnt", "media", // Handled by DEFAULT_SYSTEM_PATHS
    // Other common ignores
    ".vagrant",
    ".terraform",
];

// Default paths that are almost always too noisy or problematic to scan,
// especially if `st` is run from `/` or a very high-level directory.
// These are typically mount points for virtual filesystems or system-critical areas.
const DEFAULT_SYSTEM_PATHS: &[&str] = &[
    "/proc",
    "/sys",
    "/dev",
    "/run",
    "/tmp",
    "/var/tmp",
    "/lost+found",
    "/mnt",
    "/media",
    "/snap", // Common mount points or special dirs
];

// Specific individual files (absolute paths) that should always be ignored
// due to their special nature (e.g., virtual files representing system memory).
const DEFAULT_IGNORE_FILES: &[&str] = &[
    "/proc/kcore",    // Virtual file representing physical memory, can be huge & slow.
    "/proc/kmsg",     // Kernel messages, can be an infinite stream.
    "/proc/kallsyms", // Kernel symbols, can be large.
];

/// # Scanner: The Rock Star of our Show
///
/// BEHOLD! The `Scanner` itself! This is the main act. It takes the config,
/// the ignore lists, and a path, and it puts on a spectacular show of directory
/// traversal. It's fast, it's smart, and it knows all the best moves.
pub struct Scanner {
    /// The configuration for this scanning operation.
    config: ScannerConfig,
    /// Compiled `GlobSet` from `.gitignore` files, if respected and found.
    gitignore: Option<GlobSet>,
    /// Compiled `GlobSet` from our `DEFAULT_IGNORE_PATTERNS`.
    default_ignores: Option<GlobSet>,
    /// A set of absolute system paths to ignore (e.g., /proc, /sys).
    system_paths: HashSet<PathBuf>,
    /// A set of specific absolute file paths to ignore (e.g., /proc/kcore).
    ignore_files: HashSet<PathBuf>,
    /// The root path from which the scan originates.
    root: PathBuf,
}

impl Scanner {
    /// ## `get_file_category`
    /// Determines a `FileCategory` for a given path and `FileType`.
    /// This function uses a series of heuristics based on file extensions and common names
    /// to classify files into broad categories, useful for display and understanding content.
    /// It's like a quick identification guide for files!
    fn get_file_category(path: &Path, file_type: FileType) -> FileCategory {
        // Directories don't get a specific content category here; their content defines them.
        if matches!(file_type, FileType::Directory) {
            return FileCategory::Unknown;
        }

        // First, check for some very specific system file names.
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name == "swap.img"
                || name == "swapfile"
                || name.starts_with("vmlinuz")
                || name.starts_with("initrd")
            {
                return FileCategory::SystemFile;
            }
        }

        // Primary categorization is by file extension.
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                // --- Programming Languages ---
                "rs" => FileCategory::Rust,
                "py" | "pyw" | "pyx" | "pyi" => FileCategory::Python,
                "js" | "mjs" | "cjs" => FileCategory::JavaScript,
                "ts" | "tsx" => FileCategory::TypeScript,
                "java" | "class" | "jar" => FileCategory::Java,
                "c" | "h" => FileCategory::C,
                "cpp" | "cc" | "cxx" | "hpp" | "hxx" => FileCategory::Cpp,
                "go" => FileCategory::Go,
                "rb" => FileCategory::Ruby,
                "php" => FileCategory::PHP,
                "sh" | "bash" | "zsh" | "fish" | "ps1" | "bat" | "cmd" => FileCategory::Shell,

                // --- Markup/Data ---
                "md" | "markdown" => FileCategory::Markdown,
                "html" | "htm" => FileCategory::Html,
                "css" | "scss" | "sass" | "less" => FileCategory::Css,
                "json" | "jsonc" | "geojson" => FileCategory::Json,
                "yaml" | "yml" => FileCategory::Yaml,
                "xml" | "svg" | "plist" | "kml" | "gpx" => FileCategory::Xml, // SVG and others are XML-based
                "toml" => FileCategory::Toml,

                // --- Build/Config (some are also by name) ---
                "dockerfile" => FileCategory::Dockerfile, // Extension variant
                // .gitignore, .gitconfig are usually by name, handled below

                // --- Archives ---
                "zip" | "tar" | "gz" | "tgz" | "bz2" | "tbz2" | "xz" | "txz" | "7z" | "rar"
                | "iso" | "dmg" => FileCategory::Archive,

                // --- Media ---
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "ico" | "webp" | "tiff" | "tif"
                | "heic" | "heif" => FileCategory::Image,
                "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "mpeg" | "mpg" => {
                    FileCategory::Video
                }
                "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" => FileCategory::Audio,

                // --- Binary/Executable (some overlap with system, but these are common distributable/object formats) ---
                "exe" | "dll" | "so" | "dylib" | "o" | "a" | "lib" | "msi" | "deb" | "rpm"
                | "app" => FileCategory::Binary,

                _ => FileCategory::Unknown, // Extension not recognized
            }
        } else {
            // No extension, or extension parsing failed. Try common filenames.
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                match name {
                    "Makefile" | "makefile" | "GNUmakefile" => FileCategory::Makefile,
                    "Dockerfile" => FileCategory::Dockerfile, // Filename variant
                    ".gitignore" | ".gitconfig" | ".gitattributes" | ".gitmodules" => {
                        FileCategory::GitConfig
                    }
                    // If it's marked as executable by the OS, and we haven't categorized it yet, call it Binary.
                    _ => {
                        if matches!(file_type, FileType::Executable) {
                            FileCategory::Binary
                        } else {
                            FileCategory::Unknown // Truly a mystery!
                        }
                    }
                }
            } else {
                FileCategory::Unknown // Path has no filename component (should be rare for actual files).
            }
        }
    }

    /// ## `Scanner::new` - Constructor
    ///
    /// Creates a new `Scanner` instance. This involves:
    /// 1. Storing the provided `config` and `root` path.
    /// 2. Loading and compiling `.gitignore` patterns if `config.respect_gitignore` is true.
    /// 3. Compiling the `DEFAULT_IGNORE_PATTERNS` if `config.use_default_ignores` is true.
    /// 4. Initializing sets of system paths and specific files to always ignore.
    ///
    /// This setup prepares the scanner for efficient `should_ignore` checks during traversal.
    pub fn new(root: &Path, config: ScannerConfig) -> Result<Self> {
        // Load .gitignore patterns from the root directory if requested.
        let gitignore = if config.respect_gitignore {
            Self::load_gitignore(root)? // This can return None if no .gitignore or error.
        } else {
            None // Not respecting .gitignore.
        };

        // Build the GlobSet for default ignore patterns if requested.
        let default_ignores = if config.use_default_ignores {
            Self::build_default_ignores()? // This can return None if patterns are invalid (unlikely for defaults).
        } else {
            None // Not using default ignores.
        };

        // Initialize the set of system paths to ignore (e.g., /proc, /sys).
        let system_paths: HashSet<PathBuf> = if config.use_default_ignores {
            DEFAULT_SYSTEM_PATHS
                .iter()
                .map(|p_str| PathBuf::from(p_str)) // Convert string slices to PathBufs
                .collect() // Collect into a HashSet for quick lookups.
        } else {
            HashSet::new() // Empty set if not using default ignores.
        };

        // Initialize the set of specific files to ignore (e.g., /proc/kcore).
        let ignore_files: HashSet<PathBuf> = if config.use_default_ignores {
            DEFAULT_IGNORE_FILES
                .iter()
                .map(|p_str| PathBuf::from(p_str))
                .collect()
        } else {
            HashSet::new()
        };

        Ok(Self {
            config,
            gitignore,
            default_ignores,
            system_paths,
            ignore_files,
            root: root.to_path_buf(), // Store a copy of the root path.
        })
    }

    /// ## `build_default_ignores`
    ///
    /// Compiles the `DEFAULT_IGNORE_PATTERNS` array into a `GlobSet` for efficient matching.
    /// This `GlobSet` is used to quickly check if a file/directory name matches any of the
    /// common patterns we want to ignore by default (like `node_modules`, `target/`).
    /// Returns `Ok(Some(GlobSet))` on success, or `Ok(None)` if no patterns (should not happen),
    /// or an `Err` if glob compilation fails (very unlikely for our hardcoded patterns).
    fn build_default_ignores() -> Result<Option<GlobSet>> {
        let mut builder = GlobSetBuilder::new(); // Start with an empty builder.

        // Add each default pattern to the builder.
        for pattern_str in DEFAULT_IGNORE_PATTERNS {
            // Glob::new can fail if the pattern is malformed, but ours should be fine.
            if let Ok(glob) = Glob::new(pattern_str) {
                builder.add(glob);
            }
            // Silently ignore malformed default patterns, though this shouldn't occur.
        }

        // Build the GlobSet from the accumulated patterns.
        // This can fail if, for example, the set is empty or patterns are incompatible,
        // but again, highly unlikely for our predefined set.
        Ok(Some(builder.build()?))
    }

    /// ## `load_gitignore`
    ///
    /// Reads the `.gitignore` file from the specified `root` directory (if it exists)
    /// and compiles its patterns into a `GlobSet`.
    /// Lines starting with `#` (comments) and empty lines are ignored.
    /// Returns `Ok(Some(GlobSet))` if `.gitignore` is found and parsed,
    /// `Ok(None)` if no `.gitignore` file exists, or an `Err` on I/O or parsing issues.
    fn load_gitignore(root: &Path) -> Result<Option<GlobSet>> {
        let gitignore_path = root.join(".gitignore"); // Construct path to .gitignore.
        if !gitignore_path.exists() {
            return Ok(None); // No .gitignore file found, nothing to load.
        }

        let mut builder = GlobSetBuilder::new();
        // Read the entire .gitignore file into a string.
        let content = fs::read_to_string(&gitignore_path)?;

        // Process each line of the .gitignore file.
        for line in content.lines() {
            let trimmed_line = line.trim(); // Remove leading/trailing whitespace.
                                            // Ignore empty lines and lines that are comments (start with '#').
            if !trimmed_line.is_empty() && !trimmed_line.starts_with('#') {
                // Attempt to compile the line as a glob pattern.
                // If successful, add it to our GlobSet builder.
                if let Ok(glob) = Glob::new(trimmed_line) {
                    builder.add(glob);
                }
                // Malformed patterns in user's .gitignore are silently skipped.
            }
        }

        // Build the final GlobSet from all valid patterns.
        Ok(Some(builder.build()?))
    }

    /// Stream nodes as they are discovered
    /// This version of scan is optimized for the `--stream` flag.
    /// It sends `FileNode` objects through the `sender` channel as soon as they are processed.
    /// This allows the formatter to start displaying output immediately, which is great for large directories.
    /// Returns the final `TreeStats` once the scan is complete.
    pub fn scan_stream(&self, sender: mpsc::Sender<FileNode>) -> Result<TreeStats> {
        let mut stats = TreeStats::default();
        // Keep track of directories we've decided to ignore entirely to avoid processing their children.
        // Note: This was `ignored_dirs` in the original, but it's not used beyond `skip_current_dir`.
        // `walkdir` handles skipping children of explicitly ignored dirs.

        // Initialize WalkDir starting from the root path.
        // Configure it with max_depth and whether to follow symlinks from our ScannerConfig.
        let mut walker = WalkDir::new(&self.root)
            .max_depth(self.config.max_depth)
            .follow_links(self.config.follow_symlinks) // Usually false for st
            .into_iter(); // Get an iterator over directory entries.

        // Loop through each entry provided by WalkDir.
        while let Some(entry_result) = walker.next() {
            match entry_result {
                Ok(entry) => {
                    // Successfully read a directory entry.
                    let depth = entry.depth();
                    let path = entry.path();

                    // Determine if this entry should be ignored based on various rules.
                    let is_ignored_by_rules = self.should_ignore(path)?;

                    if is_ignored_by_rules {
                        // The entry matches an ignore rule.
                        if self.config.show_ignored {
                            // If we're showing ignored items, process it but mark as ignored.
                            if let Some(mut node) =
                                self.process_entry(&entry, depth, is_ignored_by_rules)?
                            {
                                // Perform content search if applicable, even for ignored files being shown.
                                if !node.is_dir && self.should_search_file(&node) {
                                    node.search_matches = self.search_in_file(&node.path);
                                }

                                // Send the (ignored) node through the channel.
                                if sender.send(node.clone()).is_err() {
                                    break; // Receiver has disconnected, stop scanning.
                                }

                                // Update stats for ignored items if they aren't permission-denied.
                                // This ensures `show_ignored` gives a full picture.
                                if !node.permission_denied {
                                    stats.update_file(&node);
                                }
                            }
                            // If this ignored item is a directory, tell WalkDir not to descend into it.
                            if entry.file_type().is_dir() {
                                // `ignored_dirs.insert(path.to_path_buf());` // Not strictly needed if just skipping.
                                walker.skip_current_dir();
                            }
                        } else {
                            // We are *not* showing ignored items, and this one is ignored.
                            // If it's a directory, skip its contents. Otherwise, just continue.
                            if entry.file_type().is_dir() {
                                walker.skip_current_dir();
                            }
                            // `continue;` // Implicitly done by not processing further.
                        }
                    } else {
                        // The entry is NOT ignored by rules. Process it normally.
                        if let Some(mut node) = self.process_entry(&entry, depth, false)? {
                            // `is_ignored` is false here
                            // Perform content search if applicable.
                            if !node.is_dir && self.should_search_file(&node) {
                                node.search_matches = self.search_in_file(&node.path);
                            }

                            // Apply filters (size, date, type, find pattern).
                            // A file is included if it's a directory, or it matches filters, or it has a search match.
                            let has_search_match = node
                                .search_matches
                                .as_ref()
                                .map_or(false, |m| !m.is_empty());
                            if node.is_dir || has_search_match || self.should_include(&node) {
                                // Send the processed node through the channel.
                                if sender.send(node.clone()).is_err() {
                                    break; // Receiver disconnected.
                                }

                                // Update statistics for included, non-permission-denied items.
                                if !node.permission_denied {
                                    stats.update_file(&node);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    // An error occurred trying to access a directory entry (e.g., permission denied).
                    if let Some(path) = e.path() {
                        // If the error is associated with a path.
                        let depth = e.depth();
                        // Create a special node representing the permission-denied entry.
                        let node = self.create_permission_denied_node(path, depth);
                        if sender.send(node.clone()).is_err() {
                            break; // Receiver disconnected.
                        }
                        // Still update stats (e.g., directory count) for permission-denied entries if shown.
                        stats.update_file(&node);
                        // Tell WalkDir not to try to descend into this unreadable directory.
                        walker.skip_current_dir();
                    }
                    // If the error is not path-specific, it might be logged or ignored depending on severity.
                    // For now, we primarily handle path-specific errors like permission issues.
                }
            }
        }
        // Scan complete, return the accumulated statistics.
        Ok(stats)
    }

    /// ## `should_search_file`
    /// Determines if the content of a given `FileNode` should be searched for the `search_keyword`.
    /// Search is performed if:
    /// 1. A `search_keyword` is configured.
    /// 2. If `file_type_filter` is set, the file's extension must match it.
    /// 3. If no `file_type_filter`, the file must be of a category typically containing text
    ///    (e.g., source code, markdown, JSON, plain text, etc.). This avoids searching binaries.
    fn should_search_file(&self, node: &FileNode) -> bool {
        // If no search keyword is provided in the config, no files should be searched.
        if self.config.search_keyword.is_none() {
            return false;
        }

        // If a specific file type filter is active (e.g., --type rs),
        // only search files matching that extension.
        if let Some(filter_ext) = &self.config.file_type_filter {
            return match node.path.extension() {
                Some(ext) => ext.to_string_lossy().eq_ignore_ascii_case(filter_ext),
                None => false, // No extension, doesn't match.
            };
        }

        // If no specific type filter, apply a heuristic: search in files that are likely text-based.
        // This is determined by checking common text file extensions first for speed,
        // then by checking the broader `FileCategory`.
        if let Some(ext_str) = node.path.extension().and_then(|e| e.to_str()) {
            match ext_str.to_lowercase().as_str() {
                // Common plain text or config-like extensions
                "txt" | "text" | "log" | "md" | "rst" | "tex" | "org" | "adoc" |
                "ini" | "cfg" | "conf" | "config" | "properties" | "env" | "pem" | "key" |
                "crt" | "csr" | "cnf" | "rules" | "policy" | "example" | "sample" |
                // Data formats often in text
                "csv" | "tsv" | "sql" | "graphql" | "proto" | "thrift" | "hql" | "psql" |
                // Shell/scripting related that are text
                "vim" | "vimrc" | "bashrc" | "zshrc" | "profile" | "gitconfig" | "editorconfig" |
                "npmrc" | "yarnrc" | "babelrc" | "eslintrc" | "prettierrc" | "stylelintrc" |
                "tf" | "tfvars" | // Terraform
                "hcl" | // HashiCorp Configuration Language
                "tfstate" | // Terraform state (JSON)
                "lock" | // Common lock file format (e.g. package-lock.json, Cargo.lock which is TOML)
                "mod" | // Go modules, often text
                "sum" | // Go checksums, text
                "gradle" | // Gradle build scripts (Groovy/Kotlin)
                "sbt" | // Scala Build Tool
                "cabal" | // Haskell Cabal
                "nix" | // Nix expressions
                "dhall" | // Dhall configuration language
                "cue" | // CUE configuration language
                "ipynb" // Jupyter notebooks (JSON-based)
                 => return true,
                _ => {} // Not a common plain text extension, fall through to category check.
            }
        }

        // Fallback to checking the pre-determined FileCategory.
        // This covers source code files and other structured text formats.
        matches!(
            node.category,
            FileCategory::Rust | FileCategory::Python | FileCategory::JavaScript |
            FileCategory::TypeScript | FileCategory::Java | FileCategory::C | // Most source code
            FileCategory::Cpp | FileCategory::Go | FileCategory::Ruby | FileCategory::PHP |
            FileCategory::Shell | FileCategory::Markdown | FileCategory::Html | // Markup
            FileCategory::Css | FileCategory::Json | FileCategory::Yaml | // Data/Style
            FileCategory::Xml | FileCategory::Toml | FileCategory::Makefile | // Config/Build
            FileCategory::Dockerfile | FileCategory::GitConfig |
            FileCategory::Unknown // If category is Unknown, it might still be a text file we don't have a specific category for.
                                  // We err on the side of searching it if no type filter is specified.
                                  // Binary files should ideally be caught by specific categories like `Binary` or `Archive`.
        )
    }

    /// ## `search_in_file`
    ///
    /// Performs a content search for `config.search_keyword` within the specified file.
    /// Reads the file line by line and records the byte offset (0-indexed) of each match.
    /// Returns `Some(Vec<usize>)` with match positions if found, or `None` if no matches
    /// or if the keyword is not set, or if the file cannot be read/is binary.
    /// The search is case-sensitive.
    /// To avoid performance issues with huge files or many matches, it limits the number of reported matches.
    fn search_in_file(&self, path: &Path) -> Option<Vec<usize>> {
        // Ensure there's a keyword to search for.
        let keyword = self.config.search_keyword.as_ref()?;
        if keyword.is_empty() {
            return None;
        } // Don't search for empty string.

        // Attempt to open the file for reading.
        let file = match fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return None, // Cannot open file, so cannot search.
        };

        let mut positions = Vec::new(); // Store byte offsets of matches.
        let reader = BufReader::new(file); // Use a buffered reader for efficiency.
        let mut current_byte_offset: usize = 0; // Track our position in the file.

        // Read and process the file line by line.
        // This is generally more memory-efficient than reading the whole file,
        // and allows us to stop early if it seems like a binary file or too many matches.
        for line_result in reader.lines() {
            match line_result {
                Ok(line_content) => {
                    // Find all occurrences of the keyword in the current line.
                    // `match_indices` gives (byte_offset_in_line, matched_string).
                    for (match_start_in_line, _matched_str) in line_content.match_indices(keyword) {
                        positions.push(current_byte_offset + match_start_in_line);

                        // Performance guard: If we find too many matches, stop early.
                        // This prevents using too much memory or time on files with dense matches.
                        // The limit (e.g., 100) can be adjusted.
                        if positions.len() > 100 {
                            // Adding a special marker or logging could indicate truncation.
                            // For now, just return the matches found so far.
                            return Some(positions);
                        }
                    }
                    // Update the byte offset for the next line.
                    // Add length of the line content + 1 for the newline character.
                    // (Note: This assumes Unix-style newlines (\n). Windows (\r\n) would be +2.
                    // However, `lines()` iterator strips newlines, so `line_content.len()` is just text.
                    // A more robust way might involve tracking bytes read from the BufReader if precision is critical
                    // across OSes, but for typical text files and search, this is a common approach.)
                    // Let's assume `+1` is a reasonable approximation for line terminator length.
                    current_byte_offset += line_content.len() + 1;
                }
                Err(_) => {
                    // An error occurred reading a line (e.g., invalid UTF-8 in a binary file).
                    // Stop searching this file.
                    break;
                }
            }

            // Another performance guard: if the file is excessively large and we're still reading,
            // perhaps cap the total bytes processed or lines read. For now, relies on match limit.
            // e.g., if current_byte_offset > SOME_LARGE_THRESHOLD { break; }
        }

        // Return the collected positions if any matches were found.
        if positions.is_empty() {
            None
        } else {
            Some(positions)
        }
    }

    /// ## `scan` - The Full Scan (Non-Streaming)
    ///
    /// Performs a complete directory scan, collecting all `FileNode`s that meet the criteria
    /// (not ignored, or shown if ignored, and pass filters if any).
    /// This method first traverses the entire directory structure defined by `config.max_depth`,
    /// creating `FileNode` objects for each entry. It then performs a second pass if filters
    /// are active to ensure that directories are only included if they (or their subdirectories)
    /// contain files that match the filters.
    /// Returns a tuple: `(Vec<FileNode>, TreeStats)`.
    /// ## `scan` - The "Scan-It-All-Then-Sort-It-Out" Method
    ///
    /// This is the classic way to scan. It's a two-act show:
    /// 1. **Act I**: Walk through every single file and directory, collecting a huge list of `FileNode`s.
    /// 2. **Act II**: If there are filters, go through that huge list and pick out only the ones that
    ///    match, making sure to keep their parent directories so the tree still makes sense.
    /// It's thorough and great for when you need the whole picture before making decisions.
    pub fn scan(&self) -> Result<(Vec<FileNode>, TreeStats)> {
        let mut all_nodes_collected = Vec::new(); // Stores all nodes initially encountered.
                                                  // `ignored_dirs` was here, but its primary use with `skip_current_dir` is within the loop.
                                                  // If we need to track them for other reasons post-loop, it could be reinstated.

        let mut walker = WalkDir::new(&self.root)
            .max_depth(self.config.max_depth)
            .follow_links(self.config.follow_symlinks)
            .into_iter();

        while let Some(entry_result) = walker.next() {
            match entry_result {
                Ok(entry) => {
                    let depth = entry.depth();
                    let path = entry.path();
                    let is_ignored_by_rules = self.should_ignore(path)?;

                    if is_ignored_by_rules {
                        if self.config.show_ignored {
                            // Process and add the ignored entry.
                            if let Some(mut node) = self.process_entry(&entry, depth, true)? {
                                if !node.is_dir && self.should_search_file(&node) {
                                    node.search_matches = self.search_in_file(&node.path);
                                }
                                all_nodes_collected.push(node);
                            }
                            if entry.file_type().is_dir() {
                                walker.skip_current_dir(); // Don't descend into ignored dirs if showing them.
                            }
                        } else {
                            // Not showing ignored, and it's a directory: skip its contents.
                            if entry.file_type().is_dir() {
                                walker.skip_current_dir();
                            }
                            // If it's a file, it's simply skipped by not adding to `all_nodes_collected`.
                        }
                    } else {
                        // Not ignored by rules, process normally.
                        if let Some(mut node) = self.process_entry(&entry, depth, false)? {
                            if !node.is_dir && self.should_search_file(&node) {
                                node.search_matches = self.search_in_file(&node.path);
                            }
                            all_nodes_collected.push(node);
                        }
                    }
                }
                Err(e) => {
                    // Handle errors like permission denied.
                    if let Some(path) = e.path() {
                        let depth = e.depth();
                        all_nodes_collected.push(self.create_permission_denied_node(path, depth));
                        if e.io_error().map_or(false, |io_err| {
                            io_err.kind() == std::io::ErrorKind::PermissionDenied
                        }) {
                            walker.skip_current_dir(); // Skip unreadable directory.
                        }
                    }
                }
            }
        }

        // If filters are active, we need a second pass to ensure directories are only included
        // if they contain (or lead to) matching files.
        // Also, calculate stats based on the *final* list of nodes.
        let (final_nodes, final_stats) = if self.has_active_filters() {
            self.filter_nodes_and_calculate_stats(all_nodes_collected)
        } else {
            // No filters, so all collected nodes are final. Calculate stats on them.
            let mut stats = TreeStats::default();
            for node in &all_nodes_collected {
                // Only update stats for non-permission-denied items, or items that are directories.
                // (Permission denied files usually have size 0 and aren't "counted" in the same way).
                if !node.permission_denied || node.is_dir {
                    stats.update_file(node);
                }
            }
            (all_nodes_collected, stats)
        };

        Ok((final_nodes, final_stats))
    }

    /// ## `has_active_filters`
    ///
    /// Helper function to quickly check if any of the primary filtering criteria
    /// (find pattern, type, size, date) are currently set in the configuration.
    /// This determines if the second filtering pass (`filter_nodes_and_calculate_stats`) is needed.
    /// Note: `search_keyword` is handled slightly differently; it can make a file appear
    /// even if other filters would exclude it, so it's part of `should_include` logic.
    fn has_active_filters(&self) -> bool {
        self.config.find_pattern.is_some()
            || self.config.file_type_filter.is_some()
            || self.config.min_size.is_some()
            || self.config.max_size.is_some()
            || self.config.newer_than.is_some()
            || self.config.older_than.is_some()
        // `search_keyword` isn't a primary filter in this sense; it's an inclusion criterion.
    }

    /// ## `filter_nodes_and_calculate_stats` (Formerly `filter_nodes_with_ancestors`)
    ///
    /// This crucial function takes all nodes collected during the initial traversal
    /// and filters them based on the `ScannerConfig`. It ensures that:
    /// 1. Files are included if they directly match all active filters OR if they contain a search match.
    /// 2. Directories are included if they themselves match a `--find` pattern OR
    ///    if they are an ancestor of an included file.
    /// It then calculates `TreeStats` based on this final, filtered list of nodes.
    /// This replaces the older `filter_nodes_with_ancestors` to integrate stat calculation
    /// and clarify the logic for directory inclusion with `--find`.
    fn filter_nodes_and_calculate_stats(
        &self,
        all_nodes_collected: Vec<FileNode>,
    ) -> (Vec<FileNode>, TreeStats) {
        let mut final_stats = TreeStats::default();
        let mut included_files_and_matching_dirs = Vec::new(); // Files that pass filters, and Dirs that match --find
        let mut required_ancestor_dirs = HashSet::new(); // Ancestors of included_files

        // --- Pass 1: Identify matching files and directories that directly match --find ---
        for node in &all_nodes_collected {
            if node.permission_denied {
                // Skip permission denied entries for filtering logic
                continue;
            }

            let has_search_match = node
                .search_matches
                .as_ref()
                .map_or(false, |m| !m.is_empty());

            if node.is_dir {
                // For directories, only the --find pattern applies directly.
                // Other filters (size, date, type) don't apply to directories themselves.
                if self
                    .config
                    .find_pattern
                    .as_ref()
                    .map_or(false, |p| p.is_match(&node.path.to_string_lossy()))
                {
                    included_files_and_matching_dirs.push(node.clone());
                    // Add ancestors of this directly matched directory
                    let mut current = node.path.parent();
                    while let Some(parent_path) = current {
                        if parent_path == self.root || required_ancestor_dirs.contains(parent_path)
                        {
                            break;
                        }
                        required_ancestor_dirs.insert(parent_path.to_path_buf());
                        current = parent_path.parent();
                    }
                }
            } else {
                // For files, check if it passes all filters OR has a search match.
                if has_search_match || self.should_include(node) {
                    included_files_and_matching_dirs.push(node.clone());
                    // Add all ancestors of this matching file to `required_ancestor_dirs`.
                    let mut current = node.path.parent();
                    while let Some(parent_path) = current {
                        // Stop if we reach the root or an already added ancestor.
                        if parent_path == self.root || required_ancestor_dirs.contains(parent_path)
                        {
                            break;
                        }
                        required_ancestor_dirs.insert(parent_path.to_path_buf());
                        current = parent_path.parent();
                    }
                }
            }
        }

        // --- Pass 2: Build the final list of nodes ---
        let mut final_node_list = Vec::new();
        let mut added_paths = HashSet::new(); // To prevent duplicates if a dir is both an ancestor and matches --find

        // Always add the root node if there's anything to show.
        if !included_files_and_matching_dirs.is_empty() {
            if let Some(root_node) = all_nodes_collected.iter().find(|n| n.path == self.root) {
                if added_paths.insert(root_node.path.clone()) {
                    final_node_list.push(root_node.clone());
                }
            }
        }

        // Add required ancestor directories and directly matching directories from `all_nodes_collected`.
        for node in &all_nodes_collected {
            if node.permission_denied {
                // Also include permission denied nodes if they are part of the path
                if required_ancestor_dirs.contains(&node.path)
                    || node.path == self.root && !final_node_list.is_empty()
                {
                    if added_paths.insert(node.path.clone()) {
                        final_node_list.push(node.clone());
                    }
                }
                continue;
            }

            if node.is_dir {
                // Is it a required ancestor OR a directory that itself matched --find?
                let is_find_match = self
                    .config
                    .find_pattern
                    .as_ref()
                    .map_or(false, |p| p.is_match(&node.path.to_string_lossy()));
                if required_ancestor_dirs.contains(&node.path)
                    || (is_find_match && node.path != self.root)
                {
                    if added_paths.insert(node.path.clone()) {
                        final_node_list.push(node.clone());
                    }
                }
            }
        }

        // Add the files that passed filters or had search matches.
        for node in included_files_and_matching_dirs {
            // If it's a directory, it was already handled above (if it matched --find).
            // If it's a file, add it now.
            if !node.is_dir {
                if added_paths.insert(node.path.clone()) {
                    final_node_list.push(node);
                }
            } else {
                // It's a directory that matched --find
                if added_paths.insert(node.path.clone()) {
                    final_node_list.push(node);
                }
            }
        }

        // Sort the final list by path for consistent output.
        final_node_list.sort_by(|a, b| a.path.cmp(&b.path));

        // --- Pass 3: Calculate stats on the final_node_list ---
        for node in &final_node_list {
            // Update stats, ensuring not to double-count or miscount permission-denied entries.
            if !node.permission_denied || node.is_dir {
                // Dirs (even denied) contribute to dir count.
                final_stats.update_file(node);
            }
        }

        (final_node_list, final_stats)
    }

    /// ## `process_entry`
    ///
    /// Converts a `walkdir::DirEntry` into our `FileNode` struct.
    /// This involves fetching metadata, determining file type, category, hidden status, etc.
    /// It also incorporates the `is_ignored_by_rules` status passed to it.
    /// Returns `Ok(Some(FileNode))` on success, `Ok(None)` if the entry should be skipped
    /// (e.g., hidden and not showing hidden), or an `Err` if metadata access fails.
    /// The `is_ignored_by_rules` parameter tells this function if `should_ignore` already determined this node is ignored.
    fn process_entry(
        &self,
        entry: &DirEntry,
        depth: usize,
        is_ignored_by_rules: bool,
    ) -> Result<Option<FileNode>> {
        let path = entry.path();

        // Determine if the file is hidden (starts with '.').
        let is_hidden = path
            .file_name()
            .and_then(|name_osstr| name_osstr.to_str()) // Convert OsStr to &str
            .map_or(false, |name_str| name_str.starts_with('.'));

        // Skip if hidden and we are not configured to show hidden files,
        // UNLESS it's an ignored item that we *are* configured to show (is_ignored_by_rules = true, config.show_ignored = true).
        // The `is_ignored_by_rules` flag takes precedence for display if `config.show_ignored` is true.
        if is_hidden && !self.config.show_hidden && !is_ignored_by_rules {
            // If it's a directory, we need to tell walkdir to skip its contents.
            if entry.file_type().is_dir() {
                // This is tricky because `process_entry` doesn't have `walker` to call `skip_current_dir()`.
                // The caller (`scan` or `scan_stream`) handles `skip_current_dir` based on `should_ignore`
                // and hidden status before calling `process_entry` or by checking the returned node.
                // For now, returning None signals to the caller that this node (and its children if a dir)
                // should not be further processed or added, unless `show_ignored` logic overrides.
            }
            return Ok(None); // Skip this hidden entry.
        }

        // Try to get metadata for the entry. This can fail (e.g., permission denied).
        let metadata = match entry.metadata() {
            Ok(md) => md,
            Err(_e) => {
                // If metadata fails, it's likely a permission issue or a broken symlink.
                // We create a special "permission_denied_node" in the calling `scan`/`scan_stream` methods
                // because they have access to `walker.skip_current_dir()`.
                // Here, we can't fully form that node, so we might return an error or a partial node.
                // For simplicity, if metadata fails here, we treat it as an inaccessible entry.
                // The main scan loops handle creating a FileNode for permission denied errors from WalkDir.
                // This specific call path implies WalkDir *could* read the entry but metadata() failed.
                // This is less common than WalkDir itself erroring.
                // Let's assume the main loops catch this via `Err(e)` from `walker.next()`.
                // If `process_entry` is called on an entry that `WalkDir` gave Ok for, but `metadata()` fails,
                // it's an edge case. We'll return a basic node marked as permission denied.
                return Ok(Some(self.create_permission_denied_node(path, depth)));
            }
        };

        let file_type = self.determine_file_type(&metadata);
        let category = Self::get_file_category(path, file_type);

        // Determine the size. For special virtual files (like in /proc or /sys),
        // reported size can be misleading (e.g., 0 or huge). We mark these as size 0.
        let size = if self.is_special_virtual_file(path, &metadata) {
            0
        } else {
            metadata.len()
        };

        Ok(Some(FileNode {
            path: path.to_path_buf(),
            is_dir: metadata.is_dir(),
            size,
            permissions: Self::get_permissions(&metadata),
            uid: Self::get_uid(&metadata),
            gid: Self::get_gid(&metadata),
            modified: metadata
                .modified()
                .unwrap_or_else(|_| SystemTime::UNIX_EPOCH), // Fallback for modified time
            is_symlink: metadata.file_type().is_symlink(), // Use file_type() for symlink check
            is_hidden,
            permission_denied: false, // If we got metadata, assume no permission error *for this node itself*.
            is_ignored: is_ignored_by_rules, // Use the pre-determined ignore status.
            depth,
            file_type,
            category,
            search_matches: None, // Search matches are added later by the caller if needed.
            filesystem_type: Self::get_filesystem_type(path),
        }))
    }

    /// ## `get_filesystem_type`
    ///
    /// Detects the filesystem type for a given path using statfs on Unix systems
    #[cfg(unix)]
    fn get_filesystem_type(path: &Path) -> FilesystemType {
        use libc::{c_char, statfs};
        use std::ffi::CString;
        use std::mem;

        // Filesystem magic numbers from statfs.h
        const EXT4_SUPER_MAGIC: i64 = 0xef53;
        const EXT3_SUPER_MAGIC: i64 = 0xef53; // Same as ext4, need to check features
        const EXT2_SUPER_MAGIC: i64 = 0xef53; // Same as ext4, need to check features
        const XFS_SUPER_MAGIC: i64 = 0x58465342;
        const BTRFS_SUPER_MAGIC: i64 = 0x9123683e;
        const ZFS_SUPER_MAGIC: i64 = 0x2fc12fc1;
        const NTFS_SB_MAGIC: i64 = 0x5346544e;
        const MSDOS_SUPER_MAGIC: i64 = 0x4d44; // FAT
        const EXFAT_SUPER_MAGIC: i64 = 0x2011bab0;
        const APFS_SUPER_MAGIC: i64 = 0x42535041; // 'APFS'
        const HFS_SUPER_MAGIC: i64 = 0x482b; // HFS+
        const NFS_SUPER_MAGIC: i64 = 0x6969;
        const SMB_SUPER_MAGIC: i64 = 0x517b;
        const TMPFS_MAGIC: i64 = 0x01021994;
        const PROC_SUPER_MAGIC: i64 = 0x9fa0;
        const SYSFS_MAGIC: i64 = 0x62656572;
        const DEVFS_SUPER_MAGIC: i64 = 0x1373;

        let path_cstr = match CString::new(path.to_string_lossy().as_bytes()) {
            Ok(s) => s,
            Err(_) => return FilesystemType::Unknown,
        };

        let mut stat_buf: libc::statfs = unsafe { mem::zeroed() };
        let result = unsafe { statfs(path_cstr.as_ptr(), &mut stat_buf) };

        if result != 0 {
            // statfs failed, fall back to path-based detection for virtual filesystems
            if let Some(path_str) = path.to_str() {
                if path_str.starts_with("/proc") {
                    return FilesystemType::Procfs;
                } else if path_str.starts_with("/sys") {
                    return FilesystemType::Sysfs;
                } else if path_str.starts_with("/dev") {
                    return FilesystemType::Devfs;
                }
            }
            return FilesystemType::Unknown;
        }

        // Check for Mem8 filesystem by looking for .mem8 marker files
        if path.join(".mem8").exists() || path.to_string_lossy().contains("mem8") {
            return FilesystemType::Mem8;
        }

        match stat_buf.f_type as i64 {
            EXT4_SUPER_MAGIC => FilesystemType::Ext4, // TODO: Distinguish ext2/3/4
            XFS_SUPER_MAGIC => FilesystemType::Xfs,
            BTRFS_SUPER_MAGIC => FilesystemType::Btrfs,
            ZFS_SUPER_MAGIC => FilesystemType::Zfs,
            NTFS_SB_MAGIC => FilesystemType::Ntfs,
            MSDOS_SUPER_MAGIC => FilesystemType::Fat32,
            EXFAT_SUPER_MAGIC => FilesystemType::ExFat,
            APFS_SUPER_MAGIC => FilesystemType::Apfs,
            HFS_SUPER_MAGIC => FilesystemType::Hfs,
            NFS_SUPER_MAGIC => FilesystemType::Nfs,
            SMB_SUPER_MAGIC => FilesystemType::Smb,
            TMPFS_MAGIC => FilesystemType::Tmpfs,
            PROC_SUPER_MAGIC => FilesystemType::Procfs,
            SYSFS_MAGIC => FilesystemType::Sysfs,
            DEVFS_SUPER_MAGIC => FilesystemType::Devfs,
            _ => FilesystemType::Unknown,
        }
    }

    #[cfg(not(unix))]
    fn get_filesystem_type(_path: &Path) -> FilesystemType {
        // On non-Unix systems, we can't easily detect filesystem type
        FilesystemType::Unknown
    }

    /// ## `is_virtual_filesystem`
    ///
    /// Checks if a path is on a virtual filesystem
    fn is_virtual_filesystem(path: &Path) -> bool {
        Self::get_filesystem_type(path).is_virtual()
    }

    /// ## `is_special_virtual_file`
    ///
    /// Checks if a file is likely a special virtual file (e.g., in /proc, /sys, /dev)
    /// where reported metadata like size might be zero, misleading, or cause issues if read.
    /// This helps in deciding to report size as 0 for such files.
    fn is_special_virtual_file(&self, path: &Path, metadata: &fs::Metadata) -> bool {
        // Check if the path starts with known virtual filesystem prefixes.
        if let Some(path_str) = path.to_str() {
            if path_str.starts_with("/proc/")
                || path_str.starts_with("/sys/")
                || path_str.starts_with("/dev/")
            {
                return true;
            }
        }

        // Check for specific problematic files by absolute path.
        if self.ignore_files.contains(path) {
            // Uses the pre-built HashSet of specific problem files.
            return true;
        }

        // On Unix, check for special file types like character devices, block devices, FIFOs, sockets.
        // These often have size 0 or non-standard size reporting.
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt; // For is_char_device(), is_block_device(), etc.
            let ft = metadata.file_type();
            if ft.is_char_device() || ft.is_block_device() || ft.is_fifo() || ft.is_socket() {
                return true;
            }
        }

        false // Not determined to be a special virtual file by these checks.
    }

    /// ## `create_permission_denied_node`
    ///
    /// Helper to create a `FileNode` representing an entry (usually a directory)
    /// that could not be accessed due to permission errors.
    /// These nodes are marked specially so formatters can indicate the issue.
    fn create_permission_denied_node(&self, path: &Path, depth: usize) -> FileNode {
        FileNode {
            path: path.to_path_buf(),
            is_dir: true, // Assume it's a directory, as that's common for permission errors during traversal.
            size: 0,      // No size info available.
            permissions: 0, // No permission info.
            uid: 0,       // No UID info.
            gid: 0,       // No GID info.
            modified: SystemTime::UNIX_EPOCH, // Default timestamp.
            is_symlink: false,
            is_hidden: false,        // Cannot determine if hidden.
            permission_denied: true, // Mark as permission denied.
            is_ignored: false,       // Not ignored by rules, but inaccessible.
            depth,
            file_type: FileType::Directory, // Assume directory.
            category: FileCategory::Unknown,
            search_matches: None,
            filesystem_type: Self::get_filesystem_type(path),
        }
    }

    /// ## `should_ignore` - The Bouncer at the Club Door
    ///
    /// This function is our tough-but-fair bouncer. It checks every file and
    /// directory against our lists (`.gitignore`, default ignores, etc.).
    /// "Sorry, `node_modules`, you're not on the list tonight."
    /// It's the first line of defense against clutter.
    fn should_ignore(&self, path: &Path) -> Result<bool> {
        // --- Rule 1: Check against specific, always-ignored files (absolute paths) ---
        if self.config.use_default_ignores && self.ignore_files.contains(path) {
            return Ok(true); // Matches a specific problematic file.
        }

        // --- Rule 2: ALWAYS skip virtual filesystems like /proc, /sys, /dev ---
        // These are checked regardless of use_default_ignores because they're not real files
        // and can cause issues (huge fake sizes, hangs, etc.)
        if Self::is_virtual_filesystem(path) {
            return Ok(true);
        }

        // --- Rule 3: Check against other system paths if using default ignores ---
        if self.config.use_default_ignores {
            // Check for exact match of a system path.
            if self.system_paths.contains(path) {
                return Ok(true);
            }
            // Check if the current path is a child of any registered system path.
            for system_root_path in &self.system_paths {
                if path.starts_with(system_root_path) {
                    return Ok(true); // It's inside /tmp, /var/tmp, etc.
                }
            }
        }

        // --- Rule 3: Check against default ignore patterns (GlobSet) ---
        // These patterns usually match file/directory names or relative paths within a project.
        if let Some(ref default_ignore_set) = self.default_ignores {
            // Check if the simple file/directory name matches any default pattern.
            // (e.g., "node_modules" will match `path/to/project/node_modules`)
            if let Some(file_name) = path.file_name() {
                if default_ignore_set.is_match(Path::new(file_name)) {
                    return Ok(true);
                }
            }
            // Also check the path relative to the scan root against default patterns.
            // This handles patterns like "*.pyc" or "build/outputs/".
            if let Ok(relative_path_to_root) = path.strip_prefix(&self.root) {
                if default_ignore_set.is_match(relative_path_to_root) {
                    return Ok(true);
                }
            }
        }

        // --- Rule 4: Check against .gitignore patterns (GlobSet) ---
        // These patterns are always relative to the root of the scan (where .gitignore is located).
        if let Some(ref gitignore_set) = self.gitignore {
            if let Ok(relative_path_to_root) = path.strip_prefix(&self.root) {
                if gitignore_set.is_match(relative_path_to_root) {
                    return Ok(true); // Matches a .gitignore pattern.
                }
            }
            // If strip_prefix fails (path is not under root), it can't match .gitignore relative patterns.
        }

        // If none of the above rules triggered, the path is not ignored.
        Ok(false)
    }

    /// ## `should_include` - The Velvet Rope
    ///
    /// Once a file gets past the bouncer (`should_ignore`), it has to get past
    /// the velvet rope. This function checks if the file meets the specific criteria
    /// for this party: "Are you a `.rs` file? Are you bigger than 1MB?"
    /// Only the coolest files that match all the rules get in.
    fn should_include(&self, node: &FileNode) -> bool {
        // --- Filter by --find pattern (applies to both files and directories) ---
        if let Some(ref find_regex_pattern) = self.config.find_pattern {
            // Convert path to string for regex matching. Lossy conversion is acceptable for matching.
            let path_str = node.path.to_string_lossy();
            if !find_regex_pattern.is_match(&path_str) {
                return false; // Path doesn't match the --find pattern.
            }
        }

        // --- Filters below only apply to files, not directories ---
        if !node.is_dir {
            // --- Filter by file extension (--type) ---
            if let Some(ref required_extension) = self.config.file_type_filter {
                match node
                    .path
                    .extension()
                    .and_then(|ext_osstr| ext_osstr.to_str())
                {
                    Some(file_ext_str) => {
                        if !file_ext_str.eq_ignore_ascii_case(required_extension) {
                            return false; // Extension doesn't match.
                        }
                    }
                    None => return false, // File has no extension, so cannot match.
                }
            }

            // --- Filter by minimum size (--min-size) ---
            if let Some(min_allowed_size) = self.config.min_size {
                if node.size < min_allowed_size {
                    return false; // File is too small.
                }
            }

            // --- Filter by maximum size (--max-size) ---
            if let Some(max_allowed_size) = self.config.max_size {
                if node.size > max_allowed_size {
                    return false; // File is too large.
                }
            }
        } // End of file-only filters

        // --- Date filters (apply to both files and directories based on their modification time) ---
        // --- Filter by newer_than date (--newer-than) ---
        if let Some(min_modification_date) = self.config.newer_than {
            if node.modified < min_modification_date {
                return false; // Entry is older than required.
            }
        }

        // --- Filter by older_than date (--older-than) ---
        if let Some(max_modification_date) = self.config.older_than {
            if node.modified > max_modification_date {
                return false; // Entry is newer than allowed.
            }
        }

        // If all applicable filters passed (or no filters were active for a category), include the node.
        true
    }

    /// ## `determine_file_type` (Helper for `process_entry`)
    ///
    /// Examines `fs::Metadata` to determine a more specific `FileType`
    /// than just `is_dir` or `is_file`. On Unix, this can identify symlinks,
    /// sockets, FIFOs, block/char devices, and executables (by permission).
    /// On non-Unix, it's simpler (dir, symlink, or regular file).
    fn determine_file_type(&self, metadata: &fs::Metadata) -> FileType {
        #[cfg(unix)] // Unix-specific detailed file type detection
        {
            use std::os::unix::fs::FileTypeExt; // For is_socket, is_fifo, etc.
            let ft = metadata.file_type(); // Get the rich FileType from metadata.

            if ft.is_dir() {
                FileType::Directory
            } else if ft.is_symlink() {
                // Check symlink before other types, as it can point to them.
                FileType::Symlink
            } else if ft.is_socket() {
                FileType::Socket
            } else if ft.is_fifo() {
                // Named pipe
                FileType::Pipe
            } else if ft.is_block_device() {
                FileType::BlockDevice
            } else if ft.is_char_device() {
                FileType::CharDevice
            // Check for executable permission (any of user, group, other execute bits are set).
            // This applies to regular files that are not dirs, symlinks, or other special types.
            } else if ft.is_file() && (metadata.permissions().mode() & 0o111 != 0) {
                FileType::Executable
            } else {
                // If none of the above, it's a regular (non-executable) file.
                FileType::RegularFile
            }
        }

        #[cfg(not(unix))] // Simpler detection for non-Unix platforms
        {
            if metadata.is_dir() {
                FileType::Directory
            } else if metadata.file_type().is_symlink() {
                // `is_symlink()` is part of stable `fs::FileType`
                FileType::Symlink
            } else {
                // No easy cross-platform way to check executable bit without external crates or OS-specific calls.
                // So, on non-Unix, we don't distinguish Executable from RegularFile here.
                FileType::RegularFile
            }
        }
    }

    // --- Platform-Dependent Metadata Helpers ---
    // These provide a consistent way to get permissions, UID, and GID,
    // with sensible defaults for non-Unix systems where these concepts might not directly apply
    // or be easily accessible via standard Rust fs::Metadata.

    #[cfg(unix)]
    fn get_permissions(metadata: &fs::Metadata) -> u32 {
        // On Unix, get the mode and mask it to get the permission bits (e.g., 0o755).
        metadata.permissions().mode() & 0o777
    }
    #[cfg(not(unix))]
    fn get_permissions(_metadata: &fs::Metadata) -> u32 {
        0o755 // A common default permission (rwxr-xr-x) for non-Unix.
    }

    #[cfg(unix)]
    fn get_uid(metadata: &fs::Metadata) -> u32 {
        metadata.uid() // Get User ID from metadata.
    }
    #[cfg(not(unix))]
    fn get_uid(_metadata: &fs::Metadata) -> u32 {
        1000 // Common default UID placeholder for non-Unix.
    }

    #[cfg(unix)]
    fn get_gid(metadata: &fs::Metadata) -> u32 {
        metadata.gid() // Get Group ID from metadata.
    }
    #[cfg(not(unix))]
    fn get_gid(_metadata: &fs::Metadata) -> u32 {
        0
    }
} // end impl Scanner

/// # `parse_size` - The Universal Translator for Sizes
///
/// This handy function takes something a human understands, like "2.5M", and
/// translates it into something a computer understands (2,621,440 bytes).
/// It's like having a Babel fish for file sizes. Why should we have to do
/// that math when the computer can do it for us?
pub fn parse_size(size_str: &str) -> Result<u64> {
    let size_str = size_str.trim().to_uppercase();
    if size_str.is_empty() {
        return Err(anyhow::anyhow!("Empty size string"));
    }

    // Find the first alphabetic character which marks the start of the unit.
    let unit_start_index = size_str
        .find(|c: char| c.is_alphabetic())
        .unwrap_or(size_str.len());
    let (num_part_str, unit_part) = size_str.split_at(unit_start_index);

    // Trim any space from the number part before parsing.
    let num_part_str = num_part_str.trim();

    if num_part_str.is_empty() {
        return Err(anyhow::anyhow!("Missing number for size string"));
    }

    let num: f64 = match num_part_str.parse() {
        Ok(n) => n,
        Err(e) => return Err(anyhow::anyhow!("Invalid number '{}': {}", num_part_str, e)),
    };

    // Check for negative numbers.
    if num.is_sign_negative() {
        return Err(anyhow::anyhow!("Size cannot be negative: {}", num));
    }

    let multiplier = match unit_part {
        "K" | "KB" => 1024.0,
        "M" | "MB" => 1024.0 * 1024.0,
        "G" | "GB" => 1024.0 * 1024.0 * 1024.0,
        "T" | "TB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        "B" | "" => 1.0,
        _ => return Err(anyhow::anyhow!("Invalid size unit: '{}'", unit_part)),
    };

    Ok((num * multiplier) as u64)
}

// --- Unit Tests: Ensuring Our Scanner Behaves ---
// Aye, even the most brilliant code needs tests to keep it honest!
// These tests cover some basic functionality of the scanner.
#[cfg(test)]
mod tests {
    use super::*; // Import everything from the parent module (scanner.rs).

    #[test]
    fn test_parse_size_valid_inputs() {
        assert_eq!(parse_size("100").unwrap(), 100);
        assert_eq!(parse_size("100B").unwrap(), 100);
        assert_eq!(parse_size("1k").unwrap(), 1024);
        assert_eq!(parse_size("1K").unwrap(), 1024);
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("2.5M").unwrap(), (2.5 * 1024.0 * 1024.0) as u64);
        assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(
            parse_size("0.5T").unwrap(),
            (0.5 * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64
        );
        assert_eq!(parse_size("  2 MB  ").unwrap(), 2 * 1024 * 1024); // Test with whitespace
    }

    #[test]
    fn test_parse_size_invalid_inputs() {
        assert!(parse_size("100X").is_err());
        assert!(parse_size("garbage").is_err());
        assert!(parse_size("-100M").is_err());
        assert!(parse_size("1..5K").is_err());
    }

    #[test]
    fn test_parse_size_zero_and_empty() {
        assert_eq!(parse_size("0").unwrap(), 0);
        assert!(parse_size("").is_err());
        assert!(parse_size("  ").is_err());
    }

    // Basic test for Scanner creation. More comprehensive tests would involve
    // creating a temporary directory structure and verifying scan results.
    #[test]
    fn test_scanner_creation_defaults() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = ScannerConfig {
            max_depth: 5,
            follow_symlinks: false,
            respect_gitignore: true,
            show_hidden: false,
            show_ignored: false,
            find_pattern: None,
            file_type_filter: None,
            min_size: None,
            max_size: None,
            newer_than: None,
            older_than: None,
            use_default_ignores: true,
            search_keyword: None,
            show_filesystems: false,
        };
        let scanner_result = Scanner::new(temp_dir.path(), config);
        assert!(scanner_result.is_ok());
    }
}
