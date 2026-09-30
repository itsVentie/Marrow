# Blind Relay Architecture

**Status:** Experimental

## 1. Purpose

The Marrow relay is intended to provide connectivity and temporary forwarding between peers without becoming a trusted endpoint for message plaintext.

The relay is therefore transport infrastructure rather than an end-to-end cryptographic participant.

## 2. Current implementation model

The experimental relay is based on Quinn and maintains in-memory state for connected peers.

Conceptually:

```text
Client A
   |
   | encrypted protocol frames
   v
+----------------------+
| Experimental Relay   |
|                      |
| Peer map             |
| Offline queues       |
| TTL eviction         |
+----------+-----------+
           |
           | encrypted protocol frames
           v
        Client B
```

The relay currently has:

- a peer-to-sender map;
- ephemeral connection state;
- bounded/offline queue behavior;
- queue TTL behavior;
- development-oriented TLS.

It is therefore more accurately described as an **ephemeral in-memory blind relay** than as a stateless relay.

## 3. Relay trust boundary

The relay may observe:

- transport endpoints;
- routing identifiers;
- connection lifecycle;
- queue state;
- timing;
- frame sizes;
- delivery behavior.

The relay should not possess:

- application message plaintext;
- Double Ratchet keys;
- identity private keys;
- storage encryption keys.

## 4. Offline delivery

The experimental design supports temporary in-memory queues for disconnected peers.

Queueing must remain bounded by:

- message/frame size;
- queue length;
- total relay memory;
- TTL;
- connection and peer limits.

Offline queues are not durable storage.

A relay restart may therefore discard queued data.

## 5. Routing

Initial handshake routing requires an explicit recipient.

A current architectural gap is that a `HandshakeInitPayload` without recipient information cannot independently tell the relay which peer should receive it.

A preferred design is an outer routing envelope:

```text
RoutingEnvelope {
    version
    recipient
    frame_type
    payload
}
```

The routing layer should contain only information required for transport routing.

## 6. Connection lifecycle

The relay maps peer identity to a connection sender.

Connection replacement must be generation-safe.

A race exists if:

1. connection A registers peer P;
2. connection B registers peer P;
3. connection A closes;
4. connection A cleanup unconditionally removes P;
5. connection B remains active but its map entry has been deleted.

A connection ID or generation token should be used so cleanup only removes the connection that owns the current entry.

## 7. Relay TLS

The current relay uses development-oriented self-signed certificate behavior.

This is not a production PKI model.

Production deployment requires:

- trusted certificate provisioning;
- certificate rotation;
- explicit trust configuration;
- authentication policy;
- secure key storage;
- operational monitoring.

## 8. Resource protection

Production relay infrastructure requires explicit limits for:

- maximum frame size;
- maximum concurrent connections;
- per-peer queue size;
- global queue memory;
- queue TTL;
- handshake rate;
- connection rate;
- malformed frames;
- idle connections;
- CPU-intensive protocol operations.

## 9. Client/relay transport architecture

The client network stack uses libp2p transports, while the experimental relay uses Quinn directly.

This creates an architectural integration boundary that must be explicitly defined before the relay can be treated as a normal production transport component.

The project should not describe the two stacks as one unified transport implementation until their integration path is actually established.

## 10. Security properties

A compromised relay should not be able to decrypt properly protected application messages.

It can still:

- drop;
- delay;
- reorder;
- duplicate;
- disconnect;
- route incorrectly;
- exhaust resources;
- observe metadata.

The relay therefore cannot guarantee availability or metadata confidentiality.

## 11. Production direction

A production relay architecture should add:

- authenticated peer registration;
- production TLS;
- connection-generation safety;
- bounded queues;
- global resource budgets;
- rate limiting;
- abuse controls;
- observability;
- health checks;
- graceful shutdown;
- explicit protocol compatibility;
- deployment isolation.
