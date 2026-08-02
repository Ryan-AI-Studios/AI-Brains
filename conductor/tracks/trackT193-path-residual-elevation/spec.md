# T193 — Path Residual Elevation (post-T190)

- **Track ID:** T193-PathResidualElevation
- **Phase:** Post-T190 residual elevation
- **Status:** 🚧 **In Progress** (implement Phases B–E; orchestrator owns Completed)
- **Depends on (hard):** T190 / ADR-0021 Accepted (cap-std 4.0 + `FollowSymlinks::No` SOOT for vault open+list)
- **Depends on (soft):** T145/T188 write pre/post reparse; T161 token reparse; T188 recovery kit write reparse; T154 `refuse_if_reparse`
- **Blocks / feeds:** Shrinks **R-12** residual register honestly; does **not** reopen closed vault-root read DoD
- **Category:** SECURITY
- **Deferred absorbed:** T190 ambient CLI / write / token path residuals; T188 artifact/migrate write residual (elevate highest-risk); R-12 residual list rewrite
- **Not absorbed:** Perfect Windows all-API TOCTOU; soft-canonicalize as security open; plugin WASI host; mass rewrite of every harness adapter ambient read; T195 multi-user pipe/UDS/HTTP-SYS; T196 systemd/launchd
- **Research date:** 2026-08-02
- **AI fold-in:** AI1 affirm (§1–5) + handle-based hardlink; AI2 **M1–M5**, **L1–L6**, **O1** (preferred replace), **O2–O3** soft. Disposition §15.
- **Ledger:** TX `a52b3a65-fe17-4553-a919-2494a1c56426` (SECURITY)

## 1. Objective

Elevate **remaining path-bearing surfaces** that T190 left as honesty residuals to the same **capability / refuse-reparse / no ambient content fallback** discipline **where product risk warrants**, and rewrite the **R-12 residual register** so claims match reality.

Success is **not** “path TOCTOU closed product-wide.” Success is:

1. Named high-risk surfaces elevated **or** permanently residualled with owners.
2. Shared write SOOT in `ai-brains-path` (or proven reject with residual).
3. No regression on T190 vault open+list.
4. R-12 / SECURITY-LIMITS / ADR-0021 residual sections updated honestly.

## 2. Live baseline (re-scan 2026-08-02)

### 2.1 Already hardened (T190 — **do not regress**)

| Asset | Mechanism |
|-------|-----------|
| `vault_fs::read_file_under_root` | `read_file_nofollow_components` only |
| `obsidian::walk_vault` | `Dir::entries` + nofollow descent; zero ambient `read_dir` |
| Hermes / Honcho export loaders | Elevated to shared list+read helpers |
| SOOT | `cap_open` + `FollowSymlinks::No` + Unix `O_NOFOLLOW` / Windows `FILE_FLAG_OPEN_REPARSE_POINT` |
| Deps | Workspace `cap-std = "4.0"` → lock **4.0.2**; `cap-fs-ext = "4.0"` → **4.0.2** |

### 2.2 Residual inventory (ranked)

| Rank | Surface | Live mechanism | Risk | T193 disposition (freeze) |
|------|---------|----------------|------|---------------------------|
| **P0** | `artifact_security::write_protected_artifact` | Pre-check → `std::fs::write` → post reparse; ProgramData wrappers / `daemon.env` | **Critical** — SYSTEM-executed scripts | **Elevate** write SOOT |
| **P0** | `ai-brains-api-server` `token.rs` load/write | Reparse check + ambient `read_to_string` / `write` + post reparse | **High** — bearer secret | **Elevate** (override T190 F6 “out”) |
| **P0** | `recovery::write_kit_file` | Parent reparse walk + ambient `OpenOptions` create/truncate | **High** — offline DataKey material | **Elevate** |
| **P1** | Migrate report / migrate-manifest paths | `refuse_if_reparse` then ambient write | Med — operator report integrity | **Elevate if free** via shared write helper; else residual |
| **P1** | Shadow dest / dogfood+evaluate report paths | Pre-check reparse then ambient | Med | Same as migrate |
| **P1** | Backup `create_dir_all` dest trees | **Zero** reparse refuse today (`backup.rs` ambient create only) | Med-low | Evaluate; at minimum parent reparse refuse if free; else residual with note |
| **P2** | Ambient CLI long-tail (concrete): adapters `antigravity.rs`/`agy.rs` `read_to_string`/`read_dir`; `commands/context.rs` `.env` write; `elevation.rs` temps; `path/discovery.rs` id-file `read_to_string`; `graph/cozo_proxy.rs` temp NDJSON `std::fs::write` | Ambient FS | Low–med | **Inventory + residual register** with file:line; elevate **only** if cheap SOOT fit without broad rewrite |
| **R** | Soft-canonicalize (`resolve_best_effort`) | Identity / display helper | N/A as open gate | **Permanent residual** — non-claim for TOCTOU |
| **R** | Soft-skip symlink proof when create privilege missing (T190 F17 / Codex P3) | Test residual | Verification only | **Keep** soft-skip; product path still fail-closed |
| **R** | Perfect all-API Windows TOCTOU | OS limits | — | **Non-claim** |
| **R** | Plugin WASI / untrusted host | ADR-0019 | — | **Out** (T182 residual) |

