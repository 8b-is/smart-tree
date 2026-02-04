// Service Manager for Smart Tree Daemon
// Cross-platform service management: Linux (systemd), macOS (launchctl), Windows (Task Scheduler)

use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tracing::{error, info, warn};

// =============================================================================
// PLATFORM DETECTION
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
    Unknown,
}

impl Platform {
    pub fn current() -> Self {
        #[cfg(target_os = "linux")]
        return Platform::Linux;

        #[cfg(target_os = "macos")]
        return Platform::MacOS;

        #[cfg(target_os = "windows")]
        return Platform::Windows;

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        return Platform::Unknown;
    }

    pub fn service_manager_name(&self) -> &'static str {
        match self {
            Platform::Linux => "systemd",
            Platform::MacOS => "launchctl",
            Platform::Windows => "Task Scheduler",
            Platform::Unknown => "unknown",
        }
    }
}

// =============================================================================
// LINUX (systemd)
// =============================================================================

const SYSTEMD_SERVICE_NAME: &str = "smart-tree-dashboard@.service";
const SYSTEMD_SERVICE_TEMPLATE: &str = "systemd/smart-tree-dashboard@.service";

fn get_systemd_user_path() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|home| home.join(".config").join("systemd").join("user"))
        .context("Could not find home directory")
}

// =============================================================================
// macOS (launchctl)
// =============================================================================

const LAUNCHD_LABEL: &str = "is.8b.smart-tree";

fn get_launchd_user_path() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|home| home.join("Library").join("LaunchAgents"))
        .context("Could not find home directory")
}

fn get_launchd_plist_path() -> Result<PathBuf> {
    let agents_dir = get_launchd_user_path()?;
    Ok(agents_dir.join(format!("{}.plist", LAUNCHD_LABEL)))
}

fn generate_launchd_plist(project_name: &str) -> String {
    let st_path = which_st().unwrap_or_else(|_| PathBuf::from("/usr/local/bin/st"));
    let working_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("/tmp"));
    let log_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".st")
        .join("daemon.log");

    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{}.{}</string>

    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>--http-daemon</string>
    </array>

    <key>WorkingDirectory</key>
    <string>{}</string>

    <key>RunAtLoad</key>
    <false/>

    <key>KeepAlive</key>
    <false/>

    <key>StandardOutPath</key>
    <string>{}</string>

    <key>StandardErrorPath</key>
    <string>{}</string>

    <key>EnvironmentVariables</key>
    <dict>
        <key>ST_PROJECT</key>
        <string>{}</string>
    </dict>
</dict>
</plist>
"#,
        LAUNCHD_LABEL,
        project_name,
        st_path.display(),
        working_dir.display(),
        log_path.display(),
        log_path.display(),
        project_name
    )
}

fn launchd_install() -> Result<()> {
    let project_name = get_project_name()?;
    let plist_path = get_launchd_plist_path()?;
    let agents_dir = get_launchd_user_path()?;

    // Create LaunchAgents directory if needed
    fs::create_dir_all(&agents_dir)
        .with_context(|| format!("Failed to create {}", agents_dir.display()))?;

    // Generate and write plist
    let plist_content = generate_launchd_plist(&project_name);
    fs::write(&plist_path, &plist_content)
        .with_context(|| format!("Failed to write {}", plist_path.display()))?;

    info!("Created LaunchAgent at {}", plist_path.display());

    println!("\n✅ Service installed for project '{}'", project_name);
    println!("\nTo start: st service start");
    println!("To stop:  st service stop");
    println!("Plist:    {}", plist_path.display());

    Ok(())
}

fn launchd_uninstall() -> Result<()> {
    let plist_path = get_launchd_plist_path()?;

    // Stop first if running
    let _ = launchd_stop();

    if plist_path.exists() {
        fs::remove_file(&plist_path)?;
        info!("Removed {}", plist_path.display());
        println!("✅ Service uninstalled");
    } else {
        println!("Service was not installed");
    }

    Ok(())
}

