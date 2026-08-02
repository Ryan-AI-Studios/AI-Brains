# T187 Internal Completion Review — R1

| Field | Value |
|-------|--------|
| **Track** | T187 — SQLCipher Page Encryption (live) |
| **Reviewer** | Internal completion reviewer (read-only code/docs audit) |
| **Date** | 2026-08-01 |
| **Scope** | F1–F22, AC1–AC13, plan DoD; focus: wrong-key fail-closed, zero-key policy, plain→encrypt via `sqlcipher_export`, unkeyed opens, scratch hygiene, hermeticity, docs honesty |
| **Method** | Static audit of implementation + docs (no code edits; no gate re-run in this review pass) |
| **Verdict** | **FAIL** |

---

## Executive summary

Core product path for live SQLCipher is largely **implemented correctly**:

- Workspace `rusqlite` is on `bundled-sqlcipher-vendored-openssl` + `backup` + `fallible_uint` (0.39.0).
- `VaultConnection::open` / `open_read_intent` enforce blank/invalid/zero-key policy, plain-header sniff → `LegacyPlaintextVault`, post-key `sqlite_master` verify → `VaultLocked`.
- Plain→encrypt uses **`sqlcipher_export`** (`ai-brains vault encrypt`), not Online Backup.
- Backup create keys source; verify is fail-closed; F-02/K-06 dual-mode `if plain` branches are **removed**.
- Scratch `check_db` / `check_vault` are gone.
- Primary claims docs (Deviations §1, COMPATIBILITY F8 body, RELEASE-CLAIMS R-F8/R-K06, SECURITY-LIMITS) correctly avoid FIPS/Purge overclaim.

**Blocking gaps:**

1. **P0 — AC7/AC9 hermetic zero-key incomplete:** many in-process library/CLI/daemon tests still open `VaultConnection` with the historical all-zero key **without** `AI_BRAINS_ALLOW_ZERO_KEY=1` via `TempEnv` or process env. `.config/nextest.toml` only has a **comment** (no env injection); CI workflow does not set the var. On a clean machine/CI this must fail zero-key open policy.
2. **P1 — AC8 docs honesty residual:** `Docs/RECOVERY-DRILLS.md` still asserts plain `bundled` SQLite and that wrong-key fail-closed is not claimable — directly contradicts T187 claims flip.
3. **P1 — AC10 process residual:** `conductor/deferred.md` §59 #8 still lists “needs page encrypt / plain bundled ignores PRAGMA key” as open.

These prevent a clean PASS until fixed and re-verified.

---

## Findings (P0–P3)

### P0 — Must fix before clearance

#### P0-1 — Zero-key escape hatch not hermetically available to in-process tests (AC7, F19, plan B5, AC9)

**Evidence**

- Production refuse is correct in `VaultConnection::enforce_key_policy` (`crates/ai-brains-store/src/connection.rs`): zero key → `VaultLocked` unless `AI_BRAINS_ALLOW_ZERO_KEY` is truthy.
- CLI hermetic helper **does** set the env for **child** CLI processes:

```62:68:crates/ai-brains-cli/tests/common/mod.rs
/// T187: sets `AI_BRAINS_ALLOW_ZERO_KEY=1` so default zero-key CLI init used by
/// hermetic tests still opens under live SQLCipher. Production refuses zero keys.
pub fn hermetic_bin() -> Command {
    let mut cmd = Command::cargo_bin("ai-brains").expect("ai-brains bin must be built for tests");
    strip_ambient(&mut cmd);
    cmd.env("AI_BRAINS_ALLOW_ZERO_KEY", "1");
```

- Many **in-process** tests still call `VaultConnection::open(..., zero_key)` with **no** `TempEnv` / env set, including:
  - `crates/ai-brains-store/tests/common/governed_fixture.rs` (`fixture_zero_sql_key`)
  - `crates/ai-brainsd/tests/daemon_dispatch_shared.rs`, `spool_replays_after_restart.rs`, `single_writer_serializes_parallel_ingest.rs`, `http_enable_smoke.rs`
  - `crates/ai-brains-brain/tests/nightly_summarizes_session.rs`, `nightly_summarizes_large_session.rs`, `nightly_consecutive_errors.rs`
  - `crates/ai-brains-cli/tests/migrate_governed.rs`, `shadow_vault_refuses_live_target.rs`, `device_replicate_cli.rs`, `cross_repo_bridge_smoke.rs`
