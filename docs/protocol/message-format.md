# Message Format

## Scope

This document describes the application-level encrypted-message representation and its relationship to the Double Ratchet.

The current serialization layer uses Serde + Bincode.

## Message path

```text
Plaintext
   │
   ▼
Double Ratchet
   ├── ratchet header
   ├── message key
   └── AEAD ciphertext
   │
   ▼
Protocol payload
   │
   ▼
Network frame
```

## Current encrypted-message representation

The current representation contains fields corresponding to:

- recipient public identity / routing identity;
- ratchet DH public key;
- sequence number;
- previous-chain length;
- nonce;
- ciphertext.

The exact Rust struct remains the source of truth for the current wire representation.

## Ratchet-header duplication

The Double Ratchet ciphertext path already carries header information such as DH public key, previous-chain length (`PN`) and message number (`N`).

Duplicating these values in an outer payload creates unnecessary consistency requirements.

Preferred shape:

```text
Outer envelope
    ├── routing metadata
    └── opaque ratchet message

Ratchet message
    ├── header: DH / PN / N
    └── ciphertext
```

If outer copies remain, equality and canonical semantics must be explicitly specified.

## Nonce handling

Every nonce field must specify:

- generator;
- authentication status;
- random/deterministic nature;
- length;
- consuming AEAD;
- persistence behavior;
- reuse guarantees.

An unused outer nonce must not be described as providing cryptographic protection.

## Authentication

Message authenticity is provided by the ratchet AEAD construction, not by the relay.

## Ordering

Network delivery may be delayed, duplicated, reordered or lost. The ratchet layer therefore handles out-of-order delivery through message counters and skipped-message-key state.

Transport delivery must not be confused with successful cryptographic processing.

## Replay

Previously accepted encrypted messages must not be indefinitely accepted as new messages. Replay resistance combines ratchet state, counters, skipped-key handling and persisted session state.

## Size limits

Network codec and protocol frame limits must be aligned. The current network codec accepts a substantially larger frame than the protocol's documented maximum, which should be corrected to reduce memory/DoS exposure.

## Serialization compatibility

Bincode configuration is part of the wire protocol. Changes to serialization configuration, field order, enum representation or integer encoding require a protocol version or explicitly versioned message encoding.