fn launchd_start() -> Result<()> {
    let project_name = get_project_name()?;
    let label = format!("{}.{}", LAUNCHD_LABEL, project_name);
    let plist_path = get_launchd_plist_path()?;

    if !plist_path.exists() {
        anyhow::bail!("Service not installed. Run 'st service install' first.");
    }

    info!("Starting service {}...", label);
    run_command("launchctl", &["load", plist_path.to_string_lossy().as_ref()])?;

    println!("✅ Service started");
    println!("\nDashboard: http://localhost:8420");
    println!("Logs:      tail -f ~/.st/daemon.log");

    Ok(())
}

fn launchd_stop() -> Result<()> {
    let project_name = get_project_name()?;
    let label = format!("{}.{}", LAUNCHD_LABEL, project_name);
    let plist_path = get_launchd_plist_path()?;

    if !plist_path.exists() {
        println!("Service not installed");
        return Ok(());
    }

    info!("Stopping service {}...", label);
    let _ = run_command("launchctl", &["unload", plist_path.to_string_lossy().as_ref()]);

    println!("✅ Service stopped");
    Ok(())
}

fn launchd_status() -> Result<()> {
    let project_name = get_project_name()?;
    let label = format!("{}.{}", LAUNCHD_LABEL, project_name);
    let plist_path = get_launchd_plist_path()?;

    println!("Smart Tree Service Status (macOS)");
    println!("─────────────────────────────────");
    println!("Label:   {}", label);
    println!("Plist:   {}", plist_path.display());

    if !plist_path.exists() {
        println!("Status:  NOT INSTALLED");
        return Ok(());
    }

    // Check if running
    let output = Command::new("launchctl")
        .args(["list"])
        .output();

    if let Ok(out) = output {
        let list = String::from_utf8_lossy(&out.stdout);
        if list.contains(&label) {
            println!("Status:  🟢 RUNNING");
        } else {
            println!("Status:  ⚪ LOADED (not running)");
        }
    }

    Ok(())
}

fn launchd_logs() -> Result<()> {
    let log_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".st")
        .join("daemon.log");

    println!("Showing logs from {}", log_path.display());
    println!("─────────────────────────────────");

    run_command("tail", &["-f", log_path.to_string_lossy().as_ref()])?;
    Ok(())
}

// =============================================================================
// WINDOWS (Task Scheduler) - Stub for now
// =============================================================================

#[cfg(target_os = "windows")]
fn windows_install() -> Result<()> {
    println!("Windows service installation not yet implemented.");
    println!("For now, run 'st --http-daemon' manually or add to startup.");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn windows_install() -> Result<()> {
    anyhow::bail!("Windows-specific function called on non-Windows platform")
}

// =============================================================================
// FIND ST BINARY
// =============================================================================

fn which_st() -> Result<PathBuf> {
    // Try current exe first
    if let Ok(exe) = std::env::current_exe() {
        return Ok(exe);
    }

    // Try PATH
    let output = Command::new("which")
        .arg("st")
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            return Ok(PathBuf::from(path));
        }
    }

    // Fallback
    Ok(PathBuf::from("/usr/local/bin/st"))
}

/// Run a shell command and log its output.
fn run_command(command: &str, args: &[&str]) -> Result<()> {
    info!("Running command: {} {}", command, args.join(" "));
    let mut cmd = Command::new(command);
    cmd.args(args);
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let status = cmd
        .status()
        .with_context(|| format!("Failed to execute command: {}", command))?;

    if !status.success() {
        error!("Command failed with status: {}", status);
        anyhow::bail!("Command failed with status: {}", status);
    }
    Ok(())
}

/// Get the project name from the current directory.
fn get_project_name() -> Result<String> {
    let cwd = env::current_dir()?;
    let project_name = cwd
        .file_name()
        .and_then(|s| s.to_str())
        .context("Could not determine project name from current directory")?;
    Ok(project_name.to_string())
}

