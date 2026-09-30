# Relay Node

Marrow includes an experimental relay server intended to provide peer routing
and bounded offline message delivery when direct peer-to-peer connectivity is
not available.

The current relay implementation is a prototype and must not be treated as a
production-ready privacy or availability service.

## Architecture

The current relay implementation is based on Quinn and maintains ephemeral
in-memory state for connected peers and offline message queues.

At a high level:

```text
Marrow Client
     |
     | connection
     v
+------------------+
|   Relay Node     |
|                  |
|  Peer Map        |
|  Offline Queues  |
|  TTL Eviction    |
+------------------+
     |
     v
Marrow Client
````

The relay is intended to forward encrypted application data without requiring
access to message plaintext.

The relay is therefore part of the transport infrastructure rather than the
end-to-end cryptographic trust boundary.

## Current Implementation

The experimental relay currently provides:

* active peer routing;
* in-memory peer connection state;
* bounded offline message queues;
* TTL-based queue eviction;
* Quinn-based transport;
* development-oriented TLS configuration.

The relay currently does not provide all protections required for a production
deployment.

## Relay State

The relay maintains ephemeral runtime state for connected peers.

This includes information required to route traffic between currently connected
peers.

Offline messages may also be retained temporarily in bounded in-memory
queues until:

* the recipient reconnects;
* the queue reaches its configured limits;
* the message expires;
* the relay process terminates.

The current implementation therefore should not be described as stateless.

## End-to-End Encryption

The relay is not intended to decrypt application message contents.

Application-level encryption is performed by the Marrow clients.

The intended data flow is:

```text
Sender
  |
  | plaintext
  v
Double Ratchet
  |
  | encrypted payload
  v
Transport
  |
  v
Relay
  |
  | opaque encrypted payload
  v
Recipient
  |
  v
Double Ratchet
  |
  | plaintext
  v
Application
```

Transport encryption and application-level encryption serve different purposes.

Transport encryption protects the network connection.

Application-level encryption protects the message payload from intermediate
transport infrastructure.

## Relay Visibility

A relay may observe transport and routing metadata required for operation.

Depending on the protocol and deployment configuration, this can include:

* connected peer identifiers;
* source and destination routing information;
* connection timestamps;
* message or frame sizes;
* message timing;
* connection duration;
* queue state;
* IP addresses visible at the transport layer.

The relay must not be assumed to hide all communication metadata.

Traffic-analysis resistance is currently incomplete and experimental.

## Offline Delivery

The experimental relay supports bounded in-memory offline queues.

Offline delivery is subject to:

* queue capacity;
* message expiration;
* process lifetime;
* relay resource limits.

Offline messages are not intended to provide durable message storage.

A relay restart may therefore result in queued messages being lost.

Production deployments must not depend on the current in-memory queue as a durable
messaging system.

## TLS

The current relay prototype uses development-oriented TLS configuration.

Production deployments require:

* persistent certificates;
* trusted certificate validation;
* appropriate certificate lifecycle management;
* secure private-key storage;
* certificate rotation;
* monitoring of certificate expiration.

Development certificates must not be treated as a production trust model.

## Authentication

The current relay architecture still requires additional authentication and
registration hardening.

A production relay should define:

* peer registration;
* identity verification;
* connection authorization;
* relay authorization;
* peer lifecycle management;
* connection ownership;
* duplicate connection handling.

Application identity and transport identity should be explicitly bound where
required by the protocol.

## Resource Limits

A production relay must enforce resource limits for:

* connections;
* active peers;
* queued messages;
* queue size per peer;
* total queue size;
* message size;
* frame size;
* memory consumption;
* CPU-intensive operations;
* connection establishment rate.

Without explicit limits, an internet-facing relay may be vulnerable to resource
exhaustion.

## Current Limitations

The experimental relay currently has several known limitations.

These include:

* experimental client/relay architecture;
* incomplete authenticated relay registration;
* incomplete production TLS configuration;
* incomplete rate limiting;
* incomplete resource exhaustion protection;
* incomplete abuse protection;
* duplicate-connection lifecycle concerns;
* in-memory offline queues;
* incomplete routing of initial handshake messages;
* network/protocol frame-size alignment work;
* incomplete production deployment configuration.

See the project roadmap for the current implementation status.

## Production Requirements

Before the relay can be considered suitable for production deployment, the
following areas must be addressed:

1. define a single production transport architecture;
2. implement authenticated relay registration;
3. implement connection lifecycle identifiers;
4. eliminate duplicate-connection cleanup races;
5. implement per-peer and global resource limits;
6. implement connection and message rate limiting;
7. implement backpressure;
8. implement abuse and flooding protection;
9. deploy persistent trusted TLS certificates;
10. validate relay certificates correctly;
11. define secure configuration management;
12. define logging and privacy requirements;
13. implement monitoring without collecting message contents;
14. test relay restart and network partition behavior;
15. define operational recovery procedures.

## Deployment Warning

The current relay should be treated as experimental infrastructure.

Do not expose an experimental relay directly to an untrusted public network
without understanding its current resource, authentication, TLS, and abuse
protection limitations.
