# T197 — Vault Open UX + Key Bootstrap

- **Track ID:** T197-VaultOpenUxKeyBootstrap
- **Phase:** Post-T196 CLI operator UX (from 2026-08-02 non-destructive audit)
- **Status:** 📋 **Expanded + AI fold-in** (plan-only — implement on go-ahead)
- **Depends on:** T187 SQLCipher live; T192 doctor; INSTALL/OPERATIONS key docs; workspace rusqlite **0.39.0** + `bundled-sqlcipher-vendored-openssl` (cipher **4.10.0 community** observed)
- **Blocks / feeds:** Every vault-backed CLI command; unblocks honest operator use before T198–T204
- **Category:** FEATURE / DOCS / SECURITY (operator honesty, not crypto redesign)
- **Source:** CLI audit P0 + scores &lt;7 for live `doctor` (E≈3/C≈2); series index `README-T197-T204-CLI-UX.md`
- **Deferred absorbed:** CLI vault-open SQLCipher spam + key bootstrap residual (`deferred.md`)
- **Not absorbed:** MSI / notarization / App Store; R-CI-BRANCH; DPAPI auto-unlock product; password managers; T198 empty-states; T199 daemon-status-without-vault; T201 full exit-code matrix (partial codes absorbed as free); graph feature honesty (T200); doctor `--deep` integrity (O1 future)
- **Research date:** 2026-08-02 (draft expand + live code + SQLCipher/rusqlite log APIs)
- **AI fold-in:** AI1 affirm (§1–5) + AI2 **M1–M7**, **L1–L7**, soft **O3–O4**; O1/O2 future. Disposition §14.
- **Ledger:** plan-only (no TX until implement)

## 1. Objective

1. **Stop SQLCipher native log floods** on wrong/missing key (`hmac check failed` × N) so operator stderr is CLI-owned (&lt;20 lines total on AC1 paths).  
2. **One operator-facing message family** (human + JSON codes) for: missing key / blank key / invalid format / zero key refused / wrong key (vault locked) / legacy plaintext.  
3. **Stop silent all-zero default** when `--key` / `AI_BRAINS_KEY` omitted — missing is **MissingKey**, not zero-key refuse by accident.  
4. **Validate format before open** via `try_from_raw` so bare 64-hex never reaches page decrypt.  
5. **Document key bootstrap** (process env, global `~/.ai-brains/.env` when CLI loads it, CLI `--key`; PowerShell quoting; never commit keys).  
6. Do **not** weaken zero-key refuse; do **not** log secrets; do **not** redesign crypto.

## 2. Live baseline (re-scan 2026-08-02; AI2 9/9 confirmed)

| Asset | Today |
|-------|--------|
| Key type | `SqlCipherKey` product form `x'<64 hex>'` (67 chars); `validate` / `try_from_raw` / `is_zero` / `is_blank` |
| Store open | `enforce_key_policy` → key pragmas → `verify_key` (`sqlite_master`) |
| Wrong key | `VaultLocked` after open; native **hmac check failed** spam on stderr |
| Missing key CLI | **Silent zero default** at multiple resolvers → confusing zero-refuse |
| Doctor | `open_read_intent` only; resolve defaults zero; audit ~8KB spam |
| Recall | Cleaner JSON zero-refuse — inconsistent family |
| INSTALL / OPERATIONS | Thin key shape; not full bootstrap |
| rusqlite | **0.39.0** (latest crates.io 0.40.1 — **no bump** unless blocked); features **no** `trace` today |
| SQLCipher | **4.10.0 community** — `PRAGMA cipher_log*` are **Commercial/Enterprise** (unreliable on community) |
| dotenv | CLI loads project `.env` + fallback `~/.ai-brains/.env` via dotenvy (`main.rs`) |

### 2.1 Resolve sites (F10 complete inventory — AI2 M4)

| # | Site | Today |
|---|------|--------|
| 1 | `context.rs` `AppContext::from_cli` | Silent zero |
| 2 | `doctor.rs` `resolve_sqlcipher_key` | Silent zero |
| 3 | `recovery.rs` `resolve_sqlcipher_key` | Silent zero |
| 4 | `vault.rs` rotate `resolve_sqlcipher_key` | Silent zero + zero-refuse |
| 5 | `vault.rs` encrypt path | `try_from_raw` (better) |
| 6 | `migrate.rs` `resolve_sql_key` | Silent zero / DEFAULT_SQL_KEY |
| 7 | `shadow.rs` `default_sql_key` | Silent zero |

