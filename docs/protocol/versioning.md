# Protocol Versioning

## Purpose

Versioning prevents incompatible cryptographic or serialization changes from being interpreted as compatible messages.

Application protocol versioning is separate from the underlying libp2p transport version.

## Version fields

A versioned protocol message should identify:

```text
protocol_version
message/frame type
```

Cryptographic negotiation should additionally identify the selected suite or capability set.

## Compatibility

Implementations must distinguish:

- exact version support;
- backward compatibility;
- forward compatibility;
- unsupported versions.

Do not silently downgrade to an older or weaker cryptographic construction.

## Cryptographic suites

A suite identifier should cover the complete construction:

```text
identity signature
+
classical key agreement
+
post-quantum KEM
+
KDF
+
AEAD
+
ratchet construction
```

Changing a security-critical component should normally create a new suite identifier.

## Serialization

The current protocol uses Serde + Bincode.

Changes to Bincode configuration, field order, integer encoding, enum representation or optional-field encoding can change the wire protocol.

Such changes require a protocol version or explicitly versioned encoding.

## Negotiation

Conceptually:

```text
supported_versions
supported_suites
        │
        ▼
negotiated_version
negotiated_suite
        │
        ▼
authenticated transcript
```

The selected version and suite must be bound into handshake authentication and KDF input.

## Downgrade resistance

The protocol should:

1. advertise supported versions/suites;
2. select an allowed intersection;
3. authenticate the selected result;
4. reject forbidden downgrades.

## Migration

A protocol migration must define old/new versions, minimum supported version, migration window, session behavior, storage migration and rollback behavior.

## Persisted sessions

Protocol version and suite must be associated with persisted session state. A session created under one suite must not silently be loaded as another suite.

## Current status

This document defines the policy. Complete version negotiation and compatibility enforcement remain roadmap work and must not be inferred from the existence of this document.
