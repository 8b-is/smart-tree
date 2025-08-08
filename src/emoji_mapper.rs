// -----------------------------------------------------------------------------
// 🎨 EMOJI MAPPER - Bringing Life to File Types!
// -----------------------------------------------------------------------------
// This module provides a rich, context-aware emoji mapping system that makes
// Smart Tree's output more visually appealing and informative. We map file
// categories to beautiful emojis that help users quickly identify file types.
//
// The system supports both emoji and no-emoji modes, with descriptive text
// fallbacks for environments that don't support emojis.
// -----------------------------------------------------------------------------

use crate::scanner::{FileCategory, FileNode, FileType};

/// Get an emoji (or text representation) for a file based on its type and category
pub fn get_file_emoji(node: &FileNode, no_emoji: bool) -> &'static str {
    // Handle permission denied with lock emoji
    if node.permission_denied {
        return if no_emoji { "[LOCK]" } else { "🔒" };
    }

    // Special handling for directories
    if node.is_dir {
        return if no_emoji {
            if node.size == 0 {
                "[EMPTY_DIR]"
            } else {
                "[DIR]"
            }
        } else {
            if node.size == 0 {
                "📂"
            } else {
                "📁"
            }
        };
    }

    // Map based on file category for rich, semantic emojis
    match node.category {
        // Programming Languages
        FileCategory::Rust => {
            if no_emoji {
                "[RUST]"
            } else {
                "🦀"
            }
        }
        FileCategory::Python => {
            if no_emoji {
                "[PY]"
            } else {
                "🐍"
            }
        }
        FileCategory::JavaScript => {
            if no_emoji {
                "[JS]"
            } else {
                "📜"
            }
        }
        FileCategory::TypeScript => {
            if no_emoji {
                "[TS]"
            } else {
                "📘"
            }
        }
        FileCategory::Java => {
            if no_emoji {
                "[JAVA]"
            } else {
                "☕"
            }
        }
        FileCategory::C => {
            if no_emoji {
                "[C]"
            } else {
                "🔷"
            }
        }
        FileCategory::Cpp => {
            if no_emoji {
                "[CPP]"
            } else {
                "🔷"
            }
        }
        FileCategory::Go => {
            if no_emoji {
                "[GO]"
            } else {
                "🐹"
            }
        }
        FileCategory::Ruby => {
            if no_emoji {
                "[RB]"
            } else {
                "💎"
            }
        }
        FileCategory::PHP => {
            if no_emoji {
                "[PHP]"
            } else {
                "🐘"
            }
        }
        FileCategory::Shell => {
            if no_emoji {
                "[SH]"
            } else {
                "🐚"
            }
        }

        // Markup & Data
        FileCategory::Markdown => {
            if no_emoji {
                "[MD]"
            } else {
                "📝"
            }
        }
        FileCategory::Html => {
            if no_emoji {
                "[HTML]"
            } else {
                "🌐"
            }
        }
        FileCategory::Css => {
            if no_emoji {
                "[CSS]"
            } else {
                "🎨"
            }
        }
        FileCategory::Json => {
            if no_emoji {
                "[JSON]"
            } else {
                "📊"
            }
        }
        FileCategory::Yaml => {
            if no_emoji {
                "[YAML]"
            } else {
                "📋"
            }
        }
        FileCategory::Xml => {
            if no_emoji {
                "[XML]"
            } else {
                "📰"
            }
        }
        FileCategory::Toml => {
            if no_emoji {
                "[TOML]"
            } else {
                "🔧"
            }
        }
        FileCategory::Csv => {
            if no_emoji {
                "[CSV]"
            } else {
                "📊"
            }
        }

        // Build & Config
        FileCategory::Makefile => {
            if no_emoji {
                "[MAKE]"
            } else {
                "🔨"
            }
        }
        FileCategory::Dockerfile => {
            if no_emoji {
                "[DOCKER]"
            } else {
                "🐳"
            }
        }
        FileCategory::GitConfig => {
            if no_emoji {
                "[GIT]"
            } else {
                "📝"
            }
        }
        FileCategory::Config => {
            if no_emoji {
                "[CFG]"
            } else {
                "⚙️"
            }
        }

        // Archives
        FileCategory::Archive => {
            if no_emoji {
                "[ZIP]"
            } else {
                "📦"
            }
        }

        // Media
        FileCategory::Image => {
            if no_emoji {
                "[IMG]"
            } else {
                "🖼️ "
            }
        }
        FileCategory::Video => {
            if no_emoji {
                "[VID]"
            } else {
                "🎬"
            }
        }
        FileCategory::Audio => {
            if no_emoji {
                "[AUD]"
            } else {
                "🎵"
            }
        }

        // Office & Documents
        FileCategory::Office => {
            if no_emoji {
                "[DOC]"
            } else {
                "📄"
            }
        }
        FileCategory::Spreadsheet => {
            if no_emoji {
                "[XLS]"
            } else {
                "📊"
            }
        }
        FileCategory::PowerPoint => {
            if no_emoji {
                "[PPT]"
            } else {
                "📊"
            }
        }
        FileCategory::Pdf => {
            if no_emoji {
                "[PDF]"
            } else {
                "📕"
            }
        }
        FileCategory::Ebook => {
            if no_emoji {
                "[BOOK]"
            } else {
                "📚"
            }
        }

        // Text Variants
        FileCategory::Txt => {
            if no_emoji {
                "[TXT]"
            } else {
                "📄"
            }
        }
        FileCategory::Rtf => {
            if no_emoji {
                "[RTF]"
            } else {
                "📄"
            }
        }
        FileCategory::Log => {
            if no_emoji {
                "[LOG]"
            } else {
                "📋"
            }
        }
        FileCategory::License => {
            if no_emoji {
                "[LIC]"
            } else {
                "📜"
            }
        }
        FileCategory::Readme => {
            if no_emoji {
                "[README]"
            } else {
                "📖"
            }
        }

        // Security & Crypto
        FileCategory::Certificate => {
            if no_emoji {
                "[CERT]"
            } else {
                "🔐"
            }
        }
        FileCategory::Encrypted => {
            if no_emoji {
                "[ENC]"
            } else {
                "🔒"
            }
        }

        // System & Binary
        FileCategory::SystemFile => {
            if no_emoji {
                "[SYS]"
            } else {
                "⚙️ "
            }
        }
        FileCategory::Binary => {
            if no_emoji {
                "[BIN]"
            } else {
                "⚙️ "
            }
        }

        // Database
        FileCategory::Database => {
            if no_emoji {
                "[DB]"
            } else {
                "🗄️ "
            }
        }

        // Fonts
        FileCategory::Font => {
            if no_emoji {
                "[FONT]"
            } else {
                "🔤"
            }
        }

        // Disk Images
        FileCategory::DiskImage => {
            if no_emoji {
                "[DISK]"
            } else {
                "💿"
            }
        }

        // 3D & CAD
        FileCategory::Model3D => {
            if no_emoji {
                "[3D]"
            } else {
                "🎲"
            }
        }

        // Scientific & Data
        FileCategory::Jupyter => {
            if no_emoji {
                "[JUPYTER]"
            } else {
                "📓"
            }
        }
        FileCategory::RData => {
            if no_emoji {
                "[RDATA]"
            } else {
                "📊"
            }
        }
        FileCategory::Matlab => {
            if no_emoji {
                "[MAT]"
            } else {
                "📐"
            }
        }

        // Web Assets
        FileCategory::WebAsset => {
            if no_emoji {
                "[WEB]"
            } else {
                "🌐"
            }
        }

        // Package & Dependencies
        FileCategory::Package => {
            if no_emoji {
                "[PKG]"
            } else {
                "📦"
            }
        }
        FileCategory::Lock => {
            if no_emoji {
                "[LOCK]"
            } else {
                "🔒"
            }
        }

        // Testing
        FileCategory::Test => {
            if no_emoji {
                "[TEST]"
            } else {
                "🧪"
            }
        }

        // Memory Files (MEM|8!)
        FileCategory::Memory => {
            if no_emoji {
                "[MEM8]"
            } else {
                "🧠"
            }
        }

        // Others
        FileCategory::Backup => {
            if no_emoji {
                "[BAK]"
            } else {
                "💾"
            }
        }
        FileCategory::Temp => {
            if no_emoji {
                "[TMP]"
            } else {
                "🗑️"
            }
        }
        FileCategory::Unknown => {
            // Fall back to file type for unknowns
            match node.file_type {
                FileType::Symlink => {
                    if no_emoji {
                        "[LINK]"
                    } else {
                        "🔗"
                    }
                }
                FileType::Executable => {
                    if no_emoji {
                        "[EXE]"
                    } else {
                        "⚙️ "
                    }
                }
                FileType::Socket => {
                    if no_emoji {
                        "[SOCK]"
                    } else {
                        "🔌"
                    }
                }
                FileType::Pipe => {
                    if no_emoji {
                        "[PIPE]"
                    } else {
                        "🪄"
                    }
                }
                FileType::BlockDevice => {
                    if no_emoji {
                        "[BLK]"
                    } else {
                        "💾"
                    }
                }
                FileType::CharDevice => {
                    if no_emoji {
                        "[CHR]"
                    } else {
                        "📺"
                    }
                }
                FileType::RegularFile => {
                    if node.size == 0 {
                        if no_emoji {
                            "[EMPTY]"
                        } else {
                            "🪹"
                        }
                    } else {
                        if no_emoji {
                            "[FILE]"
                        } else {
                            "📄"
                        }
                    }
                }
                FileType::Directory => {
                    if no_emoji {
                        "[DIR]"
                    } else {
                        "📁"
                    }
                }
            }
        }
    }
}

