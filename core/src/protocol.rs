use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};
use std::collections::HashMap;
use time::OffsetDateTime;
use crate::crypto::{EncryptedMessage, EncryptedIdentityKeys, DoubleRatchet};

/// Contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    pub display_name: String,
    pub public_key: [u8; 32],
    pub added_at: OffsetDateTime,
    pub last_seen: Option<OffsetDateTime>,
    pub verified: bool,
    pub blocked: bool,
}

/// Message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContent {
    Text { text: String },
    Image { data: Vec<u8>, mime_type: String, caption: Option<String> },
    File { data: Vec<u8>, filename: String, mime_type: String },
    Voice { data: Vec<u8>, duration_secs: u32 },
    Location { latitude: f64, longitude: f64, accuracy: Option<f32> },
    Contact { name: String, public_key: [u8; 32] },
}

/// Message envelope - encrypted content + metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    pub id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub timestamp: OffsetDateTime,
    pub encrypted_content: EncryptedMessage,
    pub signature: Vec<u8>,
    pub reply_to: Option<String>,
}

/// Message as stored locally (decrypted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalMessage {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub is_outgoing: bool,
    pub content: MessageContent,
    pub timestamp: OffsetDateTime,
    pub sent: bool,
    pub delivered: bool,
    pub read: bool,
    pub reply_to: Option<String>,
}

/// Conversation/session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub contact_id: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub last_message_preview: Option<String>,
    pub unread_count: u32,
    pub archived: bool,
    pub pinned: bool,
    pub ratchet_state: Option<DoubleRatchet>,
}

/// User profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub display_name: String,
    pub status_message: Option<String>,
    pub avatar: Option<Vec<u8>>,
    pub created_at: OffsetDateTime,
}

/// Device info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub platform: Platform,
    pub last_seen: OffsetDateTime,
    pub identity_key: EncryptedIdentityKeys,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Platform {
    Linux,
    Windows,
    Android,
    MacOS,
    IOS,
    Unknown,
}

/// Protocol message types for P2P communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProtocolMessage {
    /// Initial handshake - X3DH key bundle
    KeyBundle {
        identity_key: [u8; 32],
        signed_prekey: [u8; 32],
        signed_prekey_signature: Vec<u8>,
        one_time_prekeys: Vec<[u8; 32]>,
    },
    
    /// Encrypted message
    Encrypted {
        envelope: MessageEnvelope,
    },
    
    /// Delivery receipt
    DeliveryReceipt {
        message_id: String,
        timestamp: OffsetDateTime,
    },
    
    /// Read receipt
    ReadReceipt {
        message_id: String,
        timestamp: OffsetDateTime,
    },
    
    /// Typing indicator
    Typing {
        conversation_id: String,
        is_typing: bool,
    },
    
    /// Profile update
    ProfileUpdate {
        display_name: Option<String>,
        status_message: Option<String>,
        avatar_hash: Option<String>,
    },
    
    /// Contact request
    ContactRequest {
        display_name: String,
        message: String,
        key_bundle: Box<ProtocolMessage>, // KeyBundle
    },
    
    /// Contact response
    ContactResponse {
        accepted: bool,
        key_bundle: Option<Box<ProtocolMessage>>, // KeyBundle if accepted
    },
    
    /// Sync request for multi-device
    SyncRequest {
        device_id: String,
        nonce: [u8; 32],
    },
    
    /// Sync data
    SyncData {
        conversations: Vec<Conversation>,
        contacts: Vec<Contact>,
        settings: HashMap<String, String>,
    },
}

/// Generate unique ID
pub fn generate_id() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::encode(bytes)
}

impl Contact {
    pub fn new(id: String, display_name: String, public_key: [u8; 32]) -> Self {
        Self {
            id,
            display_name,
            public_key,
            added_at: OffsetDateTime::now_utc(),
            last_seen: None,
            verified: false,
            blocked: false,
        }
    }
    
    pub fn fingerprint(&self) -> String {
        let hash = blake3::hash(&self.public_key);
        format!("{}", hash.to_hex())[..32].to_string()
    }
}

impl LocalMessage {
    pub fn preview_text(&self) -> String {
        match &self.content {
            MessageContent::Text { text } => {
                if text.len() > 100 {
                    format!("{}...", &text[..100])
                } else {
                    text.clone()
                }
            }
            MessageContent::Image { caption, .. } => {
                caption.clone().unwrap_or_else(|| "📷 Image".to_string())
            }
            MessageContent::File { filename, .. } => {
                format!("📎 {}", filename)
            }
            MessageContent::Voice { .. } => {
                "🎤 Voice message".to_string()
            }
            MessageContent::Location { .. } => {
                "📍 Location".to_string()
            }
            MessageContent::Contact { name, .. } => {
                format!("👤 Contact: {}", name)
            }
        }
    }
}

impl Conversation {
    pub fn new(contact_id: String) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: generate_id(),
            contact_id,
            created_at: now,
            updated_at: now,
            last_message_preview: None,
            unread_count: 0,
            archived: false,
            pinned: false,
            ratchet_state: None,
        }
    }
}

impl MessageEnvelope {
    pub fn serialize(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .context("Failed to serialize message envelope")
    }
    
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data)
            .context("Failed to deserialize message envelope")
    }
}

use base64;

// blake3 re-export for fingerprinting
pub use blake3;
