use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hkdf::Hkdf;
use ml_kem::kem::{Decapsulate, DecapsulationKey, Encapsulate, EncapsulationKey};
use ml_kem::{Ciphertext, EncodedSizeUser, KemCore, MlKem768, MlKem768Params};
use rand_core::OsRng;
use sha2::{Digest, Sha256};
use thiserror::Error;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret};
use zeroize::{Zeroize, ZeroizeOnDrop};

pub const X25519_PK_SIZE: usize = 32;
pub const ML_KEM_PK_SIZE: usize = 1184;
pub const ML_KEM_CT_SIZE: usize = 1088;
pub const SHARED_SECRET_SIZE: usize = 32;
pub const ED25519_SIG_SIZE: usize = 64;

pub const HANDSHAKE_VERSION: u8 = 1;
pub const HANDSHAKE_SUITE: &[u8] = b"X25519-MLKEM768-ED25519-HKDF-SHA256";

const INIT_DOMAIN: &[u8] = b"Marrow-Handshake-Init";
const RESPONSE_DOMAIN: &[u8] = b"Marrow-Handshake-Response";

#[derive(Error, Debug)]
pub enum HandshakeError {
    #[error("Invalid X25519 public key length")]
    InvalidX25519KeyLength,

    #[error("Invalid ML-KEM public key length")]
    InvalidMlKemKeyLength,

    #[error("Invalid ML-KEM ciphertext length")]
    InvalidMlKemCiphertextLength,

    #[error("Invalid Ed25519 signature")]
    InvalidSignature,

    #[error("Invalid identity public key")]
    InvalidIdentityKey,

    #[error("Recipient identity mismatch")]
    RecipientMismatch,

    #[error("Peer identity mismatch")]
    PeerIdentityMismatch,

    #[error("Handshake context is not initialized")]
    ContextNotInitialized,

    #[error("Non-contributory X25519 exchange")]
    NonContributoryX25519,

    #[error("Decapsulation failed")]
    DecapsulationFailed,

    #[error("Key derivation failed")]
    KdfFailed,
}

#[derive(ZeroizeOnDrop)]
pub struct MasterSecret(pub [u8; SHARED_SECRET_SIZE]);

pub struct HandshakeInitiator {
    x25519_secret: EphemeralSecret,
    x25519_public: X25519PublicKey,
    ml_kem_decapskey: DecapsulationKey<MlKem768Params>,
    ml_kem_encapskey: EncapsulationKey<MlKem768Params>,

    initiator_identity: Option<[u8; 32]>,
    responder_identity: Option<[u8; 32]>,
}

pub struct InitiatorOutput {
    pub recipient_pubkey: [u8; 32],
    pub x25519_public: [u8; X25519_PK_SIZE],
    pub ml_kem_public: Vec<u8>,
    pub signature: [u8; ED25519_SIG_SIZE],
}

impl Default for HandshakeInitiator {
    fn default() -> Self {
        Self::new()
    }
}

impl HandshakeInitiator {
    pub fn new() -> Self {
        let x25519_secret = EphemeralSecret::random_from_rng(OsRng);
        let x25519_public = X25519PublicKey::from(&x25519_secret);
        let (ml_kem_decapskey, ml_kem_encapskey) = MlKem768::generate(&mut OsRng);

        Self {
            x25519_secret,
            x25519_public,
            ml_kem_decapskey,
            ml_kem_encapskey,
            initiator_identity: None,
            responder_identity: None,
        }
    }

    pub fn generate_init_payload(
        &mut self,
        signing_key: &SigningKey,
        recipient_pubkey: &[u8; 32],
    ) -> InitiatorOutput {
        let initiator_identity = *signing_key.verifying_key().as_bytes();

        self.initiator_identity = Some(initiator_identity);
        self.responder_identity = Some(*recipient_pubkey);

        let x25519_bytes = self.x25519_public.as_bytes();
        let ml_kem_bytes = self.ml_kem_encapskey.as_bytes();

        let transcript = build_init_transcript(
            &initiator_identity,
            recipient_pubkey,
            x25519_bytes,
            ml_kem_bytes.as_slice(),
        );

        let signature = signing_key.sign(&transcript);

        InitiatorOutput {
            recipient_pubkey: *recipient_pubkey,
            x25519_public: *x25519_bytes,
            ml_kem_public: ml_kem_bytes.as_slice().to_vec(),
            signature: signature.to_bytes(),
        }
    }

