//! SecureChat Core Library
//! 
//! End-to-end encrypted messaging library with:
//! - AES-256-GCM encryption
//! - X3DH key agreement
//! - Double Ratchet for forward secrecy
//! - P2P networking via libp2p
//! - Local encrypted storage

pub mod crypto;
pub mod protocol;
pub mod storage;
pub mod network;

use anyhow::{Result, Context};
use crypto::{IdentityKeyPair, MessageKeyPair, EncryptedIdentityKeys};
use protocol::{Contact, Conversation, LocalMessage, MessageContent, UserProfile, DeviceInfo, Platform};
use storage::SecureStorage;
use network::{NetworkManager, NetworkConfig, NetworkCommand, NetworkEvent};
use time::OffsetDateTime;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use futures::channel::mpsc as futures_mpsc;

/// Application state
pub struct SecureChat {
    storage: Arc<Mutex<SecureStorage>>,
    identity: Arc<RwLock<Option<IdentityKeyPair>>>,
    message_keys: Arc<RwLock<Option<MessageKeyPair>>>,
    network: Arc<Mutex<Option<NetworkManager>>>,
    network_cmd_tx: Arc<Mutex<Option<futures_mpsc::Sender<NetworkCommand>>>>,
    profile: Arc<RwLock<Option<UserProfile>>>,
    device_id: String,
}

/// Event types for UI updates
#[derive(Debug, Clone)]
pub enum ChatEvent {
    MessageReceived { conversation_id: String, message: LocalMessage },
    MessageSent { conversation_id: String, message_id: String },
    MessageDelivered { conversation_id: String, message_id: String },
    MessageRead { conversation_id: String, message_id: String },
    ContactOnline { contact_id: String },
    ContactOffline { contact_id: String },
    ContactRequestReceived { contact_id: String, display_name: String, message: String },
    SyncCompleted,
    Error { message: String },
}

impl SecureChat {
    /// Create new chat instance (without opening database)
    pub fn new(device_id: Option<String>) -> Self {
        Self {
            storage: Arc::new(Mutex::new(None)),
            identity: Arc::new(RwLock::new(None)),
            message_keys: Arc::new(RwLock::new(None)),
            network: Arc::new(Mutex::new(None)),
            network_cmd_tx: Arc::new(Mutex::new(None)),
            profile: Arc::new(RwLock::new(None)),
            device_id: device_id.unwrap_or_else(|| protocol::generate_id()),
        }
    }
    
    /// Initialize database with new password (first time setup)
    pub async fn create_account<P: AsRef<Path>>(
        &self,
        db_path: P,
        password: &str,
        display_name: &str,
    ) -> Result<()> {
        // Create storage
        let storage = SecureStorage::create(db_path, password)
            .context("Failed to create database")?;
        
        *self.storage.lock().await = storage;
        
        // Generate identity keys
        let mut rng = rand::thread_rng();
        let identity = IdentityKeyPair::generate(&mut rng);
        let master_key = self.storage.lock().await.master_key;
        let encrypted_identity = identity.encrypt(&master_key, &mut rng)
            .context("Failed to encrypt identity")?;
        
        self.storage.lock().await.store_identity(&encrypted_identity)?;
        *self.identity.write().await = Some(identity);
        
        // Generate message keys
        let message_keys = MessageKeyPair::generate();
        *self.message_keys.write().await = Some(message_keys);
        
        // Create profile
        let profile = UserProfile {
            display_name: display_name.to_string(),
            status_message: None,
            avatar: None,
            created_at: OffsetDateTime::now_utc(),
        };
        self.storage.lock().await.store_profile(&profile)?;
        *self.profile.write().await = Some(profile);
        
        // Store device info
        let device = DeviceInfo {
            device_id: self.device_id.clone(),
            device_name: format!("{}'s Device", display_name),
            platform: detect_platform(),
            last_seen: OffsetDateTime::now_utc(),
            identity_key: encrypted_identity,
        };
        self.storage.lock().await.store_device(&device)?;
        
        Ok(())
    }
    
    /// Unlock existing account
    pub async fn unlock_account<P: AsRef<Path>>(
        &self,
        db_path: P,
        password: &str,
    ) -> Result<()> {
        // Unlock storage
        let storage = SecureStorage::unlock(db_path, password)
            .context("Failed to unlock database")?;
        
        *self.storage.lock().await = storage;
        
        // Decrypt identity
        let encrypted_identity = self.storage.lock().await.get_identity()
            .context("Failed to get identity")?
            .ok_or_else(|| anyhow::anyhow!("No identity found"))?;
        
        let master_key = self.storage.lock().await.master_key;
        let identity = IdentityKeyPair::decrypt(&encrypted_identity, &master_key)
            .context("Failed to decrypt identity")?;
        
        *self.identity.write().await = Some(identity);
        
        // Generate message keys (ephemeral, not stored)
        let message_keys = MessageKeyPair::generate();
        *self.message_keys.write().await = Some(message_keys);
        
        // Load profile
        let profile = self.storage.lock().await.get_profile()
            .context("Failed to get profile")?;
        *self.profile.write().await = profile;
        
        Ok(())
    }
    
