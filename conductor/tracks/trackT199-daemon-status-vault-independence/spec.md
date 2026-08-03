# T199 — Daemon Status Vault Independence

- **Track ID:** T199-DaemonStatusVaultIndependence
- **Phase:** Post-T198 CLI UX series (P1)
- **Status:** 📋 **Expanded + AI fold-in / Pending** (plan-only; implement on go)
- **Depends on:** T161/T195 daemon IPC; T188 robust probe; T192 doctor `daemon_reachable`; T197 key resolve (CLI SOOT; status must **not** depend on it)
- **Blocks / feeds:** Operator UX for locked vaults; soft honesty for daemon `AI_BRAINS_VAULT_KEY` residual; T201 exit matrix (no new codes here)
- **Category:** FEATURE / DOCS
- **Source:** CLI audit 2026-08-02 P1 + live `daemon status` no key (E≈4/C≈4); deferred residual “`daemon status` requires vault key”; T197 handoff
- **Deferred absorbed:** `daemon status` requires vault key; doctor/status probe helper alignment (shared SOOT; doctor stays **Safety**)
- **Not absorbed:** Full daemon silent-zero SOOT (honesty only); MSI / R-CI-BRANCH; T200 graph install; T201 full exit matrix; multi-user pipe ACL redesign (T195 residuals)
- **Research date:** 2026-08-03 (expand + live re-scan)
- **AI fold-in:** AI1 affirms F1–F3 / probe / keyless vault / hermetic. AI2 **M1–M7** accepted; **L2/L6–L8** + soft **O1/O2**. **O4** declined (no Unix PID). Disposition §14.
- **Ledger:** plan-only (no TX until implement)

## 1. Objective

Make **`ai-brains daemon status`** answer “is the daemon process / IPC up?” **without opening the vault and without requiring a key**, so operators can diagnose when the vault is locked, the key is missing, or they are on a machine with only process visibility.

Align **doctor `daemon_reachable`**, **status**, and **Safety** callers to the **same shared IPC probe helper** (same Ping→Pong truth; policy selects attempt/timeout constants). Doctor keeps **Safety** robustness; only interactive **status** uses the fast **Status** policy.

## 2. Live baseline (re-scan 2026-08-03; AI2 6/6 confirmed)

| Path | Live behavior | Gap |
|------|---------------|-----|
| `daemon status` routing | Goes through `AppContext::from_cli` in `run()` → **requires** key resolve + vault open/`migrate()` | No key → `VAULT_KEY_MISSING` / vault-lock JSON **before** any status line |
| `run_status` body | `DaemonClient::probe(200ms)` once; `Status: Running\|Stopped`; backends TCP (sync 5-attempt + jitter); Windows PID via `tasklist` with `.output()?`; vault path/size/memories only if running | Path/size = metadata only; memories = `count_pinned_memories(&AppContext)` SQL; **tasklist `?` can exit 1** |
| Doctor `daemon_reachable` | Already vault-independent via `probe_restore_daemon_busy` before vault open | Safety 3×1000ms; not shared SOOT with status 200ms single-shot |
| Hermetic tests | `hermetic_bin()` always sets zero key + `ALLOW_ZERO_KEY` | Existing tests **never prove** no-key path (false-positive risk) |
| Operator live | preflight/recall without key → key refuse / missing | Post-T197 CLI SOOT; status still blocked at AppContext |
| Daemon start key | `AI_BRAINS_VAULT_KEY` defaults to all-zero + `from_raw` (main + windows_service) | Residual honesty; **not** status DoD |

### 2.1 Routing map (implement targets)

