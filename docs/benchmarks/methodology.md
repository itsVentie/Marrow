# Benchmark Methodology

This document defines how Marrow performance measurements should be collected,
reported, and compared.

The goal is to make benchmark results reproducible and prevent isolated
measurements from being presented as general performance guarantees.

## 1. Reproducibility Requirements

Every published result must include:

| Property | Required |
| --- | --- |
| Commit hash | Yes |
| Date | Yes |
| Operating system | Yes |
| CPU model | Yes |
| CPU architecture | Yes |
| RAM | Yes |
| Rust version | Yes |
| Compiler version | Yes |
| Dependency state | Yes |
| Build profile | Yes |
| Benchmark command | Yes |
| Benchmark parameters | Yes |
| Sample count | Yes |
| Warm-up configuration | Yes |

If a value is not available, the benchmark result should not be presented as
fully reproducible.

## 2. Build Configuration

Performance benchmarks should normally use an optimized release build.

The exact build configuration must be recorded with the result.

Debug builds must not be compared directly with release builds.

Where benchmark-specific compiler flags or CPU features are enabled, they must
be documented explicitly.

## 3. Environment

Benchmarks should be performed on a system with:

- stable power conditions;
- minimal background CPU load;
- minimal background disk activity;
- no unrelated resource-intensive applications;
- stable thermal conditions where possible.

Virtual machines and heavily loaded systems should be explicitly identified.

## 4. Warm-up

Benchmarks involving cryptographic libraries, allocators, storage, or search
engines should perform an appropriate warm-up before collecting measurements.

Warm-up iterations must not be included in reported measurements.

The number of warm-up iterations should be documented.

## 5. Repetitions

A benchmark should perform enough iterations to reduce measurement noise.

For latency-sensitive operations, results should report distribution statistics
rather than only a single average.

Where supported, report:

- median / p50;
- p90;
- p95;
- p99;
- minimum;
- maximum.

For throughput-oriented operations, report:

- operations per second;
- bytes per second where applicable;
- total processed data.

## 6. Statistical Reporting

The arithmetic mean alone should not be used as the only latency metric.

Latency distributions can be affected by:

- operating-system scheduling;
- background processes;
- CPU frequency changes;
- allocator behavior;
- page faults;
- storage latency;
- network scheduling.

For this reason, percentile-based measurements are preferred for latency.

## 7. Cryptographic Benchmarks

Cryptographic benchmarks should measure individual primitives separately.

The following operations should be measured independently:

### X25519

- key generation;
- key agreement.

### ML-KEM-768

- key generation;
- encapsulation;
- decapsulation.

### Ed25519

- key generation;
- signing;
- verification.

### XChaCha20-Poly1305

- encryption;
- decryption.

Benchmark inputs should include fixed, documented payload sizes.

Where relevant, multiple payload sizes should be tested.

## 8. Double Ratchet Benchmarks

Double Ratchet measurements should distinguish between:

- initial session construction;
- sending-chain advancement;
- receiving-chain advancement;
- DH ratchet;
- skipped-message-key handling;
- message encryption;
- message decryption.

The benchmark must not accidentally measure unrelated network or storage
operations when measuring the ratchet itself.

## 9. Handshake Benchmarks

Handshake measurements should distinguish between:

1. cryptographic primitive cost;
2. local handshake processing;
3. complete application-level handshake latency;
4. network round-trip latency.

A network benchmark must document network topology and transport.

A local benchmark must not be described as network handshake latency.

## 10. Storage Benchmarks

Storage benchmarks should identify:

- database size;
- record size;
- payload size;
- encryption enabled/disabled where applicable;
- read/write operation type;
- sequential or random access;
- cold or warm cache conditions.

Encrypted storage measurements must include the encryption overhead as part of
the measured operation when the purpose is to evaluate application-level
storage performance.

## 11. Search Benchmarks

Tantivy benchmarks should distinguish:

- indexing;
- index commit;
- query execution;
- result retrieval.

Search benchmarks should record:

- number of indexed documents;
- average document size;
- query type;
- index size;
- whether the operating-system filesystem cache was warm.

If encrypted or otherwise protected search indexes are implemented later,
those measurements must be reported separately.

## 12. Network Benchmarks

Network benchmarks should record:

- transport;
- connection type;
- network topology;
- packet loss;
- latency;
- bandwidth;
- number of concurrent connections;
- payload size.

For local benchmarks, the environment must be identified as local.

For internet benchmarks, the geographic/network relationship between endpoints
should be documented where relevant.

## 13. Relay Benchmarks

Relay benchmarks should measure:

- active peer routing;
- forwarding throughput;
- offline queue insertion;
- offline queue delivery;
- TTL eviction;
- concurrent connections;
- memory usage;
- connection limits;
- backpressure.

Relay benchmarks must clearly identify the relay implementation and revision.

Experimental relay results must not be presented as production relay capacity.

## 14. Memory Measurements

Memory measurements should distinguish between:

- process resident memory;
- heap allocation;
- persistent storage;
- memory mapped files;
- operating-system filesystem cache.

A single RSS measurement should not be presented as the total memory
requirement of the application.

Memory results must specify:

- application state;
- number of contacts;
- number of messages;
- active sessions;
- connected peers;
- search-index size.

## 15. End-to-End Measurements

End-to-end measurements should cover the complete application path where
possible:

```text
identity
  ↓
handshake
  ↓
session establishment
  ↓
encryption
  ↓
transport
  ↓
delivery
  ↓
decryption
  ↓
storage
````

The benchmark should identify each stage separately when practical.

This prevents a fast cryptographic primitive from being incorrectly presented
as equivalent to low end-to-end message latency.

## 16. Regression Tracking

Benchmark results should eventually be associated with repository commits.

Performance regressions should be investigated when a change produces a
consistent degradation under the same benchmark environment.

Single noisy measurements should not automatically be treated as regressions.

## 17. Benchmark Integrity

Benchmarks must not intentionally disable security mechanisms solely to obtain
a better performance number unless the configuration is explicitly identified.

For example, the following must be distinguished:

* encrypted vs. unencrypted storage;
* authenticated vs. unauthenticated protocol paths;
* release vs. debug builds;
* protected vs. unprotected search indexes;
* production-like vs. development relay configuration.

Security-relevant configuration must always be stated.

## 18. Publishing Results

Published results should use the following format:

```text
Commit:
Date:
Environment:
Build:
Benchmark:
Parameters:
Iterations:
Warm-up:
Result:
Percentiles:
Notes:
```

Results without sufficient environment information should be treated as
informal measurements rather than reproducible benchmark results.

````