    pub fn process_response(
        self,
        responder_verifying_key_bytes: &[u8],
        recipient_pubkey_bytes: &[u8; 32],
        responder_x25519_pk_bytes: &[u8; X25519_PK_SIZE],
        ml_kem_ct_bytes: &[u8],
        responder_signature_bytes: &[u8],
    ) -> Result<MasterSecret, HandshakeError> {
        if ml_kem_ct_bytes.len() != ML_KEM_CT_SIZE {
            return Err(HandshakeError::InvalidMlKemCiphertextLength);
        }

        let initiator_identity = self
            .initiator_identity
            .ok_or(HandshakeError::ContextNotInitialized)?;

        let expected_responder_identity = self
            .responder_identity
            .ok_or(HandshakeError::ContextNotInitialized)?;

        let actual_responder_identity: [u8; 32] = responder_verifying_key_bytes
            .try_into()
            .map_err(|_| HandshakeError::InvalidIdentityKey)?;

        if actual_responder_identity != expected_responder_identity {
            return Err(HandshakeError::PeerIdentityMismatch);
        }

        if *recipient_pubkey_bytes != initiator_identity {
            return Err(HandshakeError::RecipientMismatch);
        }

        let verifying_key = VerifyingKey::from_bytes(&actual_responder_identity)
            .map_err(|_| HandshakeError::InvalidIdentityKey)?;

        let sig_bytes: &[u8; ED25519_SIG_SIZE] = responder_signature_bytes
            .try_into()
            .map_err(|_| HandshakeError::InvalidSignature)?;

        let signature = Signature::from_bytes(sig_bytes);

        let initiator_x25519_bytes = self.x25519_public.as_bytes();
        let initiator_ml_kem_bytes = self.ml_kem_encapskey.as_bytes();

        let transcript = build_response_transcript(
            &initiator_identity,
            &expected_responder_identity,
            recipient_pubkey_bytes,
            initiator_x25519_bytes,
            responder_x25519_pk_bytes,
            initiator_ml_kem_bytes.as_slice(),
            ml_kem_ct_bytes,
        );

        verifying_key
            .verify(&transcript, &signature)
            .map_err(|_| HandshakeError::InvalidSignature)?;

        let responder_x25519_pk = X25519PublicKey::from(*responder_x25519_pk_bytes);

        let x25519_dh_secret = self.x25519_secret.diffie_hellman(&responder_x25519_pk);

        if !x25519_dh_secret.was_contributory() {
            return Err(HandshakeError::NonContributoryX25519);
        }

        let ct_array: &[u8; ML_KEM_CT_SIZE] = ml_kem_ct_bytes
            .try_into()
            .map_err(|_| HandshakeError::InvalidMlKemCiphertextLength)?;

        let ciphertext = Ciphertext::<MlKem768>::from(*ct_array);

        let ml_kem_secret = self
            .ml_kem_decapskey
            .decapsulate(&ciphertext)
            .map_err(|_| HandshakeError::DecapsulationFailed)?;

        derive_master_secret(
            x25519_dh_secret.as_bytes(),
            ml_kem_secret.as_slice(),
            &transcript,
        )
    }
}

pub struct ResponderOutput {
    pub x25519_secret: StaticSecret,
    pub x25519_public: [u8; X25519_PK_SIZE],
    pub ml_kem_ciphertext: Vec<u8>,
    pub signature: [u8; ED25519_SIG_SIZE],
    pub master_secret: MasterSecret,
}

pub struct HandshakeResponder;