- `.config/nextest.toml` comments that nextest “supports profile env via this map form when available” but **does not define any env map**; nextest profile schema does not provide a simple profile-level `env = { … }` for test processes.
- `.github/workflows/ci.yml` global `env` does **not** include `AI_BRAINS_ALLOW_ZERO_KEY`.

**Impact**

- AC7 (“escape hatch + hermetic TempEnv/helper”) is only half-done (CLI children only).
- AC9 full gate green cannot be claimed hermetically: zero-key `VaultConnection::open` must error without the escape hatch.
- Developers who already export `AI_BRAINS_ALLOW_ZERO_KEY=1` in the shell may mask a red gate.

**Required fix (implementer)**

Pick one durable approach (prefer both for defense-in-depth):

1. Inject `AI_BRAINS_ALLOW_ZERO_KEY=1` in CI + document local gate requirement **or** use a nextest setup-script / wrapper that exports it; **and/or**
2. Add a shared test helper (`allow_zero_key_for_test() -> TempEnv`) and apply it at every zero-key `VaultConnection::open` site (or migrate those fixtures to non-zero random keys).

Do **not** weaken production refuse.

**Status:** `open`

---

### P1 — High; must fix or re-scope before clearance

#### P1-1 — `Docs/RECOVERY-DRILLS.md` still claims plain `bundled` residual (AC8, F8)

**Evidence**

- §3 encryption honesty block still states workspace uses `bundled` (plain SQLite), `PRAGMA key` is a no-op, and wrong-key fail-closed must not be claimed.
- §7 wrong-key class still says “Not enforceable on plain `bundled` SQLite”.
- §11 residual # row still: “current `bundled` plain SQLite ignores `PRAGMA key`”.

**Impact**

- Direct contradiction of Deviations §1 closed, COMPATIBILITY F8, RELEASE-CLAIMS R-F8/R-K06, and the strict F-02/K-06 tests now in tree.
- Operator/doc readers can deny page-encryption claims after T187 shipped them.

**Required fix**

Rewrite RECOVERY-DRILLS honesty + residual table for live SQLCipher (wrong-key class enforceable; plain header is legacy-only; point to `vault encrypt`). Keep CE pre-erase / NIST Purge non-claims.

**Status:** `open`

#### P1-2 — deferred §59 #8 not struck (AC10, plan E3)

**Evidence**

- `conductor/deferred.md` promotion table still maps §59 #8 → T187.
- §59 residual list item 8 still claims wrong-key needs page encrypt / plain `bundled` ignores `PRAGMA key`.

**Impact**

- Track closeout incomplete; residual ledger lies relative to implementation.

**Required fix**

Strike/close §59 #8 with T187 evidence note; leave T188-owned residuals alone.

**Status:** `open`

---

### P2 — Medium; fix by default

#### P2-1 — COMPATIBILITY non-claims table still says SQLCipher non-claim “while `bundled`” (AC8)

**Evidence:** `Docs/COMPATIBILITY.md` §12 row: “SQLCipher page-level on all tiers | Non-claim while `bundled`”.

**Impact:** Contradicts §4 F8 live wording.

**Fix:** Update row to live on T1 builds + remaining non-claims (FIPS, not equal tiers without evidence if any).

**Status:** `open`

#### P2-2 — GHA Windows job does not explicitly ensure Perl on PATH (F13, AC9)

**Evidence**

- `scripts/dev-check.ps1` and `Docs/ci-tooling.md` document Perl for MSVC/openssl-src.
- `.github/workflows/ci.yml` `gate-windows` has no Perl install/verify step.
- Spec F13: “GHA windows job: ensure Perl on PATH.”