| File | Role |
|------|------|
| `crates/ai-brains-cli/src/main.rs` | Early-route `DaemonCommands::Status` in **`async fn run()`** before AppContext (~after doctor early-route). **`is_vault_path_free` unchanged** (status stays async) |
| `crates/ai-brains-cli/src/commands/daemon.rs` | `run_status(StatusOptions)`; soft tasklist; optional vault section; **leave `run_update` direct `client.probe` alone** |
| `crates/ai-brains-cli/src/daemon_probe.rs` (new) **or** `daemon_client` ext | SOOT: `probe_daemon_reachable` + `DaemonProbePolicy` + **pub const** attempt/timeouts |
| `crates/ai-brains-cli/src/commands/backup.rs` | Keep `probe_restore_daemon_busy` as **thin Safety wrapper** (doctor/recovery/vault imports unchanged) |
| `crates/ai-brains-cli/src/commands/doctor.rs` | Call shared helper via wrapper or `Safety` policy directly; keep injectable `daemon_up` |
| `crates/ai-brains-cli/tests/*` | No-key hermetic (F15); AC5 const unit; soft AC7 running+no-key |
| `Docs/OPERATIONS.md` + `CHANGELOG.md` | Status needs no key; soft F16/F17 |

## 3. Research summary (2026-08-03)

| Finding | Application to T199 |
|---------|---------------------|
| **Liveness vs readiness** | IPC Ping = liveness; vault open / memory count = optional readiness-adjacent |
| **T188 Safety** | 3×≥1000ms for destructive gates — **do not weaken** |
| **Interactive status latency** | Keep near current ~200ms single-shot; freeze **1×300ms** (AI2 M2) |
| **Doctor = health audit** | Prefer **Safety** robustness over speed (AI2 M6); severity still info-only |
| **assert_cmd env** | `hermetic_bin()` sets key; test must `env_remove` key **and** ALLOW or use no-key helper (M7) |
| **Workspace deps** | rusqlite 0.39 / clap 4.5; crates.io has 0.40.1 / 4.6.5 — **no bumps** |
| **T198** | Graph exit 0→2 shipped; status exit **0** both up/down |

## 4. Frozen decisions (F1–F30)

