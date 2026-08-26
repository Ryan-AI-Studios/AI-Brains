# T305 — rusqlite 0.39.0 → 0.40.2 (SQLCipher 4.14)

- **Track ID:** T305-Rusqlite040
- **Status:** **Planned** (Pending until **go**)
- **Category:** DEPS / SECURITY / STORE
- **Owner:** Grok
- **Source:** Dependabot `#61` rusqlite 0.39.0→**0.40.2**. Standing **decline** through T285–T300 (T213 L4 `table_exists`). **Owner reopened** 2026-08-25 (“make tracks to address the dependabot prs”).
- **Depends on:** T187 SQLCipher `bundled-sqlcipher-vendored-openssl` + `backup` + `fallible_uint` + `trace`. Observed `PRAGMA cipher_version` **`4.10.0 community`** (`Docs/COMPATIBILITY.md` F8 `:80`). Workspace pin is **exact** `0.39.0` (`Cargo.toml:57`) — **must** edit toml. Remote branch name is **`dependabot/cargo/rusqlite-0.40.1`** (title 0.40.2).
- **F0:** Plan-only until go. **Stop-Before** if encrypt/open KATs fail or `cipher_version` is empty. Do **not** merge `dependabot/cargo/rusqlite-0.40.1`.
- **Ledger:** series DOCS TX `30b7ca9d-4932-4f00-97b8-82d5d25e633b`. Fold-in DOCS TX `db5c6f11-6bfa-45c6-b0ed-1ccfcc0ecedf`. Implement starts **DEPS** (or SECURITY) TX on go.
- **AI fold-in:** 2026-08-26 `agy-review.md` + `opencode-review.md` (HEAD `561113c`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** **Agree:** Agy m3 `--precise 0.40.2`; OpenCode m1 last-PR `#221`; OpenCode extras libsqlite3-sys/hashlink; OpenCode local `fn table_exists` ≠ rusqlite API. **Already:** Agy m1 = F1/AC1; Agy m2 = F2/AC2 (observe, don’t guess); Agy O1/O2 = AC2–AC4/AC8. **Partial:** Agy “transparent 4.10→4.14 open” — [Zetetic major-version format compat](https://github.com/sqlcipher/sqlcipher) is the **reason we expect success**; **F9 KATs are the proof**. Disposition **§13**.

## 1. Objective

Upgrade rusqlite to **0.40.2** with the **same feature set**. Re-probe `PRAGMA cipher_version` (expect SQLCipher **4.14.x** per rusqlite 0.40.0 `#1837`). Update COMPATIBILITY F8 **observed** string. Keep capture independence (store still SQLCipher; no models). Do **not** adopt `Connection::table_exists` unless a one-line compile break requires it (T213 L4 was optional).

## 2. Live baseline (2026-08-26 fold-in)

| Pin | Workspace | Lock | crates.io | Action |
|-----|-----------|------|-----------|--------|
| rusqlite | **exact 0.39.0** + sqlcipher features (`Cargo.toml:57`) | **0.39.0** / libsqlite3-sys **0.37.0** / hashlink **0.11.0** | **0.40.2** (2026-08-08) | Exact **`0.40.2`** + `--precise` |
| tokio / tower-http / clap | 1.53 / 0.7 (+ dual 0.6.11 reqwest) / 4.5 | 1.53.1 / 0.7.0+0.6.11 / 4.6.1 | — | **Do not revert** (F6) |

**`cargo pkgid rusqlite`** → unique `rusqlite@0.39.0`. `--precise` is hygiene.

**Code (verified):**

- `connection.rs:20` `VaultConnection::open` / `:49` `open_read_intent` — `rusqlite::{Connection, OpenFlags}`.
- `pragmas.rs:22/:42` `PRAGMA cipher_compatibility = 4`; `:49` `cipher_version`. T187-V-01 (`connection.rs:397`) asserts **non-empty** and **`4.`** — not a frozen `4.10.0` string.
- `encrypt.rs:106` and `rotate.rs` `SELECT sqlcipher_export(...)` — SQL batch, not a rusqlite constructor. SQLCipher **4.13.0** corrected `sqlcipher_export` registration encoding (between 4.10 and 4.14) — encrypt/rotate KATs are load-bearing.
- `ai-brains-brain/src/backup.rs:178` `rusqlite::backup::Backup::new` + `run_to_completion` (T277).
- `graph_density.rs:281` / `backup.rs:615` hand-written `sqlite_master` queries. Test-local `fn table_exists` wrappers are **not** `rusqlite::Connection::table_exists`. F5 stands.
- `rg vtab` in `crates/**/*.rs`: **zero** matches. AC6 unused-branch holds.
- No product `SAVEPOINT` / `savepoint(` API — 0.40.1 `#1854` is not on our path.

**Research (verified fold-in; re-read rusqlite 0.40.0–0.40.2 + COMPATIBILITY F8 at execute):**

- [rusqlite 0.40.0](https://github.com/rusqlite/rusqlite/releases/tag/v0.40.0): VTab macros→constructors `#1823/#1824/#1826/#1832/#1835`; bundled SQLCipher **4.14.0** `#1837`; bundled SQLite 3.53.1. **0.40.1:** SAVEPOINT injection `#1854`; SQLite 3.53.2; hashlink bump. **0.40.2:** MSRV **1.88** (repo **1.95.0**). Features `bundled-sqlcipher-vendored-openssl` / `backup` / `fallible_uint` / `trace` still exist.
- [SQLCipher compatibility](https://github.com/sqlcipher/sqlcipher): **format compatible within the same major version** (4.10 vaults are 4.x). `cipher_compatibility = 4` is the 4.x default. **Not** a substitute for F9. [4.14.0](https://www.zetetic.net/blog/2026/03/17/sqlcipher-4.14.0-release/) updates SQLite baseline to **3.51.3** (WAL-reset corruption fix — **WAL users strongly advised to upgrade**; we set `journal_mode = WAL`). Zetetic: test thoroughly before production. SQLCipher’s own pragma test expects `4.14.0 community` — **record observed**, do not pre-write that string as if measured.
- `#61` is `Cargo.toml` 1/1 + `Cargo.lock` 29/26. Expected extras: libsqlite3-sys **0.37.0→0.38.2**, hashlink **0.11.0→0.12.1**, plus windows-sys/socket2/windows-core re-resolutions from an older base. Live HEAD after T303/T304 **will not** match `#61` byte-for-byte. Accept resolver; do not hand-edit; do not revert tokio/tower-http.

last-PR Cursor: **`#221`** (T304, HEAD `561113c`) — comments/reviews **empty**. **No T306.** `#58` closed/superseded — do not re-absorb.

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0** | Plan-only until go. |
| **F1** | Target **0.40.2** exact. Keep features `bundled-sqlcipher-vendored-openssl`, `backup`, `fallible_uint`, `trace`. |
| **F2** | Re-run T187 cipher_version smoke. Update `Docs/COMPATIBILITY.md` F8 **observed** string + date. Do not claim FIPS. Do not pre-fill `4.14.0 community` until the probe returns it. |
| **F3** | Existing vault must still open with current `AI_BRAINS_KEY` (hermetic + **optional** live `doctor` vault_open — no print key). |
| **F4** | VTab API unused today — no product rewrite. If a dep uses vtab, fix that crate only. |
| **F5** | `rusqlite::Connection::table_exists` adoption is **optional** (T213 L4). Local test helpers named `table_exists` are unrelated. Do not churn `has_graph_tables` / `has_core_tables` unless compile-forced. |
| **F6** | No clap / GHA this track. Do **not** revert tokio 1.53.1 or T304 dual tower-http. |
| **F7** | `cargo deny` + `audit` green. License: SQLCipher BSD-style already allowed. |
| **F8** | Do not merge Dependabot remote `dependabot/cargo/rusqlite-0.40.1`. Never `git push origin main`. |
| **F9** | Stop-Before: if `vault encrypt` / open / **rotate sqlcipher_export** KATs fail, or `cipher_version` empty, **halt** — do not ship a lock that cannot open T187 fixtures. |
| **F10** | CHANGELOG + COMPATIBILITY F8. Manual: hermetic encrypt/open; live `doctor --summary` `cipher_page` / `vault_open` **ok** (no key in logs). |
| **F11** | Cross-model: SECURITY/DEPS — `codex-review` after Phase-1. |
| **F12** | `cargo update -p rusqlite --precise 0.40.2` ([cargo-update `--precise`](https://doc.rust-lang.org/cargo/commands/cargo-update.html)). |
| **F13** | Expected lock extras: libsqlite3-sys **0.38.2**, hashlink **0.12.x**, plus F9-style windows-* / socket2. Live graph may differ from `#61`. Do not hand-edit. |

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | workspace + lock rusqlite **0.40.2**. Features unchanged. |
| **AC2** | T187-V-01 still green (`cipher_version` non-empty + `4.`). COMPATIBILITY F8 records **observed** string (expect `4.14.x community` shape). |
| **AC3** | Store encrypt/open/wrong-key tests stay green. Include sqlcipher_export paths (`encrypt.rs` + rotate). |
| **AC4** | Backup create/classify hermetic stay-green (T277 `Backup::new`). |
| **AC5** | Workspace clippy `-D warnings`, nextest, deny, audit. |
| **AC6** | `rg vtab` in `crates/**/*.rs` still unused **or** updated to 0.40 constructors. |
| **AC7** | CHANGELOG + COMPATIBILITY. |
| **AC8** | Manual live `doctor --summary`: `vault_open` / `cipher_page` not fail (degraded from graph_density/recovery_kit is OK). No key in output. |

## 5–12

**Non-goals:** clap 5; Cozo; rekey live vault; `vault encrypt` the live vault; adopting rusqlite `table_exists` as a drive-by; CSRF/tower-http steal.

**Risk:** SQLCipher 4.10→4.14 on-disk. Mitigation: Zetetic **same-major** format compat + `cipher_compatibility=4` + hermetic KATs + live doctor; **Stop-Before F9**. WAL-reset fix in 4.14 is a reason to upgrade, not a skip of KATs.

**§9:** **Reopen** T285–T300 rusqlite 0.40 decline **because owner asked**. Absorb `#61`. Decline clap 5. last-PR `#221` N/A empty — **no T306**. T213 L4 optional F5.

**Touch:** `Cargo.toml` / `Cargo.lock`; maybe store compile fixes; `Docs/COMPATIBILITY.md`; CHANGELOG; conductor. Comment on T187-V-01 observed version if it still says 4.10.0.

**Isolation:** Do not print `AI_BRAINS_KEY`. Do not `vault encrypt` the live vault. Do not `cargo install` unless owner asks after this bump.

---

## 13. AI fold-in

Inputs (not edited): `agy-review.md` + `opencode-review.md` (HEAD `561113c`). Fold-in verify: exact pin `Cargo.toml:57`; lock rusqlite 0.39.0 / libsqlite3-sys 0.37.0; `#61` head `dependabot/cargo/rusqlite-0.40.1`; COMPATIBILITY `:80` `4.10.0 community`; pragmas `cipher_compatibility=4`; T187-V-01 `4.` shape; `rg vtab` empty; local `fn table_exists` only; `Backup::new` `:178`; `#221` comments empty; rusqlite 0.40.0–0.40.2 + SQLCipher 4.14 notes; Zetetic same-major format compat.

### Pins locked by fold-in

1. **F12 (Agy m3):** `--precise 0.40.2`.
2. **F13 (OpenCode extras):** libsqlite3-sys 0.38.2 + hashlink 0.12.x + windows-* extras; live ≠ `#61`.
3. **F2 (Agy m2 restated):** COMPATIBILITY F8 records **observed** cipher_version; do not pre-write `4.14.0 community`.
4. **F5 (OpenCode):** test-local `table_exists` is not rusqlite’s API.
5. **F9 (Agy “seamless open” partial):** Zetetic same-major format is expected; KATs prove it. sqlcipher_export KATs cover 4.13 encoding fix.
6. **§2 / §9 (OpenCode m1):** last-PR Cursor is `#221`; empty; no T306. F0 remote name **`rusqlite-0.40.1`** is correct.

### Per-AI disposition

| Source | Item | Disposition |
|--------|------|-------------|
| Agy | B / M | None filed |
| Agy | **m1** workspace `0.40.2` exact | **Already** F1 / AC1 |
| Agy | **m2** COMPATIBILITY F8 `4.14.0 community` | **Partial** — **already** F2/AC2; decline pre-filling the string; SQLCipher test shape is the *expectation* |
| Agy | **m3** `--precise 0.40.2` | **Folded** F12 |
| Agy | **O1** nextest store/d/cli | **Already** AC3/AC4; plan names `-p ai-brains-store` + encrypt/rotate |
| Agy | **O2** live doctor no key | **Already** AC8 / F10 |
| Agy | “4.14 opens 4.10 transparently” | **Partial** — Zetetic same-major format; **F9 is proof**. Re-trigger: observed cipher_version empty or open KAT red |
| OpenCode | B / M | None filed |
| OpenCode | **m1** last-PR `#216` → `#221` | **Folded** §2 / §9 |
| OpenCode | **O1** changelog re-read is the gate | **Already** Phase 0 |
| OpenCode | extras libsqlite3-sys/hashlink; local table_exists; no SAVEPOINT | **Folded** F13 / F5 / §2 |
| both | last-PR Cursor empty | **Affirm** — `#221` N/A; **no T306** |

No Blockers/Majors to decline. No new placeholder. Do **not** edit `*-review.md`. Do **not** execute until go.
