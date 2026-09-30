# Debugging

## General rule

Debugging must not weaken the security model or expose secret material.

Never solve a debugging problem by printing:

- private keys;
- mnemonics;
- root keys;
- chain keys;
- message keys;
- plaintext messages;
- storage encryption keys;
- session secrets.

## Recommended debugging layers

Debug from the outside inward:

```text
UI
 ↓
Tauri IPC
 ↓
application state
 ↓
network
 ↓
protocol
 ↓
cryptographic/session state
 ↓
storage
```

This helps isolate whether a failure is transport, protocol, state or cryptographic.

## Build failures

Run:

```powershell
cargo check --workspace --all-features
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

For frontend failures:

```powershell
cd apps/Marrow
pnpm install
pnpm exec tsc --noEmit
pnpm build
```

## IPC debugging

Verify:

1. command name;
2. frontend argument shape;
3. Rust command input type;
4. returned error;
5. application-state availability;
6. command lifecycle.

Do not add broad filesystem permissions just to diagnose an IPC failure.

## Identity debugging

Check:

```text
identity file exists
        ↓
file parses
        ↓
local protection succeeds
        ↓
Ed25519 identity reconstructs correctly
        ↓
runtime state initializes
```

Use public identity fingerprints for diagnostics rather than private key material.

## Handshake debugging

Use non-secret state transitions:

```text
handshake created
handshake sent
handshake received
identity verification result
key agreement result
session initialized
handshake completed
```

Never log the raw shared secrets.

For failures, compare:

- protocol version;
- suite;
- role;
- public-key fingerprints;
- message lengths;
- state transitions;
- timeout behavior.

A critical invariant is that the responder's ratchet DH private key must correspond to the public key advertised by the handshake.

## Ratchet debugging

Useful safe diagnostics:

```text
role
DH public-key fingerprint
message number
previous-chain length
chain direction
skipped-key count
session state version
```

Do not print chain keys or message keys.

When messages fail to decrypt, determine whether the problem is:

1. wrong session;
2. wrong role initialization;
3. mismatched DH key;
4. incorrect chain direction;
5. incorrect counter;
6. skipped-key handling;
7. persisted-state rollback;
8. authentication failure.

## Network debugging

Inspect:

- peer IDs;
- connection lifecycle;
- frame type;
- frame length;
- route target;
- queue depth;
- timeout;
- reconnect state.

Do not log full encrypted payloads in production.

## Relay debugging

Safe relay diagnostics include:

```text
peer identifier
connection generation
route target
queue length
frame length
connection state
```

Do not log decrypted application data because the relay should not have access to it.

## Storage debugging

For encrypted storage problems, distinguish:

```text
database unavailable
storage key unavailable
record authentication failed
record missing
record corrupted
schema migration failed
search index unavailable
```

Do not dump encrypted records together with the storage key.

## Search debugging

The Tantivy search index is a separate data surface from encrypted redb records.

If search returns unexpected data, inspect:

- indexing event;
- commit lifecycle;
- document ID;
- query parser;
- index state.

Do not assume the encryption of primary storage automatically protects the search index.

## Logging policy

### Safe

```text
session initialized
peer connected
frame rejected: oversized
handshake timeout
storage migration completed
```

### Unsafe

```text
root_key=...
private_key=...
mnemonic=...
plaintext=...
storage_key=...
```

## Reproduction

For a bug report, capture:

- Marrow version/commit;
- operating system;
- Rust toolchain;
- frontend/runtime version;
- exact reproduction steps;
- relevant non-secret logs;
- expected behavior;
- actual behavior.

For protocol bugs, include the protocol version and suite if available.

## Minimal reproduction

Prefer a deterministic test over a long manual sequence.

A good debugging workflow is:

```text
reproduce
  ↓
reduce
  ↓
write failing test
  ↓
fix
  ↓
run regression suite
  ↓
document security/compatibility impact
```
