pub mod event_loop;

use std::fmt::Display;

#[inline]
pub fn map_err_str<E: Display>(err: E) -> String {
    err.to_string()
}

pub fn pubkey_hex_to_peer_id(pubkey_hex: &str) -> Result<libp2p::PeerId, String> {
    let bytes = hex::decode(pubkey_hex).map_err(map_err_str)?;
    let ed25519_pk = libp2p::identity::ed25519::PublicKey::try_from_bytes(&bytes)
        .map_err(|_| "Invalid Ed25519 public key bytes")?;
    let public_key = libp2p::identity::PublicKey::from(ed25519_pk);
    Ok(public_key.to_peer_id())
}

pub fn derive_network_keypair(identity: &r_crypto::Identity) -> Result<libp2p::identity::Keypair, String> {
    let mut secret_bytes = identity.secret_bytes();
    let ed25519_sk = libp2p::identity::ed25519::SecretKey::try_from_bytes(&mut secret_bytes)
        .map_err(|_| "Failed to derive network keypair")?;
    let keypair = libp2p::identity::Keypair::from(libp2p::identity::ed25519::Keypair::from(ed25519_sk));
    Ok(keypair)
}

pub fn parse_peer_pk_array(pubkey_hex: &str) -> Result<[u8; 32], String> {
    let bytes = hex::decode(pubkey_hex).map_err(map_err_str)?;
    let array: [u8; 32] = bytes
        .try_into()
        .map_err(|_| "Public key hex must be 32 bytes".to_string())?;
    Ok(array)
}