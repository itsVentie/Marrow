use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce,
};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use sha2::Sha256;
use std::collections::HashMap;
use thiserror::Error;
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::{Zeroize, ZeroizeOnDrop};

const MAX_SKIPPED_KEYS: usize = 2000;
const PRK_INFO: &[u8] = b"Marrow_Ratchet_HKDF_PRK";
const ROOT_INFO: &[u8] = b"Marrow_Ratchet_Root_Chain";
const CHAIN_INFO: &[u8] = b"Marrow_Ratchet_Chain_Step";
const MSG_KEY_INFO: &[u8] = b"Marrow_Ratchet_Message_Key";

#[derive(Error, Debug)]
pub enum RatchetError {
    #[error("Decryption failed")]
    DecryptionFailed,

    #[error("Too many skipped keys")]
    TooManySkippedKeys,

    #[error("Key not found for message")]
    KeyNotFound,

    #[error("Invalid public key")]
    InvalidPublicKey,
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SymmetricKey(pub [u8; 32]);

pub struct Header {
    pub dh_pub: PublicKey,
    pub pn: u32,
    pub n: u32,
}

pub struct EncryptedMessage {
    pub header: Header,
    pub ciphertext: Vec<u8>,
}

#[derive(Clone, Hash, Eq, PartialEq)]
struct SkippedKeyKey {
    dh_pub: [u8; 32],
    n: u32,
}

pub struct DoubleRatchet {
    dhs: StaticSecret,
    dhr: Option<PublicKey>,
    rk: [u8; 32],
    cks: Option<[u8; 32]>,
    ckr: Option<[u8; 32]>,
    ns: u32,
    nr: u32,
    pn: u32,
    mkskipped: HashMap<SkippedKeyKey, SymmetricKey>,
}

impl DoubleRatchet {
    pub fn new_ventie(shared_key: [u8; 32], anek_dh_pub: PublicKey) -> Self {
        let dhs = StaticSecret::random_from_rng(OsRng);

        let dh_out = dhs.diffie_hellman(&anek_dh_pub);
        let (rk, cks) = kdf_rk(&shared_key, dh_out.as_bytes());

        Self {
            dhs,
            dhr: Some(anek_dh_pub),
            rk,
            cks: Some(cks),
            ckr: None,
            ns: 0,
            nr: 0,
            pn: 0,
            mkskipped: HashMap::new(),
        }
    }

    pub fn new_anek(shared_key: [u8; 32], anek_dh: StaticSecret) -> Self {
        Self {
            dhs: anek_dh,
            dhr: None,
            rk: shared_key,
            cks: None,
            ckr: None,
            ns: 0,
            nr: 0,
            pn: 0,
            mkskipped: HashMap::new(),
        }
    }

    pub fn encrypt(
        &mut self,
        plaintext: &[u8],
        ad: &[u8],
    ) -> Result<EncryptedMessage, RatchetError> {
        let cks = self.cks.as_mut().ok_or(RatchetError::DecryptionFailed)?;
        let (next_cks, mk) = kdf_ck(cks);
        *cks = next_cks;

        let header = Header {
            dh_pub: PublicKey::from(&self.dhs),
            pn: self.pn,
            n: self.ns,
        };
        self.ns += 1;

        let ciphertext = aead_encrypt(&mk, plaintext, &header_ad(&header, ad), header.n)?;

        Ok(EncryptedMessage { header, ciphertext })
    }

    pub fn decrypt(&mut self, msg: &EncryptedMessage, ad: &[u8]) -> Result<Vec<u8>, RatchetError> {
        let header_bytes = header_ad(&msg.header, ad);

        if let Some(mk) = self.mkskipped.remove(&SkippedKeyKey {
            dh_pub: msg.header.dh_pub.to_bytes(),
            n: msg.header.n,
        }) {
            return aead_decrypt(&mk, &msg.ciphertext, &header_bytes, msg.header.n);
        }

        self.skip_message_keys(msg.header.dh_pub, msg.header.n)?;

        if self.dhr.as_ref() != Some(&msg.header.dh_pub) {
            self.skip_message_keys_current_chain(msg.header.pn)?;
            self.dh_ratchet(msg.header.dh_pub)?;
        }

        self.skip_message_keys_current_chain(msg.header.n)?;

        let ckr = self.ckr.as_mut().ok_or(RatchetError::DecryptionFailed)?;
        let (next_ckr, mk) = kdf_ck(ckr);
        *ckr = next_ckr;
        self.nr += 1;

        aead_decrypt(&mk, &msg.ciphertext, &header_bytes, msg.header.n)
    }

    fn dh_ratchet(&mut self, remote_dh_pub: PublicKey) -> Result<(), RatchetError> {
        self.pn = self.ns;
        self.ns = 0;
        self.nr = 0;
        self.dhr = Some(remote_dh_pub);

        let dh_out = self.dhs.diffie_hellman(&remote_dh_pub);
        let (next_rk, ckr) = kdf_rk(&self.rk, dh_out.as_bytes());
        self.rk = next_rk;
        self.ckr = Some(ckr);

        self.dhs = StaticSecret::random_from_rng(OsRng);
        let dh_out_new = self.dhs.diffie_hellman(&remote_dh_pub);
        let (next_rk2, cks) = kdf_rk(&self.rk, dh_out_new.as_bytes());
        self.rk = next_rk2;
        self.cks = Some(cks);

        Ok(())
    }

