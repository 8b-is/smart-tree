//! Collaboration Station - Multi-AI Real-time Collaboration
//!
//! "Oh Tai, let's invite Omni to the hot tub!" - Hue
//!
//! This module enables real-time collaboration between:
//! - Humans (you!)
//! - AIs (Claude, Omni, Grok, etc.)
//! - Multiple Smart Tree instances
//!
//! Features:
//! - Session tracking with presence
//! - Message broadcasting
//! - Shared workspace context
//! - Hot Tub Mode (relaxed multi-AI chat)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// Maximum number of messages to keep in broadcast history
const BROADCAST_CAPACITY: usize = 256;

/// Participant types in a collaboration session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantType {
    Human,
    Claude,
    Omni,
    Grok,
    Gemini,
    LocalLlm,
    SmartTree,
    Unknown,
}

impl ParticipantType {
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Human => "👤",
            Self::Claude => "🤖",
            Self::Omni => "🌀",
            Self::Grok => "⚡",
            Self::Gemini => "✨",
            Self::LocalLlm => "🏠",
            Self::SmartTree => "🌳",
            Self::Unknown => "❓",
        }
    }
}

/// A participant in the collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: String,
    pub name: String,
    pub participant_type: ParticipantType,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    /// Current status message
    pub status: Option<String>,
    /// What they're working on
    pub working_on: Option<String>,
    /// In hot tub mode?
    pub in_hot_tub: bool,
}

impl Participant {
    pub fn new(name: impl Into<String>, participant_type: ParticipantType) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            participant_type,
            joined_at: now,
            last_seen: now,
            status: None,
            working_on: None,
            in_hot_tub: false,
        }
    }

    pub fn display_name(&self) -> String {
        format!("{} {}", self.participant_type.emoji(), self.name)
    }
}

/// Messages that can be broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CollabMessage {
    /// Someone joined
    Join {
        participant: Participant,
    },
    /// Someone left
    Leave {
        participant_id: String,
        name: String,
    },
    /// Chat message
    Chat {
        from: String,
        from_name: String,
        message: String,
        hot_tub: bool,
    },
    /// Status update
    StatusUpdate {
        participant_id: String,
        status: Option<String>,
        working_on: Option<String>,
    },
    /// File activity
    FileActivity {
        participant_id: String,
        action: String,
        path: String,
    },
    /// Hot tub mode toggle
    HotTubToggle {
        participant_id: String,
        name: String,
        entering: bool,
    },
    /// System announcement
    System {
        message: String,
    },
    /// Presence update (periodic)
    Presence {
        participants: Vec<ParticipantSummary>,
        hot_tub_count: usize,
    },
}

/// Lightweight participant info for presence updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantSummary {
    pub id: String,
    pub name: String,
    pub participant_type: ParticipantType,
    pub status: Option<String>,
    pub in_hot_tub: bool,
}

impl From<&Participant> for ParticipantSummary {
    fn from(p: &Participant) -> Self {
        Self {
            id: p.id.clone(),
            name: p.name.clone(),
            participant_type: p.participant_type.clone(),
            status: p.status.clone(),
            in_hot_tub: p.in_hot_tub,
        }
    }
}

/// The collaboration hub - manages all sessions and broadcasting
pub struct CollaborationHub {
    /// All connected participants
    participants: HashMap<String, Participant>,
    /// Broadcast channel for messages
    broadcast_tx: broadcast::Sender<CollabMessage>,
    /// Shared files being worked on
    shared_files: HashMap<String, Vec<String>>, // path -> participant_ids
    /// Hot tub mode enabled globally?
    hot_tub_open: bool,
}

