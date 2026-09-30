# Production Deployment

Marrow is currently an experimental project.

This document defines the requirements that must be satisfied before Marrow
components can be deployed as production security infrastructure.

It does not imply that the current implementation satisfies these requirements.

## Production Readiness

Production deployment requires more than a successful build.

The following areas must be verified:

```text
Build
  ↓
Protocol
  ↓
Cryptography
  ↓
Authentication
  ↓
Storage
  ↓
Networking
  ↓
Resource limits
  ↓
Security testing
  ↓
Operational security
  ↓
Independent review
````

A production security claim should only be made after the relevant stages have
been independently verified.

## Production Components

A complete Marrow deployment may contain:

```text
+----------------------+
|      Marrow Client   |
|                      |
|  UI                  |
|  Crypto              |
|  Protocol            |
|  Storage             |
|  Networking          |
+----------+-----------+
           |
           | E2EE transport
           |
           v
+----------------------+
|     Relay Node       |
|                      |
|  Routing             |
|  Queueing            |
|  Resource limits     |
|  TLS                 |
+----------------------+
```

The relay is transport infrastructure and should not be part of the
application-level plaintext trust boundary.

## Client Requirements

Before a production client release, the following should be verified:

### Identity

* identity generation;
* identity persistence;
* identity backup;
* identity restoration;
* identity deletion semantics;
* compromise handling.

### Cryptography

* Ed25519 identity authentication;
* X25519 key agreement;
* ML-KEM-768 hybrid key establishment;
* authenticated transcript;
* secure key derivation;
* Double Ratchet integration;
* key zeroization;
* secure failure handling.

### Protocol

* protocol versioning;
* canonical serialization;
* authenticated message format;
* replay protection;
* malformed frame rejection;
* size limits;
* protocol compatibility behavior.

### Storage

* encrypted persistent records;
* storage key lifecycle;
* storage key destruction on lock/logout;
* crash recovery;
* schema migrations;
* backup and restore;
* corruption handling;
* search-index protection.

### Desktop Security

* strict Tauri CSP;
* minimized Tauri capabilities;
* audited filesystem access;
* secure lock/unlock behavior;
* sensitive clipboard handling;
* secure notification behavior;
* controlled logging;
* crash-data review.

## Relay Requirements

Before exposing a relay to the public internet, the following must be verified:

### Authentication

* authenticated peer registration;
* transport/application identity binding;
* connection ownership;
* duplicate connection handling;
* lifecycle identifiers.

### TLS

* production certificates;
* trusted certificate validation;
* certificate rotation;
* private-key protection;
* certificate expiration monitoring.

### Resource Protection

* connection limits;
* per-peer limits;
* global limits;
* queue limits;
* message-size limits;
* frame-size limits;
* rate limiting;
* backpressure;
* abuse protection.

### Availability

* connection recovery;
* network partition handling;
* relay restart behavior;
* queue expiration;
* graceful shutdown;
* resource exhaustion behavior.

### Privacy

Relay infrastructure must not log or persist application plaintext.

Operational telemetry must be reviewed for unnecessary metadata collection.

## Host Security

Production relay hosts should use a dedicated service account and minimize
the privileges available to the relay process.

Recommended controls include:

* restricted filesystem permissions;
* dedicated service user;
* firewall rules;
* restricted administrative access;
* secure SSH configuration where applicable;
* automatic security updates;
* time synchronization;
* disk monitoring;
* memory monitoring;
* CPU monitoring;
* certificate expiration monitoring.

The exact host hardening procedure depends on the target operating system and
deployment environment.

## Network Exposure

Only required ports and protocols should be exposed.

A production relay should not expose development or administrative interfaces
to the public internet.

Administrative access should use a separate controlled management path.

## Containers

If containerized deployment is introduced, production images should:

* use minimal base images;
* run as a non-root user;
* contain no development secrets;
* pin relevant dependencies;
* expose only required ports;
* use read-only filesystems where practical;
* define CPU and memory limits;
* provide health checks;
* generate reproducible image metadata.

Containerization does not replace application-level security controls.

## Observability

Production infrastructure requires sufficient observability to detect:

* connection failures;
* abnormal connection rates;
* queue exhaustion;
* memory pressure;
* CPU exhaustion;
* repeated protocol failures;
* certificate problems;
* service restarts;
* network failures.

Observability must not require collecting message contents.

Metrics should be designed to minimize unnecessary metadata retention.

## Incident Response

A production deployment must define procedures for:

* compromised relay;
* compromised identity;
* leaked server credentials;
* TLS private-key compromise;
* protocol vulnerability;
* dependency vulnerability;
* denial-of-service;
* storage corruption;
* unexpected service termination.

Security incidents should include:

1. detection;
2. containment;
3. investigation;
4. remediation;
5. credential/key rotation where necessary;
6. affected-version assessment;
7. post-incident review.

## Backup and Recovery

Production infrastructure should define what state requires backup.

The experimental relay currently relies on in-memory queues, meaning queued
messages may be lost after process termination.

Durable relay message storage must not be assumed unless explicitly implemented
and documented.

Critical deployment configuration and certificates should have appropriate
backup and recovery procedures.

Private keys must be backed up only when the threat model and operational
requirements justify doing so.

## Updates

Production deployments should use a controlled update process.

Before deploying an update:

* review protocol changes;
* review cryptographic changes;
* run automated tests;
* run security checks;
* review dependency changes;
* verify migration behavior;
* verify rollback procedures.

Security-sensitive changes should not be deployed solely because a build
succeeds.

## Release Gates

A Marrow release intended for production security claims should satisfy the
following gates.

### Gate 1 — Build Verification

* reproducible or documented release build;
* formatting passes;
* linting passes;
* tests pass;
* frontend build passes;
* release artifacts verified.

### Gate 2 — Protocol Verification

* protocol serialization tested;
* versioning defined;
* malformed frames rejected;
* size limits enforced;
* replay handling tested;
* compatibility behavior documented.

### Gate 3 — Cryptographic Verification

* authenticated handshake implemented;
* transcript binding verified;
* identity binding verified;
* Double Ratchet integration verified;
* key lifecycle audited;
* sensitive material zeroization reviewed.

### Gate 4 — End-to-End Verification

At minimum:

```text
Identity creation
       ↓
