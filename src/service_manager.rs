// Service Manager for Smart Tree Daemon
// Handles installing, uninstalling, and controlling the systemd user service.

use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tracing::{error, info, warn};

const SERVICE_FILE_NAME: &str = "smart-tree-dashboard@.service";
const SERVICE_TEMPLATE_PATH: &str = "systemd/smart-tree-dashboard@.service";

/// Get the path for the systemd user service files.
fn get_systemd_user_path() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|home| home.join(".config").join("systemd").join("user"))
        .context("Could not find home directory")
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

/// Install the systemd user service.
pub fn install() -> Result<()> {
    info!("Installing systemd user service...");

    // 1. Check if the template file exists
    let template_path = PathBuf::from(SERVICE_TEMPLATE_PATH);
    if !template_path.exists() {
        error!(
            "Service template not found at '{}'",
            template_path.display()
        );
        anyhow::bail!(
            "Service template not found at '{}'. Make sure you are running from the project root.",
            template_path.display()
        );
    }

    // 2. Get systemd user path and create it if it doesn't exist
    let systemd_path = get_systemd_user_path()?;
    fs::create_dir_all(&systemd_path)
        .with_context(|| format!("Failed to create systemd directory at {:?}", systemd_path))?;

    // 3. Copy the file
    let dest_path = systemd_path.join(SERVICE_FILE_NAME);
    info!(
        "Copying '{}' to '{}'",
        template_path.display(),
        dest_path.display()
    );
    fs::copy(&template_path, &dest_path).with_context(|| {
        format!(
            "Failed to copy service file from {} to {}",
            template_path.display(),
            dest_path.display()
        )
    })?;

    // 4. Reload systemd daemon
    run_command("systemctl", &["--user", "daemon-reload"])?;

    info!("Service installed successfully!");
    println!("\nYou may need to edit the service file to configure paths:");
    println!("   {}", dest_path.display());
    println!("\nThen, to start the service for this project, run:");
    println!("   st service start");

    Ok(())
}

/// Uninstall the systemd user service.
pub fn uninstall() -> Result<()> {
    info!("Uninstalling systemd user service...");

    // 1. Get paths
    let systemd_path = get_systemd_user_path()?;
    let dest_path = systemd_path.join(SERVICE_FILE_NAME);

    if !dest_path.exists() {
        warn!(
            "Service file not found at '{}'. Already uninstalled?",
            dest_path.display()
        );
        return Ok(());
    }

    // 2. Remove the file
    info!("Removing '{}'", dest_path.display());
    fs::remove_file(&dest_path)
        .with_context(|| format!("Failed to remove service file at {}", dest_path.display()))?;

    // 3. Reload systemd daemon
    run_command("systemctl", &["--user", "daemon-reload"])?;

    info!("Service uninstalled successfully!");
    Ok(())
}

/// Start the systemd user service for the current project.
pub fn start() -> Result<()> {
    let project_name = get_project_name()?;
    let service_instance = format!("smart-tree-dashboard@{}.service", project_name);
    info!("Starting service for project '{}'...", project_name);
    run_command("systemctl", &["--user", "start", &service_instance])?;
    info!("Service started.");
    println!("\nTo check its status, run:");
    println!("   st service status");
    Ok(())
}

/// Stop the systemd user service for the current project.
pub fn stop() -> Result<()> {
    let project_name = get_project_name()?;
    let service_instance = format!("smart-tree-dashboard@{}.service", project_name);
    info!("Stopping service for project '{}'...", project_name);
    run_command("systemctl", &["--user", "stop", &service_instance])?;
    info!("Service stopped.");
    Ok(())
}

/// Show the status of the systemd user service for the current project.
pub fn status() -> Result<()> {
    let project_name = get_project_name()?;
    let service_instance = format!("smart-tree-dashboard@{}.service", project_name);
    info!("Checking status for service '{}':", service_instance);
    // We don't mind if this command fails (e.g., service not running)
    let _ = run_command(
        "systemctl",
        &["--user", "status", &service_instance, "--no-pager"],
    );
    Ok(())
}

/// Show recent logs for the systemd user service.
pub fn logs() -> Result<()> {
    let project_name = get_project_name()?;
    let service_instance = format!("smart-tree-dashboard@{}.service", project_name);
    info!("Showing logs for service '{}':", service_instance);
    let _ = run_command(
        "journalctl",
        &[
            "--user",
            "-u",
            &service_instance,
            "-n",
            "50",
            "--no-pager",
            "-f",
        ],
    );
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
