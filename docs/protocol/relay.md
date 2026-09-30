# Relay Protocol

## Purpose

The relay provides an intermediate network path when direct peer connectivity is unavailable or relay routing is selected.

It is intended to be blind to application plaintext.

## Current implementation

The experimental relay is based on Quinn and maintains ephemeral in-memory peer state using connected-peer mappings, send channels, bounded/offline queue behavior and TTL-style cleanup.

Therefore the current relay is not stateless.

Accurate description:

> ephemeral in-memory blind relay

## Relay visibility

A relay may observe metadata required for forwarding, potentially including transport metadata, transport identity, recipient routing identifier, frame sizes, timing, connection duration and queue behavior.

It should not possess application plaintext, Double Ratchet root/message keys or identity private keys.

## Initial handshake routing

The initial handshake needs an explicit recipient.

Recommended shape:

```text
RoutingEnvelope
├── version
├── recipient
├── frame_type
└── opaque payload
```

The relay routes using the envelope and does not interpret application cryptographic content.

## Offline delivery

Offline queues require explicit limits for:

- queue size;
- frame lifetime;
- eviction order;
- restart behavior;
- duplicate handling;
- memory;
- sender backpressure.

Offline relay state should remain ephemeral unless durable relay storage is deliberately introduced.

## Connection lifecycle

Duplicate connections require generation-safe cleanup.

Unsafe:

```text
insert(peer_id, new_connection)
old_connection closes
remove(peer_id)
```

Safer:

```text
peer_id -> { connection_id, sender }
cleanup(peer_id, connection_id)
```

Only the matching connection generation may remove the mapping.

## TLS

The current relay prototype uses development-oriented self-signed TLS material. This is not a production trust model.

Production requires certificate lifecycle, validation, rotation, secure key storage and documented client trust.

## Authentication

Transport encryption is not peer authorization. Production relay deployment needs explicit connection authentication, registration policy, quotas and abuse prevention.

## Resource limits

Enforce limits for frame size, connection count, per-peer queue, global queue, queue lifetime, bandwidth, idle connections and handshake rate.

## Security model

A compromised relay should not reveal plaintext when application-layer E2E encryption is correctly implemented.

A malicious relay can still drop, delay, reorder or replay frames, inject malformed input and exhaust resources. Clients must therefore fail safely under hostile relay behavior.
