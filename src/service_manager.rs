// Service Manager for Smart Tree Daemon
// Cross-platform system-level service management:
//   Linux  → systemd (/etc/systemd/system/)
//   macOS  → launchd (/Library/LaunchDaemons/)
//   Windows → sc.exe (Windows Service)
//
// `st service install` auto-escalates privileges on all platforms.

use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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
            Platform::Windows => "sc.exe",
            Platform::Unknown => "unknown",
        }
    }
}

// =============================================================================
// CONSTANTS
// =============================================================================

const DAEMON_PORT: u16 = 28428;

// Linux
const SYSTEMD_DAEMON_SERVICE: &str = "smart-tree-daemon.service";
const SYSTEMD_SYSTEM_PATH: &str = "/etc/systemd/system";

// macOS
const LAUNCHD_DAEMON_LABEL: &str = "is.8b.smart-tree-daemon";
const LAUNCHD_DAEMON_PLIST: &str = "/Library/LaunchDaemons/is.8b.smart-tree-daemon.plist";

// Windows
#[cfg(target_os = "windows")]
const WINDOWS_SERVICE_NAME: &str = "SmartTreeDaemon";

// =============================================================================
// PRIVILEGE ESCALATION
// =============================================================================

/// Check if running as root/admin
fn is_elevated() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(windows)]
    {
        // Check for admin by trying to open a privileged registry key
        use std::process::Command;
        Command::new("net")
            .args(["session"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

/// Re-execute the current command with elevated privileges.
/// Returns Ok(true) if we re-launched (caller should exit), Ok(false) if already elevated.
fn escalate_privileges(args: &[&str]) -> Result<bool> {
    if is_elevated() {
        return Ok(false);
    }

    let exe = env::current_exe().context("Could not determine executable path")?;

    println!("This operation requires elevated privileges.");
    println!();

    #[cfg(target_os = "linux")]
    {
        let has_gui = env::var_os("DISPLAY").is_some() || env::var_os("WAYLAND_DISPLAY").is_some();
        // Prefer sudo in CLI/SSH sessions; only use pkexec when a GUI session is active or sudo is absent
        let cmd =
            if (!has_gui && which_binary("sudo").is_some()) || which_binary("pkexec").is_none() {
                println!("Requesting permission via sudo...");
                let mut c = Command::new("sudo");
                c.arg(&exe);
                c.args(args);
                c
            } else {
                println!("Requesting permission via pkexec...");
                let mut c = Command::new("pkexec");
                c.arg(&exe);
                c.args(args);
                c
            };
        let status = run_elevated(cmd)?;
        if !status.success() {
            anyhow::bail!("Elevated command failed (exit code: {:?})", status.code());
        }
        Ok(true)
    }

    #[cfg(target_os = "macos")]
    {
        // Use sudo only when stdin is an interactive TTY; otherwise use osascript GUI prompt
        let is_interactive_tty = unsafe { libc::isatty(0) == 1 };
        if is_interactive_tty
            && (env::var("TERM_PROGRAM").is_ok() || env::var("SSH_CONNECTION").is_ok())
        {
            println!("Requesting permission via sudo...");
            let mut cmd = Command::new("sudo");
            cmd.arg(&exe);
            cmd.args(args);
            let status = run_elevated(cmd)?;
            if !status.success() {
                anyhow::bail!("Elevated command failed");
            }
        } else {
            // Use osascript for GUI prompt
            let args_str = std::iter::once(exe.to_string_lossy().to_string())
                .chain(args.iter().map(|a| a.to_string()))
                .collect::<Vec<_>>()
                .join("' '");
            let script = format!(
                "do shell script \"'{}' \" with administrator privileges",
                args_str
            );
            let mut cmd = Command::new("osascript");
            cmd.args(["-e", &script]);
            let status = run_elevated(cmd)?;
            if !status.success() {
                anyhow::bail!("Elevated command failed");
            }
        }
        Ok(true)
    }

    #[cfg(target_os = "windows")]
    {
        // Use PowerShell Start-Process -Verb RunAs for UAC prompt
        let args_joined = args.join(" ");
        let ps_cmd = format!(
            "Start-Process -FilePath '{}' -ArgumentList '{}' -Verb RunAs -Wait",
            exe.display(),
            args_joined
        );
        println!("Requesting administrator permission...");
        let mut cmd = Command::new("powershell");
        cmd.args(["-Command", &ps_cmd]);
        let status = run_elevated(cmd)?;
        if !status.success() {
            anyhow::bail!("Elevated command failed");
        }
        Ok(true)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    anyhow::bail!("Privilege escalation not supported on this platform")
}

fn run_elevated(mut cmd: Command) -> Result<std::process::ExitStatus> {
    cmd.stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .status()
        .context("Failed to run elevated command")
}

// =============================================================================
// CROSS-PLATFORM PUBLIC API
// =============================================================================

/// Install the Smart Tree daemon as a system-level service.
/// Auto-escalates privileges if needed.
pub fn install() -> Result<()> {
    let platform = Platform::current();
    println!(
        "Installing Smart Tree daemon as system service ({})...",
        platform.service_manager_name()
    );
    println!();

    match platform {
        Platform::Linux => linux_install(),
        Platform::MacOS => macos_install(),
        Platform::Windows => windows_install(),
        Platform::Unknown => anyhow::bail!("Unsupported platform for service management"),
    }
}

/// Uninstall the system-level service.
pub fn uninstall() -> Result<()> {
    match Platform::current() {
        Platform::Linux => linux_uninstall(),
        Platform::MacOS => macos_uninstall(),
        Platform::Windows => windows_uninstall(),
        Platform::Unknown => anyhow::bail!("Unsupported platform"),
    }
}

/// Start the service.
pub fn start() -> Result<()> {
    match Platform::current() {
        Platform::Linux => linux_start(),
        Platform::MacOS => macos_start(),
        Platform::Windows => windows_start(),
        Platform::Unknown => anyhow::bail!("Unsupported platform"),
    }
}

/// Stop the service.
pub fn stop() -> Result<()> {
    match Platform::current() {
        Platform::Linux => linux_stop(),
        Platform::MacOS => macos_stop(),
        Platform::Windows => windows_stop(),
        Platform::Unknown => anyhow::bail!("Unsupported platform"),
    }
}

/// Show service status.
pub fn status() -> Result<()> {
    match Platform::current() {
        Platform::Linux => linux_status(),
        Platform::MacOS => macos_status(),
        Platform::Windows => windows_status(),
        Platform::Unknown => anyhow::bail!("Unsupported platform"),
    }
}

/// Show service logs.
pub fn logs() -> Result<()> {
    match Platform::current() {
        Platform::Linux => linux_logs(),
        Platform::MacOS => macos_logs(),
        Platform::Windows => windows_logs(),
        Platform::Unknown => anyhow::bail!("Unsupported platform"),
    }
}

/// Deprecated: use install() instead. Kept for --daemon-install backward compat.
pub fn daemon_install_system() -> Result<()> {
    eprintln!("Note: --daemon-install is deprecated, use `st service install` instead.");
    install()
}

// =============================================================================
// LINUX (systemd) — /etc/systemd/system/
// =============================================================================

fn linux_install() -> Result<()> {
    // Escalate if not root
    if escalate_privileges(&["service", "install"])? {
        return Ok(()); // child process did the work
    }

    // We're root now
    info!("Installing systemd system service...");

    // 1. Find/copy binary to /usr/local/bin
    let install_path = install_binary()?;

    // 2. Create state directory
    fs::create_dir_all("/var/lib/smart-tree").context("Failed to create /var/lib/smart-tree")?;

    // 3. Store integrity hash
    let hash = compute_file_hash(&install_path)?;
    fs::write("/var/lib/smart-tree/daemon.sha256", &hash)?;
    if Path::new("/var/lib/smart-tree/guardian.sha256").exists() {
        fs::write("/var/lib/smart-tree/guardian.sha256", &hash)?;
    }

    // Ensure existing system token is readable by local CLI clients
    #[cfg(unix)]
    {
        let token_file = Path::new("/var/lib/smart-tree/daemon.token");
        if token_file.exists() {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(token_file, fs::Permissions::from_mode(0o644));
        }
    }

    // 4. Write the systemd service file
    let service_dest = PathBuf::from(SYSTEMD_SYSTEM_PATH).join(SYSTEMD_DAEMON_SERVICE);
    let service_content = generate_systemd_unit(&install_path);
    fs::write(&service_dest, &service_content)
        .with_context(|| format!("Failed to write {}", service_dest.display()))?;

    // 5. Reload, enable, and restart so upgrades replace any running daemon instance
    run_command("systemctl", &["daemon-reload"])?;
    run_command("systemctl", &["enable", SYSTEMD_DAEMON_SERVICE])?;
    run_command("systemctl", &["restart", SYSTEMD_DAEMON_SERVICE])?;

    // If guardian service is active, restart it to pick up the updated binary
    if Command::new("systemctl")
        .args(["is-active", "--quiet", "smart-tree-guardian.service"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        let _ = run_command("systemctl", &["restart", "smart-tree-guardian.service"]);
    }

    // Reset any failed smart-tree-updater unit state after manual upgrade
    let _ = Command::new("systemctl")
        .args(["reset-failed", "smart-tree-updater.service"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    print_install_success(
        "systemctl status smart-tree-daemon",
        "journalctl -u smart-tree-daemon -f",
    );
    Ok(())
}

fn linux_uninstall() -> Result<()> {
    if escalate_privileges(&["service", "uninstall"])? {
        return Ok(());
    }

    let _ = run_command("systemctl", &["stop", SYSTEMD_DAEMON_SERVICE]);
    let _ = run_command("systemctl", &["disable", SYSTEMD_DAEMON_SERVICE]);

    let service_path = PathBuf::from(SYSTEMD_SYSTEM_PATH).join(SYSTEMD_DAEMON_SERVICE);
    if service_path.exists() {
        fs::remove_file(&service_path)?;
    }
    run_command("systemctl", &["daemon-reload"])?;

    println!("Service uninstalled.");
    Ok(())
}

fn linux_start() -> Result<()> {
    if escalate_privileges(&["service", "start"])? {
        return Ok(());
    }
    run_command("systemctl", &["start", SYSTEMD_DAEMON_SERVICE])?;
    println!(
        "Service started. Dashboard: http://localhost:{}",
        DAEMON_PORT
    );
    Ok(())
}

fn linux_stop() -> Result<()> {
    if escalate_privileges(&["service", "stop"])? {
        return Ok(());
    }
    run_command("systemctl", &["stop", SYSTEMD_DAEMON_SERVICE])?;
    println!("Service stopped.");
    Ok(())
}

fn linux_status() -> Result<()> {
    // status doesn't need root — systemctl status works for any user
    println!("Smart Tree Daemon Status (Linux/systemd)");
    println!("─────────────────────────────────────────");
    let _ = run_command(
        "systemctl",
        &["status", SYSTEMD_DAEMON_SERVICE, "--no-pager"],
    );
    Ok(())
}

fn linux_logs() -> Result<()> {
    let _ = run_command(
        "journalctl",
        &["-u", SYSTEMD_DAEMON_SERVICE, "-n", "50", "--no-pager", "-f"],
    );
    Ok(())
}

fn generate_systemd_unit(binary_path: &Path) -> String {
    format!(
        r#"[Unit]
Description=Smart Tree Daemon - AI Context Service
Documentation=https://github.com/8b-is/smart-tree
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart={binary} --http-daemon

StateDirectory=smart-tree
StateDirectoryMode=0755
RuntimeDirectory=smart-tree
RuntimeDirectoryMode=0755

Environment=RUST_LOG=info
Environment=HOME=/var/lib/smart-tree
Environment=ST_TOKEN_PATH=/var/lib/smart-tree/daemon.token

Restart=always
RestartSec=10
TimeoutStopSec=30

StandardOutput=journal
StandardError=journal
SyslogIdentifier=smart-tree-daemon

WorkingDirectory=/var/lib/smart-tree

NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=read-only
PrivateTmp=yes
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectControlGroups=yes
ReadOnlyPaths=/home

[Install]
WantedBy=multi-user.target
"#,
        binary = binary_path.display(),
    )
}

// =============================================================================
// macOS (launchd) — /Library/LaunchDaemons/
// =============================================================================

fn macos_install() -> Result<()> {
    if escalate_privileges(&["service", "install"])? {
        return Ok(());
    }

    info!("Installing LaunchDaemon...");

    // 1. Install binary
    let install_path = install_binary()?;

    // 2. Create state directory
    let state_dir = PathBuf::from("/var/lib/smart-tree");
    fs::create_dir_all(&state_dir).context("Failed to create /var/lib/smart-tree")?;

    // 3. Store integrity hash
    let hash = compute_file_hash(&install_path)?;
    fs::write(state_dir.join("daemon.sha256"), &hash)?;

    // 4. Create log directory
    let log_dir = PathBuf::from("/var/log/smart-tree");
    fs::create_dir_all(&log_dir).context("Failed to create /var/log/smart-tree")?;

    // 5. Write the LaunchDaemon plist
    let plist = generate_launchd_daemon_plist(&install_path);
    fs::write(LAUNCHD_DAEMON_PLIST, &plist)
        .with_context(|| format!("Failed to write {}", LAUNCHD_DAEMON_PLIST))?;

    // 6. Set ownership (must be owned by root:wheel for launchd)
    run_command("chown", &["root:wheel", LAUNCHD_DAEMON_PLIST])?;
    run_command("chmod", &["644", LAUNCHD_DAEMON_PLIST])?;

    // 7. Load or restart the daemon so upgrades replace any running instance
    let kickstart = Command::new("launchctl")
        .args([
            "kickstart",
            "-k",
            &format!("system/{}", LAUNCHD_DAEMON_LABEL),
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    if !kickstart.map(|s| s.success()).unwrap_or(false) {
        let load_result = Command::new("launchctl")
            .args(["bootstrap", "system", LAUNCHD_DAEMON_PLIST])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();

        match load_result {
            Ok(s) if s.success() => {}
            _ => {
                info!("bootstrap failed, trying legacy load...");
                let _ = run_command("launchctl", &["load", "-w", LAUNCHD_DAEMON_PLIST]);
                let _ = Command::new("launchctl")
                    .args([
                        "kickstart",
                        "-k",
                        &format!("system/{}", LAUNCHD_DAEMON_LABEL),
                    ])
                    .status();
            }
        }
    }

    print_install_success(
        "sudo launchctl list | grep smart-tree",
        "tail -f /var/log/smart-tree/daemon.log",
    );
    Ok(())
}

fn macos_uninstall() -> Result<()> {
    if escalate_privileges(&["service", "uninstall"])? {
        return Ok(());
    }

    // Try modern bootout, fall back to legacy unload
    let _ = Command::new("launchctl")
        .args(["bootout", "system", LAUNCHD_DAEMON_PLIST])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = Command::new("launchctl")
        .args(["unload", LAUNCHD_DAEMON_PLIST])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if PathBuf::from(LAUNCHD_DAEMON_PLIST).exists() {
        fs::remove_file(LAUNCHD_DAEMON_PLIST)?;
    }

    println!("Service uninstalled.");
    Ok(())
}

fn macos_start() -> Result<()> {
    if escalate_privileges(&["service", "start"])? {
        return Ok(());
    }
    // kickstart forces immediate start
    let result = Command::new("launchctl")
        .args(["kickstart", &format!("system/{}", LAUNCHD_DAEMON_LABEL)])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match result {
        Ok(s) if s.success() => {}
        _ => {
            // Fallback: bootstrap loads + starts
            run_command("launchctl", &["load", "-w", LAUNCHD_DAEMON_PLIST])?;
        }
    }

    println!(
        "Service started. Dashboard: http://localhost:{}",
        DAEMON_PORT
    );
    Ok(())
}

fn macos_stop() -> Result<()> {
    if escalate_privileges(&["service", "stop"])? {
        return Ok(());
    }
    let _ = Command::new("launchctl")
        .args([
            "kill",
            "SIGTERM",
            &format!("system/{}", LAUNCHD_DAEMON_LABEL),
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    println!("Service stopped.");
    Ok(())
}

fn macos_status() -> Result<()> {
    println!("Smart Tree Daemon Status (macOS/launchd)");
    println!("─────────────────────────────────────────");

    if !PathBuf::from(LAUNCHD_DAEMON_PLIST).exists() {
        println!("Status:  NOT INSTALLED");
        println!("Install: st service install");
        return Ok(());
    }

    let output = Command::new("launchctl")
        .args(["print", &format!("system/{}", LAUNCHD_DAEMON_LABEL)])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            if text.contains("state = running") {
                println!("Status:  RUNNING");
            } else {
                println!("Status:  LOADED (not running)");
            }
            // Show PID if available
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("pid =") || trimmed.starts_with("state =") {
                    println!("  {}", trimmed);
                }
            }
        }
        _ => {
            // Fallback: check launchctl list
            let list = Command::new("launchctl").args(["list"]).output();
            if let Ok(out) = list {
                let text = String::from_utf8_lossy(&out.stdout);
                if text.contains(LAUNCHD_DAEMON_LABEL) {
                    println!("Status:  LOADED");
                } else {
                    println!("Status:  NOT LOADED");
                }
            }
        }
    }

    Ok(())
}

fn macos_logs() -> Result<()> {
    let log_path = "/var/log/smart-tree/daemon.log";
    if PathBuf::from(log_path).exists() {
        println!("Showing logs from {}", log_path);
        println!("─────────────────────────────────");
        run_command("tail", &["-f", log_path])?;
    } else {
        // Try system log
        println!("Showing logs from system log...");
        run_command(
            "log",
            &[
                "show",
                "--predicate",
                "process == \"st\"",
                "--last",
                "1h",
                "--style",
                "compact",
            ],
        )?;
    }
    Ok(())
}

fn generate_launchd_daemon_plist(binary_path: &Path) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>

    <key>ProgramArguments</key>
    <array>
        <string>{binary}</string>
        <string>--http-daemon</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>WorkingDirectory</key>
    <string>/var/lib/smart-tree</string>

    <key>StandardOutPath</key>
    <string>/var/log/smart-tree/daemon.log</string>

    <key>StandardErrorPath</key>
    <string>/var/log/smart-tree/daemon.log</string>

    <key>EnvironmentVariables</key>
    <dict>
        <key>RUST_LOG</key>
        <string>info</string>
        <key>ST_TOKEN_PATH</key>
        <string>/var/lib/smart-tree/daemon.token</string>
    </dict>

    <key>ThrottleInterval</key>
    <integer>10</integer>
</dict>
</plist>
"#,
        label = LAUNCHD_DAEMON_LABEL,
        binary = binary_path.display(),
    )
}

// =============================================================================
// WINDOWS (sc.exe Windows Service)
// =============================================================================

fn windows_install() -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        anyhow::bail!("Windows service management is only available on Windows");
    }

    #[cfg(target_os = "windows")]
    {
        if escalate_privileges(&["service", "install"])? {
            return Ok(());
        }

        info!("Installing Windows service...");

        // 1. Find/copy binary
        let install_path = install_binary()?;

        // 2. Create state directory
        let state_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\ProgramData"))
            .join("SmartTree");
        fs::create_dir_all(&state_dir)?;

        // 3. Store integrity hash
        let hash = compute_file_hash(&install_path)?;
        fs::write(state_dir.join("daemon.sha256"), &hash)?;

        // 4. Create the Windows service using sc.exe
        let bin_arg = format!(
            "\"{}\" --http-daemon --daemon-port {}",
            install_path.display(),
            DAEMON_PORT
        );

        // Delete existing service first (ignore error if it doesn't exist)
        let _ = Command::new("sc.exe")
            .args(["delete", WINDOWS_SERVICE_NAME])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        run_command(
            "sc.exe",
            &[
                "create",
                WINDOWS_SERVICE_NAME,
                &format!("binPath= {}", bin_arg),
                "start= auto",
                "DisplayName= Smart Tree Daemon",
            ],
        )?;

        // Set description
        run_command(
            "sc.exe",
            &[
                "description",
                WINDOWS_SERVICE_NAME,
                "Smart Tree - AI-friendly directory visualization daemon",
            ],
        )?;

        // Set recovery: restart on failure
        run_command(
            "sc.exe",
            &[
                "failure",
                WINDOWS_SERVICE_NAME,
                "reset= 86400",
                "actions= restart/10000/restart/30000/restart/60000",
            ],
        )?;

        // Start the service
        run_command("sc.exe", &["start", WINDOWS_SERVICE_NAME])?;

        print_install_success(
            "sc.exe query SmartTreeDaemon",
            "Get-EventLog -LogName Application -Source SmartTreeDaemon",
        );
        Ok(())
    }
}

fn windows_uninstall() -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        anyhow::bail!("Windows service management is only available on Windows");
    }

    #[cfg(target_os = "windows")]
    {
        if escalate_privileges(&["service", "uninstall"])? {
            return Ok(());
        }

        let _ = Command::new("sc.exe")
            .args(["stop", WINDOWS_SERVICE_NAME])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        run_command("sc.exe", &["delete", WINDOWS_SERVICE_NAME])?;
        println!("Service uninstalled.");
        Ok(())
    }
}

fn windows_start() -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        anyhow::bail!("Windows service management is only available on Windows");
    }

    #[cfg(target_os = "windows")]
    {
        if escalate_privileges(&["service", "start"])? {
            return Ok(());
        }
        run_command("sc.exe", &["start", WINDOWS_SERVICE_NAME])?;
        println!(
            "Service started. Dashboard: http://localhost:{}",
            DAEMON_PORT
        );
        Ok(())
    }
}