// =============================================================================
// CROSS-PLATFORM PUBLIC API
// =============================================================================

/// Install the service (cross-platform)
pub fn install() -> Result<()> {
    let platform = Platform::current();
    println!("Installing Smart Tree service using {}...", platform.service_manager_name());

    match platform {
        Platform::Linux => systemd_install(),
        Platform::MacOS => launchd_install(),
        Platform::Windows => {
            #[cfg(target_os = "windows")]
            return windows_install();
            #[cfg(not(target_os = "windows"))]
            anyhow::bail!("Windows not supported on this platform")
        }
        Platform::Unknown => anyhow::bail!("Unsupported platform for service management"),
    }
}

/// Uninstall the service (cross-platform)
pub fn uninstall() -> Result<()> {
    match Platform::current() {
        Platform::Linux => systemd_uninstall(),
        Platform::MacOS => launchd_uninstall(),
        Platform::Windows => {
            println!("Windows uninstall not yet implemented");
            Ok(())
        }
        Platform::Unknown => anyhow::bail!("Unsupported platform"),
    }
}

/// Start the service (cross-platform)
pub fn start() -> Result<()> {
    match Platform::current() {
        Platform::Linux => systemd_start(),
        Platform::MacOS => launchd_start(),
        Platform::Windows => {
            println!("Run 'st --http-daemon' manually on Windows");
            Ok(())
        }
        Platform::Unknown => anyhow::bail!("Unsupported platform"),
    }
}

/// Stop the service (cross-platform)
pub fn stop() -> Result<()> {
    match Platform::current() {
        Platform::Linux => systemd_stop(),
        Platform::MacOS => launchd_stop(),
        Platform::Windows => {
            println!("Stop the process manually on Windows");
            Ok(())
        }
        Platform::Unknown => anyhow::bail!("Unsupported platform"),
    }
}

/// Show service status (cross-platform)
pub fn status() -> Result<()> {
    match Platform::current() {
        Platform::Linux => systemd_status(),
        Platform::MacOS => launchd_status(),
        Platform::Windows => {
            println!("Check Task Manager on Windows");
            Ok(())
        }
        Platform::Unknown => anyhow::bail!("Unsupported platform"),
    }
}

/// Show service logs (cross-platform)
pub fn logs() -> Result<()> {
    match Platform::current() {
        Platform::Linux => systemd_logs(),
        Platform::MacOS => launchd_logs(),
        Platform::Windows => {
            println!("Check ~/.st/daemon.log on Windows");
            Ok(())
        }
        Platform::Unknown => anyhow::bail!("Unsupported platform"),
    }
}

// =============================================================================
// LINUX (systemd) IMPLEMENTATION
// =============================================================================

