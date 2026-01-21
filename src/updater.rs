// -----------------------------------------------------------------------------
// Self-Update Module for Smart Tree
// Checks for updates from GitHub releases and installs new versions
// -----------------------------------------------------------------------------

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// GitHub repository for releases
const GITHUB_REPO: &str = "8b-is/smart-tree";

/// GitHub API endpoint for latest release
const GITHUB_RELEASES_API: &str = "https://api.github.com/repos/8b-is/smart-tree/releases/latest";

/// Rate limit: check for updates at most once per 24 hours
const UPDATE_CHECK_INTERVAL_SECS: u64 = 86400;

/// Binaries included in the release tarball
const BINARIES: &[&str] = &["st", "mq", "m8", "tree"];

/// Current version from Cargo.toml
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub release response (partial)
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// Update check cache
#[derive(Debug, Default, Serialize, Deserialize)]
struct UpdateCache {
    #[serde(default)]
    last_check: u64,
    #[serde(default)]
    latest_version: Option<String>,
}

/// Get the cache file path (~/.st/update_check.json)
fn get_cache_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let st_dir = home.join(".st");
    fs::create_dir_all(&st_dir)?;
    Ok(st_dir.join("update_check.json"))
}

/// Load the update cache
fn load_cache() -> UpdateCache {
    let cache_path = match get_cache_path() {
        Ok(p) => p,
        Err(_) => return UpdateCache::default(),
    };

    match fs::read_to_string(&cache_path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => UpdateCache::default(),
    }
}

/// Save the update cache
fn save_cache(cache: &UpdateCache) -> Result<()> {
    let cache_path = get_cache_path()?;
    let contents = serde_json::to_string_pretty(cache)?;
    fs::write(&cache_path, contents)?;
    Ok(())
}

/// Get current timestamp in seconds
fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Check if we should perform an update check (rate limiting)
pub fn should_check_update() -> bool {
    let cache = load_cache();
    let now = now_secs();
    now.saturating_sub(cache.last_check) > UPDATE_CHECK_INTERVAL_SECS
}

/// Compare version strings (semver-like)
fn is_newer_version(current: &str, latest: &str) -> bool {
    // Strip 'v' prefix if present
    let current = current.strip_prefix('v').unwrap_or(current);
    let latest = latest.strip_prefix('v').unwrap_or(latest);

    let parse_version = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<u32> = v
            .split('.')
            .filter_map(|p| p.parse().ok())
            .collect();
        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };

    let (curr_major, curr_minor, curr_patch) = parse_version(current);
    let (lat_major, lat_minor, lat_patch) = parse_version(latest);

    (lat_major, lat_minor, lat_patch) > (curr_major, curr_minor, curr_patch)
}

/// Check for available updates (network call)
pub fn check_for_update() -> Result<Option<String>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("smart-tree-updater")
        .timeout(Duration::from_secs(10))
        .build()?;

    let response: GitHubRelease = client
        .get(GITHUB_RELEASES_API)
        .send()
        .context("Failed to connect to GitHub")?
        .json()
        .context("Failed to parse GitHub response")?;

    // Update cache
    let mut cache = load_cache();
    cache.last_check = now_secs();
    cache.latest_version = Some(response.tag_name.clone());
    let _ = save_cache(&cache);

    let latest = response.tag_name;
    if is_newer_version(CURRENT_VERSION, &latest) {
        Ok(Some(latest))
    } else {
        Ok(None)
    }
}

/// Check for update using cache if within rate limit
pub fn check_for_update_cached() -> Option<String> {
    let cache = load_cache();

    if should_check_update() {
        // Perform actual check
        match check_for_update() {
            Ok(Some(version)) => Some(version),
            Ok(None) => None,
            Err(_) => None, // Silently fail on network errors
        }
    } else {
        // Use cached result
        cache.latest_version.filter(|v| is_newer_version(CURRENT_VERSION, v))
    }
}

