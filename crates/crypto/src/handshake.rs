use hkdf::Hkdf;
use ml_kem::kem::{Decapsulate, DecapsulationKey, Encapsulate, EncapsulationKey};
use ml_kem::{Ciphertext, EncodedSizeUser, KemCore, MlKem768, MlKem768Params};
use rand_core::OsRng;
use sha2::Sha256;
use thiserror::Error;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey};
use zeroize::ZeroizeOnDrop;

pub const X25519_PK_SIZE: usize = 32;
pub const ML_KEM_PK_SIZE: usize = 1184;
pub const ML_KEM_CT_SIZE: usize = 1088;
pub const SHARED_SECRET_SIZE: usize = 32;

#[derive(Error, Debug)]
pub enum HandshakeError {
    #[error("Invalid X25519 public key length")]
    InvalidX25519KeyLength,

    #[error("Invalid ML-KEM public key length")]
    InvalidMlKemKeyLength,

    #[error("Invalid ML-KEM ciphertext length")]
    InvalidMlKemCiphertextLength,

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

    pub fn generate_init_payload(&self) -> InitiatorOutput {
        InitiatorOutput {
            x25519_public: *self.x25519_public.as_bytes(),
            ml_kem_public: self.ml_kem_encapskey.as_bytes().as_slice().to_vec(),
        }
    }

    pub fn process_response(
        self,
        responder_x25519_pk_bytes: &[u8; X25519_PK_SIZE],
        ml_kem_ct_bytes: &[u8],
    ) -> Result<MasterSecret, HandshakeError> {
        if ml_kem_ct_bytes.len() != ML_KEM_CT_SIZE {
            return Err(HandshakeError::InvalidMlKemCiphertextLength);
        }

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
    pub x25519_public: [u8; X25519_PK_SIZE],
    pub ml_kem_ciphertext: Vec<u8>,
    pub master_secret: MasterSecret,
}

pub struct HandshakeResponder;

impl HandshakeResponder {
    pub fn process_init_and_respond(
        initiator_x25519_pk_bytes: &[u8; X25519_PK_SIZE],
        initiator_ml_kem_pk_bytes: &[u8],
    ) -> Result<ResponderOutput, HandshakeError> {
        if initiator_ml_kem_pk_bytes.len() != ML_KEM_PK_SIZE {
            return Err(HandshakeError::InvalidMlKemKeyLength);
        }

        let my_x25519_secret = EphemeralSecret::random_from_rng(OsRng);
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

        Ok(ResponderOutput {
            x25519_public: *my_x25519_public.as_bytes(),
            ml_kem_ciphertext: ml_kem_ct.as_slice().to_vec(),
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

    hk.expand(b"master secret", &mut okm)
        .map_err(|_| HandshakeError::KdfFailed)?;

    Ok(MasterSecret(okm))
}