Backup and most commands go through **AppContext** (site 1) once fixed.

### 2.2 Root causes

| Problem | Cause |
|---------|--------|
| Spam | Native SQLite errlog / SQLCipher noise on wrong-key page HMAC |
| “Missing” = zero refuse | Silent default `x'000…0'` |
| Inconsistency | 7 duplicated resolvers |
| Format late | Many use `from_raw` + store validate only at open |

## 3. Research summary (2026-08-02 + fold-in)

| Source | Finding | T197 application |
|--------|---------|------------------|
| Zetetic API | `cipher_log` / `cipher_log_level` under **Commercial/Enterprise** | **Not primary** on community 4.10.0 (M1) |
| rusqlite `trace::config_log` | Process-wide SQLite errlog; needs feature `trace`; `unsafe` + threading rules | **Primary** spam control (M1/M2/M5) |
| dotenvy in CLI | Project `.env` + `~/.ai-brains/.env` fallback | F4/F12 document real paths (L6) |
| Product SOOT | `enforce_key_policy` already format+zero | CLI resolve must stop silent zero; use `try_from_raw` |

## 4. Frozen decisions (F1–F32)

| ID | Decision |
|----|----------|
| **F1 — Spam control SOOT (rewritten M1)** | **Primary:** enable workspace rusqlite feature **`trace`** (0.39.x; not a new crate). Install process-wide `rusqlite::trace::config_log` that **drops** known noise substrings (`hmac check failed`, and similar “file is encrypted or is not a database” flood lines when they are pure noise during wrong-key open) and routes **other** messages to `tracing::debug!` only (never stderr with key material). **Secondary (optional, Commercial-forward):** attempt `PRAGMA cipher_log_level = NONE` in a fire-and-forget `let _ = …` after open if ever linked Commercial — **must not** be required for AC1 on community. Do **not** disable page HMAC. |
| **F2 — No silent zero default** | When CLI `--key` and `AI_BRAINS_KEY` are both **absent/empty** after trim → **`KeyResolveError::Missing`**. Never substitute all-zero. |
| **F3 — Shared resolver + variants (M7/L7)** | Module in **`ai-brains-cli`** (not store — O4 agree): `resolve_operator_sqlcipher_key(cli_key: Option<String>) -> Result<SqlCipherKey, KeyResolveError>`. Uses **`SqlCipherKey::try_from_raw`** (validates immediately — F5 by construction), not `from_raw` alone. Enum variants: **`Missing`**, **`Format`**, **`Zero`** (explicit zero without ALLOW). Store `enforce_key_policy` remains second line of defense. Replace **all 7** sites in §2.1. |
| **F4 — Resolve order + env load (L6)** | (1) CLI `--key` if non-empty after trim; (2) else process env `AI_BRAINS_KEY` if non-empty (includes values loaded by existing dotenvy: project `.env` then `~/.ai-brains/.env` fallback — document; do not invent a second secret store). (3) else **Missing**. |
| **F5 — Format gate** | Invalid shape fails at resolve via `try_from_raw` **before** `Connection::open`. Help text: `x'<64 hex chars>'` (67 chars). |
| **F6 — Zero key** | Explicit all-zero refused unless `AI_BRAINS_ALLOW_ZERO_KEY` truthy (`1`/`true`/`yes`). Message = **Zero**, not Missing. |
| **F7 — Wrong key** | Open+verify fail → single **`Vault locked:`** line; filter ensures no multi-page hmac dump. |
| **F8 — Message family + JSON codes (M7)** | Human prefixes (pin in tests): |
| | • `Vault key missing:` … set `--key` or `AI_BRAINS_KEY` (see INSTALL) |
| | • `Vault key invalid format:` … must be `x'<64 hex chars>'` |
| | • `Vault key refused:` zero key without allow |
| | • `Vault locked:` wrong key or cannot decrypt (no key material) |
| | • Legacy plaintext path unchanged (T187) |
| | **JSON codes (required, free with enum):** `KeyResolveError::Missing` → `VAULT_KEY_MISSING`; `Format` → `VAULT_KEY_FORMAT`; `Zero` → `VAULT_KEY_ZERO`; `StoreError::VaultLocked` → `VAULT_LOCKED`. Map at CLI error edge. Full multi-exit IA remains T201. |
| **F9 — Doctor missing vs wrong (M6)** | (a) **Missing key:** still emit structured report; `vault_exists` runs; `vault_open` = **skipped** (key missing) + hint to set key; exit 1. (b) **Wrong key:** `vault_open` = **fail** (vault locked) + hint; exit 1. (c) Both **no spam**. Operator can tell “set a key” vs “fix the key.” |
| **F10 — All 7 resolve sites** | Must convert: AppContext, doctor, recovery, vault rotate, vault encrypt (align), **migrate**, **shadow**. Backup via AppContext. Any additional site found at implement → same SOOT. |
| **F11 — Daemon log silence (L3)** | Install log policy in **`ai-brainsd` `main`** and **`windows_service` run path** before any `VaultConnection::open`. OnceLock makes double-install safe (M5). |
| **F12 — Docs** | INSTALL bootstrap: generate placeholder, PowerShell `$env:AI_BRAINS_KEY = "x'…'"`, bash `export`, `--key`, project `.env` + `~/.ai-brains/.env` (actual CLI load). OPERATIONS env table expand. Doctor help link. |
| **F13 — Secrets** | Never log key material; docs use placeholders only. |
| **F14 — Capture independence** | No models/graph. |
| **F15 — Deps** | Enable rusqlite **`trace`** only; **no** new crates; **no** rusqlite 0.40 bump unless blocked (L5). |
| **F16 — Exit codes** | Key resolve / vault locked → exit **1**. T201 owns richer matrix. |
| **F17 — TTY prompt** | Soft; default off; not DoD. |
| **F18 — Bare hex** | Do **not** auto-wrap bare 64-hex in v1. |
| **F19 — init path (L4 freeze)** | **Generate + print once** when `init` has no key: generate non-zero random product key, create vault, print PowerShell/bash set examples to **stdout** (not stderr), warn to store offline. If key **provided**, use it (validate). Do **not** silent-zero new vaults. Align/update init tests. |
| **F20 — Hermetic tests + inventory (M3)** | Process tests: missing / format / wrong-key (&lt;20 stderr, no hmac string) / zero refuse / ALLOW zero. **Phase B1:** inventory tests that pass `None` to resolvers / `from_cli` — classify (a) already explicit key or ALLOW, (b) will break → fix with explicit zero+ALLOW or real key. |
| **F21 — Claims** | No F8 encryption claim change. |
| **F22 — Filter policy (L1)** | Allow-by-default; drop **known noise** only (hmac flood / known wrong-key noise). Wrong-key and page corruption both surface as single **Vault locked:** (corruption residual honesty: no separate corruption claim in T197). Never disable HMAC. |
| **F23 — Not T199** | daemon status w/o vault → T199. |
| **F24 — Not T198** | Empty states → T198. |
| **F25 — Not packaging** | MSI / App Store / R-CI-BRANCH out. |
| **F26 — Callback allocation (L2)** | Callback: `fn(c_int, &str)` only; `message.contains(...)` (no `String::from` / `Vec` accumulation); non-noise via `tracing::debug!` only; **no** SQLite calls; no unbounded buffers. |
| **F27 — config_log safety (M2)** | (a) Install in process entry **before** any `Connection::open` and **before** tokio runtime spawn. (b) Callback is plain `fn` pointer (no captures). (c) No SQLite re-entry in callback. (d) `SAFETY` comment on `unsafe { config_log(...) }`. (e) **`OnceLock` / once** install (M5) shared by CLI, daemon, tests that assert no-spam. |
| **F28 — Series position** | First of T197–T204. |
| **F29 — install() home (M5)** | `sqlcipher_log_policy::install()` lives in **`ai-brains-store`** (or thin module re-exported) so CLI, daemon, and store tests can call it without duplicating unsafe. Idempotent OnceLock. |
| **F30 — Store purity** | Resolver stays in **cli** (arg+env); store keeps `enforce_key_policy` only (O4). |
| **F31 — Future doctor deep** | Out: `cipher_integrity_check` / `--deep` (O1); `cipher_status` needs ≥4.12 (O2). |
| **F32 — High findings** | Silent zero remains; hmac spam remains; secrets logged; HMAC disabled; migrate/shadow left on zero default; doctor missing conflated with wrong key. |

