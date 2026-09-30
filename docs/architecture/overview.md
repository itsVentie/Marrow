# Marrow Architecture Overview

**Status:** Experimental / architecture reference

## 1. System overview

Marrow is a desktop communication application built around a Rust core, a Tauri desktop shell, a TypeScript/Preact frontend, local encrypted persistence, cryptographic session state, and an experimental peer/relay network.

Conceptually:

```text
+-------------------------------+
|          Marrow UI            |
|      Preact / TypeScript      |
+---------------+---------------+
                |
             Tauri IPC
                |
+---------------v---------------+
|          Rust Core             |
|                               |
| Identity | Crypto | Protocol  |
| Network  | Storage | Search   |
+----+----------+---------+-----+
     |          |         |
     |          |         +--> Tantivy index
     |          |
     |          +------------> encrypted redb records
     |
     +-----------------------> libp2p network
                                  |
                         +--------+--------+
                         |                 |
                    direct peer       experimental relay
```

The diagram is conceptual. Individual implementation boundaries and lifecycle behavior remain under active development.

## 2. Major components

### Frontend

The frontend is responsible for:

- application screens;
- contacts and conversations;
- message presentation;
- user input;
- settings and desktop UI state;
- invoking typed Tauri commands;
- receiving application events.

The frontend should not become the authority for cryptographic validation or security-sensitive state transitions.

### Tauri shell

Tauri provides the desktop application boundary between the web frontend and native Rust implementation.

Security-sensitive operations should remain on the Rust side.

### Rust application state

The Rust layer coordinates:

- identity state;
- cryptographic sessions;
- network runtime;
- protocol serialization;
- persistent storage;
- application events;
- IPC command handling.

State initialization and teardown must be consistent across identity creation, identity unlock, logout, and application shutdown.

### Crypto layer

The crypto layer contains:

- Ed25519 identity material;
- X25519 key agreement;
- ML-KEM-768;
- password-derived keys;
- AEAD operations;
- Double Ratchet state and transitions.

Primitive availability does not by itself prove end-to-end protocol security.

### Protocol layer

The protocol layer defines:

- frames;
- payload types;
- handshake structures;
- encrypted message structures;
- relay-related messages;
- serialization;
- protocol versions and limits.

The wire representation should have one authoritative representation for each security-sensitive field.

### Network layer

The client network stack uses libp2p transports and protocols, including TCP and QUIC-related functionality.

Relevant libp2p components include transport/security/multiplexing and discovery/connectivity facilities.

The relay implementation is a separate experimental component based on Quinn.

### Storage layer

Persistent application records use redb with authenticated encryption performed before persistence.

The search index is maintained separately through Tantivy and therefore represents a separate security/data-at-rest surface.

## 3. Data flow

A simplified message flow is:

```text
User input
   |
   v
Frontend
   |
   | Tauri command
   v
Rust application state
   |
   +--> session lookup
   |
   +--> handshake if required
   |
   +--> Double Ratchet
   |
   +--> protocol serialization
   |
   v
Network
   |
   v
Remote peer / relay
```

On receipt:

```text
Network frame
   |
   v
Protocol decode
   |
   v
Session / ratchet processing
   |
   v
AEAD verification + decrypt
   |
   +--> encrypted persistence
   |
   +--> search/index handling
   |
   +--> frontend event
   v
UI
```

## 4. Lifecycle

Important application lifecycle states include:

```text
No identity
    |
    v
Identity creation / unlock
    |
    v
Runtime initialized
    |
    v
Network + sessions + storage available
    |
    +--> lock / logout
    |
    v
Sensitive state cleared
```

Identity creation and identity unlock must eventually use the same network/runtime initialization path. Any divergence can leave a valid identity with an uninitialized network runtime.

## 5. Security boundaries

Primary boundaries are:

1. frontend ↔ Rust IPC;
2. Rust process ↔ filesystem;
3. application ↔ cryptographic state;
4. application ↔ network;
5. client ↔ relay;
6. relay ↔ remote client;
7. persistent database ↔ search index.

All network and IPC input is untrusted.

## 6. Architectural principles

### Endpoint cryptographic ownership

Long-term identity and ratchet secrets remain at endpoints.

### Relay as transport infrastructure

The relay is not intended to become a cryptographic trust anchor.

### Explicit state ownership

Each security-sensitive state object should have an explicit owner and lifecycle.

### Fail closed

Malformed protocol input, authentication failures, invalid cryptographic state, and unsupported versions should fail without silently degrading security.

### Versioned boundaries

Protocol and persistence formats should be versioned independently where their compatibility requirements differ.

## 7. Current architectural issues

Known architectural work includes:

- authenticated hybrid handshake;
- responder ephemeral-key/ratchet key unification;
- explicit handshake recipient routing;
- unified outer routing envelope;
- relay connection lifecycle race handling;
- client/relay transport integration;
- application-level E2E verification;
- storage/search key lifecycle;
- frontend/backend startup consistency;
- protocol frame-size alignment;
- complete IPC/CSP review.

## 8. Non-goals

The current architecture does not establish:

- anonymous communication;
- metadata-free communication;
- production-grade relay infrastructure;
- complete group messaging;
- voice/video;
- onion routing.

Those are separate future architectural areas.
