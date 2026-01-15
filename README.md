# Archivist

**Archivist** is an encrypted, deduplicating, snapshot-based backup tool written in Rust.  
It is designed to be **privacy-first, incremental, and production-ready**.  

---

## Features

- **Encrypted storage**: all objects (blobs, trees, snapshots) are encrypted with AEAD using a repository key derived from a user password via Argon2.  
- **Deduplication**: content-addressed storage ensures the same file is stored only once.  
- **Incremental snapshots**: only changes are recorded between backups.  
- **Immutable snapshots**: each snapshot is a read-only, verifiable object.  
- **Repository integrity verification**: the `check` command ensures all blobs and trees exist and are untampered.  
- **CLI-first UX**: works on Linux/macOS with `--verbose` and `--quiet` modes.  
- **Atomic operations**: temporary files + fsync + rename to prevent partial writes.  
- **Single-writer safety**: repository locking prevents concurrent writes.  

---

## Architecture

```

┌───────────┐
│   CLI     │
│ (Commands)│
└─────┬─────┘
      │
      ▼
┌───────────────┐
│ Backup Engine │
│ - Scans FS    │
│ - Builds Trees│
└─────┬─────────┘
      │
      ▼
┌───────────────────────────┐
│ Object Model              │
│ ┌─────────┐               │
│ │ Blob    │  <-- Files    │
│ └─────────┘               │
│ ┌─────────┐               │
│ │ Tree    │  <-- Dir meta │
│ └─────────┘               │
│ ┌─────────┐               │
│ │Snapshot │               │
│ └─────────┘               │
└─────┬─────────────────────┘
      │
      ▼
┌───────────────┐
│ Local Backend │
│  (FS Storage) │
└─────┬─────────┘
      │
      ▼
┌───────────────┐
│ Crypto Core   │
│ - AEAD        │
│ - Argon2 KDF  │
└───────────────┘

````

---

## Threat Model

| Threat | Mitigation |
|--------|------------|
| Unauthorized read | AEAD encryption with repo key derived via Argon2 |
| Object tampering | SHA256 hash verification + AEAD integrity |
| Simultaneous writes | `.lock` file to enforce single-writer |
| Partial writes | Atomic write via temporary files + fsync + rename |
| Repo corruption | `check` command verifies all snapshots, trees, and blobs |

---

## Installation

1. **Clone the repository**

```bash
git clone https://github.com/yourusername/archivist.git
cd archivist
````

2. **Build**

```bash
cargo build --release
```

3. **Binary location**

```bash
./target/release/archivist
```

Or create a local symlink:

```bash
ln -s $(pwd)/target/release/archivist /usr/local/bin/archivist
```

---

## Usage

1. **Initialize a repository**

```bash
archivist init ~/myrepo
```

2. **Backup a directory**

```bash
archivist backup ~/Documents ~/myrepo
```

* Use `--verbose` to see internal hashes and debug info:

```bash
archivist --verbose backup ~/Documents ~/myrepo
```

* Use `--quiet` for silent cron-friendly backups:

```bash
archivist --quiet backup ~/Documents ~/myrepo
```

3. **List snapshots**

```bash
archivist snapshots ~/myrepo
```

4. **Restore a snapshot**

```bash
archivist restore <snapshot_hash> ~/myrepo ~/restore-target
```

* To restore the latest snapshot:

```bash
archivist restore latest ~/myrepo ~/restore-target
```

5. **Check repository integrity**

```bash
archivist check ~/myrepo
```

---


## Notes

* Target platforms: **Linux & macOS** (POSIX file metadata used)
* Rust toolchain: **1.71+ recommended**
* Designed for **CLI / scripting use**, no GUI

---

## License

MIT License © Mesam E Tamaar Khan