**Impact:** Fragile Windows build if runner image lacks Perl; local/CI divergence.

**Fix:** Add a step that asserts `perl -v` (or installs Strawberry Perl) before `cargo clippy` / nextest.

**Status:** `open`

#### P2-3 — License re-verify / deny output not archived for this track (F11, plan A3)

**Evidence:** No track review log previously; this R1 does not include pasted `cargo deny check` / `cargo audit` output (review is static-only).

**Impact:** F11 requires re-verify after feature flip; cannot mark AC9/F11 complete without evidence.

**Fix:** Run `cargo deny check ; cargo audit` after SQLCipher flip; paste SPDX-relevant lines into next review log revision.

**Status:** `open` (evidence missing in R1)

#### P2-4 — Stale K-06 module comment still describes dual-mode plain residual (docs/test honesty)

**Evidence:** `crates/ai-brains-store/tests/recovery_drills.rs` header comment for K-06 still describes plain `bundled` dual-mode residual even though the test body is strict T187.

**Impact:** Misleading maintainers; low security risk; medium for review honesty of AC4 narrative.

**Fix:** Update comment to “T187 strict fail-closed”.

**Status:** `open`

#### P2-5 — `AppContext` still documents “degraded mode / dummy zero key for rusqlite-bundled” (F9 honesty)

**Evidence:** `crates/ai-brains-cli/src/context.rs` comments + default zero-key when `--key` omitted. Runtime refuse now happens inside `VaultConnection`, so production default without escape hatch fails closed — correct — but comment is pre-T187.

**Impact:** Operator confusion; may encourage setting `AI_BRAINS_ALLOW_ZERO_KEY` casually.

**Fix:** Comment that omitted key still defaults to zero material and is **refused** unless escape hatch; prefer requiring explicit key in a later track if desired (out of T187 if not frozen).

**Status:** `open`

---

### P3 — Low / deferrable

#### P3-1 — Exact `PRAGMA cipher_version` string not recorded (plan D2b)

COMPATIBILITY / SECURITY-LIMITS say “non-empty”; no pinned observed string (e.g. `4.x.x community`). Soft residual.

**Status:** `deferred` (acceptable if tracked)

#### P3-2 — `nextest.toml` misleading comment about non-existent profile env map

Comment implies config might inject `ALLOW_ZERO_KEY`; it does not. Fix with P0-1.

**Status:** `open` (tied to P0-1)

#### P3-3 — `vault encrypt` dry-run accepts zero-key material without refuse path

Dry-run returns before `encrypt_plaintext_vault` zero-key refuse (after format validate only). Low risk; confirm path refuses.

**Status:** `deferred`

#### P3-4 — Plan.md checkboxes still all open / track status text still “planning only”

Implementation landed but plan/spec status headers not flipped. Process hygiene only.

**Status:** `deferred`

#### P3-5 — Historical wording outside elevated set

`Docs/Implementation-Plan.md` and similar archives still mention pre-T187 F8 residual. Soft re-grep residual (T185 class), not security regression.

**Status:** `deferred`

#### P3-6 — `cipher_integrity_check` on backup verify not productized

Explicit non-goal / soft residual (spec L1). OK.

**Status:** `out_of_scope`

---

## AC matrix