fn windows_stop() -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        anyhow::bail!("Windows service management is only available on Windows");
    }

    #[cfg(target_os = "windows")]
    {
        if escalate_privileges(&["service", "stop"])? {
            return Ok(());
        }
        run_command("sc.exe", &["stop", WINDOWS_SERVICE_NAME])?;
        println!("Service stopped.");
        Ok(())
    }
}

fn windows_status() -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        anyhow::bail!("Windows service management is only available on Windows");
    }

    #[cfg(target_os = "windows")]
    {
        println!("Smart Tree Daemon Status (Windows)");
        println!("──────────────────────────────────");
        let _ = run_command("sc.exe", &["query", WINDOWS_SERVICE_NAME]);
        Ok(())
    }
}

fn windows_logs() -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        anyhow::bail!("Windows service management is only available on Windows");
    }

    #[cfg(target_os = "windows")]
    {
        let log_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\ProgramData"))
            .join("SmartTree")
            .join("daemon.log");

        if log_path.exists() {
            println!("Showing logs from {}", log_path.display());
            run_command(
                "powershell",
                &[
                    "-Command",
                    &format!("Get-Content -Path '{}' -Tail 50 -Wait", log_path.display()),
                ],
            )?;
        } else {
            println!("Log file not found at {}", log_path.display());
            println!("Try: Get-EventLog -LogName Application -Source SmartTreeDaemon -Newest 50");
        }
        Ok(())
    }
}

