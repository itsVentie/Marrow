# Double Ratchet Architecture

**Status:** Implemented primitives / application integration under hardening

## 1. Purpose

The Double Ratchet provides evolving session keys for 1-to-1 encrypted communication.

Its purpose is to avoid using a single static message key for an entire conversation and to support key evolution across messages and DH ratchet steps.

## 2. High-level state

Conceptually:

```text
              +----------------+
              |   Root Key     |
              +-------+--------+
                      |
          +-----------+-----------+
          |                       |
          v                       v
   Sending Chain             Receiving Chain
          |                       |
          v                       v
     Message Keys            Message Keys
```

DH ratchet transitions update the root-key state and establish new chain material.

## 3. Roles

The implementation distinguishes initiator and responder initialization.

The role-specific constructors are:

```text
init_initiator
init_responder
```

Role separation is important because the two endpoints must derive complementary sending and receiving state.

## 4. State

Relevant ratchet state includes:

- root key;
- local DH ratchet key;
- remote DH public key;
- sending chain key;
- receiving chain key;
- send counter;
- receive counter;
- previous-chain length;
- skipped message keys.

The state must be treated as sensitive cryptographic material.

## 5. Message processing

Conceptually:

```text
Outgoing message
   |
   v
Sending chain
   |
   v
Message-key derivation
   |
   v
AEAD
   |
   v
Encrypted protocol frame
```

Incoming messages follow the inverse process with header processing, skipped-key handling, and AEAD verification.

## 6. Out-of-order messages

Skipped-message handling allows a receiver to process messages that arrive out of order.

This requires bounded skipped-key storage.

Unbounded storage would create a remote resource-exhaustion vector.

## 7. DH ratchet

A DH ratchet transition introduces new DH material into root-key evolution.

This is an important component of forward secrecy and post-compromise recovery.

The property depends on correct integration with the authenticated handshake and correct persistence of ratchet state.

## 8. Critical integration issue

The responder's handshake ephemeral X25519 key must be exactly the private key corresponding to the public key advertised during the handshake.

If handshake code generates keypair A and responder Double Ratchet initialization receives private key B, the two endpoints derive incompatible initial DH material.

This is a security/correctness blocker and must be fixed before claiming a verified end-to-end ratchet session.

## 9. Persistence

A ratchet session may need to survive application restart.

Persistence must preserve sufficient state to continue the ratchet without:

- key reuse;
- counter rollback;
- accidental session fork;
- replay acceptance.

Persistent ratchet state is encrypted at rest but still requires correct lifecycle cleanup.

## 10. Key lifecycle

Ratchet keys should have minimized memory lifetime.

Requirements include:

- zeroization where practical;
- deletion of consumed message keys;
- bounded skipped-key state;
- cleanup on logout;
- no plaintext/key logging;
- careful serialization of persistent session state.

## 11. Protocol representation

The encrypted message currently contains fields overlapping with the ratchet header.

A future canonical wire format should avoid duplicating:

- DH public key;
- previous-chain length;
- message sequence number;
- nonce-related information.

There should be one authoritative representation for each ratchet header field.

## 12. Verification

The ratchet requires tests for:

- initiator/responder initialization;
- normal bidirectional messaging;
- multiple messages in both directions;
- out-of-order delivery;
- skipped keys;
- duplicate messages;
- replay;
- dropped messages;
- delayed messages;
- DH ratchet transitions;
- persistence/restart;
- corrupted state;
- malformed headers;
- concurrent send/receive behavior.

A complete Alice-application → network → Bob-application test is still required for end-to-end verification.
