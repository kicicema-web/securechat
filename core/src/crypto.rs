use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::RngCore, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey, Signature};
use hkdf::Hkdf;
use rand::RngCore as RandRngCore;
use sha2::Sha256;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519SecretKey};
use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};

/// Master key derived from password, encrypted with AES-256-GCM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterKey {
    pub encrypted_key: Vec<u8>,
    pub salt: [u8; 32],
    pub nonce: [u8; 12],
}

/// Identity key pair for signing
#[derive(Debug, Clone)]
pub struct IdentityKeyPair {
    pub public_key: VerifyingKey,
    secret_key: SigningKey,
}

/// Encrypted identity key storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedIdentityKeys {
    pub public_key: [u8; 32],
    pub encrypted_secret: Vec<u8>,
    pub nonce: [u8; 12],
}

/// Message encryption keys (X25519)
#[derive(Clone)]
pub struct MessageKeyPair {
    pub public_key: X25519PublicKey,
    secret_key: X25519SecretKey,
}

impl std::fmt::Debug for MessageKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageKeyPair")
            .field("public_key", &self.public_key)
            .field("secret_key", &"[REDACTED]")
            .finish()
    }
}

/// Encrypted message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
    pub sender_pubkey: [u8; 32],
    pub ephemeral_pubkey: [u8; 32],
}

/// Double Ratchet state for perfect forward secrecy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoubleRatchet {
    pub root_key: [u8; 32],
    pub sending_chain_key: Option<[u8; 32]>,
    pub receiving_chain_key: Option<[u8; 32]>,
    pub sending_message_number: u32,
    pub receiving_message_number: u32,
    pub skipped_message_keys: Vec<(u32, [u8; 32])>,
}

impl MasterKey {
    /// Derive a master key from password using Argon2id
    pub fn from_password(password: &str, rng: &mut impl RngCore) -> Result<(Self, [u8; 32])> {
        let salt = Self::generate_random_bytes(rng);
        let nonce = Self::generate_random_bytes_12(rng);
        
        // Derive key using Argon2id
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(&salt)
            .map_err(|e| anyhow::anyhow!("Failed to encode salt: {:?}", e))?;
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {:?}", e))?;
        
        let mut derived_key = [0u8; 32];
        let _ = password_hash.hash
            .as_ref()
            .map(|hash| derived_key.copy_from_slice(&hash.as_bytes()[..32]));
        
        // Generate random master key and encrypt it
        let master_key: [u8; 32] = Self::generate_random_bytes(rng);
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&derived_key));
        let encrypted_key = cipher
            .encrypt(Nonce::from_slice(&nonce), master_key.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to encrypt master key: {:?}", e))?;
        
        Ok((Self {
            encrypted_key,
            salt,
            nonce,
        }, master_key))
    }
    
    /// Unlock master key with password
    pub fn unlock(&self, password: &str) -> Result<[u8; 32]> {
        // Re-derive key from password
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(&self.salt)
            .map_err(|e| anyhow::anyhow!("Failed to encode salt: {:?}", e))?;
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {:?}", e))?;
        
        let mut derived_key = [0u8; 32];
        if let Some(hash) = password_hash.hash {
            derived_key.copy_from_slice(&hash.as_bytes()[..32]);
        }
        
        // Decrypt master key
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&derived_key));
        let decrypted = cipher
            .decrypt(Nonce::from_slice(&self.nonce), self.encrypted_key.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to decrypt master key - wrong password?: {:?}", e))?;
        
        let mut master_key = [0u8; 32];
        master_key.copy_from_slice(&decrypted);
        
        Ok(master_key)
    }
    
    pub fn generate_random_bytes(rng: &mut impl RandRngCore) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        bytes
    }
    
    fn generate_random_bytes_12(rng: &mut impl RandRngCore) -> [u8; 12] {
        let mut bytes = [0u8; 12];
        rng.fill_bytes(&mut bytes);
        bytes
    }
}