// =============================================================================
// SHARED UTILITIES
// =============================================================================

/// Atomically stage and replace an executable so upgrading a running binary
/// on Linux never fails with ETXTBSY ("Text file busy").
/// Returns `Ok(true)` if the binary was copied, `Ok(false)` if `src` and `dst` are already the same file.
fn atomic_install_executable(src: &Path, dst: &Path) -> Result<bool> {
    let src_canon = src.canonicalize().unwrap_or_else(|_| src.to_path_buf());
    if let Ok(dst_canon) = dst.canonicalize() {
        if src_canon == dst_canon {
            return Ok(false);
        }
    }

    let parent = dst
        .parent()
        .context("Destination path has no parent directory")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create directory {}", parent.display()))?;

    let file_name = dst.file_name().and_then(|n| n.to_str()).unwrap_or("st");
    let stage_path = parent.join(format!(".{}.install.{}.tmp", file_name, std::process::id()));

    let copy_result = (|| -> Result<()> {
        fs::copy(&src_canon, &stage_path).with_context(|| {
            format!(
                "Failed to stage binary from {} to {}",
                src_canon.display(),
                stage_path.display()
            )
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&stage_path, fs::Permissions::from_mode(0o755))?;
        }

        if let Ok(f) = fs::File::open(&stage_path) {
            let _ = f.sync_all();
        }

        fs::rename(&stage_path, dst).with_context(|| {
            format!(
                "Failed to atomically replace {} with {}",
                dst.display(),
                stage_path.display()
            )
        })?;
        Ok(())
    })();

    if copy_result.is_err() {
        let _ = fs::remove_file(&stage_path);
    }
    copy_result?;
    Ok(true)
}

