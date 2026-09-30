# Marrow Benchmarks

This directory contains performance benchmarks and measurement methodology
for the Marrow project.

Benchmarks are intended to provide reproducible measurements for cryptographic
operations, session establishment, message processing, local storage, search,
networking, and relay infrastructure.

Performance claims are not considered valid unless they can be reproduced from
a documented benchmark procedure and associated with a specific repository
revision.

## Benchmark Categories

### Cryptography

Cryptographic primitive benchmarks cover:

- X25519 key agreement;
- ML-KEM-768 encapsulation;
- ML-KEM-768 decapsulation;
- Ed25519 signing;
- Ed25519 verification;
- XChaCha20-Poly1305 encryption;
- XChaCha20-Poly1305 decryption.

### Session Security

Session-level benchmarks cover:

- Double Ratchet message processing;
- ratchet state transitions;
- message encryption;
- message decryption;
- skipped-message-key processing;
- handshake processing;
- hybrid session establishment.

### Local Storage

Storage benchmarks cover:

- encrypted record writes;
- encrypted record reads;
- contact persistence;
- session persistence;
- message persistence;
- storage initialization;
- storage recovery operations where applicable.

### Local Search

Search benchmarks cover:

- Tantivy document indexing;
- search query latency;
- search throughput;
- index size;
- indexing overhead.

Search benchmarks must distinguish between plaintext indexing and any future
encrypted or protected search-index implementation.

### Networking

Network benchmarks cover:

- connection establishment;
- handshake latency;
- message delivery latency;
- transport throughput;
- reconnection;
- concurrent connections;
- network overhead.

### Relay

Relay benchmarks cover:

- active peer routing;
- offline queue operations;
- message forwarding throughput;
- queue eviction;
- connection capacity;
- memory consumption per connected peer;
- backpressure behavior.

Relay measurements must identify whether the benchmark uses the current
experimental relay implementation or a future production implementation.

## Benchmark Status

| Area | Status |
| --- | --- |
| Cryptographic primitives | Planned |
| Hybrid handshake | Planned |
| Double Ratchet | Planned |
| Encrypted storage | Planned |
| Tantivy search | Planned |
| Network transport | Planned |
| Relay throughput | Planned |
| Relay memory usage | Planned |
| End-to-end message latency | Planned |

No performance figures are published yet.

## Reproducibility

Every published benchmark result should identify:

- repository commit;
- benchmark version;
- operating system;
- CPU;
- RAM;
- Rust toolchain;
- compiler version;
- relevant dependency versions;
- build profile;
- benchmark command;
- benchmark parameters;
- sample size;
- warm-up configuration;
- statistical method.

See [`methodology.md`](methodology.md) for the measurement requirements.

See [`results.md`](results.md) for published benchmark results.

## Interpreting Results

Benchmark results describe measured behavior under a specific environment.
They do not constitute universal performance guarantees.

Results can vary because of:

- CPU architecture;
- operating system;
- compiler version;
- CPU frequency scaling;
- thermal throttling;
- memory pressure;
- background processes;
- storage device;
- network conditions;
- dependency versions;
- build configuration.

Comparisons between different commits should therefore use the same benchmark
methodology and, where possible, the same hardware and software environment.

## Performance Claims

Marrow does not currently make fixed claims about:

- memory consumption;
- message throughput;
- handshake latency;
- search latency;
- relay throughput;
- startup time;
- CPU usage.

Such claims will only be added after reproducible measurements are available.