fn systemd_install() -> Result<()> {
    info!("Installing systemd user service...");

    let template_path = PathBuf::from(SYSTEMD_SERVICE_TEMPLATE);
    if !template_path.exists() {
        // Generate a basic service file
        let project_name = get_project_name()?;
        let st_path = which_st()?;
        let working_dir = env::current_dir()?;

        let service_content = format!(r#"[Unit]
Description=Smart Tree Dashboard for %i
After=network.target

[Service]
Type=simple
ExecStart={} --http-daemon
WorkingDirectory={}
Environment=ST_PROJECT=%i
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
"#, st_path.display(), working_dir.display());

        let systemd_path = get_systemd_user_path()?;
        fs::create_dir_all(&systemd_path)?;

        let dest_path = systemd_path.join(SYSTEMD_SERVICE_NAME);
        fs::write(&dest_path, service_content)?;

        run_command("systemctl", &["--user", "daemon-reload"])?;

        println!("✅ Service installed for project '{}'", project_name);
        println!("\nTo start: st service start");
        return Ok(());
    }

    // Original template-based installation
    let systemd_path = get_systemd_user_path()?;
    fs::create_dir_all(&systemd_path)?;

    let dest_path = systemd_path.join(SYSTEMD_SERVICE_NAME);
    fs::copy(&template_path, &dest_path)?;
    run_command("systemctl", &["--user", "daemon-reload"])?;

    println!("✅ Service installed");
    Ok(())
}

fn systemd_uninstall() -> Result<()> {
    let systemd_path = get_systemd_user_path()?;
    let dest_path = systemd_path.join(SYSTEMD_SERVICE_NAME);

    if dest_path.exists() {
        fs::remove_file(&dest_path)?;
        run_command("systemctl", &["--user", "daemon-reload"])?;
        println!("✅ Service uninstalled");
    } else {
        println!("Service was not installed");
    }
    Ok(())
}

fn systemd_start() -> Result<()> {
    let project_name = get_project_name()?;
    let service_instance = format!("smart-tree-dashboard@{}.service", project_name);
    run_command("systemctl", &["--user", "start", &service_instance])?;
    println!("✅ Service started");
    println!("\nDashboard: http://localhost:8420");
    Ok(())
}

fn systemd_stop() -> Result<()> {
    let project_name = get_project_name()?;
    let service_instance = format!("smart-tree-dashboard@{}.service", project_name);
    run_command("systemctl", &["--user", "stop", &service_instance])?;
    println!("✅ Service stopped");
    Ok(())
}

fn systemd_status() -> Result<()> {
    let project_name = get_project_name()?;
    let service_instance = format!("smart-tree-dashboard@{}.service", project_name);
    let _ = run_command("systemctl", &["--user", "status", &service_instance, "--no-pager"]);
    Ok(())
}

fn systemd_logs() -> Result<()> {
    let project_name = get_project_name()?;
    let service_instance = format!("smart-tree-dashboard@{}.service", project_name);
    let _ = run_command("journalctl", &["--user", "-u", &service_instance, "-n", "50", "--no-pager", "-f"]);
    Ok(())
}

// =============================================================================
// AI GUARDIAN - Root daemon for system-wide protection
// =============================================================================

const GUARDIAN_SERVICE_NAME: &str = "smart-tree-guardian.service";
const GUARDIAN_SYSTEM_PATH: &str = "/etc/systemd/system";

/// Check if running as root
fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

/// Compute SHA256 hash of a file for integrity verification
fn compute_file_hash(path: &std::path::Path) -> Result<String> {
    use sha2::{Sha256, Digest};
    use std::io::Read;

    let mut file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open {} for hashing", path.display()))?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Verify the installed binary hasn't been tampered with
pub fn guardian_verify_integrity() -> Result<bool> {
    let installed_path = PathBuf::from("/usr/local/bin/st");

    if !installed_path.exists() {
        warn!("Guardian binary not found at /usr/local/bin/st");
        return Ok(false);
    }

    // Get hash of installed binary
    let installed_hash = compute_file_hash(&installed_path)?;

    // Check against known hash stored during install
    let hash_file = PathBuf::from("/var/lib/smart-tree/guardian.sha256");

    if hash_file.exists() {
        let stored_hash = std::fs::read_to_string(&hash_file)?.trim().to_string();

        if installed_hash != stored_hash {
            error!("⚠️  INTEGRITY VIOLATION: Guardian binary has been modified!");
            error!("   Expected: {}", stored_hash);
            error!("   Found:    {}", installed_hash);
            return Ok(false);
        }

        info!("✅ Guardian binary integrity verified");
        Ok(true)
    } else {
        warn!("No stored hash found - cannot verify integrity");
        Ok(true) // Can't verify without stored hash
    }
}

// =============================================================================
// GPG SIGNATURE VERIFICATION - Official build trust chain
// =============================================================================

/// 8bit-wraith's official GPG key fingerprint for signed releases
/// wraith@8b.is - The 8b-IS team signing identity
pub const OFFICIAL_GPG_FINGERPRINT: &str = "wraith@8b.is";

/// Check if this is an officially signed build
pub fn verify_gpg_signature() -> SignatureStatus {
    // Check for detached signature file
    let sig_path = PathBuf::from("/usr/local/bin/st.sig");
    let binary_path = PathBuf::from("/usr/local/bin/st");

    if !sig_path.exists() {
        return SignatureStatus::Unsigned;
    }

    // Try to verify with gpg
    let output = Command::new("gpg")
        .args(["--verify", sig_path.to_string_lossy().as_ref(), binary_path.to_string_lossy().as_ref()])
        .output();

    match output {
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);

            if result.status.success() {
                // Check if it's signed by the official key
                if stderr.contains(OFFICIAL_GPG_FINGERPRINT) {
                    SignatureStatus::OfficialBuild
                } else {
                    // Signed but not by official key
                    SignatureStatus::CommunityBuild(extract_signer(&stderr))
                }
            } else if stderr.contains("BAD signature") {
                SignatureStatus::TamperedOrInvalid
            } else {
                SignatureStatus::Unsigned
            }
        }
        Err(_) => SignatureStatus::GpgNotAvailable,
    }
}

