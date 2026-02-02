//! Collaboration WebSocket Handler for Dashboard
//!
//! Handles real-time collaboration between humans and AIs
//! through the web dashboard.

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};

use crate::collaboration::{CollabMessage, Participant, ParticipantType, SharedCollabHub};

/// Request to join collaboration
#[derive(Debug, Deserialize)]
pub struct JoinRequest {
    pub name: String,
    #[serde(default)]
    pub participant_type: String,
}

/// WebSocket message from client
#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ClientMessage {
    Join { name: String, participant_type: Option<String> },
    Chat { message: String },
    Status { status: Option<String>, working_on: Option<String> },
    HotTub,
    Ping,
}

/// WebSocket message to client
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Welcome { participant_id: String, name: String },
    Error { message: String },
    Collab(CollabMessage),
    Pong,
}

fn parse_participant_type(s: &str) -> ParticipantType {
    match s.to_lowercase().as_str() {
        "human" | "user" => ParticipantType::Human,
        "claude" => ParticipantType::Claude,
        "omni" => ParticipantType::Omni,
        "grok" => ParticipantType::Grok,
        "gemini" => ParticipantType::Gemini,
        "local" | "local_llm" => ParticipantType::LocalLlm,
        "smart_tree" | "st" => ParticipantType::SmartTree,
        _ => ParticipantType::Unknown,
    }
}

/// WebSocket handler for collaboration
pub async fn collab_handler(
    ws: WebSocketUpgrade,
    State(hub): State<SharedCollabHub>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_collab_socket(socket, hub))
}

async fn handle_collab_socket(socket: WebSocket, hub: SharedCollabHub) {
    let (mut sender, mut receiver) = socket.split();

    // Wait for join message
    let participant_id = loop {
        match receiver.next().await {
            Some(Ok(Message::Text(text))) => {
                if let Ok(ClientMessage::Join { name, participant_type }) = serde_json::from_str(&text) {
                    let ptype = participant_type
                        .map(|s| parse_participant_type(&s))
                        .unwrap_or(ParticipantType::Unknown);
                    let participant = Participant::new(name.clone(), ptype);
                    let id = hub.write().await.join(participant);

                    // Send welcome
                    let welcome = ServerMessage::Welcome {
                        participant_id: id.clone(),
                        name,
                    };
                    if let Ok(json) = serde_json::to_string(&welcome) {
                        let _ = sender.send(Message::Text(json)).await;
                    }

                    break id;
                } else {
                    let err = ServerMessage::Error {
                        message: "First message must be a join request".to_string(),
                    };
                    if let Ok(json) = serde_json::to_string(&err) {
                        let _ = sender.send(Message::Text(json)).await;
                    }
                }
            }
            Some(Ok(Message::Close(_))) | None => return,
            _ => continue,
        }
    };

    // Subscribe to broadcast
    let mut broadcast_rx = hub.read().await.subscribe();

    // Spawn task to forward broadcasts to client
    let mut send_task = {
        let participant_id = participant_id.clone();
        tokio::spawn(async move {
            while let Ok(msg) = broadcast_rx.recv().await {
                let server_msg = ServerMessage::Collab(msg);
                if let Ok(json) = serde_json::to_string(&server_msg) {
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
            participant_id
        })
    };

    // Handle incoming messages
    let hub_clone = hub.clone();
    let participant_id_clone = participant_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        match client_msg {
                            ClientMessage::Chat { message } => {
                                hub_clone.read().await.chat(&participant_id_clone, message);
                            }
                            ClientMessage::Status { status, working_on } => {
                                hub_clone.write().await.update_status(
                                    &participant_id_clone,
                                    status,
                                    working_on,
                                );
                            }
                            ClientMessage::HotTub => {
                                hub_clone.write().await.toggle_hot_tub(&participant_id_clone);
                            }
                            ClientMessage::Ping => {
                                // Handled by send_task
                            }
                            ClientMessage::Join { .. } => {
                                // Already joined
                            }
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
        participant_id_clone
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        }
        _ = (&mut recv_task) => {
            send_task.abort();
        }
    }

    // Clean up - remove participant
    hub.write().await.leave(&participant_id);
}