| AC | Criterion | Result | Notes |
|----|-----------|--------|-------|
| **AC1** | Workspace SQLCipher vendored-openssl feature set | **PASS** | Root `Cargo.toml`: `rusqlite` 0.39.0 features `bundled-sqlcipher-vendored-openssl`, `backup`, `fallible_uint`. |
| **AC2** | New vaults not plain `SQLite format 3` | **PASS** (code+test) | `open__new_vault_header_not_plain_after_write`; F14 no `cipher_plaintext_header_size`. |
| **AC3** | Wrong key open/verify fails `VaultLocked` class | **PASS** (code+tests) | `open__wrong_key__vault_locked`; CLI `backup_verify__wrong_key__wrong_key_class`; backup verify applies key + `sqlite_master`. |
| **AC4** | F-02/K-06 strict; dual-mode `if plain` removed | **PASS** | CLI + store recovery_drills assert fail-closed; no residual success branch. (Comment stale → P2-4.) |
| **AC5** | Legacy plain → `LegacyPlaintextVault` + `vault encrypt` smoke | **PASS** | Header sniff + error type; `encrypt_plaintext_vault__content_smoke`; CLI `vault encrypt` dry-run/`--confirm`/`--destination`. |
| **AC6** | Backup create/restore under SQLCipher; keyed source; list surfaces key fail | **PASS w/ note** | `run_backup` keys+verifies source; list warns on key fail (not silent ignore). Restore hard-fail-while-daemon remains T188. |
| **AC7** | Zero-key refuse at VaultConnection; hermetic escape | **FAIL** | Product refuse **PASS**; hermetic coverage **FAIL** (P0-1). Unit tests for refuse/allow exist with `TempEnv`; bulk suite does not. |
| **AC8** | Docs/claims; Deviations §1; no FIPS/Purge overclaim | **FAIL** | Deviations/COMPATIBILITY F8 body/RELEASE-CLAIMS/SECURITY-LIMITS good; **RECOVERY-DRILLS** + COMPAT non-claims row contradict (P1-1, P2-1). |
| **AC9** | Full gate Windows+Linux; Perl docs; deny/audit | **FAIL / incomplete** | Perl docs + `dev-check.ps1` **PASS**; GHA Perl ensure incomplete (P2-2); hermetic tests incomplete (P0-1); deny/audit evidence not archived (P2-3). Static review did not re-run gate. |
| **AC10** | SECURITY clean; deferred §59 #8 struck | **FAIL** | §59 #8 still open (P1-2); SECURITY review not yet clean given P0/P1. |
| **AC11** | `PRAGMA cipher_version` smoke | **PASS** | `cipher_version__non_empty_when_sqlcipher_linked`. |
| **AC12** | `SqlCipherKey::validate` / `is_zero` | **PASS** | Implemented + unit tests; vault encrypt uses `try_from_raw`. |
| **AC13** | Scratch `check_db` / `check_vault` deleted or gated | **PASS** | No matches; `ai-brains-brain/src/bin` empty of those bins. |

---

## Frozen decisions (F1–F22) audit

| ID | Status | Evidence / gap |
|----|--------|----------------|
| **F1** rusqlite features | **Met** | Workspace `Cargo.toml` |
| **F2** hold 0.39.0 | **Met** | `version = "0.39.0"` |
| **F3** post-key schema verify | **Met** | `verify_key` + backup verify / run_backup |
| **F4** plain→encrypt `sqlcipher_export` | **Met** | `encrypt.rs` sequence; not Online Backup |
| **F5** encrypted backup Online Backup | **Met** | `BackupService` still uses `rusqlite::backup` |
| **F6** keyed vault/backup opens + audit | **Mostly met** | Known gaps fixed; symbol_bridge ledgerful unkeyed = plain-exempt; encrypt plain source unkeyed = intentional |
| **F7** delete dual-mode residual | **Met** | recovery_drills strict |
| **F8** claims flip; no FIPS/Purge | **Partial** | Primary claims good; RECOVERY-DRILLS lag |
| **F9** zero refuse at VaultConnection | **Met (product)** / **hermetic gap** | P0-1 |
| **F10** blank key refuse | **Met** | `is_blank` in enforce_key_policy |
| **F11** deny/audit re-verify | **Evidence missing** | P2-3 |
| **F12** capture independence | **Not regressed** (static) | Capture path unchanged by store crypto flip |
| **F13** Perl docs + CI PATH | **Partial** | docs + dev-check; GHA explicit ensure missing |
| **F14** cipher_compat 4; no plaintext header size | **Met** | `pragmas.rs` comments + no pragma set |
| **F15** no CE/DataKey rotation | **Met** | out of scope |
| **F16** no auto-encrypt on open | **Met** | refuse + migrate hint only |
| **F17** header sniff both open paths | **Met** | `refuse_legacy_plaintext_if_present` in open + open_read_intent |
| **F18** `vault encrypt` CLI | **Met** | main.rs VaultCommands + vault.rs |
| **F19** hermetic zero-key tests | **Partial** | hermetic_bin yes; library sites no |
| **F20** validate format | **Met** | validate / try_from_raw |
| **F21** cipher_version smoke | **Met** | unit test |
| **F22** scratch hygiene | **Met** | deleted |

