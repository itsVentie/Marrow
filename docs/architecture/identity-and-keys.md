# Identity and Key Architecture

**Status:** Experimental / lifecycle hardening required

## 1. Identity model

Marrow uses a local application identity based on Ed25519.

The identity is distinct from ephemeral session and ratchet keys.

Conceptually:

```text
                    Application Identity
                         Ed25519
                            |
             +--------------+--------------+
             |                             |
        Authentication                 Identity storage
             |
             v
       Session handshake
             |
       +-----+-----+
       |           |
    X25519       ML-KEM
       |           |
       +-----+-----+
             |
        Session state
             |
        Double Ratchet
```

## 2. Key classes

The system contains multiple cryptographic key classes:

| Key class | Purpose | Lifetime |
|---|---|---|
| Ed25519 identity key | Application identity/signatures | Long-lived |
| X25519 ephemeral key | Handshake/DH | Short-lived |
| ML-KEM key material | PQ handshake | Session/identity dependent |
| Ratchet root key | Session evolution | Session |
| Chain keys | Message-key derivation | Short-lived / evolving |
| Message keys | Individual messages | Very short-lived |
| Storage key | Local persistent encryption | Application/storage lifecycle |
| Password-derived material | Unlock/derivation | Unlock operation |

Keys must not be reused across purposes without an explicit, reviewed construction.

## 3. BIP39 recovery material

BIP39 mnemonic generation is used for recovery material.

The exact identity derivation function must remain explicitly documented.

The implementation should not be described as standard wallet/BIP32 derivation unless it actually follows those semantics.

## 4. Password protection

Password-based protection uses Argon2id.

The password should never be treated as a cryptographic key directly.

A versioned derivation configuration is required so that future parameter changes can coexist with existing encrypted identity data.

## 5. Identity creation

Identity creation should:

1. generate identity/recovery material;
2. derive or establish protected identity storage;
3. persist the identity securely;
4. initialize application state;
5. start the network runtime through the common startup path.

Identity creation and identity unlock should converge on the same runtime initialization logic.

## 6. Identity unlock

Unlocking should:

1. load the protected identity;
2. derive the required key from the user's credential;
3. decrypt and validate identity material;
4. initialize Rust application state;
5. initialize network/session/storage components;
6. expose only the minimum required frontend state.

Filesystem paths must be validated and should preferably originate from a controlled file-selection mechanism rather than unrestricted frontend-provided paths.

## 7. Identity authentication

Possessing an Ed25519 private key should authenticate the application identity through signatures over the handshake transcript.

This property is not established merely by storing an Ed25519 key.

The handshake must explicitly use it.

## 8. Identity vs transport identity

A network transport may have its own identity.

The architecture must define whether and how:

```text
Application identity
        |
        +---- cryptographically binds to ----+
                                             |
                                      Transport identity
```

If these identities are independent, an attacker could potentially cause confusing identity substitution unless the application layer explicitly authenticates the peer.

## 9. Key lifecycle

Sensitive material must be cleared on:

- logout;
- lock;
- failed authentication where applicable;
- session destruction;
- application shutdown where practical.

Required cleanup includes:

- storage encryption key;
- ratchet sessions;
- pending handshakes;
- temporary private keys;
- identity-derived sensitive buffers.

## 10. Zeroization

Rust memory safety does not automatically mean cryptographic memory is zeroized.

Sensitive types should use appropriate zeroization facilities where practical.

The architecture should minimize:

- unnecessary copies;
- serialization of plaintext;
- conversion to unprotected generic buffers;
- long-lived references to secret material.

## 11. Recovery and backup

A complete production identity architecture requires:

- explicit backup flow;
- restore flow;
- backup verification;
- recovery failure behavior;
- passphrase-change semantics;
- identity rotation semantics;
- secure deletion semantics.

These are distinct from simply generating a BIP39 mnemonic.

## 12. Threat boundary

A local identity protects against remote impersonation only when the remote protocol verifies its signatures correctly.

It does not protect against:

- compromised local OS;
- malware;
- memory disclosure;
- frontend compromise;
- stolen unlocked credentials;
- screenshots or keylogging.
