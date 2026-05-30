---
title: AI-Brains Project Skill
description: How to build, develop, and operate the AI-Brains local-first memory vault.
category: devops
version: 1.0.0
---

# AI-Brains Project Skill

## What This Is

AI-Brains is a local-first, event-sourced memory vault with:
- **SQLCipher** encrypted SQLite database (`vault.db`)
- **FTS5** full-text search over conversation turns and ingested documents
- **Nightly summarization** job for session compaction
- **ChangeGuard bridge** for safety signals (HOTSPOT, DECISION, CONSTRAINT)
- **Graph projection** layer for relational queries
- **Rust workspace** with ~15 crates under `crates/`

## Where It Lives

| Component | Location |
|-----------|----------|
| Source code | `C:\dev\AI-Brains` |
| Cargo binary | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` |
| Vault database | `C:\dev\ai-brains\vault.db` |
| Conductor tracks | `C:\dev\AI-Brains\conductor\tracks\` |
| Obsidian vault | `C:\Users\RyanB\Documents\Hermes\` |

## Build Requirements

### Windows Cross-Compilation (Primary Target)

The project targets `x86_64-pc-windows-gnu` for Windows builds:

```bash
# 1. Install MinGW-w64 toolchain on WSL
sudo apt-get update
sudo apt-get install -y gcc-mingw-w64-x86-64 binutils-mingw-w64-x86-64

# 2. Configure Cargo linker
cat > ~/.cargo/config.toml << 'EOF'
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-ar"
EOF

# 3. Verify
which x86_64-w64-mingw32-gcc
x86_64-w64-mingw32-gcc --version

# 4. Build
cargo build --target x86_64-pc-windows-gnu -p ai-brains-cli
```

### Linux Native Build

```bash
cargo build --release -p ai-brains-cli
```

## Common Commands

### Vault Operations
```bash
# Initialize a new vault
ai-brains init --vault-path C:\dev\ai-brains\vault.db

# Ingest a file or directory
ai-brains ingest --vault-path C:\dev\ai-brains\vault.db --project-id <ID> --path C:\path\to\notes\n
# Query with FTS5
ai-brains recall --vault-path C:\dev\ai-brains\vault.db "your query" --limit 5

# Preflight (project-scoped safety signals)
ai-brains preflight --vault-path C:\dev\ai-brains\vault.db --project-id <ID> --summary

# Nightly summarization
ai-brains nightly --vault-path C:\dev\ai-brains\vault.db

# Check nightly status
ai-brains nightly --status --vault-path C:\dev\ai-brains\vault.db
```

### Testing
```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p ai-brains-path
cargo test -p ai-brains-store
cargo test -p ai-brains-cli

# Run with specific target
cargo test --target x86_64-pc-windows-gnu -p ai-brains-cli
```

### Conductor Workflow
All tracks follow the Conductor pattern:
1. **Spec** in `conductor/tracks/trackTNN-<name>/spec.md`
2. **Plan** in `conductor/tracks/trackTNN-<name>/plan.md`
3. Branch from main: `track-tNN-<name>`
4. Implement → test → lint
5. Update `conductor/conductor.md` registry
6. Commit → push → PR

## Key Architecture

### Crate Layout
| Crate | Purpose |
|-------|---------|
| `ai-brains-core` | IDs, privacy types, session model |
| `ai-brains-store` | SQLCipher event store + projections |
| `ai-brains-path` | Windows/WSL/UNC path normalization |
| `ai-brains-capture` | CLI/daemon capture pipeline |
| `ai-brains-retrieval` | FTS5 search + preflight assembly |
| `ai-brains-graph` | CozoProxy + graph projection |
| `ai-brains-cli` | Main CLI binary |
| `ai-brainsd` | Background daemon |

### Preflight vs Recall
- **Preflight**: Project-scoped, structured data only (HOTSPOT, DECISION, CONSTRAINT), requires `project_id`
- **Recall**: Cross-project FTS5 search, returns all content types, no project scoping

For cross-project context (e.g., "what is Ryan working on?"), use `recall`.
For repo-specific safety signals when coding in a specific project, use `preflight`.

## Windows-Specific Notes

1. **Paths**: Use forward slashes or escaped backslashes in PowerShell: `C:\\dev\\ai-brains`
2. **PowerShell encoding**: Always set `$OutputEncoding = [System.Text.Encoding]::UTF8`
3. **Cargo config**: `~/.cargo/config.toml` is required for `x86_64-pc-windows-gnu` linker
4. **Binary location**: After `cargo install`, binary is at `C:\Users\RyanB\.cargo\bin\ai-brains.exe`

## Troubleshooting

### "linker not found" error
```bash
# MinGW not installed
sudo apt-get install gcc-mingw-w64-x86-64 binutils-mingw-w64-x86-64
```

### Vault locked / SQLite busy
```bash
# Check for running daemon
ps aux | grep ai-brainsd
# Or: ai-brains sync query --vault-path <path>  # auto-starts daemon if needed
```

### Tests fail on path canonicalization
See Track T58: Unix absolute paths need `starts_with('/')` check in `canonical.rs`.

## Safety & Privacy

- **Privacy filter** runs on all ingested content — never expose API keys, passwords, or tokens
- **Redaction** automatically masks `sk-...`, `ghp_...`, `SG.*` patterns
- **Pinned memories** (HOTSPOT, CONSTRAINT, DECISION) are promoted to preflight context
- Use `--quiet` flag on bridge commands to suppress stderr noise