    /// Start networking
    pub async fn start_network(&self, config: NetworkConfig) -> Result<mpsc::Receiver<ChatEvent>> {
        let (manager, event_rx, cmd_tx) = NetworkManager::new(config)
            .context("Failed to create network manager")?;
        
        *self.network.lock().await = Some(manager);
        *self.network_cmd_tx.lock().await = Some(cmd_tx);
        
        // Spawn network task
        let network = self.network.clone();
        tokio::spawn(async move {
            if let Some(manager) = network.lock().await.take() {
                if let Err(e) = manager.run().await {
                    log::error!("Network error: {}", e);
                }
            }
        });
        
        // Convert network events to chat events
        let (chat_tx, chat_rx) = mpsc::channel(100);
        tokio::spawn(Self::network_event_loop(event_rx, chat_tx));
        
        Ok(chat_rx)
    }
    
    /// Stop networking
    pub async fn stop_network(&self) -> Result<()> {
        if let Some(tx) = self.network_cmd_tx.lock().await.as_mut() {
            tx.send(NetworkCommand::Shutdown).await.ok();
        }
        Ok(())
    }
    
    async fn network_event_loop(
        mut event_rx: futures_mpsc::Receiver<NetworkEvent>,
        chat_tx: mpsc::Sender<ChatEvent>,
    ) {
        while let Some(event) = event_rx.recv().await {
            let chat_event = match event {
                NetworkEvent::MessageReceived { peer_id, message } => {
                    // Handle protocol message
                    Self::handle_protocol_message(peer_id, message).await
                }
                NetworkEvent::PeerConnected { peer_id } => {
                    Some(ChatEvent::ContactOnline { contact_id: peer_id })
                }
                NetworkEvent::PeerDisconnected { peer_id } => {
                    Some(ChatEvent::ContactOffline { contact_id: peer_id })
                }
                _ => None,
            };
            
            if let Some(evt) = chat_event {
                chat_tx.send(evt).await.ok();
            }
        }
    }
    
    async fn handle_protocol_message(peer_id: String, message: protocol::ProtocolMessage) -> Option<ChatEvent> {
        match message {
            protocol::ProtocolMessage::ContactRequest { display_name, message: msg, .. } => {
                Some(ChatEvent::ContactRequestReceived {
                    contact_id: peer_id,
                    display_name,
                    message: msg,
                })
            }
            _ => None,
        }
    }
    
    /// Send text message
    pub async fn send_text_message(&self, conversation_id: &str, text: &str) -> Result<String> {
        let conversation = self.storage.lock().await
            .get_conversation(conversation_id)?
            .ok_or_else(|| anyhow::anyhow!("Conversation not found"))?;
        
        let contact = self.storage.lock().await
            .get_contact(&conversation.contact_id)?
            .ok_or_else(|| anyhow::anyhow!("Contact not found"))?;
        
        let message_id = protocol::generate_id();
        let timestamp = OffsetDateTime::now_utc();
        
        // Create message
        let content = MessageContent::Text { text: text.to_string() };
        let local_message = LocalMessage {
            id: message_id.clone(),
            conversation_id: conversation_id.to_string(),
            sender_id: "self".to_string(),
            is_outgoing: true,
            content,
            timestamp,
            sent: false,
            delivered: false,
            read: false,
            reply_to: None,
        };
        
        // Store locally
        self.storage.lock().await.store_message(&local_message)?;
        
        // Encrypt for network (placeholder - real implementation would use proper X3DH)
        // self.encrypt_and_send(&contact, &local_message).await?;
        
        Ok(message_id)
    }
    
    /// Get all conversations
    pub async fn get_conversations(&self) -> Result<Vec<Conversation>> {
        self.storage.lock().await.get_all_conversations()
    }
    
    /// Get messages for a conversation
    pub async fn get_messages(&self, conversation_id: &str, limit: usize) -> Result<Vec<LocalMessage>> {
        self.storage.lock().await.get_messages(conversation_id, limit)
    }
    
    /// Create or get conversation with contact
    pub async fn get_or_create_conversation(&self, contact_id: &str) -> Result<Conversation> {
        if let Some(conv) = self.storage.lock().await
            .get_conversation_by_contact(contact_id)? {
            return Ok(conv);
        }
        
        let conversation = Conversation::new(contact_id.to_string());
        self.storage.lock().await.store_conversation(&conversation)?;
        
        Ok(conversation)
    }
    
