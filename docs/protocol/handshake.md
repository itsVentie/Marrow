# Double Ratchet Protocol

## Purpose

Marrow uses a Double Ratchet session model for ongoing 1-to-1 message encryption.

The implementation contains root-key evolution, sending and receiving chains, DH ratchets, message counters, previous-chain length, skipped-message-key handling, bounded skip state and AEAD encryption/decryption.

Application integration remains a hardening item.

## State model

```text
Session
├── Root Key
├── DH self keypair
├── DH remote public key
├── Sending Chain
│    ├── chain key
│    └── message number
├── Receiving Chain
│    ├── chain key
│    └── message number
├── Previous Chain Length
└── Skipped Message Keys
```

The exact Rust representation is authoritative.

## Message encryption

```text
chain_key
    │
    ├── derive message key
    └── advance chain key
           │
           ▼
message_key + plaintext
           │
           ▼
          AEAD
           │
           ▼
ciphertext + ratchet header
```

A message key must not be reused for multiple messages.

## Message decryption

The receiver:

1. parses the ratchet header;
2. determines whether a DH ratchet is required;
3. derives skipped keys where necessary;
4. derives the requested message key;
5. verifies/decrypts the ciphertext;
6. advances state.

Failed authentication must not silently advance state in an unsafe way.

## Out-of-order messages

Skipped-message-key state permits delayed messages from an earlier sending chain to be processed after later messages.

The implementation uses a bounded skip policy. The maximum skip must remain explicit to prevent attacker-controlled unbounded state growth.

## DH ratchet

When the remote DH public key changes, the session performs a DH ratchet and derives fresh root/chain material.

The ordering of DH computation, root-key KDF and chain initialization must be identical for both roles.

## Initiator/responder initialization

The repository uses explicit role-specific constructors:

```text
init_initiator(...)
init_responder(...)
```

The responder's initial DH private key must correspond to the public key authenticated during the handshake.

## Persistence

Ratchet state must be persisted in a way that prevents crashes from causing message-key reuse or unsafe rollback.

The storage layer must define commit ordering, crash recovery, migration and corruption behavior.

## Key lifecycle

Sensitive ratchet state should be zeroized where practical, including root keys, chain keys, message keys, skipped keys and ephemeral DH private keys.

A systematic zeroization audit remains required.

## Limits

Define explicit limits for skipped messages, header size, ciphertext size, pending sessions and persisted state.

## Reference

The conceptual design should be reviewed against the Signal Double Ratchet specification:

https://signal.org/docs/specifications/doubleratchet/

Marrow-specific wire and state behavior is not claimed to be Signal-interoperable.
