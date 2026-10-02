use ed25519_dalek::SigningKey;
use r_crypto::handshake::{
    HandshakeError, HandshakeInitiator, HandshakeResponder, InitiatorOutput, ResponderOutput,
};
use rand::rngs::OsRng;

struct HandshakeFixture {
    initiator: HandshakeInitiator,
    initiator_pubkey: [u8; 32],
    responder_pubkey: [u8; 32],
    responder_sk: SigningKey,
    init: InitiatorOutput,
    response: ResponderOutput,
}

fn build_handshake() -> HandshakeFixture {
    let initiator_sk = SigningKey::generate(&mut OsRng);
    let responder_sk = SigningKey::generate(&mut OsRng);

    let initiator_pubkey = *initiator_sk.verifying_key().as_bytes();
    let responder_pubkey = *responder_sk.verifying_key().as_bytes();

    let mut initiator = HandshakeInitiator::new();

    let init = initiator.generate_init_payload(&initiator_sk, &responder_pubkey);

    let response = HandshakeResponder::process_init_and_respond(
        &responder_sk,
        &initiator_pubkey,
        &init.recipient_pubkey,
        &init.x25519_public,
        &init.ml_kem_public,
        &init.signature,
    )
    .expect("Responder must accept a valid handshake init");

    HandshakeFixture {
        initiator,
        initiator_pubkey,
        responder_pubkey,
        responder_sk,
        init,
        response,
    }
}

#[test]
fn test_authenticated_handshake_roundtrip() {
    let HandshakeFixture {
        initiator,
        initiator_pubkey,
        responder_pubkey,
        response,
        ..
    } = build_handshake();

    let initiator_secret = initiator
        .process_response(
            &responder_pubkey,
            &initiator_pubkey,
            &response.x25519_public,
            &response.ml_kem_ciphertext,
            &response.signature,
        )
        .expect("Initiator must accept a valid handshake response");

    assert_eq!(
        initiator_secret.0, response.master_secret.0,
        "Initiator and responder must derive the same master secret"
    );
}

#[test]
fn test_wrong_init_recipient_rejected() {
    let HandshakeFixture {
        initiator_pubkey,
        responder_sk,
        init,
        ..
    } = build_handshake();

    let wrong_recipient = [0xA5; 32];

    let result = HandshakeResponder::process_init_and_respond(
        &responder_sk,
        &initiator_pubkey,
        &wrong_recipient,
        &init.x25519_public,
        &init.ml_kem_public,
        &init.signature,
    );

    assert!(matches!(result, Err(HandshakeError::RecipientMismatch)));
}

#[test]
fn test_init_sender_identity_substitution_rejected() {
    let HandshakeFixture {
        responder_sk, init, ..
    } = build_handshake();

    let attacker = SigningKey::generate(&mut OsRng);
    let attacker_pubkey = *attacker.verifying_key().as_bytes();

    let result = HandshakeResponder::process_init_and_respond(
        &responder_sk,
        &attacker_pubkey,
        &init.recipient_pubkey,
        &init.x25519_public,
        &init.ml_kem_public,
        &init.signature,
    );

    assert!(matches!(result, Err(HandshakeError::InvalidSignature)));
}

#[test]
fn test_init_x25519_tampering_rejected() {
    let HandshakeFixture {
        initiator_pubkey,
        responder_sk,
        init,
        ..
    } = build_handshake();

    let mut tampered_x25519 = init.x25519_public;
    tampered_x25519[0] ^= 0x01;

    let result = HandshakeResponder::process_init_and_respond(
        &responder_sk,
        &initiator_pubkey,
        &init.recipient_pubkey,
        &tampered_x25519,
        &init.ml_kem_public,
        &init.signature,
    );

    assert!(matches!(result, Err(HandshakeError::InvalidSignature)));
}

#[test]
fn test_init_ml_kem_tampering_rejected() {
    let HandshakeFixture {
        initiator_pubkey,
        responder_sk,
        init,
        ..
    } = build_handshake();

    let mut tampered_ml_kem = init.ml_kem_public.clone();
    tampered_ml_kem[0] ^= 0x01;

    let result = HandshakeResponder::process_init_and_respond(
        &responder_sk,
        &initiator_pubkey,
        &init.recipient_pubkey,
        &init.x25519_public,
        &tampered_ml_kem,
        &init.signature,
    );

    assert!(matches!(result, Err(HandshakeError::InvalidSignature)));
}

#[test]
fn test_init_malformed_ml_kem_key_length_rejected() {
    let HandshakeFixture {
        initiator_pubkey,
        responder_sk,
        init,
        ..
    } = build_handshake();

    let malformed_ml_kem = &init.ml_kem_public[..init.ml_kem_public.len() - 1];

    let result = HandshakeResponder::process_init_and_respond(
        &responder_sk,
        &initiator_pubkey,
        &init.recipient_pubkey,
        &init.x25519_public,
        malformed_ml_kem,
        &init.signature,
    );

    assert!(matches!(result, Err(HandshakeError::InvalidMlKemKeyLength)));
}