/// Get a descriptive name for a file category (used in stats and summaries)
pub fn get_category_name(category: &FileCategory) -> &'static str {
    match category {
        // Programming Languages
        FileCategory::Rust => "Rust Source",
        FileCategory::Python => "Python Source",
        FileCategory::JavaScript => "JavaScript",
        FileCategory::TypeScript => "TypeScript",
        FileCategory::Java => "Java Source",
        FileCategory::C => "C Source",
        FileCategory::Cpp => "C++ Source",
        FileCategory::Go => "Go Source",
        FileCategory::Ruby => "Ruby Source",
        FileCategory::PHP => "PHP Source",
        FileCategory::Shell => "Shell Script",

        // Markup & Data
        FileCategory::Markdown => "Markdown",
        FileCategory::Html => "HTML",
        FileCategory::Css => "CSS",
        FileCategory::Json => "JSON",
        FileCategory::Yaml => "YAML",
        FileCategory::Xml => "XML",
        FileCategory::Toml => "TOML",
        FileCategory::Csv => "CSV Data",

        // Build & Config
        FileCategory::Makefile => "Makefile",
        FileCategory::Dockerfile => "Dockerfile",
        FileCategory::GitConfig => "Git Config",
        FileCategory::Config => "Configuration",

        // Archives
        FileCategory::Archive => "Archive",

        // Media
        FileCategory::Image => "Image",
        FileCategory::Video => "Video",
        FileCategory::Audio => "Audio",

        // Office & Documents
        FileCategory::Office => "Document",
        FileCategory::Spreadsheet => "Spreadsheet",
        FileCategory::PowerPoint => "Presentation",
        FileCategory::Pdf => "PDF Document",
        FileCategory::Ebook => "E-Book",

        // Text Variants
        FileCategory::Txt => "Text File",
        FileCategory::Rtf => "Rich Text",
        FileCategory::Log => "Log File",
        FileCategory::License => "License",
        FileCategory::Readme => "README",

        // Security & Crypto
        FileCategory::Certificate => "Certificate",
        FileCategory::Encrypted => "Encrypted",

        // System & Binary
        FileCategory::SystemFile => "System File",
        FileCategory::Binary => "Binary",

        // Database
        FileCategory::Database => "Database",

        // Fonts
        FileCategory::Font => "Font",

        // Disk Images
        FileCategory::DiskImage => "Disk Image",

        // 3D & CAD
        FileCategory::Model3D => "3D Model",

        // Scientific & Data
        FileCategory::Jupyter => "Jupyter Notebook",
        FileCategory::RData => "R Data",
        FileCategory::Matlab => "MATLAB",

        // Web Assets
        FileCategory::WebAsset => "Web Asset",

        // Package & Dependencies
        FileCategory::Package => "Package File",
        FileCategory::Lock => "Lock File",

        // Testing
        FileCategory::Test => "Test File",

        // Memory Files
        FileCategory::Memory => "MEM|8 File",

        // Others
        FileCategory::Backup => "Backup",
        FileCategory::Temp => "Temporary",
        FileCategory::Unknown => "Unknown",
    }
}