/// Canonical root installation directory on Unix systems.
pub const CANONICAL_UNIX_BIN_DIR: &str = "/usr/local/bin";

/// Canonical root `st` executable path on Unix systems.
pub const CANONICAL_UNIX_ST_BIN: &str = "/usr/local/bin/st";

/// Managed Smart Tree binary names that belong exclusively in the canonical root bin directory.
pub const MANAGED_ST_BINARIES: &[&str] = &["st", "std", "m8", "n8x", "mq", "smart-tree"];

/// Return the canonical system-wide `st` executable path for the current platform.
pub fn canonical_st_binary_path() -> PathBuf {
    #[cfg(unix)]
    {
        PathBuf::from(CANONICAL_UNIX_ST_BIN)
    }
    #[cfg(windows)]
    {
        let prog = env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
        PathBuf::from(prog).join("SmartTree").join("st.exe")
    }
}

/// Collect user home directories (current user, `SUDO_USER`, and `/home/*` or `/Users/*`)
/// so root/sudo installations can clean up per-user duplicate binaries and agent configs.
fn collect_candidate_home_dirs() -> Vec<PathBuf> {
    let mut homes = Vec::new();
    let mut push_unique = |p: PathBuf| {
        if !p.as_os_str().is_empty() && !homes.contains(&p) {
            homes.push(p);
        }
    };

    if let Some(h) = dirs::home_dir() {
        push_unique(h);
    }
    if let Ok(sudo_user) = env::var("SUDO_USER") {
        let trimmed = sudo_user.trim();
        if !trimmed.is_empty() && trimmed != "root" {
            let linux_home = PathBuf::from("/home").join(trimmed);
            if linux_home.is_dir() {
                push_unique(linux_home);
            }
            let mac_home = PathBuf::from("/Users").join(trimmed);
            if mac_home.is_dir() {
                push_unique(mac_home);
            }
        }
    }

    for root in ["/home", "/Users"] {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    push_unique(p);
                }
            }
        }
    }

    homes
}

