# Security Properties

**Status:** Experimental / verification matrix

This document records security properties as implementation and verification status rather than treating the presence of a cryptographic primitive as proof of an end-to-end guarantee.

## 1. Property matrix

| Property | Current status | Notes |
|---|---|---|
| Message confidentiality | Partial | Depends on complete handshake and application ratchet integration |
| Message integrity | Implemented at primitive level | AEAD/ratcher primitives exist; application path still requires full verification |
| Identity authentication | Not complete | Identity must be cryptographically bound to the handshake transcript |
| Forward secrecy | Partial | Double Ratchet primitives support the required model; complete application integration requires verification |
| Post-compromise security | Partial | Requires verified DH-ratchet integration and key lifecycle handling |
| Relay content confidentiality | Intended | Relay should transport ciphertext without message plaintext |
| Relay metadata confidentiality | Not provided | Relay necessarily handles routing/transport metadata |
| Local-at-rest encryption | Implemented | Persistent records use authenticated encryption |
| Search-index confidentiality | Pending | Tantivy is a separate plaintext data surface and requires protection |
| Replay resistance | Partial | Requires complete protocol-level and application-level tests |
| Out-of-order delivery | Primitive support | Ratchet skipped-key handling exists; full network/storage path requires testing |
| Key zeroization | Partial | Several key lifetimes still require audit and hardening |
| Memory locking | Planned | OS memory locking is not currently assumed |
| Traffic-analysis resistance | Limited | Padding/jitter reduce selected leakage but do not provide anonymity |
| Anonymity | Not provided | No onion-routing or equivalent anonymity system |
| Endpoint compromise resistance | Not provided | Compromised endpoints can expose plaintext and key material |
| Protocol versioning | Planned / evolving | Version negotiation and compatibility rules require completion |
| Cryptographic agility | Planned | Algorithm/suite migration needs explicit protocol support |

## 2. Confidentiality

Message confidentiality depends on the complete cryptographic path:

```text
Authenticated handshake
        |
        v
Initial shared state
        |
        v
Double Ratchet
        |
        v
Per-message AEAD
```

A correct individual primitive is insufficient if the surrounding key agreement or identity binding is incorrect.

## 3. Integrity and authenticity

AEAD provides ciphertext integrity under correct key and nonce usage.

This is distinct from **peer identity authentication**. A valid AEAD ciphertext proves possession of the corresponding cryptographic state; it does not automatically establish which human or application identity controls the peer.

The handshake therefore requires explicit Ed25519 authentication and transcript binding.

## 4. Forward secrecy

The Double Ratchet design contains:

- root-key evolution;
- sending and receiving chains;
- DH ratchet state;
- message counters;
- skipped-message-key handling.

These primitives are intended to provide forward secrecy when correctly integrated and when ephemeral/DH keys are securely handled.

The application-level property remains subject to integration testing.

## 5. Local storage protection

Persistent records are encrypted with authenticated encryption before being stored in the local database.

This protects database contents against straightforward offline inspection.

It does not automatically protect:

- plaintext held in process memory;
- search indexes;
- logs;
- crash dumps;
- screenshots;
- compromised operating systems;
- already-decrypted application state.

## 6. Relay security property

The relay is an untrusted transport component.

Its security role is therefore limited to forwarding and temporarily storing protocol frames. End-to-end cryptographic keys should remain at endpoints.

A malicious relay can nevertheless deny service and observe metadata.

## 7. Verification requirements

Before a property is promoted from "partial" to "verified", the repository should contain reproducible tests covering at least:

- normal handshake;
- authenticated identity verification;
- successful message encryption/decryption;
- out-of-order delivery;
- duplicated frames;
- replay;
- dropped frames;
- delayed frames;
- DH ratchet;
- endpoint restart;
- persisted ratchet state;
- invalid public keys;
- malformed frames;
- authentication failure;
- corrupted storage;
- relay/client restart.

## 8. Claiming policy

README and documentation should use conservative terminology:

- **Implemented and verified** — implementation exists and has reproducible verification.
- **Partially implemented** — implementation exists but an important integration or hardening step remains.
- **Planned** — design intent only.
- **Blocked** — dependent on an unresolved security or architectural issue.

No security property should be advertised solely because a dependency or cryptographic primitive exists in the source tree.