impl HandshakeResponder {
    pub fn process_init_and_respond(
        signing_key: &SigningKey,
        initiator_verifying_key_bytes: &[u8],
        recipient_pubkey_bytes: &[u8; 32],
        initiator_x25519_pk_bytes: &[u8; X25519_PK_SIZE],
        initiator_ml_kem_pk_bytes: &[u8],
        initiator_signature_bytes: &[u8],
    ) -> Result<ResponderOutput, HandshakeError> {
        if initiator_ml_kem_pk_bytes.len() != ML_KEM_PK_SIZE {
            return Err(HandshakeError::InvalidMlKemKeyLength);
        }

        let initiator_identity: [u8; 32] = initiator_verifying_key_bytes
            .try_into()
            .map_err(|_| HandshakeError::InvalidIdentityKey)?;

        let responder_identity = *signing_key.verifying_key().as_bytes();

        if *recipient_pubkey_bytes != responder_identity {
            return Err(HandshakeError::RecipientMismatch);
        }

        let init_transcript = build_init_transcript(
            &initiator_identity,
            &responder_identity,
            initiator_x25519_pk_bytes,
            initiator_ml_kem_pk_bytes,
        );

        let init_verifying_key = VerifyingKey::from_bytes(&initiator_identity)
            .map_err(|_| HandshakeError::InvalidIdentityKey)?;

        let sig_bytes: &[u8; ED25519_SIG_SIZE] = initiator_signature_bytes
            .try_into()
            .map_err(|_| HandshakeError::InvalidSignature)?;

        let init_signature = Signature::from_bytes(sig_bytes);

        init_verifying_key
            .verify(&init_transcript, &init_signature)
            .map_err(|_| HandshakeError::InvalidSignature)?;

        let my_x25519_secret = StaticSecret::random_from_rng(OsRng);

        let my_x25519_public = X25519PublicKey::from(&my_x25519_secret);

        let initiator_x25519_pk = X25519PublicKey::from(*initiator_x25519_pk_bytes);

        let x25519_dh_secret = my_x25519_secret.diffie_hellman(&initiator_x25519_pk);

        if !x25519_dh_secret.was_contributory() {
            return Err(HandshakeError::NonContributoryX25519);
        }

        let pk_bytes: &[u8; ML_KEM_PK_SIZE] = initiator_ml_kem_pk_bytes
            .try_into()
            .map_err(|_| HandshakeError::InvalidMlKemKeyLength)?;

        let initiator_ml_kem_pk = EncapsulationKey::<MlKem768Params>::from_bytes(pk_bytes.into());

        let (ml_kem_ct, ml_kem_secret) = initiator_ml_kem_pk
            .encapsulate(&mut OsRng)
            .map_err(|_| HandshakeError::KdfFailed)?;

        let response_transcript = build_response_transcript(
            &initiator_identity,
            &responder_identity,
            &responder_identity,
            initiator_x25519_pk_bytes,
            my_x25519_public.as_bytes(),
            initiator_ml_kem_pk_bytes,
            ml_kem_ct.as_slice(),
        );

        let master_secret = derive_master_secret(
            x25519_dh_secret.as_bytes(),
            ml_kem_secret.as_slice(),
            &response_transcript,
        )?;

        let signature = signing_key.sign(&response_transcript);

        Ok(ResponderOutput {
            x25519_secret: my_x25519_secret,
            x25519_public: *my_x25519_public.as_bytes(),
            ml_kem_ciphertext: ml_kem_ct.as_slice().to_vec(),
            signature: signature.to_bytes(),
            master_secret,
        })
    }
}

fn build_init_transcript(
    initiator_identity: &[u8; 32],
    responder_identity: &[u8; 32],
    initiator_x25519: &[u8; 32],
    initiator_ml_kem_pk: &[u8],
) -> Vec<u8> {
    let mut transcript = Vec::with_capacity(
        INIT_DOMAIN.len() + 1 + HANDSHAKE_SUITE.len() + 32 + 32 + 32 + initiator_ml_kem_pk.len(),
    );

    transcript.extend_from_slice(INIT_DOMAIN);
    transcript.push(HANDSHAKE_VERSION);
    transcript.extend_from_slice(HANDSHAKE_SUITE);
    transcript.extend_from_slice(initiator_identity);
    transcript.extend_from_slice(responder_identity);
    transcript.extend_from_slice(initiator_x25519);
    transcript.extend_from_slice(initiator_ml_kem_pk);

    transcript
}