| ID | Decision |
|----|----------|
| **F1 — No vault open for liveness** | `daemon status` **must not** call `AppContext::from_cli` / `from_resolved_key` / migrate for process/IPC/backend/PID. |
| **F2 — No key required** | Missing `--key` / `AI_BRAINS_KEY` is **not** an error for status. Never sole outcome = vault-lock / `VAULT_KEY_*`. |
| **F3 — Early-route in `run()` only (M1)** | Insert early-route in **`async fn run()`** before `AppContext::from_cli` (mirror doctor at ~doctor early-route). **`is_vault_path_free` / `run_sync_path_free` unchanged** — status is async (`probe_daemon_reachable`). Nested match: `Commands::Daemon { command: DaemonCommands::Status }`. |
| **F4 — Shared probe + thin Safety wrapper (M5)** | SOOT `probe_daemon_reachable(client, policy)` in **`daemon_probe` module** (or `daemon_client` extension). **`probe_restore_daemon_busy` stays in `backup.rs`** as: `probe_daemon_reachable(client, Safety).await` — preserves doctor/recovery/vault imports. Status calls SOOT with **Status** policy. Doctor calls shared path via Safety wrapper or `Safety` directly (same SOOT). |
| **F5 — Probe policies (M2/M6)** | **`Safety`:** 3 attempts × ≥1000ms + 50ms backoff (T188). **`Status`:** **1 attempt × 300ms** (single-shot; match current behavior, slightly wider timeout — **no 2-attempt latency regression**). **Doctor uses `Safety`** (health audit robustness). **Only interactive `daemon status` uses `Status`.** Pub const for attempts/timeouts (O1/L6). |
| **F6 — Vault path optional** | `--vault-path` / `AI_BRAINS_VAULT_PATH` **optional** for status. Absent → omit vault section. Present + Running → path + size via `fs::metadata` (no open). Doctor still requires path. |
| **F7 — Memories optional open (M3)** | Stopped: no vault lines (T128). Running + path: Vault + Vault size always (metadata). Memories via: ```fn try_count_pinned_optional(path: &Path, key: Option<String>) -> Option<u64>``` using **only** `.ok()?` swallow chain: `resolve_operator_sqlcipher_key(key).ok()?` → `VaultConnection::open_read_intent(...).ok()?` → lock → `COUNT(*) … pinned`. **Never** `?` propagate; **never** AppContext/migrate. On `None`: print **`Memories: skipped (vault key missing or vault not openable)`** (pinned string). Exit still 0. |
| **F8 — Exit 0 + soft tasklist (M4)** | Exit **0** for Running, Stopped, missing key. **Required:** change Windows `tasklist` `.output()?` → soft `if let Ok(output) = …` (skip PID line on Err). No non-zero for inactive. |
| **F9 — Human strings** | Keep `Status: Running` / `Status: Stopped`. |
| **F10 — start/stop/install/update unchanged (L8)** | Out of scope. **`run_update` keeps direct `client.probe` calls** — do not migrate to policy helper. `run_start` / `run_stop` unchanged. |
| **F11 — Doctor alignment** | `daemon_reachable` uses **shared SOOT** with **Safety** policy (via thin wrapper OK). Injectable `run_with_daemon_state` remains. Messages `"up"`/`"down"`; never sole hard-fail. |
| **F12 — No new exit codes / JSON** | T201 later. Soft: structure vault section as separate fn for future JSON (O2). |
| **F13 — Capture independence** | Unchanged. |
| **F14 — Zero new deps** | No rusqlite/clap bumps. |
| **F15 — Hermetic no-key proof (M7)** | Must **not** leave hermetic default key set. Required pattern: `hermetic_bin()` then **`env_remove("AI_BRAINS_KEY")` + `env_remove("AI_BRAINS_ALLOW_ZERO_KEY")`**, **or** soft `common::hermetic_bin_no_key()` that never sets them. Assert exit 0; Status Running\|Stopped; not sole vault-lock / `VAULT_KEY_MISSING` / zero-refuse. |
| **F16 — Daemon silent-zero** | **Honesty docs only** (not DoD; not soft product SOOT in T199). OPERATIONS: CLI `AI_BRAINS_KEY` vs daemon `AI_BRAINS_VAULT_KEY` (already partial). |
| **F17 — service-only ACL** | Soft: one OPERATIONS line near daemon lifecycle — interactive status may show Stopped while `sc query AI-Brains-Daemon` is Running when `AI_BRAINS_PIPE_ACL=service-only`. |
| **F18 — sc query** | Soft/out of DoD. IPC Ping = SOOT for “reachable to this CLI.” |
| **F19 — Backend probes (L2)** | Keep existing TCP probes. Accepted debt: **sync** connect + `thread::sleep` inside async `run_status` (pre-existing; not T199 fix). Document latency only. |
| **F20 — Contracts** | No DTO change. DoctorReport shape unchanged. |
| **F21 — High findings** | Status still requires key; vault-lock sole error; Safety weakened; memories open propagates Err; tasklist still `?` → exit 1; hermetic no-key false positive (key still set); doctor diverges from SOOT. |
| **F22 — Series** | After T198; parallel T200 OK (main.rs early-route vs graph stubs — careful). Land before T201. |
| **F23 — Determinism** | Stable labels; backend jitter not in output (O3 OK). |
| **F24 — Review** | FEATURE; Codex if Safety wrapper refactor risks restore. |
| **F25 — Policy constants (L6/O1)** | `DaemonProbePolicy` exposes `pub const` attempts + per-attempt duration (or associated consts). AC5 unit: `Safety.attempts >= 3 && Safety.per_attempt >= 1000ms`; Status: attempts == 1 && per_attempt == 300ms. |
| **F26 — AC7 separate test (L7)** | Existing T128 smoke (has hermetic key) stays. **New** test for Running+path+no-key memories skip (unit inject `is_running` preferred if live daemon flaky). |
| **F27 — No Unix PID (O4 declined)** | Windows tasklist only; non-Windows omit PID (current). No `pgrep` in T199. |
| **F28 — open_read_intent only** | Memories path never migrates. |
| **F29 — AI1 affirm** | Early routing, StatusOptions, shared probe, metadata size, hermetic remove key — all in freezes above. |
| **F30 — No FEATURE_UNAVAILABLE on status** | Status is not feature-gated; T198 string unused here. |

