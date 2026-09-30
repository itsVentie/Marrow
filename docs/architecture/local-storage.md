# Local Storage Architecture

**Status:** Encrypted persistent records implemented; search protection and lifecycle hardening pending

## 1. Purpose

Marrow persists application data locally so that conversations, contacts, and session-related information can survive application restarts.

The storage architecture separates:

1. encrypted persistent records;
2. search indexing;
3. in-memory application/session state.

These surfaces have different security properties.

## 2. Storage architecture

Conceptually:

```text
Rust Application
      |
      +--------------------+
      |                    |
      v                    v
 Encrypted redb        Tantivy index
      |                    |
      v                    v
Persistent records     Search data
```

The Tantivy index must not be assumed to inherit the encryption properties of the redb database.

## 3. Persistent records

Application records are serialized and encrypted before being written to redb.

The intended properties are:

- confidentiality at rest;
- authenticated integrity;
- fresh nonce usage;
- separation of storage keys from session keys.

The database itself should therefore not contain ordinary plaintext message records when accessed directly.

## 4. Encryption

XChaCha20-Poly1305 is used for encrypted persistent records.

A fresh random nonce is required for each encryption operation.

Associated data should be defined consistently if record type/version binding is used.

The storage format should include enough version/context information to support future migrations.

## 5. Storage key lifecycle

The storage encryption key is sensitive application state.

It must not:

- be logged;
- be persisted in plaintext;
- remain available after logout unnecessarily;
- be reused as a ratchet or identity key.

A complete lifecycle should be:

```text
Unlock
  |
  v
Derive/load storage key
  |
  v
Use
  |
  v
Lock/logout
  |
  v
Zeroize / release
```

Current implementation requires further hardening around key cleanup.

## 6. Search index

Tantivy is a separate storage surface.

If message plaintext is indexed and stored by Tantivy, the search index can expose plaintext independently of encrypted redb records.

Therefore:

> Encrypted local database records do not imply encrypted local search.

Before claiming encrypted local search, the project must either:

- encrypt/protect the index;
- redesign indexing so sensitive content is not stored in plaintext;
- or explicitly document the index as a local plaintext data surface.

## 7. Search writer architecture

Search indexing should avoid creating and committing a heavyweight writer for every individual message.

A persistent writer/worker model is preferable:

```text
Message event
     |
     v
Bounded indexing queue
     |
     v
Persistent Tantivy writer
     |
     v
Periodic commit
```

The queue must have bounded memory and defined shutdown behavior.

## 8. Storage consistency

Message handling should define the order between:

1. ratchet state update;
2. message persistence;
3. search indexing;
4. frontend event emission.

Incorrect ordering can create inconsistent states after crashes.

A recovery strategy is required for interrupted operations.

## 9. Crash recovery

Storage must account for:

- interrupted writes;
- partially completed message handling;
- database corruption;
- stale search entries;
- missing search entries;
- inconsistent ratchet/session state;
- migration failures.

A production design should have explicit recovery procedures rather than assuming process termination is always clean.

## 10. Logout and cleanup

Logout should clear or invalidate:

- storage encryption keys;
- in-memory session state;
- pending handshakes;
- ratchet state;
- sensitive identity-derived buffers;
- network runtime state where appropriate.

Search/index state also requires explicit policy.

Deleting only the database key is not equivalent to deleting every local plaintext representation.

## 11. Backups

A backup architecture must define:

- what is backed up;
- encryption of backups;
- key ownership;
- restore procedure;
- version compatibility;
- corruption detection;
- recovery from incomplete backups.

The current application should not imply that arbitrary filesystem copies constitute secure backups.

## 12. Metadata

Even encrypted records can reveal some metadata through:

- file size;
- index size;
- timestamps;
- record counts;
- filesystem behavior;
- backup copies.

Metadata protection is separate from content encryption.

## 13. Storage security checklist

Before production readiness:

- [ ] encrypted records verified;
- [ ] storage key zeroization verified;
- [ ] search-index protection resolved;
- [ ] migrations tested;
- [ ] crash recovery tested;
- [ ] corruption handling tested;
- [ ] interrupted writes tested;
- [ ] logout cleanup tested;
- [ ] backup/restore semantics documented;
- [ ] sensitive logging reviewed;
- [ ] temporary-file behavior reviewed.