## 5. Residual disposition

| Residual | Disposition |
|----------|-------------|
| SQLCipher spam + key bootstrap | **Absorb** |
| Silent zero default | **Absorb** F2/F3/F10 |
| Doctor vs recall inconsistency | **Absorb** F8/F9 |
| Full exit matrix | **T201** (codes partially absorbed F8) |
| daemon status w/o key | **T199** |
| Empty states | **T198** |
| Bare-hex wrap | Soft F18 |
| doctor `--deep` integrity | Future F31 |
| MSI / R-CI-BRANCH | Out |

## 6. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | Wrong/missing key on `doctor` → **&lt;20 lines** stderr; **zero** repeated `hmac check failed` lines | Process + install() in test setup |
| **AC2** | AppContext paths (`recall` / `preflight` / `project list`) share F8 prefixes | Tests |
| **AC3** | Invalid format fails at resolve **before** open (`try_from_raw`) | Unit |
| **AC4** | Missing key → **Missing** / `VAULT_KEY_MISSING`, not silent zero | Unit + process |
| **AC5** | INSTALL + OPERATIONS bootstrap + dotenv paths; doctor help link | Docs |
| **AC6** | Hermetic: missing / format / wrong-key / zero / ALLOW | nextest |
| **AC7** | No secrets logged; gate green; no crypto redesign | Review + gate |
| **AC8** | Explicit zero refused without ALLOW; wrong key fail-closed | Tests |
| **AC9** | Full gate if code changed | Process |
| **AC10** | All **7** resolve sites use SOOT (F10) | Grep / review |
| **AC11** | Doctor: missing → vault_open **skipped**; wrong → vault_open **fail** (F9) | Unit/process |
| **AC12** | JSON codes mapped from `KeyResolveError` / VaultLocked (F8) | Unit |
| **AC13** | Log policy installed CLI + both daemon entries + no-spam tests (F11/F27/F29) | Code review |