### 2.3 Claims surface today

| Doc | R-12 text |
|-----|-----------|
| `Docs/RELEASE-CLAIMS.md` | Implemented-with-residuals: vault open+list hardened; residuals = ambient CLI, soft-canon, token path, T188 write pre/post |
| `Docs/SECURITY-LIMITS.md` §5 | Same residual list |
| ADR-0021 §3–4 | Write + token + soft-canon residualled |
| `conductor/deferred.md` | T190 ambient/write/token → **T193** |

## 3. Research summary (2026-08-02)

| Source | Finding | T193 application |
|--------|---------|------------------|
| crates.io **cap-std 4.0.2** (2026-02-15) | Still latest stable 4.0 line; MIT/Apache+LLVM | **Hold** workspace pin `4.0` / lock 4.0.2 — zero forced bump |
| crates.io **cap-fs-ext 4.0.2** | `OpenOptionsFollowExt::follow(FollowSymlinks::No)` | **Required** on write/create opens same as read (T190 P0 lesson) |
| docs.rs cap-std `OpenOptions` | Has `write` / `create` / `create_new` / `truncate`; no public `follow_symlinks` | Use `cap-fs-ext` + platform `custom_flags`; never invent APIs |
| ADR-0021 / T190 security reviews | Default `FollowSymlinks::Yes` software-follows after OS nofollow probe | Write path **must** reuse `nofollow_*_options()` pattern |
| Windows reparse literature | Check-then-write remains racy; open-time `FILE_FLAG_OPEN_REPARSE_POINT` + refuse is open-time control | Prefer handle open with reparse flag then write on handle |
| **MS CreateFileW (AI2 M1)** | **`FILE_FLAG_OPEN_REPARSE_POINT` cannot be used with `CREATE_ALWAYS`**. With `TRUNCATE_EXISTING` + OPEN_REPARSE_POINT, the **reparse point itself** is truncated at open time — before user-code handle inspect | **Ban** Windows nofollow write+truncate replace; mandate create_new / delete-regular-then-create_new / temp-rename (F9 / F31) |
| docs.rs cap-std OpenOptions | **No** `open()` method — must `Dir::open_with` | Write helpers take `&Dir` or open ambient parent first (F8 / F32 / M3) |
| T190 F5 | Evaluate T188 write; elevate preferred not hard-required if tests insufficient | T193 makes **P0 elevate hard**; P1 preferred |
| Product AGENTS | Path normalization mandatory; no unwrap/expect in prod; SECURITY review for path TOCTOU class | Cross-model SECURITY review required |

## 4. Frozen decisions (F1–F35)