/// Print update available banner
pub fn print_update_banner(latest_version: &str) {
    let current = format!("v{}", CURRENT_VERSION);
    eprintln!();
    eprintln!("\x1b[36m╭─────────────────────────────────────────────────────╮\x1b[0m");
    eprintln!("\x1b[36m│\x1b[0m \x1b[32m🌳 Smart Tree {} is available!\x1b[0m (you have {})", latest_version, current);
    eprintln!("\x1b[36m│\x1b[0m    Run '\x1b[1mst --update\x1b[0m' to upgrade");
    eprintln!("\x1b[36m╰─────────────────────────────────────────────────────╯\x1b[0m");
    eprintln!();
}

/// Detect the current platform for download
fn get_platform() -> Result<(&'static str, &'static str)> {
    let os = if cfg!(target_os = "macos") {
        "apple-darwin"
    } else if cfg!(target_os = "linux") {
        "unknown-linux-gnu"
    } else if cfg!(target_os = "windows") {
        "pc-windows-msvc"
    } else {
        bail!("Unsupported operating system");
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        bail!("Unsupported architecture");
    };

    Ok((arch, os))
}

/// Create a temporary directory for the update
fn create_temp_dir() -> Result<PathBuf> {
    let base = env::temp_dir();
    let unique_name = format!("st-update-{}", now_secs());
    let temp_dir = base.join(unique_name);
    fs::create_dir_all(&temp_dir).context("Failed to create temp directory")?;
    Ok(temp_dir)
}

/// Clean up a temporary directory
fn cleanup_temp_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

/// Find where the current binary is installed
fn find_install_dir() -> Result<PathBuf> {
    // Try to find where 'st' is installed
    let current_exe = env::current_exe().context("Could not determine current executable path")?;
    let install_dir = current_exe.parent()
        .context("Could not determine installation directory")?
        .to_path_buf();

    Ok(install_dir)
}

/// Check if we need elevated permissions
fn needs_sudo(install_dir: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(meta) = install_dir.metadata() {
            // Check if we're the owner or if we can write
            let uid = unsafe { libc::getuid() };
            if meta.uid() != uid {
                // Not owner, check if writable
                return fs::metadata(install_dir)
                    .and_then(|_| fs::OpenOptions::new().write(true).open(install_dir.join(".test_write")))
                    .is_err();
            }
        }
        false
    }
    #[cfg(not(unix))]
    {
        false
    }
}

