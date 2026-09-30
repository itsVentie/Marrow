# Hybrid PQC Handshake

**Status:** Experimental / integration and authentication hardening required

## 1. Purpose

The Marrow handshake is intended to establish initial session key material using a hybrid construction combining:

- X25519 for classical Diffie-Hellman;
- ML-KEM-768 for post-quantum key encapsulation;
- Ed25519 for long-term application identity authentication.

The combination is intended to avoid relying on a single key-establishment primitive.

## 2. Conceptual flow

```text
Initiator                         Responder
   |                                 |
   | X25519 / ML-KEM material        |
   |-------------------------------> |
   |                                 |
   | authenticated transcript        |
   |<-------------------------------> |
   |                                 |
   | derive hybrid secret             |
   |                                 |
   +----------> Ratchet <-------------+
```

The exact wire flow must be defined by the versioned protocol specification.

## 3. Cryptographic roles

### Ed25519

Provides long-term identity authentication.

### X25519

Provides classical ephemeral DH material.

### ML-KEM-768

Provides post-quantum KEM material.

### KDF

Combines the independently established secret material into session initialization material.

The KDF input must be domain-separated and bound to the authenticated transcript.

## 4. Transcript authentication

A hybrid key exchange without identity authentication is vulnerable to active identity substitution.

The handshake should authenticate a canonical transcript containing, at minimum:

- protocol version;
- selected cryptographic suite;
- initiator identity;
- responder identity;
- X25519 public keys;
- ML-KEM-related public/ciphertext material;
- roles;
- transcript context.

Ed25519 signatures should authenticate the transcript.

The resulting transcript hash should also be incorporated into session key derivation.

## 5. Identity binding

Application identity should be bound to the network identity where applicable.

For libp2p-based transport this means defining how the application-level identity relates to the libp2p `PeerId` or transport-level authenticated identity.

Without an explicit binding, transport authentication and application authentication remain separate trust domains.

## 6. Critical responder-key issue

The responder currently has an architectural risk in which handshake processing can generate an ephemeral X25519 keypair and advertise its public key while a separate keypair is passed into Double Ratchet initialization.

These must be the same keypair.

Required invariant:

```text
advertised_responder_public
        ==
public(responder_private_used_by_ratchet)
```

This should be enforced by construction rather than by convention.

## 7. Transcript and downgrade protection

The handshake must authenticate:

- protocol version;
- algorithm suite;
- roles;
- all key-agreement inputs.

This prevents an attacker from modifying security-relevant negotiation fields without detection.

Unsupported versions and suites must fail closed.

## 8. Replay and state handling

Handshake processing must define:

- unique session context;
- replay detection;
- timeout;
- duplicate-init behavior;
- simultaneous initiation;
- malformed input behavior;
- failed-authentication cleanup;
- pending-handshake limits.

Handshake state must not remain indefinitely in memory.

## 9. Key derivation

The hybrid secret should not be used directly as a message key.

A domain-separated KDF should derive independent values for:

- initial ratchet state;
- authentication/context binding where required;
- protocol-specific purposes.

Context/version identifiers should be part of the derivation inputs.

## 10. Failure handling

Handshake failure should not reveal sensitive details unnecessarily.

Examples include:

- invalid identity signature;
- invalid key material;
- invalid transcript;
- unsupported version;
- invalid ML-KEM material;
- invalid X25519 result;
- timeout;
- duplicate/replayed handshake.

Failed state should be destroyed or expired.

## 11. Verification requirements

Required tests include:

- successful initiator/responder handshake;
- identity signature verification;
- transcript tampering;
- suite/version tampering;
- identity substitution;
- invalid public keys;
- invalid ML-KEM ciphertext;
- invalid/low-order DH inputs where applicable;
- replay;
- duplicate initiation;
- simultaneous initiation;
- timeout;
- malformed frames;
- responder-key consistency;
- deterministic test vectors where practical.

## 12. Current security status

The primitive composition is substantial, but the complete authenticated handshake should not be considered production-verified until:

1. Ed25519 authentication is bound to the transcript;
2. application identity is bound to transport identity where required;
3. responder keypair consistency is enforced;
4. replay and malformed-input handling are tested;
5. handshake-to-ratchet integration is tested end to end.
