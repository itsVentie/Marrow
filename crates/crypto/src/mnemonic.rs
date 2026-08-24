use crate::{CryptoError, Identity};
use bip39::{Language, Mnemonic};
use rand::RngCore;
use zeroize::Zeroize;

pub struct MnemonicKey;

impl MnemonicKey {
    pub fn generate_phrase_12() -> String {
        let mut entropy = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut entropy);
        let mnemonic =
            Mnemonic::from_entropy(&entropy).expect("12 words mnemonic generation failed");
        entropy.zeroize();
        mnemonic.to_string()
    }

    pub fn generate_phrase_24() -> String {
        let mut entropy = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut entropy);
        let mnemonic =
            Mnemonic::from_entropy(&entropy).expect("24 words mnemonic generation failed");
        entropy.zeroize();
        mnemonic.to_string()
    }

    pub fn derive_identity(
        phrase: &str,
        passphrase: Option<&str>,
    ) -> Result<Identity, CryptoError> {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, phrase)
            .map_err(|_| CryptoError::InvalidKey)?;

        let seed_bytes = mnemonic.to_seed(passphrase.unwrap_or(""));

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&seed_bytes[..32]);

        let identity = Identity::from_bytes(&key_bytes);
        key_bytes.zeroize();

        Ok(identity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mnemonic_generation_and_derivation() {
        let phrase_12 = MnemonicKey::generate_phrase_12();
        assert_eq!(phrase_12.split_whitespace().count(), 12);

        let phrase_24 = MnemonicKey::generate_phrase_24();
        assert_eq!(phrase_24.split_whitespace().count(), 24);

        let id1 = MnemonicKey::derive_identity(&phrase_12, None).unwrap();
        let id2 = MnemonicKey::derive_identity(&phrase_12, None).unwrap();

        assert_eq!(
            id1.verifying_key().to_bytes(),
            id2.verifying_key().to_bytes()
        );

        let id_passphrase = MnemonicKey::derive_identity(&phrase_12, Some("pass123")).unwrap();
        assert_ne!(
            id1.verifying_key().to_bytes(),
            id_passphrase.verifying_key().to_bytes()
        );
    }
}
