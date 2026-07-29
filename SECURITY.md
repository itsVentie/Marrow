# Security Policy

`marrow` is built with a zero-trust design philosophy. As a post-quantum messaging client dealing with asymmetric cryptography and sensitive communication state, security is our highest priority.

---

## Supported Versions

Only the latest commit on the `main` branch and the most recent release tag are supported with security updates.

| Version | Supported |
| --- | --- |
| Main Branch | :white_check_mark: Yes |
| Pre-release Tags (`v0.x`) | :white_check_mark: Yes |
| Old Commits | :x: No |

---

## Reporting a Vulnerability

> [!CAUTION]
> **DO NOT** create public GitHub Issues or discussions for security vulnerabilities or cryptographic defects.

If you discover a security flaw, vulnerability, or cryptographic weakness in `marrow`:

1. Contact the primary maintainer directly via encrypted channels or private message.
2. Provide a detailed summary containing:
   * Affected component (`crates/crypto`, `crates/protocol`, `apps/relay`, IPC bridge, etc.)
   * Vector analysis or Proof of Concept (PoC)
   * Impact estimation (e.g., Memory disclosure, Key leakage, Relay impersonation, Denial of Service)
3. Allow up to **48 hours** for an initial response acknowledging the report.

---

## Scope of Security Concerns

We are particularly interested in reports covering:

* Hybrid Handshake bypasses (X25519 + ML-KEM-768 flaws)
* Double Ratchet state desynchronization or key exposure
* Memory retention of keys (failure to zeroize secret states)
* Side-channel timing attacks in payload decryption
* Blind Relay metadata leakage or unauthenticated packet injection