## 7. Non-goals

- Changing SQLCipher algorithms, page size, or HMAC off  
- Multi-user key vaults / IdP  
- Auto-prompt TTY as default  
- Graph / governed discovery (T200/T203)  
- Full exit-code IA (T201)  
- `daemon status` without vault (T199)  
- MSI / notarization / App Store  
- DPAPI product unlock / password managers  
- Auto-wrapping bare 64-hex  
- Relying on Commercial `cipher_log_level` for AC1  
- doctor `--deep` integrity check  
- rusqlite 0.40 bump unless blocked  

## 8. Handoffs

| To | What |
|----|------|
| deferred CLI spam/bootstrap | Strike on ship |
| T198–T204 | Unblocked operator vault use |
| T201 | May extend exit codes beyond exit 1 |
| T199 | Daemon status independence |
| Future doctor deep | F31 |

## 9. Implementation sketch

### 9.1 Shared resolve (CLI)

```text
enum KeyResolveError { Missing, Format(...), Zero }

resolve_operator_sqlcipher_key(cli_key: Option<String>) -> Result<SqlCipherKey, KeyResolveError>
  pick = non_empty(cli_key) or non_empty(env AI_BRAINS_KEY) or return Missing
  key = try_from_raw(pick)?  // Format on err
  if key.is_zero() && !allow → Zero
  Ok(key)
```

### 9.2 Log silence

```text
// ai-brains-store::sqlcipher_log_policy
static INSTALL: OnceLock<()>
pub fn install() {
  INSTALL.get_or_init(|| {
    // SAFETY: before any Connection::open / before multi-threaded SQLite use;
    // callback is fn pointer, no SQLite re-entry, no unbounded alloc.
    unsafe { rusqlite::trace::config_log(Some(filter_cb)) }
  });
}
fn filter_cb(code: c_int, msg: &str) {
  if msg.contains("hmac check failed") { return; }
  // optional other known noise substrings
  tracing::debug!(code, msg, "sqlite errlog");
}
```

CLI `main`, daemon `main`, `windows_service` startup, and AC1 tests call `install()` first.

### 9.3 Doctor

```text
match resolve(...) {
  Err(Missing) => vault_open skipped + hint; exit 1 after report
  Ok(key) => open_read_intent; Err(VaultLocked) => vault_open fail
}
```