/// Extract signer info from GPG output
fn extract_signer(gpg_output: &str) -> String {
    for line in gpg_output.lines() {
        if line.contains("Good signature from") {
            return line.to_string();
        }
    }
    "Unknown signer".to_string()
}

#[derive(Debug, Clone, PartialEq)]
pub enum SignatureStatus {
    /// Signed by Hue's official key - this is an authentic release
    OfficialBuild,
    /// Signed by someone else - community/custom build
    CommunityBuild(String),
    /// No signature found
    Unsigned,
    /// Signature doesn't match - potential tampering
    TamperedOrInvalid,
    /// GPG not installed
    GpgNotAvailable,
}

/// Print signature verification banner on first run
pub fn print_signature_banner() {
    let first_run_marker = dirs::data_dir()
        .map(|d| d.join("smart-tree").join(".first_run_complete"))
        .unwrap_or_else(|| PathBuf::from("/tmp/.st_first_run"));

    // Skip if already shown
    if first_run_marker.exists() {
        return;
    }

    let status = verify_gpg_signature();

    println!();
    match status {
        SignatureStatus::OfficialBuild => {
            println!("╔═══════════════════════════════════════════════════════════════════════════════╗");
            println!("║  ✅ OFFICIAL BUILD - Signed by 8bit-wraith (wraith@8b.is)                     ║");
            println!("║                                                                               ║");
            println!("║  This Smart Tree binary is cryptographically signed and verified.             ║");
            println!("║  You can trust this is an authentic release from the 8b-IS team.              ║");
            println!("╚═══════════════════════════════════════════════════════════════════════════════╝");
        }
        SignatureStatus::CommunityBuild(signer) => {
            println!("╔═══════════════════════════════════════════════════════════════════════════════╗");
            println!("║  🔵 COMMUNITY BUILD - Custom signed release                                   ║");
            println!("║                                                                               ║");
            println!("║  This build is signed, but NOT by the official 8b.is key.                     ║");
            println!("║  Signer: {}                                                                   ", &signer[..signer.len().min(60)]);
            println!("║                                                                               ║");
            println!("║  This may be a legitimate fork or custom build. Verify you trust the signer.  ║");
            println!("╚═══════════════════════════════════════════════════════════════════════════════╝");
        }
        SignatureStatus::Unsigned => {
            println!("╔═══════════════════════════════════════════════════════════════════════════════╗");
            println!("║  ⚪ UNSIGNED BUILD - No cryptographic signature                               ║");
            println!("║                                                                               ║");
            println!("║  This Smart Tree binary has no GPG signature attached.                        ║");
            println!("║  This is normal for development builds or self-compiled versions.             ║");
            println!("║                                                                               ║");
            println!("║  For verified official releases, download from: https://i1.is/smart-tree      ║");
            println!("╚═══════════════════════════════════════════════════════════════════════════════╝");
        }
        SignatureStatus::TamperedOrInvalid => {
            println!("╔═══════════════════════════════════════════════════════════════════════════════╗");
            println!("║  ⛔ WARNING: SIGNATURE VERIFICATION FAILED                                    ║");
            println!("║                                                                               ║");
            println!("║  This binary has a signature that DOES NOT MATCH the file contents!           ║");
            println!("║  This could indicate:                                                         ║");
            println!("║    - The binary was modified after signing (POTENTIAL TAMPERING)              ║");
            println!("║    - Corrupted download                                                       ║");
            println!("║    - Signature file mismatch                                                  ║");
            println!("║                                                                               ║");
            println!("║  RECOMMENDATION: Re-download from https://i1.is/smart-tree                    ║");
            println!("╚═══════════════════════════════════════════════════════════════════════════════╝");
        }
        SignatureStatus::GpgNotAvailable => {
            println!("╔═══════════════════════════════════════════════════════════════════════════════╗");
            println!("║  ℹ️  GPG not available - signature verification skipped                       ║");
            println!("║                                                                               ║");
            println!("║  Install GPG to enable cryptographic verification of official builds.         ║");
            println!("║  Arch: sudo pacman -S gnupg                                                   ║");
            println!("║  Ubuntu: sudo apt install gnupg                                               ║");
            println!("╚═══════════════════════════════════════════════════════════════════════════════╝");
        }
    }
    println!();

    // Mark first run complete
    if let Some(parent) = first_run_marker.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&first_run_marker, "shown");
}