| ID | Decision |
|----|----------|
| **F1 — Problem** | T190 closed vault **read+list** TOCTOU; residual **writes** and **token** still check-then-ambient, and many CLI ambient paths remain. R-12 honesty lists them. |
| **F2 — Goal honesty** | Do **not** claim product-wide path TOCTOU closed. Claim only named elevated surfaces + updated residual register. |
| **F3 — Dep pin** | **Hold** `cap-std`/`cap-fs-ext` **4.0.x** (lock 4.0.2). Zero new production deps unless spike proves impossible (then F3b hand-roll + residual). |
| **F4 — SOOT reuse** | Extend `ai-brains-path::cap_open` for **write/create** with the same component nofollow rules as read. Core primitives (names illustrative): `nofollow_write_options()`, `create_file_component_nofollow(parent: &Dir, ...)`, `write_file_nofollow_leaf`. **Path convenience** wrappers must open parent as `Dir` first (`open_ambient_*`) then `Dir::open_with` — cap-std OpenOptions has **no** path `open()` (AI2 M3). |
| **F5 — FollowSymlinks::No mandatory** | Every write/create `OpenOptions` **must** set `follow(FollowSymlinks::No)` via `cap-fs-ext`. High finding if missing. |
| **F6 — Platform flags + post-open type** | Unix: `O_NOFOLLOW` (+ `O_DIRECTORY` for dirs). Windows: `FILE_FLAG_OPEN_REPARSE_POINT` (+ `FILE_FLAG_BACKUP_SEMANTICS` for dirs). After successful nofollow write-open: verify handle metadata is a **regular file** (`is_file()` && not symlink/reparse); refuse `NotAFile` otherwise — same class as read helpers (AI2 **M4**). **Do not** set `maybe_dir(true)` on write opens (AI2 **L4**). |
| **F7 — No silent ambient fallback** | On elevated surfaces: open/write failure → typed error; **never** fall back to `std::fs::write`/`read` after cap failure (same class as T190 F26). |
| **F8 — Trusted ambient once** | For absolute paths (token, ProgramData, kit path): open **parent** with ambient authority once, then relative nofollow create/write of the **leaf**. Do not multi-segment ambient path open for the file. Helper naming: `open_ambient_vault_dir` is **functionally generic** (`Dir::open_ambient_dir`); add thin alias `open_ambient_dir` / `open_ambient_trusted_dir` for non-vault call sites **or** document that vault-named helper is safe for any trusted parent (AI2 **M5** / **O3** soft — prefer alias, avoid mass rename of T190 call sites). |
| **F9 — create_new vs replace (AI2 M1/M2 rewrite)** | **Create path:** `create_new(true)` + nofollow flags only. If leaf exists → EEXIST / ERROR_FILE_EXISTS → fail closed **or** fall to replace algorithm. **Replace path (force / D0.5 re-schedule / kit `--force`):** **one of** (preferred first): **(a) O1 temp-rename** — `create_new` a temp leaf under same parent Dir → write+sync → atomic rename to final name (Windows `MoveFileEx` REPLACE_EXISTING same-volume; Unix `rename`); **(b) delete-regular-then-create_new** — open existing **read-only** nofollow → verify regular file + nlink==1 on handle → close → delete → `create_new` + nofollow write. **FORBIDDEN on all platforms for replace:** sole ambient `path.exists()` then `truncate(true)` (AI2 **L5**). **FORBIDDEN on Windows:** `TRUNCATE_EXISTING` / `CREATE_ALWAYS` combined with `FILE_FLAG_OPEN_REPARSE_POINT` (truncates reparse **at open**, before refuse). Unix `O_NOFOLLOW`+`O_TRUNC` may refuse symlink with ELOOP, but product SOOT still uses (a) or (b) for cross-platform consistency — **do not** ship Windows truncate+reparse path. |
| **F10 — Handle-based hardlink refuse (AI1 #2)** | After nofollow open (create path: on new handle before/after write; replace path: on read-only probe handle before delete): refuse if nlink > 1. **Prefer handle-bound:** Windows `GetFileInformationByHandle` → `nNumberOfLinks`; Unix `MetadataExt::nlink` on **handle** metadata. Ambient pre-check may remain defense-in-depth but is **not** sufficient alone. |
| **F11 — ACL ordering** | After successful handle write: existing ACL apply+verify (artifact SYSTEM+BA; token owner-only) unchanged. Reparse refuse is **not** a substitute for ACL. Kit `restrict_windows_acl_best_effort` (icacls) stays post-write best-effort (AI2 **L6** — out of mechanism change; verify path/handle still works). Convert cap `File` via `into_std()` if ACL APIs need `std::fs::File`. |
| **F12 — P0 elevate hard** | **Must** elevate: (1) `write_protected_artifact`, (2) `token` load+write, (3) `recovery::write_kit_file`. Partial ship that only documents them is **fail**. Live bug class: ambient `std::fs::write` can overwrite through a swapped symlink **before** post-write delete (AI1 §1). |
| **F13 — P1 elevate preferred** | Migrate report/manifest, shadow dest, dogfood/evaluate report writes: use shared write helper if free; else residual with owner in AC11 register. **Backup** (`backup.rs`) has **zero** reparse refuse today — P1 evaluate must note that baseline is pure-ambient; adding parent refuse is the minimum bar if elevated (AI2 **L3**). |
| **F14 — P2 inventory residual** | Concrete long-tail (not vague labels): `adapters/antigravity.rs`, `adapters/agy.rs`, `cli/commands/context.rs`, `cli/elevation.rs`, `path/discovery.rs` (id-file read), `graph/cozo_proxy.rs` (temp NDJSON write). **Not** hard DoD to rewrite. Residual register lists file:line examples. Optional opportunistic SOOT only if zero scope creep (AI2 **L2**). |
| **F15 — Token in scope** | **Overrides T190 F6 “token out”.** Bearer secret is high enough value. Load: ambient parent Dir → nofollow **read** open leaf → `Read::read_to_end` on handle — **not** sole `std::fs::read_to_string` after check. Write: F9 create/replace SOOT + owner ACL. |
| **F16 — Soft-canon permanent residual** | `resolve_best_effort` stays identity/display; **never** marketed as TOCTOU close. No code change required unless tests confuse the claim. |
| **F17 — No dual permanent stacks** | One write SOOT in `cap_open` (or documented F3 hand-roll). Do not leave permanent parallel “check-then-write” + “cap-write” without migration of P0. |
| **F18 — T190 regression ban** | Vault_fs / walk_vault / hermes / honcho remain on existing read helpers. Any change that reintroduces ambient `std::fs::read`/`read_dir` on those paths = **high**. |
| **F19 — Errors** | Map write errors to existing domains where possible (`CapOpenError` + CLI `Box<dyn Error>` / `TokenError`). Actionable messages; no panics; no unwrap/expect in production. |
| **F20 — Capture independence** | Path-only; no models/graph deps. |
| **F21 — Contracts** | No daemon DTO change expected. Token file format unchanged (opaque bearer string). Kit JSON schema unchanged (T194). |
| **F22 — ADR** | Prefer **short amend** to ADR-0021 residual table (write+token elevated; new residual list) over a full ADR-0022 unless write invents a different stack. |
| **F23 — Claims rewrite** | On ship: R-12 residual list **shrinks** P0 items; remains Implemented-with-residuals if any residual remains (expected: soft-canon + P2 ambient long-tail + perfect-Windows + parent create residual). |
| **F24 — Review category** | SECURITY cross-model review required. |
| **F25 — Soft-skip tests** | Symlink/junction create proofs may soft-skip when privilege missing (same as T190); product fail-closed must still be unit-proven with pure helpers + available FS cases. |
| **F26 — Parent walk residual** | Where only leaf is elevated and parents are ambient-created (`create_dir_all`), document residual: parent chain create is still check-then-create class; mitigate with post-create reparse refuse on parent before leaf open (already partially present). Full dir-cap mkdir chain is **stretch** (P1/P2), not P0 blocker if leaf open is nofollow. |
| **F27 — Unix gating (AI2 L1)** | `write_protected_artifact` remains **Windows-only fail-closed** (no fake ProgramData). Shared `cap_open` **write helpers MUST work on Unix** (token + kit are cross-platform). **Do not** `#[cfg(windows)]` the shared SOOT helpers. |
| **F28 — Determinism** | Tests hermetic (tempdir); no ambient path pollution; soft-skip documented. |
| **F29 — Doctor kit path** | Doctor **read** of kit already reparse-refuses; optional elevate to nofollow open if free — not P0. |
| **F30 — Scope cap** | If P0+docs+R-12 rewrite + gate clear, track may complete with P1 residualled. Do not expand into T195 multi-user or WASI. |
| **F31 — Windows truncate trap ban (AI2 M1)** | Implementer **must not** use `TRUNCATE_EXISTING` + `FILE_FLAG_OPEN_REPARSE_POINT` as the replace strategy. Review finding = **high** if present on P0. |
| **F32 — API parent type consistency (AI2 M3)** | Leaf write helpers that take `&Path` for parent **must** document and implement ambient `Dir` open first. Prefer public API shape: `write_file_nofollow_leaf(parent: &Dir, file_name: &str, bytes, mode)` + thin Path convenience. Optional soft: `write_file_nofollow_components` mirroring read (AI2 **O2** — not hard DoD). |
| **F33 — Optional multi-component write (O2)** | Soft: if free, add component-walk write helper. Not required for AC clearance. |
| **F34 — Ambient dir alias (O3)** | Soft preferred: `pub use open_ambient_vault_dir as open_ambient_dir` (or new thin wrapper). Hard rename of all T190 call sites **not** required. |
| **F35 — AC replace proof** | Mandatory test: force/replace path with symlink leaf **refuses** and **does not** truncate/destroy reparse target content (AI2 M1 verification matrix). Soft-skip only when symlink create privilege missing. |

## 5. API sketch (normative shape — implement may rename)

```text
// ai-brains-path::cap_open (extend)

fn nofollow_write_options_create_new() -> OpenOptions
  // write(true); create_new(true); follow(FollowSymlinks::No); + platform flags
  // NEVER: truncate(true) with Windows OPEN_REPARSE_POINT

fn open_ambient_dir(path: &Path) -> Result<Dir, CapOpenError>
  // alias or wrapper over open_ambient_vault_dir / Dir::open_ambient_dir

fn create_file_component_nofollow(parent: &Dir, name: &str) -> Result<File, CapOpenError>
  // create_new + nofollow; post-open is_file; handle nlink==1

fn write_file_nofollow_leaf(parent: &Dir, file_name: &str, bytes: &[u8], mode: CreateMode)
  -> Result<(), CapOpenError>
  // CreateNew: create_file_component_nofollow → write_all → sync?
  // Replace: (preferred) temp create_new + write + rename
  //       OR delete-regular-then-create_new (read-only nofollow probe first)
  // Post: never ambient std::fs::write fallback

// Convenience (optional):
fn write_file_nofollow_under_parent_path(parent_dir: &Path, file_name: &str, bytes, mode)
  // = open_ambient_dir(parent_dir)? then write_file_nofollow_leaf

// Call sites
write_protected_artifact → write SOOT (+ ACL after)  // Windows-only caller gate
token write/load → SOOT (+ owner ACL)                // cross-platform
write_kit_file → SOOT (+ best-effort ACL)            // cross-platform
```

`CreateMode`: **`CreateNew` | `Replace`** only.  
**Removed:** `TruncateExisting` / `CreateOrTruncate` as open flags (unsafe on Windows with reparse flag — F9/F31).

## 6. Acceptance criteria

| AC | Criterion |
|----|-----------|
| **AC1** | Shared write SOOT in `cap_open` (or F3 hand-roll recorded) with `FollowSymlinks::No` + platform flags |
| **AC2** | `write_protected_artifact` uses SOOT; no sole ambient `std::fs::write` as success path |
| **AC3** | Token load+write elevated; refuse reparse open-time; no sole ambient read/write success path |
| **AC4** | `write_kit_file` elevated same class |
| **AC5** | Proof tests: symlink/reparse leaf refused on each P0 surface (soft-skip create privilege OK) |
| **AC6** | No silent ambient fallback on P0 (code + review pin) |
| **AC7** | T190 vault read/list tests still green; no ambient reintroduction |
| **AC8** | R-12 + SECURITY-LIMITS + ADR-0021 residual sections rewritten; deferred T193 row struck on ship |
| **AC9** | Soft-canon remains non-claim |
| **AC10** | deny + audit green; hold cap-std 4.0.x |
| **AC11** | Residual register completed (P1/P2/R items with owners + concrete file paths for P2) |
| **AC12** | Full gate + SECURITY review (internal + cross-model) |
| **AC13** | **Force/replace** with symlink leaf: refuse; **target content not truncated** (F35 / M1) — mandatory where symlink creatable |
| **AC14** | Hardlink refuse uses **handle-bound** nlink (or documented equivalent on open handle); nlink>1 fails closed before content commit |

## 7. Non-goals

- Claiming #12 / R-12 “fully closed for entire product”
- Soft-canonicalize as TOCTOU security open
- Plugin WASI / untrusted connector host
- Rewriting all harness adapter FS I/O
- T195 pipe SDDL / UDS / LocalSystem token multi-user work
- MSI packaging / systemd units (T196)
- Raising symlink-create privilege requirements for CI

## 8. Threats / anti-patterns

| Threat | Mitigation |
|--------|------------|
| Swap leaf to symlink between check and ambient write | Open-time nofollow create_new / replace SOOT (F5–F9) |
| Software-follow on write open (T190 P0 class) | `FollowSymlinks::No` mandatory (F5) |
| **Windows TRUNCATE_EXISTING + OPEN_REPARSE_POINT destroys reparse at open** | F9/F31 ban; AC13 proof |
| Handle reparse refuse **after** truncate-on-open | Never truncate-on-open for replace (F9/M2) |
| Silent ambient fallback | F7 high finding |
| Ambient `path.exists()` then truncate | L5 ban; use CreateNew or Replace algorithms |
| Elevate docs only without P0 code | F12 hard fail |
| Gate shared write helpers Windows-only | F27 — helpers cross-platform |
| Scope explosion into all ambient CLI | F14 / F30 |
| Regress vault read SOOT | F18 |
| Claim soft-canon closed TOCTOU | F16 / AC9 |
| `maybe_dir(true)` on write open | L4 ban; post-open is_file (F6) |

## 9. Verification plan

| Layer | Proof |
|-------|-------|
| Unit | Pure refuse helpers; cap_open write component tests (symlink refuse, happy create_new write, **replace does not truncate symlink target**, no ambient fallback, handle nlink) |
| Integration | artifact_security reparse/hardlink retargeted; token security tests; recovery export parent/leaf refuse + force refuse |
| Regression | nextest path + sources + cli packages; T190 vault_fs/obsidian/hermes/honcho |
| Claims | Diff review RELEASE-CLAIMS R-12, SECURITY-LIMITS, ADR-0021, deferred.md |
| Gate | fmt, clippy -D warnings, nextest workspace, deny, audit, ledgerful verify |
| Review | SECURITY category; cross-model before final clear |

## 10. Affected crates / docs

| Area | Touch |
|------|-------|
| `ai-brains-path` | `cap_open` write helpers + tests |
| `ai-brains-cli` | `artifact_security`, `commands/recovery` (+ P1 if free) |
| `ai-brains-api-server` | `token.rs` |
| Docs | ADR-0021 residual amend; RELEASE-CLAIMS R-12; SECURITY-LIMITS; deferred.md; optional OPERATIONS one-liner |
| Conductor | status → Expanded / Completed on ship |

## 11. Handoffs

| To | What |
|----|------|
| R-12 | Residual shrink after P0 |
| T195 | Multi-user pipe/UDS/token location (not path nofollow) |
| T196 | Units/docs |
| Soft-canon | Stays non-claim forever unless new ADR |

## 12. Deferred roll-in matrix

| Item | Disposition |
|------|-------------|
| T190 residual ambient CLI / write / token | **Absorb** — core of track |
| T188 write pre/post reparse residual | **Absorb P0** (artifact + kit); migrate report **P1** |
| R-12 residual rewrite | **Absorb** on ship |
| Soft-skip symlink proof (T190 Codex P3) | **Keep** as verification residual |
| Soft-canonicalize TOCTOU | **Not absorbed** (permanent residual) |
| T195 R-PIPE-IU / R-UDS-TMP / R-HTTP-SYS / R-MULTI | **Not absorbed** |
| T196 units | **Not absorbed** |
| WASI plugin host | **Not absorbed** |
| #40 workspace dep bumps | **Not absorbed** |

## 13. Residual register (filled on ship)

| ID | Surface | Status after T193 | Owner |
|----|---------|-------------------|-------|
| R-WRITE-PARENT | Parent `create_dir_all` chain (artifact/token/kit/report parents) | **Residual** (F26) — leaf nofollow elevated; parent mkdir still ambient create class | path / CLI |
| R-SOFT-CANON | `resolve_best_effort` soft-canonicalize | **Permanent residual** — non-claim for TOCTOU (F16) | path |
| R-AMBIENT-CLI | Long-tail ambient FS (concrete): `adapters/antigravity.rs` `read_dir`/`read_to_string` (~102,148,163,219,367); `adapters/agy.rs` `read_to_string` (~22); `cli/commands/context.rs` `.env` write (~170); `cli/elevation.rs` temps (~65,106–114); `path/discovery.rs` id-file `read_to_string` (~42); `graph/cozo_proxy.rs` temp NDJSON `std::fs::write` (~168,236) | **Inventory residual** — not mass-rewritten (F14) | CLI/graph |
| R-P1-REPORT | migrate report/manifest; shadow-manifest; dogfood compare out; evaluate report | **Elevated** via `write_file_nofollow_under_parent_path` (CreateMode::Replace) | CLI |
| R-P1-BACKUP | `backup.rs` `create_dir_all` for `~/.ai-brains` + sentinel | **Partial** — parent reparse refuse added; dest tree / backup blob I/O still ambient (L3) | CLI |
| R-WIN-PERFECT | Perfect all-API Windows TOCTOU | **Non-claim** | product |
| R-SOFT-SKIP | Symlink create privilege in CI (AC5/AC13 soft-skip) | **Verification residual** — product path still fail-closed | tests |
| R-P0-ELEVATED | artifact / token / kit write+load | **Elevated** (AC2–AC4) — not residual | path/CLI/api |

## 14. Docs touch list (Phase E1)

| File | Change |
|------|--------|
| `Docs/RELEASE-CLAIMS.md` R-12 | P0 write+token elevated; residual list shrink |
| `Docs/SECURITY-LIMITS.md` §5 | Same |
| `Docs/DECISIONS/ADR-0021-path-capability-open.md` | Residual table amend (write/token) |
| `conductor/deferred.md` | Strike T193 residual row on ship |
| `conductor/conductor.md` | Status Completed on ship |
| Optional OPERATIONS | One-liner if operator-facing |

## 15. AI fold-in disposition (2026-08-02)

### AI1 — Affirm + one elevation

| Item | Disposition |
|------|-------------|
| §1 P0 surfaces (artifact / token / kit) | **Affirm** — already F12; AI1 details overwrite-before-post-delete bug class noted under F12 |
| §2 Handle-based hardlink/ACL inspect | **Accept → F10** — nlink on open handle; ACL remains post-write (F11) |
| §3 Shared write helpers sketch | **Affirm** — F4/F5/F6; refined by AI2 M3 for `&Dir` |
| §4 No silent ambient fallback | **Affirm** — F7 / AC6 |
| §5 Honest R-12 residual rewrite | **Affirm** — F23 / AC8 |
| Summary table items 1–5 | **Accept as implement checklist** — plan Phase B/C/E |

### AI2 — Required mediums

| ID | Disposition |
|----|-------------|
| **M1** Windows TRUNCATE + OPEN_REPARSE trap | **Accept → F9 rewrite + F31 + AC13** |
| **M2** Post-open refuse too late for truncate-on-open | **Accept → F9** (create vs replace algorithms; never O_TRUNC/TRUNCATE replace) |
| **M3** Path helper must open parent Dir first | **Accept → F4 + F32 + §5 API** |
| **M4** Post-open is_file check | **Accept → F6** |
| **M5** open_ambient_vault_dir for non-vault parents | **Accept → F8 + F34** (alias; no forced T190 mass rename) |

### AI2 — Lows

| ID | Disposition |
|----|-------------|
| **L1** Shared SOOT Unix-capable; only artifact Windows-gated | **Accept → F27** |
| **L2** Concrete P2 file paths | **Accept → F14 + §2.2 P2 row** |
| **L3** Backup zero reparse | **Accept → F13 note + R-P1-BACKUP** |
| **L4** No maybe_dir(true) | **Accept → F6** |
| **L5** path.exists() then truncate TOCTOU | **Accept → F9 ban** |
| **L6** icacls ACL post-write | **Accept note → F11** (no mechanism change) |

### AI2 — Opportunities

| ID | Disposition |
|----|-------------|
| **O1** Atomic temp-rename replace | **Accept as preferred Replace strategy → F9(a)** |
| **O2** write_file_nofollow_components | **Soft → F33** (not hard DoD) |
| **O3** Rename open_ambient_vault_dir | **Soft alias → F34** (no mass rename) |

### Declined / not folded

| Item | Why |
|------|-----|
| Mass rewrite of P2 ambient CLI | Out of scope (F14/F30) |
| Claiming product-wide TOCTOU closed | Contradicts F2 |
| New ADR-0022 solely for alias rename | F22 short amend sufficient |

### Net freeze delta

- Freezes **F1–F30** expanded/amended; **F31–F35** added.
- AC **AC13–AC14** added.
- `CreateMode` reduced to **CreateNew | Replace**; truncate-open removed.
