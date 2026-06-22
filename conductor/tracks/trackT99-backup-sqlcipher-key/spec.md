# Track T99: Fix `backup create` — Use VaultConnection with SQLCipher Key

**Status:** Complete
**Owner:** Claude
**Started:** 2026-06-22
**Started:** —
**Owner:** —
**Priority:** P1 — backups are the user's recovery mechanism; currently broken (hangs indefinitely).
**Source:** Non-destructive command test 2026-06-22.

---

## Problem Statement

`ai-brains backup create` hangs indefinitely. Root cause: `BackupService::run_backup()` in `crates/ai-brains-brain/src/backup.rs:42` opens the vault via `rusqlite::Connection::open(&self.vault_path)` **without applying SQLCipher key pragmas**. The vault is SQLCipher-encrypted, so this raw open cannot read the database. Additionally, the daemon may hold a write lock on the vault, and the raw connection has no `PRAGMA busy_timeout` (T96 only added it to `VaultConnection::open`), so SQLite blocks indefinitely instead of returning `SQLITE_BUSY`.

The same issue affects `run_restore` at `backup.rs:60` — it opens the vault with `rusqlite::Connection::open` without the key.

## Acceptance Criteria

**AC1:** `BackupService::run_backup()` uses `VaultConnection::open` (or applies the SQLCipher key pragmas manually) so the backup can read the encrypted vault.

**AC2:** `BackupService::run_restore()` uses `VaultConnection::open` for the destination vault so the restore writes an encrypted database.

**AC3:** `backup create` completes in <10 seconds on a 12MB vault and produces a valid, readable backup (passes `PRAGMA integrity_check`).

**AC4:** `backup restore --dry-run` works on the resulting backup file (verifies integrity without overwriting).

**AC5:** The backup file is SQLCipher-encrypted (not a plaintext copy).

**AC6:** `backup create` works even when the daemon is running (does not hang on lock contention). The raw connection used for backup must set `PRAGMA busy_timeout = 5000` alongside the key pragma (same as T96's `VaultConnection::open`).

**AC7:** `backup restore` checks if the daemon is running before overwriting the vault. If the daemon is running, it prints a warning to stderr advising the user to stop the daemon first. The probe uses `DaemonClient::probe()` (available in `crates/ai-brains-cli/src/daemon_client.rs:97`). Note: the probe is global (not per-vault), so it warns rather than aborts to avoid false positives when restoring a different vault file.

**AC8:** `backup create` deletes any existing backup file at the same timestamp path before creating the new one, preventing merge/corruption from a stale destination.

## Design Notes

- The simplest fix: change `BackupService` to accept the `SqlCipherKey` (or the `AppContext`) and use `VaultConnection::open` instead of `rusqlite::Connection::open`.
- The `BackupService::new` constructor currently takes only `vault_path: PathBuf`. Add a `key: SqlCipherKey` field.
- Update `backup.rs` (CLI) to pass `ctx._key` to `BackupService::new`.
- The SQLite backup API (`rusqlite::backup::Backup::new`) needs raw `Connection` objects. `VaultConnection` wraps a `Mutex<Connection>`. Access the inner connection via `conn.lock()` and pass references to the backup API.
- Alternatively, apply the key pragmas manually on the raw `rusqlite::Connection` before the backup: `conn.execute_batch(&format!("PRAGMA key = \"{}\"", key.expose_secret()))?` — this avoids changing `BackupService`'s structure.
- The manual pragma approach is simpler and avoids holding a lock on the `VaultConnection` mutex during the backup. Prefer this approach.

## Files

- `crates/ai-brains-brain/src/backup.rs` — add key to `BackupService`, apply pragmas (key + busy_timeout) before backup/restore, delete existing backup file before creating.
- `crates/ai-brains-cli/src/commands/backup.rs` — pass `ctx._key` to `BackupService::new`, add daemon-running check to `run_restore` using `DaemonClient::probe()`.
- `crates/ai-brains-brain/src/backup.rs` tests — update test to use a SQLCipher-encrypted vault.

## Tests (TDD)

**Red:** `backup_create__encrypted_vault__produces_valid_encrypted_backup` — creates a vault with `VaultConnection::open` (encrypted), writes a memory, runs `backup create`, verifies the backup file exists and is encrypted (opening without key fails, opening with key succeeds and passes integrity_check).

**Green:** Apply the key pragmas in `run_backup` and `run_restore`. Test passes.

## Verification

- `cargo nextest run -p ai-brains-brain`
- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains backup create` completes in <10s while daemon is running.
- Manual: `ai-brains backup restore --dry-run` on the resulting file succeeds.

## Out of Scope

- Changes to the daemon's lock behavior.
- Incremental or online backup strategies.
- Backup compression.