/// Replace `stale` with `canonical` only when `stale` appears as a complete path token
/// (i.e. not preceded or followed by path/identifier characters like alphanumeric, `/`, `.`, `_`, `-`).
/// This prevents `"/bin/st"` from matching as a suffix inside `"/usr/local/bin/st"` or `"/home/user/.local/bin/st"`.
fn replace_st_path_tokens(content: &str, stale: &str, canonical: &str) -> String {
    if stale.is_empty() || stale == canonical || !content.contains(stale) {
        return content.to_string();
    }
    let is_path_char =
        |c: char| c.is_ascii_alphanumeric() || matches!(c, '/' | '.' | '_' | '-' | '~');

    let mut out = String::with_capacity(content.len());
    let mut rest = content;
    let mut prefix_last_char: Option<char> = None;

    while let Some(pos) = rest.find(stale) {
        let before = &rest[..pos];
        if let Some(ch) = before.chars().last() {
            prefix_last_char = Some(ch);
        }
        let after = &rest[pos + stale.len()..];
        let next_char = after.chars().next();

        let prev_is_path = prefix_last_char.map(is_path_char).unwrap_or(false);
        let next_is_path = next_char.map(is_path_char).unwrap_or(false);

        out.push_str(before);
        if !prev_is_path && !next_is_path {
            out.push_str(canonical);
            prefix_last_char = canonical.chars().last();
        } else {
            out.push_str(stale);
            prefix_last_char = stale.chars().last();
        }
        rest = after;
    }
    out.push_str(rest);
    out
}

/// Rewrite any agent MCP/hook configuration files (`mcp_config.json`, `hooks.json`,
/// `opencode.json`, `crush.json`, `.claude/settings.json`) that reference non-canonical
/// `st` paths (`~/.local/bin/st`, `~/.cargo/bin/st`, `~/bin/st`, `/usr/bin/st`, `/opt/homebrew/bin/st`)
/// so they point to `canonical_st` (`/usr/local/bin/st`).
pub fn rewrite_agent_configs_to_canonical_st(canonical_st: &Path) -> Vec<PathBuf> {
    let canonical_str = canonical_st.to_string_lossy();
    let homes = collect_candidate_home_dirs();
    let mut updated_files = Vec::new();

    let rel_configs = [
        ".gemini/config/mcp_config.json",
        ".gemini/antigravity/mcp_config.json",
        ".gemini/mcp_config.json",
        ".gemini/config/hooks.json",
        ".agents/hooks.json",
        ".claude/settings.json",
        ".claude/hooks.json",
        ".config/opencode/opencode.json",
        ".config/crush/crush.json",
        ".config/Claude/claude_desktop_config.json",
        "Library/Application Support/Claude/claude_desktop_config.json",
    ];

    let mut stale_paths = vec![
        "/usr/local/usr/local/bin/st".to_string(),
        "/opt/homebrew/bin/st".to_string(),
        "/usr/bin/st".to_string(),
        "/bin/st".to_string(),
    ];
    for home in &homes {
        for sub in [
            ".local/usr/local/bin/st",
            ".cargo/usr/local/bin/st",
            ".local/bin/st",
            ".cargo/bin/st",
            "bin/st",
        ] {
            stale_paths.push(home.join(sub).to_string_lossy().to_string());
        }
    }
    stale_paths.sort_by_key(|b| std::cmp::Reverse(b.len()));
    stale_paths.dedup();

    for home in &homes {
        for rel in &rel_configs {
            let cfg_path = home.join(rel);
            if !cfg_path.is_file() {
                continue;
            }
            let Ok(content) = fs::read_to_string(&cfg_path) else {
                continue;
            };
            let mut new_content = content.clone();
            for stale in &stale_paths {
                new_content = replace_st_path_tokens(&new_content, stale, canonical_str.as_ref());
            }
            if new_content != content && fs::write(&cfg_path, new_content).is_ok() {
                println!(
                    "  🔧 Updated agent config to canonical root binary: {}",
                    cfg_path.display()
                );
                updated_files.push(cfg_path);
            }
        }
    }

    updated_files
}

/// Remove duplicate Smart Tree binaries (`st`, `std`, `m8`, `n8x`, `mq`, `smart-tree`)
/// from the specified `candidate_dirs` while strictly preserving `canonical_dir`.
pub fn purge_duplicate_st_binaries_in_dirs(
    canonical_dir: &Path,
    candidate_dirs: &[PathBuf],
    allow_sudo: bool,
) -> Vec<PathBuf> {
    let canonical_dir_resolved = canonical_dir
        .canonicalize()
        .unwrap_or_else(|_| canonical_dir.to_path_buf());
    let macos_helper_dir = PathBuf::from("/Library/PrivilegedHelperTools/is.8b.smart-tree");
    let macos_helper_resolved = macos_helper_dir
        .canonicalize()
        .unwrap_or(macos_helper_dir.clone());

    let mut seen_dirs = Vec::new();
    let mut removed = Vec::new();

    // Also retire legacy `std` binary inside `canonical_dir` (`/usr/local/bin/std`)
    let legacy_canonical_std = canonical_dir.join("std");
    if legacy_canonical_std.exists() || fs::symlink_metadata(&legacy_canonical_std).is_ok() {
        let unlink_ok = match fs::remove_file(&legacy_canonical_std) {
            Ok(()) => true,
            Err(_) if allow_sudo && !is_elevated() => Command::new("sudo")
                .args(["rm", "-f", legacy_canonical_std.to_string_lossy().as_ref()])
                .status()
                .map(|s| s.success())
                .unwrap_or(false),
            Err(_) => false,
        };
        if unlink_ok {
            println!(
                "  🧹 Retired legacy binary: {}",
                legacy_canonical_std.display()
            );
            removed.push(legacy_canonical_std);
        }
    }

    for dir in candidate_dirs {
        if !dir.exists() {
            continue;
        }
        let dir_resolved = dir.canonicalize().unwrap_or_else(|_| dir.clone());
        // Never purge inside the canonical root directory or macOS PrivilegedHelperTools
        if dir_resolved == canonical_dir_resolved || dir_resolved == macos_helper_resolved {
            continue;
        }
        if seen_dirs.contains(&dir_resolved) {
            continue;
        }
        seen_dirs.push(dir_resolved);

        for bin_name in MANAGED_ST_BINARIES {
            let dup_path = dir.join(bin_name);
            let Ok(meta) = fs::symlink_metadata(&dup_path) else {
                continue;
            };
            if meta.is_dir() {
                continue;
            }
            let canonical_bin = canonical_dir.join(bin_name);
            if dup_path == canonical_bin {
                continue;
            }

            // If the canonical root binary does not exist yet and this duplicate is a real file,
            // do not delete the last surviving `st` binary before `/usr/local/bin/st` exists.
            if *bin_name == "st" && !canonical_bin.exists() {
                continue;
            }

            let unlink_ok = match fs::remove_file(&dup_path) {
                Ok(()) => true,
                Err(_) if allow_sudo && !is_elevated() => Command::new("sudo")
                    .args(["rm", "-f", dup_path.to_string_lossy().as_ref()])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false),
                Err(_) => false,
            };

            if unlink_ok {
                println!("  🧹 Removed duplicate binary: {}", dup_path.display());
                removed.push(dup_path);
            }
        }
    }

    removed
}

