use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

pub mod handshake;
pub mod mnemonic;
pub mod ratchet;

pub use mnemonic::MnemonicKey;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Failed to generate keypair")]
    KeyGenError,
    #[error("Key derivation failed")]
    DerivationError,
    #[error("Encryption failed")]
    EncryptionError,
    #[error("Decryption failed or invalid password")]
    DecryptionError,
    #[error("Invalid key length or format")]
    InvalidKey,
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Identity {
    secret: [u8; 32],
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EncryptedVault {
    pub salt: [u8; 16],
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
}

impl Identity {
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        Self {
            secret: signing_key.to_bytes(),
        }
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self { secret: *bytes }
    }

    pub fn secret_bytes(&self) -> [u8; 32] {
        self.secret
    }

    pub fn signing_key(&self) -> SigningKey {
        SigningKey::from_bytes(&self.secret)
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key().verifying_key()
    }

    pub fn public_hex(&self) -> String {
        hex::encode(self.verifying_key().to_bytes())
    }

    pub fn export_encrypted(&self, password: &[u8]) -> Result<EncryptedVault, CryptoError> {
        let mut salt = [0u8; 16];
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce_bytes);

        let mut derived_key = [0u8; 32];
        let params =
            Params::new(65536, 3, 1, Some(32)).map_err(|_| CryptoError::DerivationError)?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        argon2
            .hash_password_into(password, &salt, &mut derived_key)
            .map_err(|_| CryptoError::DerivationError)?;

        let cipher = ChaCha20Poly1305::new_from_slice(&derived_key)
            .map_err(|_| CryptoError::EncryptionError)?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, self.secret.as_ref())
            .map_err(|_| CryptoError::EncryptionError)?;

        derived_key.zeroize();

        Ok(EncryptedVault {
            salt,
            nonce: nonce_bytes,
            ciphertext,
        })
    }

    pub fn import_encrypted(vault: &EncryptedVault, password: &[u8]) -> Result<Self, CryptoError> {
        let mut derived_key = [0u8; 32];
        let params =
            Params::new(65536, 3, 1, Some(32)).map_err(|_| CryptoError::DerivationError)?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        argon2
            .hash_password_into(password, &vault.salt, &mut derived_key)
            .map_err(|_| CryptoError::DerivationError)?;

        let cipher = ChaCha20Poly1305::new_from_slice(&derived_key)
            .map_err(|_| CryptoError::DecryptionError)?;
        let nonce = Nonce::from_slice(&vault.nonce);

        let plaintext = cipher
            .decrypt(nonce, vault.ciphertext.as_ref())
            .map_err(|_| CryptoError::DecryptionError)?;

        derived_key.zeroize();

        if plaintext.len() != 32 {
            return Err(CryptoError::InvalidKey);
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&plaintext);

        let identity = Self::from_bytes(&key_bytes);
        key_bytes.zeroize();

        Ok(identity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_encryption_cycle() {
        let identity = Identity::generate();
        let password = b"super_secret_master_password";

        let vault = identity.export_encrypted(password).unwrap();
        let decrypted = Identity::import_encrypted(&vault, password).unwrap();

        assert_eq!(
            identity.verifying_key().to_bytes(),
            decrypted.verifying_key().to_bytes()
        );
    }

    #[test]
    fn test_invalid_password_fails() {
        let identity = Identity::generate();
        let password = b"correct_password";
        let wrong_password = b"wrong_password";

        let vault = identity.export_encrypted(password).unwrap();
        assert!(Identity::import_encrypted(&vault, wrong_password).is_err());
    }
}