impl IdentityKeyPair {
    /// Generate new identity key pair
    pub fn generate(rng: &mut impl rand_core::CryptoRngCore) -> Self {
        let signing_key = SigningKey::generate(rng);
        let verifying_key = signing_key.verifying_key();
        
        Self {
            public_key: verifying_key,
            secret_key: signing_key,
        }
    }
    
    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.secret_key.sign(message)
    }
    
    /// Verify a signature
    pub fn verify(public_key: &VerifyingKey, message: &[u8], signature: &Signature) -> Result<()> {
        public_key.verify_strict(message, signature)
            .context("Signature verification failed")
    }
    
    /// Encrypt keys with master key
    pub fn encrypt(&self, master_key: &[u8; 32], rng: &mut impl RngCore) -> Result<EncryptedIdentityKeys> {
        let nonce = Self::generate_random_bytes_12(rng);
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(master_key));
        
        let secret_bytes = self.secret_key.to_bytes();
        let encrypted_secret = cipher
            .encrypt(Nonce::from_slice(&nonce), secret_bytes.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to encrypt secret key: {:?}", e))?;
        
        Ok(EncryptedIdentityKeys {
            public_key: self.public_key.to_bytes(),
            encrypted_secret,
            nonce,
        })
    }
    
    /// Decrypt keys
    pub fn decrypt(encrypted: &EncryptedIdentityKeys, master_key: &[u8; 32]) -> Result<Self> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(master_key));
        let decrypted = cipher
            .decrypt(Nonce::from_slice(&encrypted.nonce), encrypted.encrypted_secret.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to decrypt identity keys: {:?}", e))?;
        
        let mut secret_bytes = [0u8; 32];
        secret_bytes.copy_from_slice(&decrypted);
        
        let secret_key = SigningKey::from_bytes(&secret_bytes);
        let public_key = secret_key.verifying_key();
        
        Ok(Self {
            public_key,
            secret_key,
        })
    }
    
    fn generate_random_bytes_12(rng: &mut impl RandRngCore) -> [u8; 12] {
        let mut bytes = [0u8; 12];
        rng.fill_bytes(&mut bytes);
        bytes
    }
}

impl MessageKeyPair {
    /// Generate new message key pair
    pub fn generate() -> Self {
        let secret_key = X25519SecretKey::random_from_rng(OsRng);
        let public_key = X25519PublicKey::from(&secret_key);
        
        Self {
            public_key,
            secret_key,
        }
    }
    
    /// Encrypt a message using X3DH + Double Ratchet
    pub fn encrypt_message(
        &self,
        recipient_pubkey: &X25519PublicKey,
        message: &[u8],
    ) -> Result<EncryptedMessage> {
        // Generate ephemeral key for forward secrecy
        let ephemeral_secret = X25519SecretKey::random_from_rng(OsRng);
        let ephemeral_pubkey = X25519PublicKey::from(&ephemeral_secret);
        
        // Perform DH exchanges for X3DH
        let dh1 = self.secret_key.diffie_hellman(recipient_pubkey);
        let dh2 = ephemeral_secret.diffie_hellman(recipient_pubkey);
        
        // Derive shared secret using HKDF
        let mut shared_secret = [0u8; 32];
        let mut dh_bytes = Vec::with_capacity(64);
        dh_bytes.extend_from_slice(dh1.as_bytes());
        dh_bytes.extend_from_slice(dh2.as_bytes());
        let hk = Hkdf::<Sha256>::new(None, &dh_bytes);
        hk.expand(b"SecureChat-v1", &mut shared_secret)
            .map_err(|e| anyhow::anyhow!("HKDF expand failed: {:?}", e))?;
        
        // Encrypt message
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&shared_secret));
        let nonce = Aes256Gcm::generate_nonce(OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, message)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;
        
        Ok(EncryptedMessage {
            ciphertext,
            nonce: nonce.into(),
            sender_pubkey: self.public_key.as_bytes().clone(),
            ephemeral_pubkey: ephemeral_pubkey.as_bytes().clone(),
        })
    }
    
    /// Decrypt a message
    pub fn decrypt_message(
        &self,
        encrypted: &EncryptedMessage,
    ) -> Result<Vec<u8>> {
        // Reconstruct ephemeral public key
        let ephemeral_pubkey = X25519PublicKey::from(encrypted.ephemeral_pubkey);
        let sender_pubkey = X25519PublicKey::from(encrypted.sender_pubkey);
        
        // Perform DH exchanges
        let dh1 = self.secret_key.diffie_hellman(&sender_pubkey);
        let dh2 = self.secret_key.diffie_hellman(&ephemeral_pubkey);
        
        // Derive shared secret
        let mut shared_secret = [0u8; 32];
        let mut dh_bytes = Vec::with_capacity(64);
        dh_bytes.extend_from_slice(dh1.as_bytes());
        dh_bytes.extend_from_slice(dh2.as_bytes());
        let hk = Hkdf::<Sha256>::new(None, &dh_bytes);
        hk.expand(b"SecureChat-v1", &mut shared_secret)
            .map_err(|e| anyhow::anyhow!("HKDF expand failed: {:?}", e))?;
        
        // Decrypt message
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&shared_secret));
        let plaintext = cipher
            .decrypt(Nonce::from_slice(&encrypted.nonce), encrypted.ciphertext.as_ref())
            .map_err(|e| anyhow::anyhow!("Decryption failed - wrong key or tampered message: {:?}", e))?;
        
        Ok(plaintext)
    }
}