/// Scan standard system & user binary directories (`/usr/bin`, `~/.local/bin`, `~/.cargo/bin`,
/// `~/bin`, `/opt/homebrew/bin`, and `PATH`) and remove duplicate copies of Smart Tree binaries
/// so `/usr/local/bin` is always the single authoritative root installation.
pub fn purge_duplicate_st_binaries(canonical_dir: &Path, allow_sudo: bool) -> Vec<PathBuf> {
    let mut candidate_dirs = vec![
        PathBuf::from("/usr/bin"),
        PathBuf::from("/bin"),
        PathBuf::from("/opt/homebrew/bin"),
    ];

    for home in collect_candidate_home_dirs() {
        candidate_dirs.push(home.join(".local/bin"));
        candidate_dirs.push(home.join(".cargo/bin"));
        candidate_dirs.push(home.join("bin"));
    }

    if let Some(path_os) = env::var_os("PATH") {
        for entry in env::split_paths(&path_os) {
            if !entry.as_os_str().is_empty() {
                candidate_dirs.push(entry);
            }
        }
    }

    purge_duplicate_st_binaries_in_dirs(canonical_dir, &candidate_dirs, allow_sudo)
}

/// Copy the current binary (and any sibling companion binaries) to the canonical root install path,
/// rewrite agent configs to point to the canonical root binary, and purge duplicate copies.
/// Returns the path where the binary was installed.
fn install_binary() -> Result<PathBuf> {
    let current_exe = env::current_exe().context("Failed to get current executable path")?;

    #[cfg(unix)]
    let install_path = {
        let usr_local_dir = PathBuf::from(CANONICAL_UNIX_BIN_DIR);
        let usr_local_st = usr_local_dir.join("st");
        let helper_dir = PathBuf::from("/Library/PrivilegedHelperTools/is.8b.smart-tree");
        if cfg!(target_os = "macos") && helper_dir.is_dir() {
            let _ = atomic_install_executable(&current_exe, &usr_local_st);
            helper_dir.join("st")
        } else {
            usr_local_st
        }
    };

    #[cfg(windows)]
    let install_path = {
        let prog = env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
        let dir = PathBuf::from(prog).join("SmartTree");
        fs::create_dir_all(&dir)?;
        dir.join("st.exe")
    };

    if atomic_install_executable(&current_exe, &install_path)? {
        println!("  Installed binary to {}.", install_path.display());
    } else {
        println!("  Binary already at {}.", install_path.display());
    }

    // Also install sibling companion binaries (m8, n8x, mq) into /usr/local/bin if present
    #[cfg(unix)]
    if let Some(src_dir) = current_exe.parent() {
        let canonical_dir = Path::new(CANONICAL_UNIX_BIN_DIR);
        for companion in ["m8", "n8x", "mq"] {
            let src_companion = src_dir.join(companion);
            let dst_companion = canonical_dir.join(companion);
            if src_companion.is_file() {
                if let Ok(true) = atomic_install_executable(&src_companion, &dst_companion) {
                    println!(
                        "  Installed companion binary to {}.",
                        dst_companion.display()
                    );
                }
            }
        }
    }

    // Rewrite agent configs and remove duplicate binaries in ~/.local/bin, ~/.cargo/bin, /usr/bin, etc.
    #[cfg(unix)]
    {
        let canonical_dir = Path::new(CANONICAL_UNIX_BIN_DIR);
        let canonical_st = canonical_dir.join("st");
        rewrite_agent_configs_to_canonical_st(&canonical_st);
        purge_duplicate_st_binaries(canonical_dir, false);
    }

    // Keep .auto-update worker synchronized and clear any stale rollback state on manual install
    #[cfg(unix)]
    if let Some(parent) = install_path.parent() {
        let auto_update_dir = parent.join(".auto-update");
        if auto_update_dir.is_dir() {
            let worker_path = auto_update_dir.join("worker");
            if worker_path.is_file() {
                let _ = atomic_install_executable(&install_path, &worker_path);
            }
            let txn_dir = auto_update_dir.join("transaction");
            if txn_dir.exists() {
                let _ = fs::remove_dir_all(&txn_dir);
            }
            let rejected = auto_update_dir.join("rejected-digest");
            if rejected.exists() {
                let _ = fs::remove_file(&rejected);
            }
        }
    }

    Ok(install_path)
}

/// Compute SHA256 hash of a file for integrity verification.
fn compute_file_hash(path: &std::path::Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut file =
        fs::File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Run a command, inheriting stdio.
fn run_command(command: &str, args: &[&str]) -> Result<()> {
    info!("Running: {} {}", command, args.join(" "));
    let status = Command::new(command)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("Failed to execute: {}", command))?;

    if !status.success() {
        error!("Command failed: {} (exit: {})", command, status);
        anyhow::bail!("Command failed: {} {}", command, args.join(" "));
    }
    Ok(())
}

/// Find a binary on PATH.
fn which_binary(name: &str) -> Option<PathBuf> {
    which::which(name).ok()
}

/// Print the post-install success banner.
fn print_install_success(status_cmd: &str, logs_cmd: &str) {
    let _ = crate::completions::maintain_service_completions(true);
    println!();
    println!("Smart Tree daemon installed and running (same server as `st up`)!");
    println!("  Shell Completions: Auto-maintained (Fish, Zsh, Bash + live folder/MEM|8 cache)");
    println!();
    println!(
        "  Dashboard:  http://localhost:{}/ (automagic, no local token)",
        DAEMON_PORT
    );
    println!(
        "  MCP (HTTP): http://localhost:{}/mcp + /mcp/sse",
        DAEMON_PORT
    );
    println!("  MCP (stdio): st --mcp");
    println!(
        "  Token:      ~/.st/daemon.token (needed only when externally bound; `st --show-token`)"
    );
    println!();
    println!("  Status:     {}", status_cmd);
    println!("  Logs:       {}", logs_cmd);
    println!("  Stop:       st service stop");
    println!("  Uninstall:  st service uninstall");
}

