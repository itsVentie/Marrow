use r_crypto::handshake::{HandshakeInitiator, HandshakeResponder};

#[test]
fn test_successful_hybrid_handshake() {
    let initiator = HandshakeInitiator::new();
    let init_payload = initiator.generate_init_payload();

    let responder_output = HandshakeResponder::process_init_and_respond(
        &init_payload.x25519_public,
        &init_payload.ml_kem_public,
    )
    .expect("Responder processing failed");

    let initiator_master_secret = initiator
        .process_response(
            &responder_output.x25519_public,
            &responder_output.ml_kem_ciphertext,
        )
        .expect("Initiator response processing failed");

    assert_eq!(initiator_master_secret.0, responder_output.master_secret.0);
}