/// Download and install the update
pub fn download_and_install(version: &str, yes: bool) -> Result<()> {
    let (arch, os) = get_platform()?;
    let install_dir = find_install_dir()?;

    println!("\x1b[36m🌳 Smart Tree Updater\x1b[0m");
    println!();
    println!("Current version: v{}", CURRENT_VERSION);
    println!("Latest version:  {}", version);
    println!("Install path:    {}", install_dir.display());
    println!("Binaries:        {}", BINARIES.join(", "));
    println!();

    if !yes {
        print!("Proceed with update? [Y/n] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if !input.is_empty() && input != "y" && input != "yes" {
            println!("Update cancelled.");
            return Ok(());
        }
    }

    let use_sudo = needs_sudo(&install_dir);
    if use_sudo {
        println!("\x1b[33m⚠ Installation directory requires elevated permissions.\x1b[0m");
        println!("  You may be prompted for your password.\n");
    }

    // Construct download URL
    let ext = if cfg!(target_os = "windows") { "zip" } else { "tar.gz" };
    let archive_name = format!("st-{}-{}-{}.{}", version, arch, os, ext);
    let download_url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        GITHUB_REPO, version, archive_name
    );

    println!("Downloading {}...", archive_name);

    // Create temp directory
    let temp_dir = create_temp_dir()?;
    let archive_path = temp_dir.join(&archive_name);

    // Download
    let client = reqwest::blocking::Client::builder()
        .user_agent("smart-tree-updater")
        .timeout(Duration::from_secs(300))
        .build()?;

    let response = client
        .get(&download_url)
        .send()
        .context("Failed to download release")?;

    if !response.status().is_success() {
        bail!("Download failed: HTTP {}", response.status());
    }

    let bytes = response.bytes()?;
    fs::write(&archive_path, &bytes)?;

    println!("Extracting...");

    // Extract archive
    #[cfg(unix)]
    {
        let output = Command::new("tar")
            .args(["-xzf", archive_path.to_str().unwrap()])
            .current_dir(&temp_dir)
            .output()
            .context("Failed to extract archive")?;

        if !output.status.success() {
            bail!("Failed to extract archive: {}", String::from_utf8_lossy(&output.stderr));
        }
    }

    #[cfg(windows)]
    {
        // On Windows, use powershell to extract zip
        let output = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    archive_path.display(),
                    &temp_dir.display()
                ),
            ])
            .output()
            .context("Failed to extract archive")?;

        if !output.status.success() {
            bail!("Failed to extract archive: {}", String::from_utf8_lossy(&output.stderr));
        }
    }

    // Install binaries
    println!("Installing binaries...");

    for binary in BINARIES {
        let binary_name = if cfg!(windows) {
            format!("{}.exe", binary)
        } else {
            binary.to_string()
        };

        // Find binary in temp dir (might be at root or in subdirectory)
        let src_path = find_binary_in_dir(&temp_dir, &binary_name)?;
        let dest_path = install_dir.join(&binary_name);

        // IMPORTANT: Remove old binary first to avoid macOS zombie process issue
        #[cfg(unix)]
        {
            if use_sudo {
                let _ = Command::new("sudo")
                    .args(["rm", "-f", dest_path.to_str().unwrap()])
                    .status();

                Command::new("sudo")
                    .args(["cp", src_path.to_str().unwrap(), dest_path.to_str().unwrap()])
                    .status()
                    .context(format!("Failed to install {}", binary))?;

                Command::new("sudo")
                    .args(["chmod", "+x", dest_path.to_str().unwrap()])
                    .status()?;
            } else {
                let _ = fs::remove_file(&dest_path);
                fs::copy(&src_path, &dest_path)
                    .context(format!("Failed to install {}", binary))?;

                // Set executable permission
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&dest_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&dest_path, perms)?;
            }
        }

        #[cfg(windows)]
        {
            // On Windows, rename old binary first (can't delete while running)
            let old_path = install_dir.join(format!("{}.old", binary_name));
            let _ = fs::remove_file(&old_path);
            let _ = fs::rename(&dest_path, &old_path);

            fs::copy(&src_path, &dest_path)
                .context(format!("Failed to install {}", binary))?;
        }

        println!("  \x1b[32m✓\x1b[0m {}", binary);
    }

    // Update cache
    let mut cache = load_cache();
    cache.latest_version = Some(version.to_string());
    let _ = save_cache(&cache);

    // Clean up temp directory
    cleanup_temp_dir(&temp_dir);

    println!();
    println!("\x1b[32m✨ Successfully updated to {}!\x1b[0m", version);

    #[cfg(windows)]
    {
        println!();
        println!("\x1b[33mNote: Please restart your terminal for the update to take effect.\x1b[0m");
    }

    Ok(())
}

/// Find a binary file within a directory (handles nested extraction)
fn find_binary_in_dir(dir: &Path, binary_name: &str) -> Result<PathBuf> {
    // Check root
    let root_path = dir.join(binary_name);
    if root_path.exists() {
        return Ok(root_path);
    }

    // Search subdirectories
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let nested = path.join(binary_name);
            if nested.exists() {
                return Ok(nested);
            }
        }
    }

    bail!("Could not find {} in downloaded archive", binary_name)
}

/// Run the update command
pub fn run_update(yes: bool) -> Result<()> {
    println!("Checking for updates...");

    match check_for_update()? {
        Some(version) => {
            download_and_install(&version, yes)?;
        }
        None => {
            println!("\x1b[32m✓\x1b[0m Already up to date! (v{})", CURRENT_VERSION);
        }
    }

    Ok(())
}

/// Get current version string
pub fn current_version() -> &'static str {
    CURRENT_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("5.5.0", "5.5.1"));
        assert!(is_newer_version("5.5.1", "5.6.0"));
        assert!(is_newer_version("5.5.1", "6.0.0"));
        assert!(is_newer_version("v5.5.0", "v5.5.1"));
        assert!(!is_newer_version("5.5.1", "5.5.1"));
        assert!(!is_newer_version("5.5.1", "5.5.0"));
        assert!(!is_newer_version("6.0.0", "5.5.1"));
    }

    #[test]
    fn test_platform_detection() {
        let result = get_platform();
        assert!(result.is_ok());
    }
}
