use r_crypto::handshake::{HandshakeInitiator, HandshakeResponder};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

#[test]
fn test_handshake() {
    let initiator_sk = SigningKey::generate(&mut OsRng);
    let responder_sk = SigningKey::generate(&mut OsRng);

    let initiator = HandshakeInitiator::new();
    let init_out = initiator.generate_init_payload(&initiator_sk);

    let responder_output = HandshakeResponder::process_init_and_respond(
        &responder_sk,
        initiator_sk.verifying_key().as_bytes(),
        &init_out.x25519_public,
        &init_out.ml_kem_public,
        &init_out.signature,
    ).unwrap();

    let master_secret = initiator.process_response(
        responder_sk.verifying_key().as_bytes(),
        &responder_output.x25519_public,
        &responder_output.ml_kem_ciphertext,
        &responder_output.signature,
    ).unwrap();

    assert_eq!(master_secret.0, responder_output.master_secret.0);
}