Authenticated handshake
       ↓
Session establishment
       ↓
Alice → Bob message
       ↓
Bob → Alice response
       ↓
Out-of-order delivery
       ↓
Replay rejection
       ↓
Application restart
       ↓
Continued messaging
```

This path must be covered by automated tests.

### Gate 5 — Security Testing

Required testing should include:

* protocol fuzzing;
* handshake fuzzing;
* ratchet state testing;
* storage corruption testing;
* malformed input testing;
* resource exhaustion testing;
* dependency auditing;
* Tauri IPC audit;
* filesystem access audit;
* CSP review.

### Gate 6 — Performance Verification

Performance claims must be backed by reproducible benchmarks.

Relevant areas include:

* handshake latency;
* message processing;
* storage operations;
* search;
* network delivery;
* relay throughput;
* relay memory consumption.

See [`../benchmarks/README.md`](../benchmarks/README.md).

### Gate 7 — Independent Review

Production security claims should require an independent cryptographic and
security review.

The review should cover at least:

* protocol design;
* cryptographic composition;
* identity authentication;
* key lifecycle;
* Double Ratchet integration;
* storage encryption;
* metadata exposure;
* relay architecture;
* Tauri security boundary.

## Current Production Status

Marrow is not currently considered production-ready.

The project still has implementation and verification work remaining in areas
including:

* authenticated hybrid handshake;
* complete Double Ratchet application integration;
* end-to-end client verification;
* search-index protection;
* key lifecycle hardening;
* relay production hardening;
* resource exhaustion protection;
* protocol versioning;
* fuzzing;
* security auditing;
* reproducible performance benchmarking.

Production readiness must be established through implementation and
verification rather than inferred from the existence of individual
cryptographic primitives.
