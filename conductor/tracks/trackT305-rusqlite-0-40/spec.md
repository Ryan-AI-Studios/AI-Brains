# T305 — rusqlite 0.39.0 → 0.40.2 (SQLCipher 4.14)

- **Track ID:** T305-Rusqlite040
- **Status:** **Planned** (Pending until **go**)
- **Category:** DEPS / SECURITY / STORE
- **Owner:** Grok
- **Source:** Dependabot `#61` rusqlite 0.39.0→**0.40.2**. Standing **decline** through T285–T300 (T213 L4 `table_exists`). **Owner reopened** 2026-08-25 (“make tracks to address the dependabot prs”).
- **Depends on:** T187 SQLCipher `bundled-sqlcipher-vendored-openssl` + `backup` + `fallible_uint` + `trace`. Observed `PRAGMA cipher_version` **`4.10.0 community`** (`Docs/COMPATIBILITY.md` F8). Workspace pin is **exact** `0.39.0` (not a caret).
- **F0:** Plan-only until go. **Stop-Before** if encrypt/open KATs fail or `cipher_version` is empty. Do **not** merge `dependabot/cargo/rusqlite-0.40.1`.
- **Ledger:** series DOCS TX `30b7ca9d-4932-4f00-97b8-82d5d25e633b`.

## 1. Objective

Upgrade rusqlite to **0.40.2** with the **same feature set**. Re-probe `PRAGMA cipher_version` (expect SQLCipher **4.14.x** per rusqlite 0.40.0 `#1837`). Update COMPATIBILITY F8 observed version. Keep capture independence (store still SQLCipher; no models). Do **not** adopt `Connection::table_exists` unless a one-line compile break requires it (T213 L4 was optional).

## 2. Live baseline (2026-08-25)

| Pin | Workspace | Lock | crates.io | Action |
|-----|-----------|------|-----------|--------|
| rusqlite | **0.39.0** exact + sqlcipher features | **0.39.0** | **0.40.2** (2026-08-08) | Exact `0.40.2` or `0.40` |

**Code:** `VaultConnection::open` / `open_read_intent` (`connection.rs`). `encrypt.rs` `sqlcipher_export`. Backup Online Backup API. CLI `graph_density` SQL COUNT. **No** `rusqlite::vtab` in store src (VTab breaking changes likely N/A — grep at execute).

**Research (snapshot):** 0.40.0 breaking = **VTab** macros/best_index/connect. Bundled SQLCipher **4.14.0**. 0.40.1 SAVEPOINT name injection fix `#1854`. 0.40.2 lowers MSRV to 1.88 (we are 1.95). lib.rs: bundled SQLite 3.53.2 as of 0.40.2. **Re-read rusqlite Changelog + COMPATIBILITY F8 at execute.**

last-PR `#216` empty.

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0** | Plan-only until go. |
| **F1** | Target **0.40.2**. Keep features `bundled-sqlcipher-vendored-openssl`, `backup`, `fallible_uint`, `trace`. |
| **F2** | Re-run T187 cipher_version smoke. Update `Docs/COMPATIBILITY.md` observed string. Do not claim FIPS. |
| **F3** | Existing vault must still open with current `AI_BRAINS_KEY` (hermetic + **optional** live `doctor` vault_open — no print key). |
| **F4** | VTab API: if unused, no product rewrite. If a dep uses vtab, fix that crate only. |
| **F5** | `table_exists` adoption is **optional** (T213 L4). Do not churn `has_graph_tables` unless compile-forced. |
| **F6** | No tokio/tower-http/clap/GHA this track. |
| **F7** | `cargo deny` + `audit` green. License: SQLCipher BSD-style already allowed. |
| **F8** | Do not merge Dependabot remote. Never `git push origin main`. |
| **F9** | Stop-Before: if `vault encrypt` / open KATs fail, **halt** and report — do not ship a lock that cannot open T187 fixtures. |
| **F10** | CHANGELOG + COMPATIBILITY F8. Manual: hermetic encrypt/open; live `doctor --summary` `cipher_page` / `vault_open` **ok** (no key in logs). |
| **F11** | Cross-model: SECURITY/DEPS — `codex-review` after Phase-1. |

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | workspace + lock rusqlite **0.40.2**. Features unchanged. |
| **AC2** | T187-class cipher_version test: `PRAGMA cipher_version` **non-empty**; recorded new string in COMPATIBILITY F8. |
| **AC3** | Store encrypt/open/wrong-key tests stay green. |
| **AC4** | Backup create/classify hermetic stay-green (T277). |
| **AC5** | Workspace clippy `-D warnings`, nextest, deny, audit. |
| **AC6** | `rg vtab` in `crates/ai-brains-store/src` still unused **or** updated to 0.40 constructors. |
| **AC7** | CHANGELOG + COMPATIBILITY. |
| **AC8** | Manual live `doctor --summary`: `vault_open` / `cipher_page` not fail (degraded from graph_density/recovery_kit is OK). |

## 5–12

**Non-goals:** clap 5; Cozo; rekey live vault; adopting rusqlite `table_exists` as a drive-by.

**Risk:** SQLCipher 4.10→4.14 on-disk compatibility (usually forward-compatible). Mitigation: hermetic + live doctor; Stop-Before F9.

**§9:** **Reopen** T285–T300 rusqlite 0.40 decline **because owner asked**. Absorb `#61`. Decline clap 5. last-PR `#216` N/A. T213 L4 optional F5.

**Touch:** `Cargo.toml` / `Cargo.lock`; maybe store compile fixes; `Docs/COMPATIBILITY.md`; CHANGELOG; conductor.

**Isolation:** Do not print `AI_BRAINS_KEY`. Do not `vault encrypt` the live vault. Do not `cargo install` unless owner asks after this bump.