## 10. Verification plan

1. Unit: resolve matrix + JSON code map.  
2. stderr capture wrong-key with install().  
3. Process doctor missing vs wrong.  
4. Grep: no remaining silent zero defaults in 7 sites.  
5. Docs bootstrap.  
6. Full gate.  

## 11. Risks

| Risk | Mitigation |
|------|------------|
| cipher_log unavailable on community | F1 primary = config_log (M1) |
| Unsafe config_log misuse | F27 + OnceLock |
| Test breakage from F2 | F20 inventory (M3) |
| migrate/shadow leftover zero | F10 AC10 |
| Filter swallows real issues | F22 + single Vault locked line |
| init UX regression | F19 generate+print |

## 12. Implement notes (for go-ahead)

1. **Order:** B1 inventory (sites + tests) → shared resolve F2/F3 → install log policy → wire 7 sites → doctor F9 → init F19 → docs → AC tests.  
2. **High findings:** F32 list.  
3. **Stop-before:** crypto redesign; T199/T201 scope; MSI.  
4. **Category:** `FEATURE`.  

## 13. Research pins (post fold-in)

| Fact | Pin |
|------|-----|
| Spam primary | `config_log` + rusqlite `trace` |
| Spam secondary | Commercial cipher_log_level only (optional) |
| Missing | Not silent zero |
| Resolver home | `ai-brains-cli` + `try_from_raw` |
| Resolve sites | **7** (incl. migrate, shadow) |
| install() | store OnceLock; CLI + both daemon paths + tests |
| Doctor | skipped vs fail |
| JSON codes | VAULT_KEY_* + VAULT_LOCKED |
| init | generate + print once |
| dotenv | project + `~/.ai-brains/.env` (existing) |
| rusqlite | 0.39.x + `trace`; no 0.40 unless blocked |

## 14. AI fold-in disposition (2026-08-02)

### AI1

| Item | Disposition |
|------|-------------|
| Enable `trace` + config_log filter | **Agree** — F1 primary (see M1 for Commercial caveat) |
| Eliminate silent zero + shared resolve | **Agree** — F2/F3 |
| Early format via try_from_raw | **Agree** — F3/F5/L7 |
| Message family prefixes | **Agree** — F8 |
| INSTALL/OPERATIONS bootstrap + PS quoting | **Agree** — F12 |

### AI2 required (M1–M7)

| ID | Disposition | Fold-in |
|----|-------------|---------|
| **M1** cipher_log Commercial | **Agree** | F1 rewrite: config_log primary |
| **M2** unsafe/threading | **Agree** | F27 expanded |
| **M3** test inventory for F2 | **Agree** | F20 + plan B1 |
| **M4** 7 resolve sites | **Agree** | §2.1 + F10 |
| **M5** OnceLock install home | **Agree** | F29 + F11 |
| **M6** doctor missing vs wrong | **Agree** | F9 + AC11 |
| **M7** KeyResolveError → JSON codes | **Agree** | F8 required codes |

### AI2 low (L1–L7)

| ID | Disposition | Fold-in |
|----|-------------|---------|
| **L1** filter vs corruption | **Agree** | F22 single Vault locked |
| **L2** allocation-free callback | **Agree** | F26 |
| **L3** daemon both entries | **Agree** | F11 |
| **L4** init generate+print | **Agree** | F19 freeze (b) |
| **L5** no 0.40 unless blocked | **Agree** | F15 |
| **L6** dotenv paths real | **Agree** | F4/F12 |
| **L7** try_from_raw not from_raw | **Agree** | F3/F5 |

### Opportunities

| ID | Disposition |
|----|-------------|
| **O1** doctor --deep integrity | **Future** F31 |
| **O2** cipher_status 4.12+ | **Future** F31 |
| **O3** Commercial cipher_log secondary | **Soft accept** F1 secondary |
| **O4** resolver in cli not store | **Agree** F30 |

### Declined / not absorbed

| Item | Why |
|------|-----|
| Making Commercial cipher_log DoD | Unavailable on community |
| Auto bare-hex wrap | F18 |
| Full T201 exit matrix | Series boundary |
| rusqlite 0.40 bump as DoD | Unrelated-Failures / F15 |
)
