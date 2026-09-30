# Identity Protocol

## Purpose

Marrow uses a local Ed25519 identity as the application-level identity of a user.

Identity is separate from transport encryption and per-session ephemeral keys.

## Identity components

```text
Local identity
├── Ed25519 signing identity
├── deterministic derivation material
├── locally protected secret material
└── public identity representation
```

The repository contains BIP-39 seed-based deterministic identity derivation and Argon2id-based local protection.

## Identity derivation

The current implementation derives identity material deterministically from BIP-39 seed material.

This is an application-specific derivation scheme and is not a claim of BIP-32/BIP-44 wallet-path compatibility.

Any derivation change must define:

- derivation version;
- domain separation;
- exact input encoding;
- output length;
- cryptographic primitive;
- migration behavior;
- compatibility behavior.

## Local protection

Private identity material is protected locally using Argon2id-derived key material.

Treat the following as secrets:

- mnemonic / seed material;
- Ed25519 private key;
- derived encryption keys;
- session secrets;
- ratchet state;
- storage encryption key.

Private key material must never be serialized into protocol frames.

## Identity authentication

An Ed25519 public key is not proof of identity by itself. An authenticated handshake should prove possession of the corresponding private key by signing a canonical transcript.

The signature input should include protocol version, suite, identities, ephemeral X25519 keys, relevant ML-KEM material, and role information.

The canonical encoding must be specified before signatures are considered interoperable.

## Identity ↔ transport binding

Application identity and libp2p transport identity are separate concepts.

Production protocol design should explicitly bind them so that a transport-level PeerId cannot silently represent a different application identity.

## Identity lifecycle

```text
Created → Locked → Unlocked → In use → Locked / Logged out → Active secrets destroyed
```

Logout must clear active session and identity-derived secret material according to the key-lifecycle policy.

## Backup and recovery

BIP-39 backup/restore semantics must specify mnemonic lengths, normalization, checksum handling, derivation version, migration, incorrect passphrase behavior and corrupted-backup behavior.

## Security requirements

- Never log private identity material.
- Never include private identity material in protocol frames.
- Do not use identity keys directly as symmetric keys.
- Domain-separate derivation contexts.
- Version identity derivation.
- Zeroize sensitive material where practical.