    /// Add contact
    pub async fn add_contact(&self, public_key: [u8; 32], display_name: &str) -> Result<Contact> {
        let contact = Contact::new(
            protocol::generate_id(),
            display_name.to_string(),
            public_key,
        );
        
        self.storage.lock().await.store_contact(&contact)?;
        
        Ok(contact)
    }
    
    /// Get all contacts
    pub async fn get_contacts(&self) -> Result<Vec<Contact>> {
        self.storage.lock().await.get_all_contacts()
    }
    
    /// Get user profile
    pub async fn get_profile(&self) -> Result<Option<UserProfile>> {
        self.storage.lock().await.get_profile()
    }
    
    /// Update profile
    pub async fn update_profile(&self, display_name: Option<&str>, status_message: Option<&str>) -> Result<()> {
        let mut profile = self.storage.lock().await
            .get_profile()?
            .unwrap_or_else(|| UserProfile {
                display_name: "Anonymous".to_string(),
                status_message: None,
                avatar: None,
                created_at: OffsetDateTime::now_utc(),
            });
        
        if let Some(name) = display_name {
            profile.display_name = name.to_string();
        }
        if let Some(status) = status_message {
            profile.status_message = Some(status.to_string());
        }
        
        self.storage.lock().await.store_profile(&profile)?;
        *self.profile.write().await = Some(profile);
        
        Ok(())
    }
    
    /// Get public identity key for sharing
    pub async fn get_public_key(&self) -> Result<[u8; 32]> {
        let identity = self.identity.read().await;
        let identity = identity.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;
        Ok(identity.public_key.to_bytes())
    }
    
    /// Export encrypted backup
    pub async fn export_backup(&self, password: &str) -> Result<Vec<u8>> {
        // Collect all data
        let contacts = self.storage.lock().await.get_all_contacts()?;
        let conversations = self.storage.lock().await.get_all_conversations()?;
        let profile = self.storage.lock().await.get_profile()?;
        
        // Serialize
        let backup_data = serde_json::json!({
            "version": 1,
            "contacts": contacts,
            "conversations": conversations,
            "profile": profile,
        });
        
        let json_data = serde_json::to_vec(&backup_data)?;
        
        // Encrypt with password
        use crypto::MasterKey;
        use rand::RngCore;
        
        let mut rng = rand::thread_rng();
        let (master_key_store, master_key) = MasterKey::from_password(password, &mut rng)?;
        
        use aes_gcm::{
            aead::{Aead, AeadCore, KeyInit},
            Aes256Gcm, Key, Nonce,
        };
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&master_key));
        let nonce = Aes256Gcm::generate_nonce(aes_gcm::aead::OsRng);
        let encrypted = cipher.encrypt(&nonce, json_data.as_ref())?;
        
        // Format: [master_key_encrypted][nonce][encrypted_data]
        let master_key_bytes = bincode::serialize(&master_key_store)?;
        let mut result = Vec::new();
        result.extend_from_slice(&(master_key_bytes.len() as u32).to_be_bytes());
        result.extend_from_slice(&master_key_bytes);
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&encrypted);
        
        Ok(result)
    }
    
    /// Close and cleanup
    pub async fn close(self) -> Result<()> {
        self.stop_network().await.ok();
        // Storage will be dropped
        Ok(())
    }
}

fn detect_platform() -> Platform {
    #[cfg(target_os = "linux")]
    return Platform::Linux;
    
    #[cfg(target_os = "windows")]
    return Platform::Windows;
    
    #[cfg(target_os = "android")]
    return Platform::Android;
    
    #[cfg(target_os = "macos")]
    return Platform::MacOS;
    
    #[cfg(target_os = "ios")]
    return Platform::IOS;
    
    #[cfg(not(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "android",
        target_os = "macos",
        target_os = "ios"
    )))]
    return Platform::Unknown;
}

// Re-exports
pub use crypto::{hash_password, verify_password};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_create_and_unlock() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let chat = SecureChat::new(None);
        chat.create_account(&db_path, "password123", "Test User").await.unwrap();
        
        // Unlock should work
        let chat2 = SecureChat::new(None);
        chat2.unlock_account(&db_path, "password123").await.unwrap();
        
        // Wrong password should fail
        let chat3 = SecureChat::new(None);
        assert!(chat3.unlock_account(&db_path, "wrong_password").await.is_err());
    }
    
    #[tokio::test]
    async fn test_contacts_and_conversations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let chat = SecureChat::new(None);
        chat.create_account(&db_path, "password", "User").await.unwrap();
        
        // Add contact
        let public_key = [1u8; 32];
        let contact = chat.add_contact(public_key, "Alice").await.unwrap();
        
        // Get conversation
        let conversation = chat.get_or_create_conversation(&contact.id).await.unwrap();
        
        // Verify
        let conversations = chat.get_conversations().await.unwrap();
        assert_eq!(conversations.len(), 1);
        
        let contacts = chat.get_contacts().await.unwrap();
        assert_eq!(contacts.len(), 1);
    }
}