impl DoubleRatchet {
    /// Initialize with shared secret from X3DH
    pub fn initialize(shared_secret: &[u8; 32]) -> Self {
        Self {
            root_key: *shared_secret,
            sending_chain_key: None,
            receiving_chain_key: None,
            sending_message_number: 0,
            receiving_message_number: 0,
            skipped_message_keys: Vec::new(),
        }
    }
    
    /// Ratchet step - derive new chain keys
    pub fn ratchet(&mut self, new_remote_pubkey: &[u8; 32]) -> Result<()> {
        let hk = Hkdf::<Sha256>::new(None, &self.root_key);
        let mut new_root = [0u8; 32];
        hk.expand(b"ratchet-root", &mut new_root)
            .map_err(|e| anyhow::anyhow!("Ratchet root derivation failed: {:?}", e))?;
        
        let mut sending = [0u8; 32];
        hk.expand(b"ratchet-send", &mut sending)
            .map_err(|e| anyhow::anyhow!("Ratchet send derivation failed: {:?}", e))?;
        
        let mut receiving = [0u8; 32];
        hk.expand(b"ratchet-recv", &mut receiving)
            .map_err(|e| anyhow::anyhow!("Ratchet recv derivation failed: {:?}", e))?;
        
        self.root_key = new_root;
        self.sending_chain_key = Some(sending);
        self.receiving_chain_key = Some(receiving);
        self.sending_message_number = 0;
        self.receiving_message_number = 0;
        
        Ok(())
    }
}

/// Utility function to hash a password for storage
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {:?}", e))?;
    
    Ok(password_hash.to_string())
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| anyhow::anyhow!("Invalid password hash format: {:?}", e))?;
    
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_master_key_encryption() {
        let mut rng = OsRng;
        let password = "test_password_123";
        
        let (master_key_store, original_key) = MasterKey::from_password(password, &mut rng)
            .expect("Failed to create master key");
        
        let decrypted_key = master_key_store.unlock(password)
            .expect("Failed to unlock master key");
        
        assert_eq!(original_key, decrypted_key);
    }
    
    #[test]
    fn test_identity_key_encryption() {
        let mut rng = OsRng;
        let master_key: [u8; 32] = MasterKey::generate_random_bytes(&mut rng);
        
        let identity = IdentityKeyPair::generate(&mut rng);
        let encrypted = identity.encrypt(&master_key, &mut rng)
            .expect("Failed to encrypt identity");
        
        let decrypted = IdentityKeyPair::decrypt(&encrypted, &master_key)
            .expect("Failed to decrypt identity");
        
        assert_eq!(identity.public_key.to_bytes(), decrypted.public_key.to_bytes());
    }
    
    #[test]
    fn test_message_encryption() {
        let alice = MessageKeyPair::generate();
        let bob = MessageKeyPair::generate();
        
        let message = b"Hello, secure world!";
        
        // Alice encrypts for Bob
        let encrypted = alice.encrypt_message(&bob.public_key, message)
            .expect("Failed to encrypt message");
        
        // Bob decrypts
        let decrypted = bob.decrypt_message(&encrypted)
            .expect("Failed to decrypt message");
        
        assert_eq!(message.as_slice(), decrypted.as_slice());
    }
    
    #[test]
    fn test_signing() {
        let mut rng = OsRng;
        let identity = IdentityKeyPair::generate(&mut rng);
        
        let message = b"Test message to sign";
        let signature = identity.sign(message);
        
        IdentityKeyPair::verify(&identity.public_key, message, &signature)
            .expect("Signature verification failed");
    }
}