#[test]
fn test_init_malformed_signature_rejected() {
    let HandshakeFixture {
        initiator_pubkey,
        responder_sk,
        init,
        ..
    } = build_handshake();

    let malformed_signature = &init.signature[..63];

    let result = HandshakeResponder::process_init_and_respond(
        &responder_sk,
        &initiator_pubkey,
        &init.recipient_pubkey,
        &init.x25519_public,
        &init.ml_kem_public,
        malformed_signature,
    );

    assert!(matches!(result, Err(HandshakeError::InvalidSignature)));
}

#[test]
fn test_response_identity_substitution_rejected() {
    let HandshakeFixture {
        initiator,
        initiator_pubkey,
        response,
        ..
    } = build_handshake();

    let attacker = SigningKey::generate(&mut OsRng);
    let attacker_pubkey = *attacker.verifying_key().as_bytes();

    let result = initiator.process_response(
        &attacker_pubkey,
        &initiator_pubkey,
        &response.x25519_public,
        &response.ml_kem_ciphertext,
        &response.signature,
    );

    assert!(matches!(result, Err(HandshakeError::PeerIdentityMismatch)));
}

#[test]
fn test_response_recipient_substitution_rejected() {
    let HandshakeFixture {
        initiator,
        responder_pubkey,
        response,
        ..
    } = build_handshake();

    let wrong_recipient = [0xA5; 32];

    let result = initiator.process_response(
        &responder_pubkey,
        &wrong_recipient,
        &response.x25519_public,
        &response.ml_kem_ciphertext,
        &response.signature,
    );

    assert!(matches!(result, Err(HandshakeError::RecipientMismatch)));
}

#[test]
fn test_response_x25519_tampering_rejected() {
    let HandshakeFixture {
        initiator,
        initiator_pubkey,
        responder_pubkey,
        response,
        ..
    } = build_handshake();

    let mut tampered_x25519 = response.x25519_public;
    tampered_x25519[0] ^= 0x01;

    let result = initiator.process_response(
        &responder_pubkey,
        &initiator_pubkey,
        &tampered_x25519,
        &response.ml_kem_ciphertext,
        &response.signature,
    );

    assert!(matches!(result, Err(HandshakeError::InvalidSignature)));
}

#[test]
fn test_response_ml_kem_ciphertext_tampering_rejected() {
    let HandshakeFixture {
        initiator,
        initiator_pubkey,
        responder_pubkey,
        response,
        ..
    } = build_handshake();

    let mut tampered_ciphertext = response.ml_kem_ciphertext.clone();
    tampered_ciphertext[0] ^= 0x01;

    let result = initiator.process_response(
        &responder_pubkey,
        &initiator_pubkey,
        &response.x25519_public,
        &tampered_ciphertext,
        &response.signature,
    );

    assert!(matches!(result, Err(HandshakeError::InvalidSignature)));
}

#[test]
fn test_response_signature_tampering_rejected() {
    let HandshakeFixture {
        initiator,
        initiator_pubkey,
        responder_pubkey,
        response,
        ..
    } = build_handshake();

    let mut tampered_signature = response.signature;
    tampered_signature[0] ^= 0x01;

    let result = initiator.process_response(
        &responder_pubkey,
        &initiator_pubkey,
        &response.x25519_public,
        &response.ml_kem_ciphertext,
        &tampered_signature,
    );

    assert!(matches!(result, Err(HandshakeError::InvalidSignature)));
}

#[test]
fn test_response_malformed_ciphertext_length_rejected() {
    let HandshakeFixture {
        initiator,
        initiator_pubkey,
        responder_pubkey,
        response,
        ..
    } = build_handshake();

    let malformed_ciphertext = &response.ml_kem_ciphertext[..1087];

    let result = initiator.process_response(
        &responder_pubkey,
        &initiator_pubkey,
        &response.x25519_public,
        malformed_ciphertext,
        &response.signature,
    );

    assert!(matches!(
        result,
        Err(HandshakeError::InvalidMlKemCiphertextLength)
    ));
}

#[test]
fn test_response_malformed_signature_rejected() {
    let HandshakeFixture {
        initiator,
        initiator_pubkey,
        responder_pubkey,
        response,
        ..
    } = build_handshake();

    let malformed_signature = &response.signature[..63];

    let result = initiator.process_response(
        &responder_pubkey,
        &initiator_pubkey,
        &response.x25519_public,
        &response.ml_kem_ciphertext,
        malformed_signature,
    );

    assert!(matches!(result, Err(HandshakeError::InvalidSignature)));
}

#[test]
fn test_invalid_responder_identity_length_rejected() {
    let HandshakeFixture {
        initiator,
        initiator_pubkey,
        response,
        ..
    } = build_handshake();

    let invalid_identity = [0u8; 31];

    let result = initiator.process_response(
        &invalid_identity,
        &initiator_pubkey,
        &response.x25519_public,
        &response.ml_kem_ciphertext,
        &response.signature,
    );

    assert!(matches!(result, Err(HandshakeError::InvalidIdentityKey)));
}
