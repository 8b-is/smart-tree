//! PTY management using portable-pty

use anyhow::{Context, Result};
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Handle to a PTY session
pub struct PtyHandle {
    pub id: String,
    pub master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    pub reader: Arc<Mutex<Box<dyn Read + Send>>>,
    pub writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pub cols: u16,
    pub rows: u16,
}

impl std::fmt::Debug for PtyHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PtyHandle")
            .field("id", &self.id)
            .field("cols", &self.cols)
            .field("rows", &self.rows)
            .finish_non_exhaustive()
    }
}

/// Spawn a new PTY shell
pub fn spawn_shell(cols: u16, rows: u16) -> Result<PtyHandle> {
    let pty_system = native_pty_system();

    let pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .context("Failed to open PTY")?;

    // Determine shell
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());

    let mut cmd = CommandBuilder::new(&shell);
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");

    // Spawn the shell
    let _child = pair.slave.spawn_command(cmd).context("Failed to spawn shell")?;

    // Get reader and writer
    let reader = pair.master.try_clone_reader().context("Failed to clone reader")?;
    let writer = pair.master.take_writer().context("Failed to take writer")?;

    let id = uuid::Uuid::new_v4().to_string();

    Ok(PtyHandle {
        id,
        master: Arc::new(Mutex::new(pair.master)),
        reader: Arc::new(Mutex::new(reader)),
        writer: Arc::new(Mutex::new(writer)),
        cols,
        rows,
    })
}

impl PtyHandle {
    /// Write data to the PTY
    pub async fn write(&self, data: &[u8]) -> Result<()> {
        let mut writer = self.writer.lock().await;
        writer.write_all(data).context("Failed to write to PTY")?;
        writer.flush().context("Failed to flush PTY")?;
        Ok(())
    }

    /// Resize the PTY
    pub async fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        let master = self.master.lock().await;
        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Failed to resize PTY")?;
        Ok(())
    }

    /// Read available data from the PTY (non-blocking read attempt)
    pub async fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let mut reader = self.reader.lock().await;
        // Note: This is blocking, we'll handle it with tokio::task::spawn_blocking in websocket
        match reader.read(buf) {
            Ok(n) => Ok(n),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(0),
            Err(e) => Err(e.into()),
        }
    }
}