/// Install Smart Tree Guardian as a root daemon
pub fn guardian_install() -> Result<()> {
    println!(r#"
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║    🛡️  SMART TREE GUARDIAN - System-wide AI Protection Daemon 🛡️            ║
║                                                                               ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║  This will install Smart Tree as a ROOT daemon that provides:                 ║
║                                                                               ║
║    • System-wide prompt injection scanning                                    ║
║    • AI context protection across all users                                   ║
║    • Real-time file monitoring for hidden instructions                        ║
║    • Unicode smuggling detection (ASCII smuggling, zero-width chars)          ║
║    • Persistent memory poisoning defense                                      ║
║                                                                               ║
║  The Guardian daemon runs as root to have full system visibility.             ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
"#);

    // Check if we need sudo
    if !is_root() {
        println!("Root access required. Running with sudo...\n");

        // Get the path to our binary
        let exe_path = std::env::current_exe()
            .context("Could not determine executable path")?;

        // Re-run with sudo
        let status = Command::new("sudo")
            .args([exe_path.to_string_lossy().as_ref(), "--guardian-install"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("Failed to run with sudo")?;

        if !status.success() {
            anyhow::bail!("Installation failed");
        }
        return Ok(());
    }

    // We're root - do the installation
    info!("Installing Guardian daemon as root...");

    // 1. Copy binary to /usr/local/bin (if not already there)
    let exe_path = std::env::current_exe()?;
    let target_bin = PathBuf::from("/usr/local/bin/st");

    if exe_path != target_bin {
        info!("Copying binary to /usr/local/bin/st");
        fs::copy(&exe_path, &target_bin)
            .context("Failed to copy binary to /usr/local/bin")?;

        // Make executable
        Command::new("chmod")
            .args(["755", "/usr/local/bin/st"])
            .status()?;
    }

    // 2. Create state directory
    fs::create_dir_all("/var/lib/smart-tree")
        .context("Failed to create /var/lib/smart-tree")?;

    // 2.5 Store hash of installed binary for integrity verification
    let binary_hash = compute_file_hash(&target_bin)?;
    let hash_file = PathBuf::from("/var/lib/smart-tree/guardian.sha256");
    fs::write(&hash_file, &binary_hash)
        .context("Failed to write integrity hash")?;
    info!("Stored integrity hash: {}", binary_hash);

    // 3. Write the service file
    let service_content = include_str!("../systemd/smart-tree-guardian.service");
    let service_path = format!("{}/{}", GUARDIAN_SYSTEM_PATH, GUARDIAN_SERVICE_NAME);

    info!("Writing service file to {}", service_path);
    fs::write(&service_path, service_content)
        .with_context(|| format!("Failed to write service file to {}", service_path))?;

    // 4. Reload systemd and enable the service
    run_command("systemctl", &["daemon-reload"])?;
    run_command("systemctl", &["enable", GUARDIAN_SERVICE_NAME])?;
    run_command("systemctl", &["start", GUARDIAN_SERVICE_NAME])?;

    println!(r#"
╔═══════════════════════════════════════════════════════════════════════════════╗
║  ✅ Guardian installed and running!                                           ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║  Commands:                                                                    ║
║    st --guardian-status     Show daemon status                                ║
║    st --guardian-scan FILE  Scan a file for injection attempts                ║
║    sudo systemctl stop smart-tree-guardian     Stop the daemon                ║
║    sudo journalctl -u smart-tree-guardian -f   Watch logs                     ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
"#);

    Ok(())
}

/// Uninstall Smart Tree Guardian
pub fn guardian_uninstall() -> Result<()> {
    println!("Uninstalling Smart Tree Guardian...\n");

    if !is_root() {
        // Re-run with sudo
        let exe_path = std::env::current_exe()?;
        let status = Command::new("sudo")
            .args([exe_path.to_string_lossy().as_ref(), "--guardian-uninstall"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        if !status.success() {
            anyhow::bail!("Uninstallation failed");
        }
        return Ok(());
    }

    // Stop and disable service
    let _ = run_command("systemctl", &["stop", GUARDIAN_SERVICE_NAME]);
    let _ = run_command("systemctl", &["disable", GUARDIAN_SERVICE_NAME]);

    // Remove service file
    let service_path = format!("{}/{}", GUARDIAN_SYSTEM_PATH, GUARDIAN_SERVICE_NAME);
    if PathBuf::from(&service_path).exists() {
        fs::remove_file(&service_path)?;
        info!("Removed {}", service_path);
    }

    // Reload systemd
    run_command("systemctl", &["daemon-reload"])?;

    println!("✅ Guardian uninstalled successfully.");
    Ok(())
}

/// Show Guardian daemon status
pub fn guardian_status() -> Result<()> {
    println!(r#"
╔═══════════════════════════════════════════════════════════════════════════════╗
║            🛡️  SMART TREE GUARDIAN STATUS 🛡️                                 ║
╚═══════════════════════════════════════════════════════════════════════════════╝
"#);

    // Check if service exists
    let service_path = format!("{}/{}", GUARDIAN_SYSTEM_PATH, GUARDIAN_SERVICE_NAME);
    if !PathBuf::from(&service_path).exists() {
        println!("  Status: NOT INSTALLED");
        println!("\n  Install with: st --guardian-install");
        return Ok(());
    }

    // Show systemctl status
    let output = Command::new("systemctl")
        .args(["status", GUARDIAN_SERVICE_NAME, "--no-pager"])
        .output();

    match output {
        Ok(out) => {
            let status = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            if status.contains("Active: active") {
                println!("  Status: 🟢 RUNNING");
            } else if status.contains("Active: inactive") {
                println!("  Status: 🔴 STOPPED");
            } else {
                println!("  Status: ⚪ UNKNOWN");
            }

            println!("\n{}", status);
            if !stderr.is_empty() {
                println!("{}", stderr);
            }
        }
        Err(_) => {
            println!("  Could not determine status. Try: sudo systemctl status {}", GUARDIAN_SERVICE_NAME);
        }
    }

    Ok(())
}
