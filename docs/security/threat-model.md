# Marrow Threat Model

**Status:** Experimental / security documentation  
**Scope:** Marrow desktop client, 1-to-1 messaging, cryptographic handshake, ratcheting, local storage, search index, and experimental relay infrastructure.

> Marrow is not currently a production-secure messaging system. This document describes the intended security model and distinguishes implemented primitives from properties that still depend on integration, hardening, or verification.

## 1. Security objectives

Marrow is designed around the following objectives:

- confidentiality of message content against network infrastructure and relays;
- integrity and authenticated encryption of protected message data;
- authenticated application identities;
- forward secrecy through ratcheting;
- post-compromise recovery through continued ratchet evolution;
- protection of persistent local records against ordinary filesystem disclosure;
- minimization of trust placed in relay infrastructure;
- explicit handling of replay, reordering, delay, duplication, and malformed input;
- cryptographic agility and protocol versioning.

These objectives are not all fully achieved by the current implementation.

## 2. Protected assets

Relevant assets include:

- Ed25519 identity private keys;
- BIP39 recovery material;
- password-derived encryption keys;
- X25519 private keys;
- ML-KEM private keys;
- handshake state;
- Double Ratchet root, sending, receiving, and skipped-message keys;
- message plaintext;
- encrypted persistent records;
- search-index contents;
- contact and peer metadata;
- relay routing state;
- application configuration and logs.

## 3. Adversary model

### Passive network observer

An observer may capture network traffic and attempt to infer message contents or communication patterns.

Marrow's cryptographic layer is intended to prevent recovery of message plaintext from captured ciphertext. Traffic analysis is a separate problem: timing, packet frequency, endpoints, and residual size information may remain observable.

### Active network attacker

An attacker may:

- inject frames;
- modify frames;
- replay frames;
- reorder frames;
- delay or drop frames;
- establish competing connections;
- send malformed or oversized data.

The protocol must reject unauthenticated or malformed cryptographic state and must bound resource consumption.

### Malicious or compromised relay

The relay is not assumed to be trusted with message plaintext.

A relay may observe routing and transport metadata, drop or delay messages, disconnect peers, or attempt resource exhaustion. End-to-end encryption is intended to prevent it from reading message content.

The current relay is an experimental stateful in-memory relay, not a production stateless service.

### Malicious peer

A remote peer may be fully controlled by an attacker. The client must treat all remote protocol input as untrusted, including:

- handshake fields;
- public keys;
- counters;
- ciphertexts;
- frame lengths;
- protocol versions;
- relay messages.

### Compromised endpoint

Endpoint compromise is outside the protection guarantees of network cryptography.

If an attacker controls the running operating system or application process, they may access plaintext, keys, memory, logs, screenshots, or user input.

Marrow therefore does not claim protection against a fully compromised endpoint.

### Stolen local storage

Persistent records are encrypted at rest using authenticated encryption. This reduces the value of a raw database copy.

The search index is a separate data surface and currently requires additional protection before encrypted-search claims can be made.

## 4. Trust boundaries

Important boundaries are:

```text
User
  |
  v
Tauri / Frontend
  |
  | IPC
  v
Rust application state
  |
  +--> Identity / Crypto
  |
  +--> Ratchet / Protocol
  |
  +--> Encrypted Storage
  |
  +--> Search Index
  |
  v
Network Client
  |
  v
Relay / Direct Peer
```

The frontend, IPC layer, filesystem, network, relay, and remote peer must all be treated as separate security boundaries.

## 5. Cryptographic trust model

The intended 1-to-1 flow is:

1. each endpoint possesses a local application identity;
2. ephemeral key material is exchanged during the handshake;
3. the handshake establishes initial shared secret material;
4. the resulting state initializes the Double Ratchet;
5. application messages are encrypted using ratchet-derived keys;
6. ratchet state evolves as messages and DH ratchets occur.

The current handshake still requires authenticated identity binding and transcript authentication before this can be considered a complete authenticated protocol.

## 6. Relay trust model

The relay is intended to provide transport assistance, not cryptographic trust.

A relay should not be able to:

- decrypt message ciphertext;
- derive ratchet keys;
- impersonate a peer without defeating endpoint authentication.

A relay can still:

- observe routing information;
- observe connection metadata;
- drop messages;
- delay messages;
- disconnect clients;
- perform denial-of-service attacks;
- accumulate queue state and metadata.

## 7. Traffic-analysis limitations

Fixed-size padding, limited jitter, and planned dummy traffic can reduce some information leakage, but they do not provide anonymity or complete traffic-flow confidentiality.

Marrow does not currently claim:

- sender anonymity;
- receiver anonymity;
- resistance to global traffic correlation;
- onion routing;
- mixnet-level protection;
- metadata-free communication.

## 8. Resource exhaustion

Remote input must be bounded at multiple layers.

Relevant controls include:

- frame-size limits;
- queue limits;
- connection limits;
- handshake timeouts;
- skipped-message limits;
- offline-message TTLs;
- relay memory limits;
- rate limiting;
- malformed-input rejection.

The network codec and protocol frame limits must remain aligned.

## 9. Current critical security gaps

The following are known blockers to stronger security claims:

1. Responder handshake ephemeral-key mismatch must be fixed.
2. Ed25519 identity authentication is not yet cryptographically bound to the hybrid handshake.
3. The complete application-level Alice-to-Bob E2E path requires verification.
4. Search-index protection is incomplete.
5. Key lifecycle and zeroization require further hardening.
6. Relay production authentication, TLS, limits, and connection lifecycle require hardening.
7. Complete replay, malformed-input, restart, corruption, and network-failure testing is still required.
8. Tauri CSP and filesystem/IPC security require further review.

## 10. Non-goals

Unless separately implemented and verified, Marrow does not claim:

- endpoint compromise resistance;
- anonymous communication;
- metadata confidentiality;
- perfect traffic-analysis resistance;
- protection against malicious operating systems;
- secure deletion on every filesystem and storage medium;
- protection from screenshots, keyloggers, or compromised desktop environments.

## 11. Security status

Security claims should be made only from verified implementation and reproducible tests.

A cryptographic primitive being present in the repository does not by itself establish an end-to-end security property.
