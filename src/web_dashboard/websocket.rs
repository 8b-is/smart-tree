//! WebSocket handlers for terminal and state updates

use super::{pty, SharedState, TerminalMessage};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;

/// WebSocket handler for terminal connections
pub async fn terminal_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_terminal(socket, state))
}

async fn handle_terminal(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();

    // Spawn PTY
    let pty_handle = match pty::spawn_shell(80, 24) {
        Ok(h) => Arc::new(h),
        Err(e) => {
            let error_msg = TerminalMessage::Error {
                message: format!("Failed to spawn shell: {}", e),
            };
            let _ = sender
                .send(Message::Text(serde_json::to_string(&error_msg).unwrap()))
                .await;
            return;
        }
    };

    // Update connection count and send welcome message
    {
        let mut s = state.write().await;
        s.connections += 1;
        let welcome_msg = TerminalMessage::System {
            message: format!("Connected to project: {}", s.cwd.to_string_lossy()),
        };
        if sender
            .send(Message::Text(serde_json::to_string(&welcome_msg).unwrap()))
            .await
            .is_err()
        {
            // Connection closed immediately, bail
            return;
        }
    }

    let pty_for_read = Arc::clone(&pty_handle);
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Spawn task to read from PTY and send to WebSocket
    let read_task = tokio::spawn(async move {
        loop {
            // Use spawn_blocking for the blocking read
            let pty_clone = Arc::clone(&pty_for_read);
            let read_result = tokio::task::spawn_blocking(move || {
                let mut reader = pty_clone.reader.blocking_lock();
                // Create a small buffer for blocking read
                let mut local_buf = [0u8; 4096];
                match std::io::Read::read(&mut *reader, &mut local_buf) {
                    Ok(n) if n > 0 => Some(local_buf[..n].to_vec()),
                    _ => None,
                }
            })
            .await;

            match read_result {
                Ok(Some(data)) => {
                    // Convert to string, lossy for binary data
                    let text = String::from_utf8_lossy(&data).to_string();
                    if tx.send(text).await.is_err() {
                        break;
                    }
                }
                Ok(None) => {
                    // EOF or empty read, small delay and continue
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
                Err(_) => break,
            }
        }
    });

    // Spawn task to forward PTY output to WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            let msg = TerminalMessage::Output { data };
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages from WebSocket
    let pty_for_write = Arc::clone(&pty_handle);
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(terminal_msg) = serde_json::from_str::<TerminalMessage>(&text) {
                    match terminal_msg {
                        TerminalMessage::Input { data } => {
                            if let Err(e) = pty_for_write.write(data.as_bytes()).await {
                                eprintln!("Failed to write to PTY: {}", e);
                                break;
                            }
                        }
                        TerminalMessage::Resize { cols, rows } => {
                            if let Err(e) = pty_for_write.resize(cols, rows).await {
                                eprintln!("Failed to resize PTY: {}", e);
                            }
                        }
                        TerminalMessage::Ping => {
                            // Client ping, could send pong back
                        }
                        _ => {}
                    }
                }
            }
            Ok(Message::Close(_)) => break,
            Err(_) => break,
            _ => {}
        }
    }

    // Cleanup
    read_task.abort();
    send_task.abort();

    {
        let mut s = state.write().await;
        s.connections = s.connections.saturating_sub(1);
    }
}
