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

    let status = cmd.status().with_context(|| format!("Failed to execute command: {}", command))?;

    if !status.success() {
        error!("Command failed with status: {}", status);
        anyhow::bail!("Command failed with status: {}", status);
    }
    Ok(())
}

/// Get the project name from the current directory.
fn get_project_name() -> Result<String> {
    let cwd = env::current_dir()?;
    let project_name = cwd.file_name()
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
        error!("Service template not found at '{}'", template_path.display());
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
    info!("Copying '{}' to '{}'", template_path.display(), dest_path.display());
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
        warn!("Service file not found at '{}'. Already uninstalled?", dest_path.display());
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
    let _ = run_command("systemctl", &["--user", "status", &service_instance, "--no-pager"]);
    Ok(())
}

/// Show recent logs for the systemd user service.
pub fn logs() -> Result<()> {
    let project_name = get_project_name()?;
    let service_instance = format!("smart-tree-dashboard@{}.service", project_name);
    info!("Showing logs for service '{}':", service_instance);
    let _ = run_command("journalctl", &["--user", "-u", &service_instance, "-n", "50", "--no-pager", "-f"]);
    Ok(())
}
