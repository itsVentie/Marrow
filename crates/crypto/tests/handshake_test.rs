use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use r_crypto::handshake::{HandshakeInitiator, HandshakeResponder};

#[test]
fn test_handshake() {
    let initiator_sk = SigningKey::generate(&mut OsRng);
    let responder_sk = SigningKey::generate(&mut OsRng);

    let responder_pubkey = *responder_sk.verifying_key().as_bytes();
    let initiator_pubkey = *initiator_sk.verifying_key().as_bytes();

    let mut initiator = HandshakeInitiator::new();

    let init_out =
        initiator.generate_init_payload(&initiator_sk, &responder_pubkey);

    let responder_output = HandshakeResponder::process_init_and_respond(
        &responder_sk,
        &initiator_pubkey,
        &init_out.recipient_pubkey,
        &init_out.x25519_public,
        &init_out.ml_kem_public,
        &init_out.signature,
    )
    .unwrap();

    let master_secret = initiator
        .process_response(
            responder_sk.verifying_key().as_bytes(),
            &init_out.recipient_pubkey,
            &responder_output.x25519_public,
            &responder_output.ml_kem_ciphertext,
            &responder_output.signature,
        )
        .unwrap();

    assert_eq!(
        master_secret.0,
        responder_output.master_secret.0
    );
}