fn build_response_transcript(
    initiator_identity: &[u8; 32],
    responder_identity: &[u8; 32],
    recipient_identity: &[u8; 32],
    initiator_x25519: &[u8; 32],
    responder_x25519: &[u8; 32],
    initiator_ml_kem_pk: &[u8],
    ml_kem_ct: &[u8],
) -> Vec<u8> {
    let mut transcript = Vec::with_capacity(
        RESPONSE_DOMAIN.len()
            + 1
            + HANDSHAKE_SUITE.len()
            + 32
            + 32
            + 32
            + 32
            + 32
            + initiator_ml_kem_pk.len()
            + ml_kem_ct.len(),
    );

    transcript.extend_from_slice(RESPONSE_DOMAIN);
    transcript.push(HANDSHAKE_VERSION);
    transcript.extend_from_slice(HANDSHAKE_SUITE);
    transcript.extend_from_slice(initiator_identity);
    transcript.extend_from_slice(responder_identity);
    transcript.extend_from_slice(recipient_identity);
    transcript.extend_from_slice(initiator_x25519);
    transcript.extend_from_slice(responder_x25519);
    transcript.extend_from_slice(initiator_ml_kem_pk);
    transcript.extend_from_slice(ml_kem_ct);

    transcript
}

fn derive_master_secret(
    x25519_ss: &[u8],
    ml_kem_ss: &[u8],
    transcript: &[u8],
) -> Result<MasterSecret, HandshakeError> {
    let transcript_hash = Sha256::digest(transcript);

    let mut ikm = Vec::with_capacity(x25519_ss.len() + ml_kem_ss.len() + 32);

    ikm.extend_from_slice(x25519_ss);
    ikm.extend_from_slice(ml_kem_ss);
    ikm.extend_from_slice(&transcript_hash);

    let hk = Hkdf::<Sha256>::new(Some(b"Marrow-PQC-Hybrid-Handshake-v1"), &ikm);

    let mut okm = [0u8; SHARED_SECRET_SIZE];

    let result = hk
        .expand(b"master secret", &mut okm)
        .map_err(|_| HandshakeError::KdfFailed);

    ikm.zeroize();

    result.map(|_| MasterSecret(okm))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_roundtrip() {
        let initiator_identity = SigningKey::generate(&mut OsRng);
        let responder_identity = SigningKey::generate(&mut OsRng);

        let responder_pubkey = *responder_identity.verifying_key().as_bytes();

        let mut initiator = HandshakeInitiator::new();

        let init_output = initiator.generate_init_payload(&initiator_identity, &responder_pubkey);

        let initiator_pubkey = *initiator_identity.verifying_key().as_bytes();

        let responder_output = HandshakeResponder::process_init_and_respond(
            &responder_identity,
            &initiator_pubkey,
            &init_output.recipient_pubkey,
            &init_output.x25519_public,
            &init_output.ml_kem_public,
            &init_output.signature,
        )
        .unwrap();

        let responder_secret = responder_output.master_secret.0;

        let initiator_secret = initiator.process_response(
            responder_output
                .signature
                .as_slice()
                .get(0..0)
                .unwrap_or_default(),
            &init_output.recipient_pubkey,
            &responder_output.x25519_public,
            &responder_output.ml_kem_ciphertext,
            &responder_output.signature,
        );

        assert!(initiator_secret.is_err());
        assert_ne!(responder_secret, [0u8; 32]);
    }

    #[test]
    fn test_full_handshake_roundtrip() {
        let initiator_identity = SigningKey::generate(&mut OsRng);
        let responder_identity = SigningKey::generate(&mut OsRng);

        let initiator_pubkey = *initiator_identity.verifying_key().as_bytes();

        let responder_pubkey = *responder_identity.verifying_key().as_bytes();

        let mut initiator = HandshakeInitiator::new();

        let init_output = initiator.generate_init_payload(&initiator_identity, &responder_pubkey);

        let responder_output = HandshakeResponder::process_init_and_respond(
            &responder_identity,
            &initiator_pubkey,
            &init_output.recipient_pubkey,
            &init_output.x25519_public,
            &init_output.ml_kem_public,
            &init_output.signature,
        )
        .unwrap();

        let initiator_secret = initiator
            .process_response(
                &responder_pubkey,
                &init_output.recipient_pubkey,
                &responder_output.x25519_public,
                &responder_output.ml_kem_ciphertext,
                &responder_output.signature,
            )
            .unwrap();

        assert_eq!(initiator_secret.0, responder_output.master_secret.0);
    }

    #[test]
    fn test_wrong_recipient_rejected() {
        let initiator_identity = SigningKey::generate(&mut OsRng);
        let responder_identity = SigningKey::generate(&mut OsRng);
        let wrong_identity = SigningKey::generate(&mut OsRng);

        let responder_pubkey = *responder_identity.verifying_key().as_bytes();

        let wrong_pubkey = *wrong_identity.verifying_key().as_bytes();

        let mut initiator = HandshakeInitiator::new();

        let init_output = initiator.generate_init_payload(&initiator_identity, &wrong_pubkey);

        let initiator_pubkey = *initiator_identity.verifying_key().as_bytes();

        let result = HandshakeResponder::process_init_and_respond(
            &responder_identity,
            &initiator_pubkey,
            &init_output.recipient_pubkey,
            &init_output.x25519_public,
            &init_output.ml_kem_public,
            &init_output.signature,
        );

        assert!(matches!(result, Err(HandshakeError::RecipientMismatch)));

        assert_ne!(wrong_pubkey, responder_pubkey);
    }

    #[test]
    fn test_response_identity_substitution_rejected() {
        let initiator_identity = SigningKey::generate(&mut OsRng);
        let responder_identity = SigningKey::generate(&mut OsRng);
        let attacker_identity = SigningKey::generate(&mut OsRng);

        let responder_pubkey = *responder_identity.verifying_key().as_bytes();

        let attacker_pubkey = *attacker_identity.verifying_key().as_bytes();

        let mut initiator = HandshakeInitiator::new();

        let init_output = initiator.generate_init_payload(&initiator_identity, &responder_pubkey);

        let initiator_pubkey = *initiator_identity.verifying_key().as_bytes();

        let responder_output = HandshakeResponder::process_init_and_respond(
            &responder_identity,
            &initiator_pubkey,
            &init_output.recipient_pubkey,
            &init_output.x25519_public,
            &init_output.ml_kem_public,
            &init_output.signature,
        )
        .unwrap();

        let result = initiator.process_response(
            &attacker_pubkey,
            &init_output.recipient_pubkey,
            &responder_output.x25519_public,
            &responder_output.ml_kem_ciphertext,
            &responder_output.signature,
        );

        assert!(matches!(result, Err(HandshakeError::PeerIdentityMismatch)));
    }

    #[test]
    fn test_init_signature_tampering_rejected() {
        let initiator_identity = SigningKey::generate(&mut OsRng);
        let responder_identity = SigningKey::generate(&mut OsRng);

        let responder_pubkey = *responder_identity.verifying_key().as_bytes();

        let mut initiator = HandshakeInitiator::new();

        let mut init_output =
            initiator.generate_init_payload(&initiator_identity, &responder_pubkey);

        init_output.ml_kem_public[0] ^= 0x01;

        let initiator_pubkey = *initiator_identity.verifying_key().as_bytes();

        let result = HandshakeResponder::process_init_and_respond(
            &responder_identity,
            &initiator_pubkey,
            &init_output.recipient_pubkey,
            &init_output.x25519_public,
            &init_output.ml_kem_public,
            &init_output.signature,
        );

        assert!(matches!(result, Err(HandshakeError::InvalidSignature)));
    }

    #[test]
    fn test_non_contributory_x25519_rejected() {
        let initiator_identity = SigningKey::generate(&mut OsRng);
        let responder_identity = SigningKey::generate(&mut OsRng);

        let initiator_pubkey = *initiator_identity.verifying_key().as_bytes();

        let responder_pubkey = *responder_identity.verifying_key().as_bytes();

        let helper = HandshakeInitiator::new();
        let ml_kem_pk = helper.ml_kem_encapskey.as_bytes();

        let zero_x25519 = [0u8; 32];

        let transcript = build_init_transcript(
            &initiator_pubkey,
            &responder_pubkey,
            &zero_x25519,
            ml_kem_pk.as_slice(),
        );

        let signature = initiator_identity.sign(&transcript);

        let result = HandshakeResponder::process_init_and_respond(
            &responder_identity,
            &initiator_pubkey,
            &responder_pubkey,
            &zero_x25519,
            ml_kem_pk.as_slice(),
            &signature.to_bytes(),
        );

        assert!(matches!(result, Err(HandshakeError::NonContributoryX25519)));
    }
}
