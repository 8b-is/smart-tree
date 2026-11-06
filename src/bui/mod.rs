//! BUI (Blender UI) Integration for Smart Tree
//!
//! Provides Blender-style GUI components for the Smart Tree dashboard and CLI.
//! Requires the `bui` feature flag AND cmake installed for C++ backend compilation.
//!
//! Note: Requires cmake to build the C++ backend. Install with:
//! - macOS: `brew install cmake`
//! - Linux: `apt-get install cmake` / `pacman -S cmake`
//! - Windows: `choco install cmake` or download from cmake.org

#[cfg(feature = "bui")]
pub mod app;

#[cfg(feature = "bui")]
pub mod components;

#[cfg(feature = "bui")]
pub mod dashboard;

#[cfg(feature = "bui")]
pub mod theme;

#[cfg(feature = "bui")]
pub use app::BuiApp;

#[cfg(feature = "bui")]
pub use components::*;

#[cfg(feature = "bui")]
pub use dashboard::BuiDashboard;

#[cfg(feature = "bui")]
pub use theme::BuiTheme;

/// Initialize BUI subsystem
#[cfg(feature = "bui")]
pub fn init_bui() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize BUI subsystems
    blenderui::init();
    Ok(())
}

/// Shutdown BUI subsystem
#[cfg(feature = "bui")]
pub fn shutdown_bui() {
    blenderui::shutdown();
}