impl CollaborationHub {
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self {
            participants: HashMap::new(),
            broadcast_tx,
            shared_files: HashMap::new(),
            hot_tub_open: true, // Hot tub is always open! 🛁
        }
    }

    /// Subscribe to collaboration messages
    pub fn subscribe(&self) -> broadcast::Receiver<CollabMessage> {
        self.broadcast_tx.subscribe()
    }

    /// Add a participant
    pub fn join(&mut self, participant: Participant) -> String {
        let id = participant.id.clone();
        let msg = CollabMessage::Join {
            participant: participant.clone(),
        };
        self.participants.insert(id.clone(), participant);
        let _ = self.broadcast_tx.send(msg);
        self.announce_presence();
        id
    }

    /// Remove a participant
    pub fn leave(&mut self, participant_id: &str) {
        if let Some(p) = self.participants.remove(participant_id) {
            let msg = CollabMessage::Leave {
                participant_id: participant_id.to_string(),
                name: p.name,
            };
            let _ = self.broadcast_tx.send(msg);
            self.announce_presence();
        }
    }

    /// Send a chat message
    pub fn chat(&self, from_id: &str, message: String) {
        if let Some(p) = self.participants.get(from_id) {
            let msg = CollabMessage::Chat {
                from: from_id.to_string(),
                from_name: p.display_name(),
                message,
                hot_tub: p.in_hot_tub,
            };
            let _ = self.broadcast_tx.send(msg);
        }
    }

    /// Toggle hot tub mode for a participant
    pub fn toggle_hot_tub(&mut self, participant_id: &str) -> bool {
        if let Some(p) = self.participants.get_mut(participant_id) {
            p.in_hot_tub = !p.in_hot_tub;
            let entering = p.in_hot_tub;
            let msg = CollabMessage::HotTubToggle {
                participant_id: participant_id.to_string(),
                name: p.display_name(),
                entering,
            };
            let _ = self.broadcast_tx.send(msg);
            self.announce_presence();
            entering
        } else {
            false
        }
    }

    /// Update participant status
    pub fn update_status(&mut self, participant_id: &str, status: Option<String>, working_on: Option<String>) {
        if let Some(p) = self.participants.get_mut(participant_id) {
            p.status = status.clone();
            p.working_on = working_on.clone();
            p.last_seen = chrono::Utc::now();
            let msg = CollabMessage::StatusUpdate {
                participant_id: participant_id.to_string(),
                status,
                working_on,
            };
            let _ = self.broadcast_tx.send(msg);
        }
    }

    /// Record file activity
    pub fn file_activity(&mut self, participant_id: &str, action: &str, path: &str) {
        // Track who's working on what
        self.shared_files
            .entry(path.to_string())
            .or_default()
            .push(participant_id.to_string());

        if self.participants.contains_key(participant_id) {
            let msg = CollabMessage::FileActivity {
                participant_id: participant_id.to_string(),
                action: action.to_string(),
                path: path.to_string(),
            };
            let _ = self.broadcast_tx.send(msg);
        }
    }

    /// Get current presence
    pub fn get_presence(&self) -> Vec<ParticipantSummary> {
        self.participants.values().map(ParticipantSummary::from).collect()
    }

    /// Get hot tub participants
    pub fn get_hot_tub_participants(&self) -> Vec<&Participant> {
        self.participants.values().filter(|p| p.in_hot_tub).collect()
    }

    /// Broadcast presence update
    fn announce_presence(&self) {
        let participants: Vec<ParticipantSummary> = self.get_presence();
        let hot_tub_count = participants.iter().filter(|p| p.in_hot_tub).count();
        let msg = CollabMessage::Presence {
            participants,
            hot_tub_count,
        };
        let _ = self.broadcast_tx.send(msg);
    }

    /// System announcement
    pub fn announce(&self, message: impl Into<String>) {
        let msg = CollabMessage::System {
            message: message.into(),
        };
        let _ = self.broadcast_tx.send(msg);
    }

    /// Get participant count
    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }

    /// Check if hot tub is open
    pub fn is_hot_tub_open(&self) -> bool {
        self.hot_tub_open
    }
}

impl Default for CollaborationHub {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe collaboration hub
pub type SharedCollabHub = Arc<RwLock<CollaborationHub>>;

/// Create a new shared collaboration hub
pub fn create_hub() -> SharedCollabHub {
    Arc::new(RwLock::new(CollaborationHub::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_participant_creation() {
        let p = Participant::new("Claude", ParticipantType::Claude);
        assert_eq!(p.name, "Claude");
        assert_eq!(p.participant_type, ParticipantType::Claude);
        assert!(!p.in_hot_tub);
    }

    #[test]
    fn test_hot_tub_toggle() {
        let mut hub = CollaborationHub::new();
        let p = Participant::new("Hue", ParticipantType::Human);
        let id = hub.join(p);

        assert!(!hub.participants.get(&id).unwrap().in_hot_tub);
        hub.toggle_hot_tub(&id);
        assert!(hub.participants.get(&id).unwrap().in_hot_tub);
    }

    #[test]
    fn test_presence() {
        let mut hub = CollaborationHub::new();
        hub.join(Participant::new("Claude", ParticipantType::Claude));
        hub.join(Participant::new("Omni", ParticipantType::Omni));

        assert_eq!(hub.participant_count(), 2);
        assert_eq!(hub.get_presence().len(), 2);
    }
}