    fn skip_message_keys(
        &mut self,
        remote_dh_pub: PublicKey,
        until_n: u32,
    ) -> Result<(), RatchetError> {
        if self.dhr.as_ref() != Some(&remote_dh_pub) {
            return Ok(());
        }
        self.skip_message_keys_current_chain(until_n)
    }

    fn skip_message_keys_current_chain(&mut self, until_n: u32) -> Result<(), RatchetError> {
        if self.ckr.is_none() {
            return Ok(());
        }

        if self.nr + (MAX_SKIPPED_KEYS as u32) < until_n {
            return Err(RatchetError::TooManySkippedKeys);
        }

        while self.nr < until_n {
            let ckr = self.ckr.as_mut().unwrap();
            let (next_ckr, mk) = kdf_ck(ckr);
            *ckr = next_ckr;

            if let Some(dhr) = self.dhr {
                if self.mkskipped.len() >= MAX_SKIPPED_KEYS {
                    return Err(RatchetError::TooManySkippedKeys);
                }
                self.mkskipped.insert(
                    SkippedKeyKey {
                        dh_pub: dhr.to_bytes(),
                        n: self.nr,
                    },
                    mk,
                );
            }
            self.nr += 1;
        }
        Ok(())
    }
}

fn kdf_rk(rk: &[u8; 32], dh_out: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let hk = Hkdf::<Sha256>::new(Some(rk), dh_out);
    let mut next_rk = [0u8; 32];
    let mut cks_or_ckr = [0u8; 32];

    hk.expand(ROOT_INFO, &mut next_rk).unwrap();
    hk.expand(CHAIN_INFO, &mut cks_or_ckr).unwrap();

    (next_rk, cks_or_ckr)
}

fn kdf_ck(ck: &[u8; 32]) -> ([u8; 32], SymmetricKey) {
    let hk = Hkdf::<Sha256>::new(Some(PRK_INFO), ck);
    let mut next_ck = [0u8; 32];
    let mut mk = [0u8; 32];

    hk.expand(CHAIN_INFO, &mut next_ck).unwrap();
    hk.expand(MSG_KEY_INFO, &mut mk).unwrap();

    (next_ck, SymmetricKey(mk))
}

fn header_ad(header: &Header, ad: &[u8]) -> Vec<u8> {
    let mut res = Vec::with_capacity(32 + 4 + 4 + ad.len());
    res.extend_from_slice(header.dh_pub.as_bytes());
    res.extend_from_slice(&header.pn.to_le_bytes());
    res.extend_from_slice(&header.n.to_le_bytes());
    res.extend_from_slice(ad);
    res
}

fn aead_encrypt(
    key: &SymmetricKey,
    plaintext: &[u8],
    ad: &[u8],
    sequence_number: u32,
) -> Result<Vec<u8>, RatchetError> {
    let cipher =
        ChaCha20Poly1305::new_from_slice(&key.0).map_err(|_| RatchetError::DecryptionFailed)?;

    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[8..12].copy_from_slice(&sequence_number.to_le_bytes());
    let nonce = Nonce::from_slice(&nonce_bytes);

    let payload = Payload {
        msg: plaintext,
        aad: ad,
    };

    cipher
        .encrypt(nonce, payload)
        .map_err(|_| RatchetError::DecryptionFailed)
}

fn aead_decrypt(
    key: &SymmetricKey,
    ciphertext: &[u8],
    ad: &[u8],
    sequence_number: u32,
) -> Result<Vec<u8>, RatchetError> {
    let cipher =
        ChaCha20Poly1305::new_from_slice(&key.0).map_err(|_| RatchetError::DecryptionFailed)?;

    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[8..12].copy_from_slice(&sequence_number.to_le_bytes());
    let nonce = Nonce::from_slice(&nonce_bytes);

    let payload = Payload {
        msg: ciphertext,
        aad: ad,
    };

    cipher
        .decrypt(nonce, payload)
        .map_err(|_| RatchetError::DecryptionFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_ratchet_basic_exchange() {
        let shared_secret = [42u8; 32];
        let anek_dh = StaticSecret::random_from_rng(OsRng);
        let anek_dh_pub = PublicKey::from(&anek_dh);

        let mut ventie = DoubleRatchet::new_ventie(shared_secret, anek_dh_pub);
        let mut anek = DoubleRatchet::new_anek(shared_secret, anek_dh);

        let ad = b"context-ad";

        let msg1 = ventie.encrypt(b"Hello anek", ad).unwrap();
        let decrypted1 = anek.decrypt(&msg1, ad).unwrap();
        assert_eq!(decrypted1, b"Hello anek");

        let msg2 = anek.encrypt(b"Hello ventie", ad).unwrap();
        let decrypted2 = ventie.decrypt(&msg2, ad).unwrap();
        assert_eq!(decrypted2, b"Hello ventie");
    }

    #[test]
    fn test_out_of_order_messages() {
        let shared_secret = [99u8; 32];
        let anek_dh = StaticSecret::random_from_rng(OsRng);
        let anek_dh_pub = PublicKey::from(&anek_dh);

        let mut ventie = DoubleRatchet::new_ventie(shared_secret, anek_dh_pub);
        let mut anek = DoubleRatchet::new_anek(shared_secret, anek_dh);

        let ad = b"test";

        let msg1 = ventie.encrypt(b"Message 1", ad).unwrap();
        let msg2 = ventie.encrypt(b"Message 2", ad).unwrap();

        let dec2 = anek.decrypt(&msg2, ad).unwrap();
        assert_eq!(dec2, b"Message 2");

        let dec1 = anek.decrypt(&msg1, ad).unwrap();
        assert_eq!(dec1, b"Message 1");
    }
}
