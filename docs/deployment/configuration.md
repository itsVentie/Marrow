# Deployment Configuration

This document describes configuration requirements and conventions for Marrow
deployment components.

The configuration model is currently evolving together with the relay and
networking architecture.

Configuration names documented here must not be interpreted as stable API
names unless they are explicitly implemented by the current release.

## Configuration Principles

Deployment configuration should follow these principles:

- secure defaults;
- explicit resource limits;
- no plaintext message logging;
- no secrets embedded in source code;
- separate development and production configuration;
- explicit TLS configuration;
- bounded memory usage;
- bounded queue sizes;
- deterministic protocol limits;
- auditable configuration changes.

## Environment Separation

Marrow deployments should distinguish between:

### Development

Development environments may use:

- local addresses;
- development certificates;
- verbose diagnostics;
- temporary storage;
- reduced resource limits;
- local-only access.

Development configuration must not automatically be reused for production.

### Testing

Testing environments should provide:

- deterministic configuration;
- isolated storage;
- controlled network conditions;
- reproducible certificates or test trust roots;
- explicit resource limits.

Testing configuration should avoid external production infrastructure unless
the test specifically targets production integration.

### Production

Production configuration requires:

- trusted TLS certificates;
- restricted filesystem permissions;
- explicit resource limits;
- secure secret management;
- controlled logging;
- monitoring;
- operational recovery procedures;
- documented network exposure.

## Network Configuration

A relay deployment must explicitly define:

- listening address;
- listening port;
- transport protocol;
- maximum frame size;
- maximum message size;
- maximum concurrent connections;
- connection timeout;
- idle timeout;
- queue limits;
- queue message TTL.

The exact configuration interface is implementation-dependent until the relay
configuration API is stabilized.

## TLS Configuration

Production TLS configuration must define:

- certificate location or secret-provider integration;
- private-key location or secret-provider integration;
- certificate validation policy;
- certificate rotation procedure;
- minimum supported protocol configuration;
- failure behavior for invalid or expired certificates.

Private keys must never be committed to the repository.

Development certificates must be clearly separated from production
certificates.

## Queue Configuration

Offline relay queues should be bounded by both:

- per-peer limits;
- global limits.

Recommended configuration concepts include:

```text
maximum messages per peer
maximum bytes per peer
maximum total queued messages
maximum total queued bytes
message TTL
queue eviction policy
````

A relay must fail closed or apply controlled eviction when resource limits are
reached rather than allowing unbounded memory growth.

## Connection Limits

Production relay configuration should include:

```text
maximum total connections
maximum connections per peer
connection establishment rate
idle connection timeout
handshake timeout
```

These limits are required to reduce resource-exhaustion risk.

## Logging

Production logging must not contain:

* plaintext message contents;
* private keys;
* session keys;
* storage encryption keys;
* passphrases;
* BIP-39 recovery phrases;
* decrypted application payloads.

Operational logs may contain limited infrastructure information such as:

* service lifecycle events;
* connection failures;
* protocol errors;
* resource-limit events;
* certificate errors;
* queue-limit events;
* version information.

Logging must be reviewed for metadata leakage before production deployment.

## Secrets

Secrets must be supplied through an appropriate secret-management mechanism.

Do not store production secrets in:

* Git;
* source files;
* Docker images;
* public configuration files;
* shell history;
* debug logs.

Sensitive values should have the minimum required permissions and lifetime.

## Filesystem Permissions

Production service accounts should have access only to the files and
directories required by the service.

Where applicable:

* configuration files should not be writable by the service;
* private keys should have restricted permissions;
* logs should use controlled ownership;
* temporary directories should be isolated;
* persistent state should use dedicated directories.

## Resource Limits

Deployments should explicitly define:

| Resource                   | Requirement  |
| -------------------------- | ------------ |
| Maximum connections        | Bounded      |
| Maximum message/frame size | Bounded      |
| Per-peer queue             | Bounded      |
| Global queue               | Bounded      |
| Connection rate            | Limited      |
| CPU-intensive operations   | Rate limited |
| Memory                     | Bounded      |
| Offline message lifetime   | TTL          |

Exact values should be determined through benchmarking and abuse testing rather
than arbitrary defaults.

## Configuration Validation

The service should validate configuration before starting.

Invalid configuration should result in startup failure rather than silently
falling back to insecure values.

Examples include:

* invalid certificate path;
* inaccessible private key;
* invalid port;
* zero or excessively large resource limits;
* invalid timeout;
* invalid queue configuration;
* incompatible protocol configuration.

## Configuration Versioning

Configuration changes that affect protocol behavior or security properties
should be documented and versioned.

Configuration compatibility must be considered separately from protocol
compatibility.

A deployment configuration change must not silently change cryptographic
semantics.

## Current Status

The Marrow deployment configuration model is not yet considered stable.

The relay currently requires additional work before a complete production
configuration interface can be documented.

Until then, implementation-specific configuration should be treated as
experimental and subject to change.