/// Get a color suggestion for terminal output (ANSI color codes)
pub fn get_category_color(category: &FileCategory) -> &'static str {
    match category {
        // Programming languages - bright colors
        FileCategory::Rust => "\x1b[38;5;208m",   // Orange
        FileCategory::Python => "\x1b[38;5;226m", // Yellow
        FileCategory::JavaScript => "\x1b[38;5;220m", // Gold
        FileCategory::TypeScript => "\x1b[38;5;33m", // Blue
        FileCategory::Java => "\x1b[38;5;166m",   // Dark Orange
        FileCategory::C | FileCategory::Cpp => "\x1b[38;5;39m", // Light Blue
        FileCategory::Go => "\x1b[38;5;51m",      // Cyan
        FileCategory::Ruby => "\x1b[38;5;196m",   // Red
        FileCategory::PHP => "\x1b[38;5;99m",     // Purple
        FileCategory::Shell => "\x1b[38;5;28m",   // Green

        // Data formats - cool colors
        FileCategory::Json | FileCategory::Yaml | FileCategory::Xml => "\x1b[38;5;45m",
        FileCategory::Markdown => "\x1b[38;5;250m",

        // Media - vibrant colors
        FileCategory::Image => "\x1b[38;5;201m", // Magenta
        FileCategory::Video => "\x1b[38;5;129m", // Purple
        FileCategory::Audio => "\x1b[38;5;213m", // Pink

        // System - muted colors
        FileCategory::Binary | FileCategory::SystemFile => "\x1b[38;5;240m",

        // Special files
        FileCategory::Memory => "\x1b[38;5;93m", // Purple (for MEM|8!)
        FileCategory::Database => "\x1b[38;5;94m", // Brown

        // Default
        _ => "\x1b[0m", // Reset
    }
}
