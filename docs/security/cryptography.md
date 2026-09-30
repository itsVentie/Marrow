# Cryptography

**Status:** Experimental / cryptographic inventory and hardening requirements

This document describes the cryptographic building blocks used or intended by Marrow. It is not a claim that the complete protocol is already secure.

## 1. Cryptographic inventory

| Primitive / mechanism | Intended role | Status |
|---|---|---|
| Ed25519 | Long-term application identity and signatures | Implemented; handshake authentication integration pending |
| X25519 | Elliptic-curve Diffie-Hellman | Implemented |
| ML-KEM-768 | Post-quantum key encapsulation | Implemented / integration hardening pending |
| XChaCha20-Poly1305 | Authenticated encryption for local storage | Implemented |
| Argon2id | Password-based key derivation | Implemented |
| BIP39 | Recovery mnemonic generation | Implemented |
| Double Ratchet | Per-session key evolution | Substantial primitive implementation; application integration requires verification |
| HKDF / KDF construction | Key derivation | Used where applicable; protocol transcript binding requires completion |

## 2. Identity keys

Ed25519 is used for long-term application identity material.

Identity keys must not be treated as interchangeable with:

- X25519 ephemeral keys;
- ML-KEM keys;
- ratchet keys;
- storage encryption keys.

The protocol must explicitly define how identity possession authenticates an ephemeral handshake.

## 3. Hybrid handshake

The intended handshake combines:

```text
Classical:
    X25519

Post-quantum:
    ML-KEM-768

Identity authentication:
    Ed25519
```

Hybrid key agreement does not automatically provide identity authentication.

The final protocol should authenticate a canonical transcript containing all security-relevant handshake parameters, including:

- protocol version;
- cipher suite;
- roles;
- application identities;
- relevant X25519 public keys;
- ML-KEM public/ciphertext material;
- transcript context.

The resulting transcript hash should be incorporated into the key derivation process.

## 4. Critical handshake requirement

The responder must use the same X25519 private key corresponding to the public key advertised during the handshake.

The current implementation has a known integration risk where handshake processing can generate one responder ephemeral keypair while the Double Ratchet responder initialization receives another private key.

Until this is fixed and tested, the handshake-to-ratchet security property must not be considered verified.

## 5. Double Ratchet

The Double Ratchet maintains evolving cryptographic state rather than reusing one long-lived message key.

Important state includes:

- root key;
- sending chain key;
- receiving chain key;
- DH ratchet keys;
- message counters;
- previous-chain length;
- skipped message keys.

Security requirements include:

- unique message-key derivation;
- correct send/receive role separation;
- authenticated headers;
- correct DH ratchet transitions;
- bounded skipped-key storage;
- safe deletion of consumed message keys;
- persistence consistency across restart.

Raw key material should be zeroized where practical.

## 6. AEAD

Authenticated encryption should provide both:

- confidentiality;
- ciphertext integrity.

Nonce handling must follow the exact requirements of the selected primitive.

The protocol should avoid redundant or ambiguous nonce fields.

The wire format should have one authoritative source of ratchet header information and one authoritative source of AEAD parameters.

## 7. Local storage encryption

Persistent records are encrypted using XChaCha20-Poly1305 with fresh random nonces.

The storage encryption key is distinct from:

- identity signing keys;
- handshake keys;
- ratchet keys.

Storage encryption protects database records at rest but does not automatically protect secondary data surfaces such as the Tantivy search index.

## 8. Password-derived keys

Argon2id is intended for password-based key derivation.

Parameters must be versioned and documented.

Changing Argon2id parameters in the future must not silently make old encrypted data undecryptable. Stored records therefore need enough version/context information to select the correct derivation parameters.

## 9. BIP39 and key derivation

BIP39 is used for mnemonic generation and recovery material.

The current identity derivation must not be described as a generic BIP32 wallet derivation unless the implementation actually follows the corresponding BIP32 path and semantics.

Cryptographic documentation should describe the exact implemented derivation function rather than relying on terminology that implies additional standards.

## 10. Randomness

Security-sensitive randomness must come from a cryptographically secure operating-system-backed RNG through the Rust cryptographic APIs.

Randomness is required for:

- ephemeral key generation;
- ML-KEM key generation;
- storage nonces;
- other protocol randomness.

Deterministic or application-level pseudo-random generators must not be substituted for security-sensitive randomness.

## 11. Key lifecycle

Sensitive key material should have explicit lifecycle rules:

```text
Generate
   |
   v
Use
   |
   v
Rotate / Ratchet
   |
   v
Expire
   |
   v
Zeroize / Release
```

Required hardening includes:

- zeroization of temporary key buffers;
- storage-key cleanup on logout;
- cleanup of ratchet state;
- cleanup of pending handshakes;
- identity-key lifecycle review;
- minimizing plaintext/key lifetime in memory;
- avoiding accidental logging or serialization of secrets.

## 12. Invalid-key handling

X25519 and other key-agreement inputs require explicit validation and handling of invalid or unacceptable shared-secret outcomes.

Tests should cover:

- malformed public keys;
- low-order / invalid inputs where applicable;
- all-zero shared-secret conditions where applicable;
- malformed ML-KEM material;
- truncated ciphertexts;
- incorrect transcript context.

The implementation must fail closed.

## 13. Cryptographic agility

Protocol versioning should permit migration when:

- a primitive is deprecated;
- a library changes its API/security assumptions;
- a cryptographic weakness is discovered;
- a post-quantum algorithm needs replacement;
- protocol parameters require adjustment.

Algorithm identifiers and protocol versions should be authenticated as part of the handshake transcript.

## 14. What cryptography does not solve

Cryptography alone does not protect against:

- compromised endpoints;
- malicious frontend code;
- insecure IPC;
- plaintext logs;
- insecure search indexes;
- memory disclosure;
- filesystem metadata;
- traffic analysis;
- denial of service;
- compromised operating systems.

These require separate system and application security controls.

## 15. Review standard

Any change to:

- handshake construction;
- KDF input;
- nonce construction;
- ratchet state transitions;
- key derivation;
- identity authentication;
- serialization of cryptographic state;

should receive focused security review and regression tests.

Security-sensitive code should prefer explicit, auditable constructions over implicit behavior.