---

## Focus-area deep dives

### Wrong-key fail-closed

| Path | Behavior |
|------|----------|
| `VaultConnection::open` | Header not plain → key pragmas → `sqlite_master` → `VaultLocked` |
| `open_read_intent` | Same policy + RO flags |
| Backup verify | Plain header refused with migrate class; else key + schema + integrity/quick_check |
| Backup create (`run_backup`) | Source key + schema verify before Online Backup |
| F-02 / K-06 tests | Assert non-zero / `is_err` + wrong-key substring class; no plain success branch |

**Result:** Product fail-closed is sound for open/verify.

### Zero-key policy

| Layer | Behavior |
|-------|----------|
| `VaultConnection` | Refuse zero unless env escape |
| `encrypt_plaintext_vault` | Same escape |
| `apply_key_pragmas` alone | **Does not** refuse zero (by design; F9 scoped to VaultConnection) |
| CLI default key | Still all-zero material when omitted → fails closed at open unless escape |
| Tests | CLI children OK; in-process zero VaultConnection **broken without env** |

### Plain→encrypt (`sqlcipher_export`, not Online Backup)

Sequence in `encrypt.rs` matches F4: plain-header check → unkeyed open → WAL checkpoint TRUNCATE → ATTACH KEY → `sqlcipher_export` → DETACH → verify not plain + keyed open. CLI defaults to dry-run; `--destination` writes aside; `--confirm` replaces with `*.bak-plain`. No `PRAGMA rekey` on plaintext.

### Unkeyed-open audit (seed sites)

| Site | Classification |
|------|----------------|
| `BackupService::run_backup` | **Fixed** — keyed + verify |
| `has_core_tables` / list metadata path | **Fixed** — key probe surfaces warn (not silent “missing”) |
| CLI backup verify/restore | **Keyed** |
| `vault encrypt` source | **Plain-exempt intentional** |
| Ledgerful `symbol_bridge` RO open | **Plain-exempt** (non-vault) |
| Daemon/CLI AppContext | **VaultConnection** (inherits zero policy) |

No remaining known product vault open that skips key+verify for encrypted vaults.

### Scratch hygiene

`check_db.rs` / `scratch/check_vault.rs` not present. **AC13 PASS.**

### Test hermeticity

| Mechanism | Status |
|-----------|--------|
| `hermetic_bin` + `ALLOW_ZERO_KEY` | Present |
| `TempEnv` on VaultConnection zero-key unit tests | Present for AC7 unit cases |
| Workspace zero-key fixtures without env | **Broken** |
| nextest profile env injection | **Documented only, not real** |

### Docs honesty (FIPS / Purge)

| Doc | FIPS/Purge | SQLCipher live |
|-----|------------|----------------|
| Deviations §1 | Explicit non-claim | Resolved T187 |
| COMPATIBILITY §4 | Explicit non-claim | Live wording |
| RELEASE-CLAIMS R-F8/R-K06 | Closed; non-claim FIPS/Purge | Live |
| SECURITY-LIMITS | Forbidden non-claim | Live |
| CLI vault after_help | not FIPS/Purge | sqlcipher_export honesty |
| RECOVERY-DRILLS | Purge non-claim OK | **Still claims plain bundled** ← fail |

No FIPS or NIST Purge product claim found in elevated claim set. CE wipe honesty retained.

---

## DoD / plan phase checklist (summary)

