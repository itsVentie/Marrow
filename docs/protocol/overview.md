# Marrow Protocol Overview

## Scope

This directory documents the application protocol used by Marrow for identity, session establishment, encrypted 1-to-1 messaging, relay routing, errors, and protocol versioning.

The documentation separates current implementation from required hardening and planned extensions.

## Protocol layers

```text
Application
  │
  ├── Identity
  │     └── Ed25519
  ├── Session establishment
  │     ├── X25519
  │     ├── ML-KEM-768
  │     └── authenticated transcript
  ├── Session cryptography
  │     └── Double Ratchet
  ├── Message framing
  │     └── protocol frames
  └── Network transport
        ├── libp2p TCP
        ├── libp2p QUIC
        └── experimental Quinn relay
```

The current serialization layer is Serde + Bincode.

## Current cryptographic building blocks

| Function | Primitive | Status |
| --- | --- | --- |
| Application identity | Ed25519 | Implemented |
| Local password protection | Argon2id | Implemented |
| Classical key agreement | X25519 | Implemented |
| Post-quantum KEM | ML-KEM-768 | Implemented |
| Message/session AEAD | XChaCha20-Poly1305 / ratchet AEAD path | Implemented |
| Session state | Double Ratchet | Implemented; integration hardening pending |
| Serialization | Serde + Bincode | Implemented |

## Security boundary

```text
Alice application
   │
   │ authenticated E2E session
   ▼
Encrypted application message
   │
   │ opaque transport / relay
   ▼
Bob application
```

The relay is not intended to receive plaintext message content or application session keys.

## Current protocol hardening items

1. Bind the hybrid handshake to Ed25519 identity authentication.
2. Ensure the responder uses the exact X25519 private key corresponding to the advertised public key.
3. Bind protocol version, suite, identities and ephemeral key material into the transcript and KDF.
4. Bind application identity to the transport identity / libp2p PeerId.
5. Add explicit recipient routing for initial handshakes.
6. Complete Alice-to-Bob application-level Double Ratchet testing.
7. Remove or formally specify duplicated ratchet-header fields.
8. Define replay, malformed-input, timeout and recovery behavior.

## Non-goals

This protocol does not claim anonymity against network observers, endpoint compromise resistance, metadata elimination, production-ready group messaging, production-ready voice/video, or production-ready onion routing.