## 5. Residual disposition

| Residual | Disposition |
|----------|-------------|
| `daemon status` requires vault key | **Absorb** |
| Divergent probe implementations | **Absorb** shared SOOT + policies |
| Doctor 3s when down | **Keep Safety** (M6) — intentional |
| Daemon silent zero | **Honesty only** F16 |
| service-only false Stopped | Soft F17 |
| Status JSON / inactive exit | **T201** |
| Backend sync sleep in async | Accepted debt F19 |
| Unix PID | Out F27 |
| MSI / graph install | Out / **T200** |

## 6. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | No key → exit 0 + `Status: Running\|Stopped` | Hermetic F15 |
| **AC2** | No sole vault-lock / `VAULT_KEY_MISSING` / zero-refuse | Assert streams |
| **AC3** | SOOT `probe_daemon_reachable` for status + doctor + Safety path (wrapper OK) | Grep + review |
| **AC4** | Doctor `daemon_reachable` up/down; never sole Fail | doctor_cli + unit |
| **AC5** | Safety ≥3×≥1000ms; Status == 1×300ms | Unit on **pub const** |
| **AC6** | Stopped: no Vault/size/Memories | Smoke/hermetic |
| **AC7** | Running + path + no key: path/size; Memories skip line; exit 0 | New test F26 |
| **AC8** | OPERATIONS: status no key; soft F16/F17 | Diff |
| **AC9** | CHANGELOG | Diff |
| **AC10** | Full gate | Process |
| **AC11** | start/stop/install/update not regressed; `run_update` probes left alone | Review |
| **AC12** | tasklist soft-skip: no `?` on `.output()`; status still exit 0 if tasklist fails | Code review (+ soft unit) |
| **AC13** | `try_count_pinned_optional` never propagates; uses open_read_intent only | Unit / review |

## 7. Non-goals

- Daemon key product SOOT / env rename  
- Pipe/UDS ACL redesign (T195)  
- Auto-start from status  
- Removing backend TCP probes / fixing async blocking sleep  
- Status JSON / non-zero-when-stopped (T201)  
- Graph install (T200)  
- Unix PID via pgrep  
- MSI / R-CI-BRANCH  
- rusqlite 0.40 / clap 4.6  

## 8. Handoffs

| To | What |
|----|------|
| deferred.md | Strike status-requires-key on ship |
| OPERATIONS | Status no key; soft F16/F17 |
| T201 | JSON status; optional inactive exit; structure vault section (O2) |
| T200 | Parallel-safe |
| Future daemon key SOOT | F16 residual |

## 9. Implementation sketch

### 9.1 Probe SOOT (`daemon_probe.rs`)

```text
pub enum DaemonProbePolicy { Status, Safety }

impl DaemonProbePolicy {
    pub const fn attempts(self) -> u32 { match self { Status => 1, Safety => 3 } }
    pub const fn per_attempt(self) -> Duration {
        match self {
            Status => Duration::from_millis(300),
            Safety => Duration::from_millis(1000),
        }
    }
}

pub async fn probe_daemon_reachable(client: &DaemonClient, policy: DaemonProbePolicy) -> bool {
    // loop policy.attempts(); client.probe(policy.per_attempt()); Safety: +50ms backoff between
}
```

`backup.rs`:
```text
pub async fn probe_restore_daemon_busy(client: &DaemonClient) -> bool {
    probe_daemon_reachable(client, DaemonProbePolicy::Safety).await
}
```

- Status command → `Status` policy.  
- Doctor production probe → keep calling `probe_restore_daemon_busy` (Safety) **or** SOOT Safety — same constants.  

### 9.2 StatusOptions + main early-route

```text
// async fn run(), before AppContext::from_cli — NOT is_vault_path_free
if let Commands::Daemon { command: DaemonCommands::Status } = cli.command.as_ref() {
    return commands::daemon::run_status(StatusOptions {
        vault_path: cli.vault_path.clone(),
        key: cli.key.clone(),
    }).await;
}
```

