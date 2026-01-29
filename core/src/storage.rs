use sled::Db;
use anyhow::{Result, Context};
use serde::{Serialize, de::DeserializeOwned};
use std::path::Path;

use crate::crypto::{EncryptedIdentityKeys, MasterKey};
use crate::protocol::{Contact, Conversation, LocalMessage, UserProfile, DeviceInfo};

/// Encrypted local storage
pub struct SecureStorage {
    db: Db,
    master_key: [u8; 32],
}

/// Key prefixes for different data types
const PREFIX_MASTER_KEY: &str = "mk:";
const PREFIX_IDENTITY: &str = "id:";
const PREFIX_CONTACT: &str = "ct:";
const PREFIX_CONVERSATION: &str = "cv:";
const PREFIX_MESSAGE: &str = "msg:";
const PREFIX_PROFILE: &str = "pf:";
const PREFIX_DEVICE: &str = "dv:";
const PREFIX_SETTINGS: &str = "st:";

impl SecureStorage {
    /// Open or create encrypted database
    pub fn open<P: AsRef<Path>>(path: P, master_key: Option<[u8; 32]>) -> Result<Self> {
        let db = sled::open(path)
            .context("Failed to open database")?;
        
        let master_key = if let Some(key) = master_key {
            key
        } else {
            // Check if we have a stored master key
            let stored = db.get(PREFIX_MASTER_KEY.as_bytes())
                .context("Failed to read master key")?;
            
            if let Some(data) = stored {
                let encrypted: MasterKey = bincode::deserialize(&data)
                    .context("Failed to deserialize master key")?;
                // This will fail if we don't have the password, caller must handle
                // For now, return error - unlock separately
                return Err(anyhow::anyhow!("Database exists but needs unlock"));
            } else {
                return Err(anyhow::anyhow!("New database needs master key generation"));
            }
        };
        
        Ok(Self { db, master_key })
    }
    
    /// Create new database with password
    pub fn create<P: AsRef<Path>>(path: P, password: &str) -> Result<Self> {
        let db = sled::open(path)
            .context("Failed to create database")?;
        
        let mut rng = rand::thread_rng();
        let (master_key_store, master_key) = MasterKey::from_password(password, &mut rng)
            .context("Failed to generate master key")?;
        
        // Store encrypted master key
        let serialized = bincode::serialize(&master_key_store)
            .context("Failed to serialize master key")?;
        db.insert(PREFIX_MASTER_KEY.as_bytes(), serialized)
            .context("Failed to store master key")?;
        
        Ok(Self { db, master_key })
    }
    
    /// Unlock existing database
    pub fn unlock<P: AsRef<Path>>(path: P, password: &str) -> Result<Self> {
        let db = sled::open(path)
            .context("Failed to open database")?;
        
        let stored = db.get(PREFIX_MASTER_KEY.as_bytes())
            .context("Failed to read master key")?
            .ok_or_else(|| anyhow::anyhow!("No master key found"))?;
        
        let encrypted: MasterKey = bincode::deserialize(&stored)
            .context("Failed to deserialize master key")?;
        
        let master_key = encrypted.unlock(password)
            .context("Failed to unlock database - wrong password?")?;
        
        Ok(Self { db, master_key })
    }
    
    /// Store encrypted value
    fn put<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let serialized = bincode::serialize(value)
            .context("Failed to serialize value")?;
        
        let encrypted = self.encrypt(&serialized)?;
        
        self.db.insert(key.as_bytes(), encrypted)
            .context("Failed to store value")?;
        
