use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hkdf::Hkdf;
use ml_kem::kem::{Decapsulate, DecapsulationKey, Encapsulate, EncapsulationKey};
use ml_kem::{Ciphertext, EncodedSizeUser, KemCore, MlKem768, MlKem768Params};
use rand_core::OsRng;
use sha2::Sha256;
use thiserror::Error;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret};
use zeroize::{Zeroize, ZeroizeOnDrop};

pub const X25519_PK_SIZE: usize = 32;
pub const ML_KEM_PK_SIZE: usize = 1184;
pub const ML_KEM_CT_SIZE: usize = 1088;
pub const SHARED_SECRET_SIZE: usize = 32;
pub const ED25519_SIG_SIZE: usize = 64;

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
}

pub struct InitiatorOutput {
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
        }
    }

    pub fn generate_init_payload(&self, signing_key: &SigningKey) -> InitiatorOutput {
        let x25519_bytes = self.x25519_public.as_bytes();
        let ml_kem_bytes = self.ml_kem_encapskey.as_bytes();

        let mut transcript = Vec::new();
        transcript.extend_from_slice(b"Marrow-Handshake-Init");
        transcript.extend_from_slice(x25519_bytes);
        transcript.extend_from_slice(ml_kem_bytes.as_slice());

        let signature = signing_key.sign(&transcript);

        InitiatorOutput {
            x25519_public: *x25519_bytes,
            ml_kem_public: ml_kem_bytes.as_slice().to_vec(),
            signature: signature.to_bytes(),
        }
    }

    pub fn process_response(
        self,
        responder_signing_key_bytes: &[u8; 32],
        responder_x25519_pk_bytes: &[u8; X25519_PK_SIZE],
        ml_kem_ct_bytes: &[u8],
        responder_signature_bytes: &[u8; ED25519_SIG_SIZE],
    ) -> Result<MasterSecret, HandshakeError> {
        if ml_kem_ct_bytes.len() != ML_KEM_CT_SIZE {
            return Err(HandshakeError::InvalidMlKemCiphertextLength);
        }

        let mut transcript = Vec::new();
        transcript.extend_from_slice(b"Marrow-Handshake-Response");
        transcript.extend_from_slice(responder_x25519_pk_bytes);
        transcript.extend_from_slice(ml_kem_ct_bytes);

        let verifying_key = VerifyingKey::from_bytes(responder_signing_key_bytes)
            .map_err(|_| HandshakeError::InvalidSignature)?;
        let signature = Signature::from_bytes(responder_signature_bytes);

        verifying_key
            .verify(&transcript, &signature)
            .map_err(|_| HandshakeError::InvalidSignature)?;

        let responder_x25519_pk = X25519PublicKey::from(*responder_x25519_pk_bytes);
        let x25519_dh_secret = self.x25519_secret.diffie_hellman(&responder_x25519_pk);

        let ct_array: &[u8; ML_KEM_CT_SIZE] = ml_kem_ct_bytes
            .try_into()
            .map_err(|_| HandshakeError::InvalidMlKemCiphertextLength)?;

        let ciphertext = Ciphertext::<MlKem768>::from(*ct_array);

        let ml_kem_secret = self
            .ml_kem_decapskey
            .decapsulate(&ciphertext)
            .map_err(|_| HandshakeError::DecapsulationFailed)?;

        derive_master_secret(x25519_dh_secret.as_bytes(), ml_kem_secret.as_slice())
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
        initiator_verifying_key_bytes: &[u8; 32],
        initiator_x25519_pk_bytes: &[u8; X25519_PK_SIZE],
        initiator_ml_kem_pk_bytes: &[u8],
        initiator_signature_bytes: &[u8; ED25519_SIG_SIZE],
    ) -> Result<ResponderOutput, HandshakeError> {
        if initiator_ml_kem_pk_bytes.len() != ML_KEM_PK_SIZE {
            return Err(HandshakeError::InvalidMlKemKeyLength);
        }

        let mut init_transcript = Vec::new();
        init_transcript.extend_from_slice(b"Marrow-Handshake-Init");
        init_transcript.extend_from_slice(initiator_x25519_pk_bytes);
        init_transcript.extend_from_slice(initiator_ml_kem_pk_bytes);

        let init_verifying_key = VerifyingKey::from_bytes(initiator_verifying_key_bytes)
            .map_err(|_| HandshakeError::InvalidSignature)?;
        let init_signature = Signature::from_bytes(initiator_signature_bytes);

        init_verifying_key
            .verify(&init_transcript, &init_signature)
            .map_err(|_| HandshakeError::InvalidSignature)?;

        let my_x25519_secret = StaticSecret::random_from_rng(OsRng);
        let my_x25519_public = X25519PublicKey::from(&my_x25519_secret);

        let initiator_x25519_pk = X25519PublicKey::from(*initiator_x25519_pk_bytes);
        let x25519_dh_secret = my_x25519_secret.diffie_hellman(&initiator_x25519_pk);

        let pk_bytes: &[u8; ML_KEM_PK_SIZE] = initiator_ml_kem_pk_bytes
            .try_into()
            .map_err(|_| HandshakeError::InvalidMlKemKeyLength)?;

        let initiator_ml_kem_pk = EncapsulationKey::<MlKem768Params>::from_bytes(pk_bytes.into());

        let (ml_kem_ct, ml_kem_secret) = initiator_ml_kem_pk
            .encapsulate(&mut OsRng)
            .map_err(|_| HandshakeError::KdfFailed)?;

        let master_secret =
            derive_master_secret(x25519_dh_secret.as_bytes(), ml_kem_secret.as_slice())?;

        let mut resp_transcript = Vec::new();
        resp_transcript.extend_from_slice(b"Marrow-Handshake-Response");
        resp_transcript.extend_from_slice(my_x25519_public.as_bytes());
        resp_transcript.extend_from_slice(ml_kem_ct.as_slice());

        let signature = signing_key.sign(&resp_transcript);

        Ok(ResponderOutput {
            x25519_secret: my_x25519_secret,
            x25519_public: *my_x25519_public.as_bytes(),
            ml_kem_ciphertext: ml_kem_ct.as_slice().to_vec(),
            signature: signature.to_bytes(),
            master_secret,
        })
    }
}

fn derive_master_secret(
    x25519_ss: &[u8],
    ml_kem_ss: &[u8],
) -> Result<MasterSecret, HandshakeError> {
    let mut ikm = Vec::with_capacity(x25519_ss.len() + ml_kem_ss.len());
    ikm.extend_from_slice(x25519_ss);
    ikm.extend_from_slice(ml_kem_ss);

    let hk = Hkdf::<Sha256>::new(Some(b"Marrow-PQC-Hybrid-Handshake-v1"), &ikm);
    let mut okm = [0u8; SHARED_SECRET_SIZE];

    let result = hk
        .expand(b"master secret", &mut okm)
        .map_err(|_| HandshakeError::KdfFailed);

    ikm.zeroize();
    result.map(|_| MasterSecret(okm))
}