### 9.3 Memories (swallow-only)

```text
fn try_count_pinned_optional(path: &Path, key: Option<String>) -> Option<u64> {
    let resolved = resolve_operator_sqlcipher_key(key).ok()?;
    let conn = VaultConnection::open_read_intent(path, &resolved).ok()?;
    let lock = conn.lock().ok()?;
    lock.query_row(
        "SELECT COUNT(*) FROM memory_projection WHERE status = 'pinned'",
        [],
        |r| r.get(0),
    ).ok()
}
```

### 9.4 Soft tasklist

```text
#[cfg(windows)]
if let Ok(output) = std::process::Command::new("tasklist")
    .args(["/FI", "IMAGENAME eq ai-brainsd.exe", "/FO", "CSV", "/NH"])
    .output()
{
    // parse PID if present
}
```

## 10. Verification plan

| Layer | What |
|-------|------|
| Unit | F25 policy consts; try_count swallow; soft tasklist review |
| Hermetic | AC1–AC2 no-key; AC6; soft AC7 |
| Doctor | Existing AC5 daemon_reachable |
| Manual | Unset key → status shows Running\|Stopped |
| Gate | Full CI + ledgerful verify |
| Review | Safety wrapper must not weaken restore |

## 11. Stop-before

- Weakening Safety below 3×1000ms  
- Key required again as hard fail  
- Daemon silent-zero product SOOT without explicit go  
- Status exit ≠ 0 when Stopped  
- Migrating `run_update` probes “for consistency” without need  

## 12. Suggested implement order

1. `daemon_probe` + pub consts + Safety wrapper in backup.rs; doctor stays on Safety path.  
2. Early-route status in `run()`; rewrite `run_status` without AppContext.  
3. Soft tasklist; F7 memories helper; optional vault section.  
4. Hermetic no-key (F15) + AC5 unit + soft AC7.  
5. OPERATIONS + CHANGELOG.  
6. Gate + review + deferred strike + ledger commit.

## 14. AI fold-in disposition (2026-08-03)

| ID | Source | Disposition |
|----|--------|-------------|
| AI1 §1–4 | Early route, StatusOptions, probe policies, keyless metadata, hermetic | **Accept** — already core; tightened by M* |
| **M1** | F3 routing: `run()` only; not `is_vault_path_free` | **Accept** → F3 |
| **M2** | Status = 1×300ms single-shot | **Accept** → F5 |
| **M3** | `try_count_pinned_optional` + `.ok()?` only | **Accept** → F7, AC13 |
| **M4** | tasklist soft-skip required | **Accept** → F8, AC12 |
| **M5** | Keep `probe_restore_daemon_busy` thin wrapper | **Accept** → F4 |
| **M6** | Doctor stays **Safety**; only status uses Status | **Accept** → F5, F11 |
| **M7** | env_remove key **and** ALLOW; or hermetic_bin_no_key | **Accept** → F15 |
| L1 | path Option handling | Affirm F6 |
| L2 | Backend sync-in-async debt | **Accept** → F19 |
| L3 | Contracts | Affirm F20 |
| L4 | Daemon silent-zero out of CLI track | Affirm F16 (no soft product SOOT) |
| L5 | F17 OPERATIONS placement | Soft F17 |
| L6/O1 | Pub policy consts | **Accept** → F25, AC5 |
| L7 | Separate AC7 test | **Accept** → F26 |
| L8 | Leave `run_update` probes | **Accept** → F10 |
| L9 | start/stop no probe migrate | Affirm F10 |
| O2 | Vault section extract for T201 JSON | Soft F12 |
| O3 | Jitter not in output | Affirm F23 |
| **O4** | Unix pgrep PID | **Decline** → F27 |

**Baseline:** AI2 confirmed 6/6 live claims + dep pins. **Verdict target:** M1–M7 folded; plan ship-ready on go.
)