        Ok(())
    }
    
    /// Retrieve and decrypt value
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        match self.db.get(key.as_bytes()) {
            Ok(Some(data)) => {
                let decrypted = self.decrypt(&data)?;
                let value: T = bincode::deserialize(&decrypted)
                    .context("Failed to deserialize value")?;
                Ok(Some(value))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    /// Delete value
    fn delete(&self, key: &str) -> Result<()> {
        self.db.remove(key.as_bytes())
            .context("Failed to delete value")?;
        Ok(())
    }
    
    /// Encrypt data with master key + per-entry salt
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        use aes_gcm::{
            aead::{Aead, AeadCore, KeyInit},
            Aes256Gcm, Key, Nonce,
        };
        use rand::RngCore;
        
        let mut salt = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt);
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.master_key));
        let nonce = Aes256Gcm::generate_nonce(aes_gcm::aead::OsRng);
        
        let ciphertext = cipher
            .encrypt(&nonce, data)
            .context("Encryption failed")?;
        
        // Format: [salt:16][nonce:12][ciphertext]
        let mut result = Vec::with_capacity(16 + 12 + ciphertext.len());
        result.extend_from_slice(&salt);
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    /// Decrypt data
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Key, Nonce,
        };
        
        if data.len() < 28 {
            return Err(anyhow::anyhow!("Invalid encrypted data"));
        }
        
        let _salt = &data[0..16];
        let nonce = &data[16..28];
        let ciphertext = &data[28..];
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.master_key));
        
        let plaintext = cipher
            .decrypt(Nonce::from_slice(nonce), ciphertext)
            .context("Decryption failed")?;
        
        Ok(plaintext)
    }
    
    // ===== Identity Operations =====
    
    pub fn store_identity(&self, identity: &EncryptedIdentityKeys) -> Result<()> {
        self.put(&format!("{}self", PREFIX_IDENTITY), identity)
    }
    
    pub fn get_identity(&self) -> Result<Option<EncryptedIdentityKeys>> {
        self.get(&format!("{}self", PREFIX_IDENTITY))
    }
    
    // ===== Contact Operations =====
    
    pub fn store_contact(&self, contact: &Contact) -> Result<()> {
        self.put(&format!("{}{}", PREFIX_CONTACT, contact.id), contact)
    }
    
    pub fn get_contact(&self, id: &str) -> Result<Option<Contact>> {
        self.get(&format!("{}{}", PREFIX_CONTACT, id))
    }
    
    pub fn get_all_contacts(&self) -> Result<Vec<Contact>> {
        let mut contacts = Vec::new();
        for item in self.db.scan_prefix(PREFIX_CONTACT.as_bytes()) {
            let (_, value) = item.context("Failed to read contact")?;
            let decrypted = self.decrypt(&value)?;
            let contact: Contact = bincode::deserialize(&decrypted)
                .context("Failed to deserialize contact")?;
            contacts.push(contact);
        }
        Ok(contacts)
    }
    
    pub fn delete_contact(&self, id: &str) -> Result<()> {
        self.delete(&format!("{}{}", PREFIX_CONTACT, id))
    }
    
    // ===== Conversation Operations =====
    
    pub fn store_conversation(&self, conversation: &Conversation) -> Result<()> {
        self.put(&format!("{}{}", PREFIX_CONVERSATION, conversation.id), conversation)
    }
    
    pub fn get_conversation(&self, id: &str) -> Result<Option<Conversation>> {
        self.get(&format!("{}{}", PREFIX_CONVERSATION, id))
    }
    
    pub fn get_conversation_by_contact(&self, contact_id: &str) -> Result<Option<Conversation>> {
        for conv in self.get_all_conversations()? {
            if conv.contact_id == contact_id {
                return Ok(Some(conv));
            }
        }
        Ok(None)
    }
    
    pub fn get_all_conversations(&self) -> Result<Vec<Conversation>> {
        let mut conversations = Vec::new();
        for item in self.db.scan_prefix(PREFIX_CONVERSATION.as_bytes()) {
            let (_, value) = item.context("Failed to read conversation")?;
            let decrypted = self.decrypt(&value)?;
            let conversation: Conversation = bincode::deserialize(&decrypted)
                .context("Failed to deserialize conversation")?;
            conversations.push(conversation);
        }
        // Sort by updated_at descending
        conversations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(conversations)
    }
    
    // ===== Message Operations =====
    
    pub fn store_message(&self, message: &LocalMessage) -> Result<()> {
        let key = format!("{}{}/{}", PREFIX_MESSAGE, message.conversation_id, message.id);
        self.put(&key, message)
    }
    
    pub fn get_message(&self, conversation_id: &str, message_id: &str) -> Result<Option<LocalMessage>> {
        let key = format!("{}{}/{}", PREFIX_MESSAGE, conversation_id, message_id);
        self.get(&key)
    }
    
    pub fn get_messages(&self, conversation_id: &str, limit: usize) -> Result<Vec<LocalMessage>> {
        let prefix = format!("{}{}/", PREFIX_MESSAGE, conversation_id);
        let mut messages = Vec::new();
        
        for item in self.db.scan_prefix(prefix.as_bytes()) {
            if messages.len() >= limit {
                break;
            }
            let (_, value) = item.context("Failed to read message")?;
            let decrypted = self.decrypt(&value)?;
            let message: LocalMessage = bincode::deserialize(&decrypted)
                .context("Failed to deserialize message")?;
            messages.push(message);
        }
        
        // Sort by timestamp ascending
        messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(messages)
    }
    
    pub fn get_messages_before(&self, conversation_id: &str, before_id: &str, limit: usize) -> Result<Vec<LocalMessage>> {
        let prefix = format!("{}{}/", PREFIX_MESSAGE, conversation_id);
        let mut messages = Vec::new();
        
        for item in self.db.scan_prefix(prefix.as_bytes()) {
            let (_, value) = item.context("Failed to read message")?;
            let decrypted = self.decrypt(&value)?;
            let message: LocalMessage = bincode::deserialize(&decrypted)
                .context("Failed to deserialize message")?;
            messages.push(message);
        }
        
        // Sort and filter
        messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        if let Some(pos) = messages.iter().position(|m| m.id == before_id) {
            let start = pos.saturating_sub(limit);
            Ok(messages[start..pos].to_vec())
        } else {
            let start = messages.len().saturating_sub(limit);
            Ok(messages[start..].to_vec())
        }
    }
    
    pub fn delete_message(&self, conversation_id: &str, message_id: &str) -> Result<()> {
        let key = format!("{}{}/{}", PREFIX_MESSAGE, conversation_id, message_id);
        self.delete(&key)
    }
    
    // ===== Profile Operations =====
    
    pub fn store_profile(&self, profile: &UserProfile) -> Result<()> {
        self.put(&format!("{}self", PREFIX_PROFILE), profile)
    }
    
    pub fn get_profile(&self) -> Result<Option<UserProfile>> {
        self.get(&format!("{}self", PREFIX_PROFILE))
    }
    
    // ===== Settings Operations =====
    
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.db.insert(
            format!("{}{}", PREFIX_SETTINGS, key).as_bytes(),
            value.as_bytes()
        ).context("Failed to store setting")?;
        Ok(())
    }
    
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        match self.db.get(format!("{}{}", PREFIX_SETTINGS, key).as_bytes()) {
            Ok(Some(data)) => {
                let value = String::from_utf8(data.to_vec())
                    .context("Invalid UTF-8 in setting")?;
                Ok(Some(value))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    // ===== Device Operations =====
    
    pub fn store_device(&self, device: &DeviceInfo) -> Result<()> {
        self.put(&format!("{}{}", PREFIX_DEVICE, device.device_id), device)
    }
    
    pub fn get_device(&self, device_id: &str) -> Result<Option<DeviceInfo>> {
        self.get(&format!("{}{}", PREFIX_DEVICE, device_id))
    }
    
    pub fn get_all_devices(&self) -> Result<Vec<DeviceInfo>> {
        let mut devices = Vec::new();
        for item in self.db.scan_prefix(PREFIX_DEVICE.as_bytes()) {
            let (_, value) = item.context("Failed to read device")?;
            let decrypted = self.decrypt(&value)?;
            let device: DeviceInfo = bincode::deserialize(&decrypted)
                .context("Failed to deserialize device")?;
            devices.push(device);
        }
        Ok(devices)
    }
    
    /// Flush all changes to disk
    pub fn flush(&self) -> Result<()> {
        self.db.flush()
            .context("Failed to flush database")?;
        Ok(())
    }
    
    /// Close the database
    pub fn close(self) -> Result<()> {
        self.db.flush()
            .context("Failed to close database")?;
        Ok(())
    }
}

use rand::RngCore;
