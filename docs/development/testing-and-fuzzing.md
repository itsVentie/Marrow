# Testing and Fuzzing

## Testing strategy

Marrow requires testing at multiple layers:

```text
Unit
  ↓
Crate integration
  ↓
Protocol / crypto integration
  ↓
Network integration
  ↓
Application E2E
  ↓
Security / fuzz / resource testing
```

A passing unit-test suite does not establish that the complete desktop communication path is correct.

## Rust unit tests

Core crates should test:

- valid cryptographic operations;
- invalid inputs;
- boundary values;
- serialization/deserialization;
- state transitions;
- storage behavior;
- network framing.

Run:

```powershell
cargo test --workspace --all-features
```

## Formatting and linting

Run:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Formatting failures should block CI rather than being treated as cosmetic.

## Cryptographic tests

Crypto tests should cover:

### Identity

- deterministic derivation;
- invalid mnemonic;
- invalid checksum;
- wrong passphrase;
- key serialization;
- identity verification.

### X25519

- valid agreement;
- invalid/all-zero shared-secret handling;
- malformed key material;
- role symmetry.

### ML-KEM

- encapsulation/decapsulation;
- malformed ciphertext;
- wrong key;
- failure handling.

### Hybrid handshake

- successful initiator/responder exchange;
- transcript modification;
- signature modification;
- wrong identity;
- wrong role;
- wrong X25519 key;
- wrong ML-KEM material;
- replay;
- timeout.

## Double Ratchet tests

Required coverage:

- sequential messages;
- bidirectional messages;
- DH ratchet;
- out-of-order messages;
- delayed messages;
- skipped-message keys;
- maximum skip boundary;
- replay;
- invalid authentication tag;
- concurrent state access;
- persisted state restore;
- crash/restart behavior.

## Storage tests

Test:

- encrypted-at-rest records;
- wrong storage key;
- corruption;
- interrupted writes;
- migration;
- session cleanup;
- logout cleanup;
- search/index behavior;
- restart.

The Tantivy index requires separate security tests because message content can exist in an index independent of the encrypted redb records.

## Network tests

Test:

- framing;
- maximum frame size;
- malformed frames;
- connection loss;
- reconnect;
- duplicate connections;
- relay routing;
- offline queue limits;
- queue expiration;
- backpressure;
- hostile relay behavior.

## Complete application E2E

The highest-value integration test is:

```text
Alice application
      ↓
identity
      ↓
handshake
      ↓
network / relay
      ↓
Bob application
      ↓
ratchet decrypt
      ↓
storage
      ↓
UI event
```

The reverse direction must also be tested.

The repository currently has crypto/integration tests but does not yet have complete application-level Alice/Bob coverage for the entire path.

## Restart tests

At minimum:

```text
Alice starts
Bob starts
handshake
message A → B
message B → A
Alice restart
Bob restart
message A → B
message B → A
```

The test must verify that ratchet state and storage state remain consistent.

## Property testing

Useful property tests include:

- encode/decode round trips;
- protocol boundary values;
- frame-size invariants;
- ratchet counter invariants;
- storage serialization invariants;
- error handling under arbitrary malformed input.

## Fuzzing targets

Recommended fuzz targets:

```text
protocol frame decoder
message decoder
handshake parser
identity-file parser
encrypted payload parser
relay frame parser
Bincode compatibility boundaries
ratchet header parser
```

Fuzzers should enforce bounded resource usage and must not require valid cryptographic secrets.

## Fuzzing principles

A fuzz target should be:

- deterministic where possible;
- bounded;
- isolated;
- free of external network dependencies;
- free of real user data.

The goal is to discover panics, excessive allocation, parser inconsistencies, state-machine violations and authentication bypasses.

## Additional verification

Security verification should eventually include:

- Miri where applicable;
- AddressSanitizer where supported;
- dependency audit;
- `cargo deny`;
- cryptographic API review;
- Tauri IPC review;
- capability review;
- filesystem audit.

## CI requirements

CI should eventually cover:

```text
Rust fmt
Rust clippy
Rust tests
Frontend install
Frontend typecheck
Frontend build
Security/dependency checks
Relevant fuzz smoke tests
```

Release CI should additionally verify built artifacts.

## Test data

Never use real identity keys, mnemonics or private messages in committed fixtures.

Use deterministic test vectors only where the test explicitly requires deterministic cryptographic material.