// =============================================================================
// GPG SIGNATURE VERIFICATION
// =============================================================================

/// 8bit-wraith's official GPG key fingerprint for signed releases
pub const OFFICIAL_GPG_FINGERPRINT: &str = "wraith@8b.is";

/// Check if this is an officially signed build
pub fn verify_gpg_signature() -> SignatureStatus {
    let sig_path = PathBuf::from("/usr/local/bin/st.sig");
    let binary_path = PathBuf::from("/usr/local/bin/st");

    if !sig_path.exists() {
        return SignatureStatus::Unsigned;
    }

    let output = Command::new("gpg")
        .args([
            "--verify",
            sig_path.to_string_lossy().as_ref(),
            binary_path.to_string_lossy().as_ref(),
        ])
        .output();

    match output {
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            if result.status.success() {
                if stderr.contains(OFFICIAL_GPG_FINGERPRINT) {
                    SignatureStatus::OfficialBuild
                } else {
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
    OfficialBuild,
    CommunityBuild(String),
    Unsigned,
    TamperedOrInvalid,
    GpgNotAvailable,
}

/// Print signature verification banner on first run
pub fn print_signature_banner() {
    let first_run_marker = dirs::data_dir()
        .map(|d| d.join("smart-tree").join(".first_run_complete"))
        .unwrap_or_else(|| PathBuf::from("/tmp/.st_first_run"));

    if first_run_marker.exists() {
        return;
    }

    let status = verify_gpg_signature();

    eprintln!();
    match status {
        SignatureStatus::OfficialBuild => {
            eprintln!("  OFFICIAL BUILD - Signed by 8bit-wraith (wraith@8b.is)");
            eprintln!("  This binary is cryptographically verified as an authentic release.");
        }
        SignatureStatus::CommunityBuild(ref signer) => {
            eprintln!("  COMMUNITY BUILD - Signed but NOT by the official 8b.is key.");
            eprintln!("  Signer: {}", &signer[..signer.len().min(70)]);
            eprintln!("  Verify you trust this signer before proceeding.");
        }
        SignatureStatus::Unsigned => {
            eprintln!("  UNSIGNED BUILD - No GPG signature found.");
            eprintln!("  This is normal for dev builds or self-compiled versions.");
            eprintln!("  Official releases: https://i1.is/smart-tree");
        }
        SignatureStatus::TamperedOrInvalid => {
            eprintln!("  WARNING: SIGNATURE VERIFICATION FAILED");
            eprintln!("  The binary signature does NOT match the file contents!");
            eprintln!("  Re-download from https://i1.is/smart-tree");
        }
        SignatureStatus::GpgNotAvailable => {
            eprintln!("  GPG not available - signature verification skipped.");
            eprintln!("  Install gnupg to enable verification of official builds.");
        }
    }
    eprintln!();

    if let Some(parent) = first_run_marker.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&first_run_marker, "shown");
}

// =============================================================================
// GUARDIAN — kept for backward compatibility
// =============================================================================

/// Verify the installed binary hasn't been tampered with
pub fn guardian_verify_integrity() -> Result<bool> {
    let installed_path = PathBuf::from("/usr/local/bin/st");

    if !installed_path.exists() {
        warn!("Binary not found at /usr/local/bin/st");
        return Ok(false);
    }

    let installed_hash = compute_file_hash(&installed_path)?;
    let hash_file = PathBuf::from("/var/lib/smart-tree/guardian.sha256");

    if hash_file.exists() {
        let stored_hash = fs::read_to_string(&hash_file)?.trim().to_string();
        if installed_hash != stored_hash {
            error!("INTEGRITY VIOLATION: Binary has been modified!");
            error!("  Expected: {}", stored_hash);
            error!("  Found:    {}", installed_hash);
            return Ok(false);
        }
        info!("Binary integrity verified");
        Ok(true)
    } else {
        warn!("No stored hash found - cannot verify integrity");
        Ok(true)
    }
}

/// Install Smart Tree Guardian as a root daemon
pub fn guardian_install() -> Result<()> {
    println!("Smart Tree Guardian - System-wide AI Protection Daemon");
    println!();

    if escalate_privileges(&["--guardian-install"])? {
        return Ok(());
    }

    info!("Installing Guardian daemon...");

    // 1. Install binary
    let target_bin = install_binary()?;

    // 2. Create state directory & store hash
    fs::create_dir_all("/var/lib/smart-tree")?;
    let hash = compute_file_hash(&target_bin)?;
    fs::write("/var/lib/smart-tree/guardian.sha256", &hash)?;

    // 3. Write service file (platform-specific)
    match Platform::current() {
        Platform::Linux => {
            let service_content = include_str!("../systemd/smart-tree-guardian.service");
            let service_path = format!("{}/{}", SYSTEMD_SYSTEM_PATH, "smart-tree-guardian.service");
            fs::write(&service_path, service_content)?;
            run_command("systemctl", &["daemon-reload"])?;
            run_command("systemctl", &["enable", "smart-tree-guardian.service"])?;
            run_command("systemctl", &["restart", "smart-tree-guardian.service"])?;
        }
        Platform::MacOS => {
            let plist = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>is.8b.smart-tree-guardian</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>--guardian-daemon</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>WorkingDirectory</key>
    <string>/var/lib/smart-tree</string>
    <key>StandardOutPath</key>
    <string>/var/log/smart-tree/guardian.log</string>
    <key>StandardErrorPath</key>
    <string>/var/log/smart-tree/guardian.log</string>
</dict>
</plist>
"#,
                target_bin.display()
            );
            let plist_path = "/Library/LaunchDaemons/is.8b.smart-tree-guardian.plist";
            fs::create_dir_all("/var/log/smart-tree")?;
            fs::write(plist_path, &plist)?;
            run_command("chown", &["root:wheel", plist_path])?;
            run_command("chmod", &["644", plist_path])?;
            let _ = Command::new("launchctl")
                .args(["bootstrap", "system", plist_path])
                .status();
        }
        _ => {
            anyhow::bail!("Guardian is currently supported on Linux and macOS only");
        }
    }

    println!();
    println!("Guardian installed and running!");
    println!("  Status: st --guardian-status");
    Ok(())
}

/// Uninstall Smart Tree Guardian
pub fn guardian_uninstall() -> Result<()> {
    if escalate_privileges(&["--guardian-uninstall"])? {
        return Ok(());
    }

    match Platform::current() {
        Platform::Linux => {
            let _ = run_command("systemctl", &["stop", "smart-tree-guardian.service"]);
            let _ = run_command("systemctl", &["disable", "smart-tree-guardian.service"]);
            let path = format!("{}/smart-tree-guardian.service", SYSTEMD_SYSTEM_PATH);
            if PathBuf::from(&path).exists() {
                fs::remove_file(&path)?;
            }
            run_command("systemctl", &["daemon-reload"])?;
        }
        Platform::MacOS => {
            let plist = "/Library/LaunchDaemons/is.8b.smart-tree-guardian.plist";
            let _ = Command::new("launchctl")
                .args(["bootout", "system", plist])
                .status();
            if PathBuf::from(plist).exists() {
                fs::remove_file(plist)?;
            }
        }
        _ => {}
    }

    println!("Guardian uninstalled.");
    Ok(())
}

/// Show Guardian daemon status
pub fn guardian_status() -> Result<()> {
    println!("Smart Tree Guardian Status");
    println!("─────────────────────────");

    match Platform::current() {
        Platform::Linux => {
            let service_path = format!("{}/smart-tree-guardian.service", SYSTEMD_SYSTEM_PATH);
            if !PathBuf::from(&service_path).exists() {
                println!("Status: NOT INSTALLED");
                println!("Install: st --guardian-install");
                return Ok(());
            }
            let _ = run_command(
                "systemctl",
                &["status", "smart-tree-guardian.service", "--no-pager"],
            );
        }
        Platform::MacOS => {
            let plist = "/Library/LaunchDaemons/is.8b.smart-tree-guardian.plist";
            if !PathBuf::from(plist).exists() {
                println!("Status: NOT INSTALLED");
                println!("Install: st --guardian-install");
                return Ok(());
            }
            let output = Command::new("launchctl")
                .args(["print", "system/is.8b.smart-tree-guardian"])
                .output();
            match output {
                Ok(out) if out.status.success() => {
                    println!("Status: RUNNING");
                    let text = String::from_utf8_lossy(&out.stdout);
                    for line in text.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("pid =") || trimmed.starts_with("state =") {
                            println!("  {}", trimmed);
                        }
                    }
                }
                _ => println!("Status: LOADED (check sudo launchctl list | grep guardian)"),
            }
        }
        _ => {
            println!("Guardian status not available on this platform.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_atomic_install_executable_replaces_existing_file() -> Result<()> {
        let dir = TempDir::new()?;
        let src = dir.path().join("st-new");
        let dst = dir.path().join("st");

        fs::write(&src, b"new-binary-v10.7.0")?;
        fs::write(&dst, b"old-binary-v10.5.0")?;

        let copied = atomic_install_executable(&src, &dst)?;
        assert!(copied);
        assert_eq!(fs::read(&dst)?, b"new-binary-v10.7.0");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&dst)?.permissions().mode() & 0o777;
            assert_eq!(mode, 0o755);
        }

        // Installing the same file onto itself is a no-op
        let copied_again = atomic_install_executable(&dst, &dst)?;
        assert!(!copied_again);
        Ok(())
    }

    #[test]
    fn test_generate_systemd_unit_uses_isolated_state_dir() {
        let unit = generate_systemd_unit(Path::new("/usr/local/bin/st"));
        assert!(unit.contains("ExecStart=/usr/local/bin/st --http-daemon"));
        assert!(unit.contains("Environment=HOME=/var/lib/smart-tree"));
        assert!(unit.contains("Environment=ST_TOKEN_PATH=/var/lib/smart-tree/daemon.token"));
        assert!(unit.contains("WorkingDirectory=/var/lib/smart-tree"));
    }

    #[test]
    fn test_purge_duplicate_st_binaries_preserves_canonical_and_removes_duplicates() -> Result<()> {
        let root = TempDir::new()?;
        let canonical = root.path().join("usr_local_bin");
        let user_local = root.path().join("home_local_bin");
        let cargo_bin = root.path().join("home_cargo_bin");
        let usr_bin = root.path().join("usr_bin");

        fs::create_dir_all(&canonical)?;
        fs::create_dir_all(&user_local)?;
        fs::create_dir_all(&cargo_bin)?;
        fs::create_dir_all(&usr_bin)?;

        fs::write(canonical.join("st"), b"canonical-st")?;
        fs::write(canonical.join("m8"), b"canonical-m8")?;

        fs::write(user_local.join("st"), b"dup-local-st")?;
        fs::write(user_local.join("m8"), b"dup-local-m8")?;
        fs::write(user_local.join("other-tool"), b"keep-me")?;

        fs::write(cargo_bin.join("st"), b"dup-cargo-st")?;
        fs::write(cargo_bin.join("n8x"), b"dup-cargo-n8x")?;

        fs::write(usr_bin.join("st"), b"dup-usr-st")?;

        let candidates = vec![
            canonical.clone(),
            user_local.clone(),
            cargo_bin.clone(),
            usr_bin.clone(),
        ];

        let removed = purge_duplicate_st_binaries_in_dirs(&canonical, &candidates, false);
        assert_eq!(removed.len(), 5);

        // Canonical binaries must remain untouched
        assert!(canonical.join("st").exists());
        assert!(canonical.join("m8").exists());

        // Duplicates must be gone, unrelated binaries preserved
        assert!(!user_local.join("st").exists());
        assert!(!user_local.join("m8").exists());
        assert!(user_local.join("other-tool").exists());
        assert!(!cargo_bin.join("st").exists());
        assert!(!cargo_bin.join("n8x").exists());
        assert!(!usr_bin.join("st").exists());
        Ok(())
    }

    #[test]
    fn test_replace_st_path_tokens_is_idempotent_and_self_heals() {
        let canonical = "/usr/local/bin/st";
        let sample = r#"{
  "command": "/home/hue/.local/bin/st --agy-hook PreToolUse",
  "corrupted": "/home/hue/.local/usr/local/bin/st --mcp",
  "already_ok": "/usr/local/bin/st --agy-hook PostToolUse",
  "bin_st": "/bin/st --version"
}"#;
        let stale_list = [
            "/home/hue/.local/usr/local/bin/st",
            "/home/hue/.local/bin/st",
            "/usr/bin/st",
            "/bin/st",
        ];
        let mut out = sample.to_string();
        for s in stale_list {
            out = replace_st_path_tokens(&out, s, canonical);
        }
        assert!(out.contains(r#""command": "/usr/local/bin/st --agy-hook PreToolUse""#));
        assert!(out.contains(r#""corrupted": "/usr/local/bin/st --mcp""#));
        assert!(out.contains(r#""already_ok": "/usr/local/bin/st --agy-hook PostToolUse""#));
        assert!(out.contains(r#""bin_st": "/usr/local/bin/st --version""#));
        assert!(!out.contains(".local/usr/local/bin/st"));
        assert!(!out.contains("/usr/local/usr/local/bin/st"));

        // Running a second time must be 100% idempotent
        let mut second = out.clone();
        for s in stale_list {
            second = replace_st_path_tokens(&second, s, canonical);
        }
        assert_eq!(out, second);
    }
}
