# Contributing

## Scope

Contributions to Marrow may affect cryptography, protocol behavior, storage, networking, the Tauri IPC boundary or the frontend.

Security-sensitive changes require more review than ordinary UI changes.

## Before starting

Read:

- project README;
- relevant architecture documentation;
- protocol documentation;
- security documentation;
- benchmark methodology when changing performance-sensitive code.

If a relevant document does not yet describe the behavior, update the documentation as part of the change.

## Branches

Use focused branches for independent changes.

Keep commits logically scoped. Avoid combining:

- protocol changes;
- unrelated frontend refactors;
- formatting-only changes;
- dependency upgrades

unless they are genuinely part of one change.

## Pull requests

A useful pull request should explain:

1. what changed;
2. why it changed;
3. affected security boundaries;
4. compatibility implications;
5. tests performed;
6. known limitations;
7. documentation updated.

## Cryptography changes

Changes involving cryptographic code must explicitly describe:

- primitive;
- inputs/outputs;
- key lifecycle;
- failure behavior;
- nonce behavior;
- authentication;
- compatibility;
- test coverage.

Do not introduce custom cryptographic primitives when a reviewed standard primitive is appropriate.

Do not describe a construction as post-quantum secure merely because it contains a PQ primitive; the complete construction and authentication binding matter.

## Protocol changes

Any change to serialized structures, frame types, handshake messages, ratchet headers or error identifiers must update:

```text
docs/protocol/
```

Protocol changes must address:

- versioning;
- compatibility;
- downgrade resistance;
- malformed inputs;
- replay;
- limits.

## Storage changes

Storage changes must consider:

- encryption at rest;
- key lifecycle;
- crash recovery;
- migrations;
- corruption;
- backups;
- search indexes;
- deletion semantics.

Do not assume that encrypting the primary database automatically encrypts secondary indexes or caches.

## Network changes

Network changes must consider:

- frame limits;
- connection limits;
- backpressure;
- queue bounds;
- timeouts;
- malformed frames;
- duplicate connections;
- relay behavior;
- metadata exposure.

## Tauri / IPC changes

IPC changes must consider:

- input validation;
- capability scope;
- filesystem access;
- secret exposure;
- error serialization;
- CSP;
- lifecycle state.

## Testing expectations

At minimum, run relevant checks before opening a PR:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

For frontend changes:

```powershell
cd apps/Marrow
pnpm exec tsc --noEmit
pnpm build
```

For security-sensitive changes, add targeted tests.

## Documentation

Documentation should distinguish:

- implemented;
- partially implemented;
- planned;
- experimental;
- blocked.

Avoid performance or security claims without reproducible evidence.

## Benchmarks

Performance claims should follow:

```text
docs/benchmarks/methodology.md
```

Do not add hand-measured numbers to README files as if they were formal benchmarks.

## Commit messages

Use focused commit messages, for example:

```text
docs(protocol): document handshake format
fix(crypto): bind responder key to handshake
test(network): add malformed-frame coverage
docs(deployment): document relay limits
```

## Security issues

Do not disclose an exploitable vulnerability in a public issue before following the project's security-reporting process.

See the repository security policy for the current reporting channel.

## Review standard

A contribution is not complete merely because it compiles.

Reviewers should consider:

- correctness;
- security;
- lifecycle;
- compatibility;
- resource usage;
- observability;
- test coverage;
- documentation.