| Phase | Assessment |
|-------|------------|
| A Spike (features/build) | Implemented in tree; deny/audit evidence not in log |
| B Open/backup/zero/migrate | Core code done; hermetic zero incomplete |
| C Tests | H-01/H-02/P-01/P-02/V-01/Z-01 unit coverage present; suite hermeticity incomplete |
| D Docs | Partial — RECOVERY-DRILLS + COMPAT non-claims lag |
| E Closeout | Not ready (P0/P1 open; deferred #8 not struck) |

---

## Positive notes (what landed well)

1. Correct technical choice of **`sqlcipher_export`** for plain→encrypt (F4) vs Online Backup for encrypted backups (F5).
2. Shared header sniff + migrate hint naming `vault encrypt` / `sqlcipher_export`.
3. Zero-key / blank / format policy centralized on `VaultConnection` (daemon/shadow/migrate inherit).
4. Strict recovery drills (no dual-mode residual in executable assertions).
5. Claims docs elevated set largely honest on non-claims (FIPS/Purge/page key ≠ DEK).
6. Perl capability check in `dev-check.ps1` + `ci-tooling.md`.
7. Scratch hygiene completed.

---

## Recommended fix order before R2

1. **P0-1** — Make zero-key tests hermetic (env injection and/or non-zero keys + TempEnv helper). Re-run `cargo nextest run --workspace --profile ci` on a shell **without** ambient `AI_BRAINS_ALLOW_ZERO_KEY`.
2. **P1-1** — Rewrite `Docs/RECOVERY-DRILLS.md` encryption honesty + residual table for T187.
3. **P1-2** — Strike deferred §59 #8.
4. **P2-1** — COMPATIBILITY non-claims row.
5. **P2-2** — GHA Windows Perl assert/install.
6. **P2-3** — Paste deny/audit into review log.
7. **P2-4/P2-5** — Stale comments.
8. Optional P3s (cipher_version pin, plan status flip, dry-run zero-key).

---

## Verdict

# **FAIL**

**Reason:** Production SQLCipher path is substantially correct, but track DoD is not met while:

- AC7 hermetic zero-key policy is incomplete (P0) → AC9 gate not hermetically green;
- AC8 claims integrity broken by stale RECOVERY-DRILLS residual language (P1);
- AC10 deferred §59 #8 not struck (P1).

Re-review as **R2** after P0 + P1 (and preferred P2s) are fixed with evidence.

---

## Appendix — key file map

| Area | Path |
|------|------|
| rusqlite features | `C:\dev\AI-Brains\Cargo.toml` |
| SqlCipherKey | `C:\dev\AI-Brains\crates\ai-brains-crypto\src\sqlcipher.rs` |
| Vault open / zero policy | `C:\dev\AI-Brains\crates\ai-brains-store\src\connection.rs` |
| Header sniff | `C:\dev\AI-Brains\crates\ai-brains-store\src\header.rs` |
| sqlcipher_export | `C:\dev\AI-Brains\crates\ai-brains-store\src\encrypt.rs` |
| PRAGMAs | `C:\dev\AI-Brains\crates\ai-brains-store\src\pragmas.rs` |
| Errors | `C:\dev\AI-Brains\crates\ai-brains-store\src\errors.rs` |
| Backup keyed source | `C:\dev\AI-Brains\crates\ai-brains-brain\src\backup.rs` |
| CLI vault encrypt | `C:\dev\AI-Brains\crates\ai-brains-cli\src\commands\vault.rs` |
| CLI wiring | `C:\dev\AI-Brains\crates\ai-brains-cli\src\main.rs` |
| CLI recovery drills | `C:\dev\AI-Brains\crates\ai-brains-cli\tests\recovery_drills.rs` |
| Store recovery drills | `C:\dev\AI-Brains\crates\ai-brains-store\tests\recovery_drills.rs` |
| Hermetic helper | `C:\dev\AI-Brains\crates\ai-brains-cli\tests\common\mod.rs` |
| nextest | `C:\dev\AI-Brains\.config\nextest.toml` |
| dev-check Perl | `C:\dev\AI-Brains\scripts\dev-check.ps1` |
