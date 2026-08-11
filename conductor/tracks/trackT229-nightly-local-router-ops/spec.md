# T229 — Nightly + local router ops (env / health / schedule)

- **Track ID:** T229-NightlyLocalRouterOps
- **Phase:** Post-audit CLI quality series (T217–T232) — **last series placeholder** (P0 ops residual + P1 bugfix + P2 product)
- **Status:** 📋 **Planning** (plan-only until **go**) — AI fold-in 2026-08-11
- **Depends on:** T132/T143/T145 schedule wrappers ✅; T135 schedule state on status ✅; T205 global dotenv gap-fill ✅; T239 multi-import + status block ✅; T100/llama_cpp timeouts ✅; T231 F32 random-project honesty (pattern) ✅
- **Blocks / feeds:** Operators can trust overnight brain against local router; closes audit “nightly not scheduled / model env only in project .env”; unblocks confidence for T233 multi-root (does **not** implement multi-root)
- **Category:** OPS / FEATURE / BUGFIX / DOCS
- **Source:** Audit 2026-08-05 nightly ops; series README T229; deferred.md Placeholder; live dogfood 2026-08-11
- **Deferred absorbed:** deferred.md “Nightly schedule + router :8081/:8083” product half → **DoD**; live **UTF-8 embed truncate panic** (exit **101**) → **hard F5**; status URL/probe/Last Result F1–F2/F6; OPERATIONS F3/F7; F4 gap-fill verify; **F13** nightly random `ProjectId` (T231 F32 class) → **hard**
- **Not absorbed:** Multi-root Ledgerful bridge / System32 cwd → **[T233](../trackT233-path-alias-multiroot-nightly/spec.md)**; bundle/start GPU router from Rust; doctor model-endpoint matrix (F8 soft); MSI; clap 5; JSON status contract (F12 soft); 50ms-per-embedding sleep latency (F14 soft residual); Router ONLOGON residual codes (F11 soft)
- **Research date:** 2026-08-11 (live dogfood + code truth + crates.io + llama.cpp + std `floor_char_boundary`)
- **AI fold-in:** 2026-08-11 — AI1 T229 **M1–M4 hard**, **L1–L2 hard**, **O1 hard**. AI2 **M1–M3 hard**, **L1–L7 hard** (L8 soft residual, L9 no-op), **O1–O8 hard** (O8 cross-model on go). Disposition **§15**.
- **Ledger:** plan-only — open TX on **go** (`ledgerful ledger start T229-nightly-local-router-ops --category FEATURE`)

## 1. Objective

1. **Ops honesty:** Machine can run nightly against local dynamic router (`c:\llm\router.bat` → completion **:8081**, embedding **:8083**) with global dotenv + scheduled task.
2. **Product honesty:** `nightly --status` shows **effective model endpoints** (host:port, credentials redacted) + **soft health probe** + **Task Scheduler Last Result** so operators see non-zero exits (e.g. 101).
3. **Run-time soft probe:** **After multi-import, before summarize** — non-fatal warn if completion/embed endpoints are down.
4. **Hard bugfix (F5):** Embedding truncate must be **UTF-8 char-boundary safe** — live panic is forbidden production behavior.
5. **Hard bug fix (F13):** Nightly project resolve must **not** invent a random UUID on missing/invalid env (same class as T231 F32); fix dead “default project” warning.
6. **Docs:** OPERATIONS documents router.bat, dual schedule paths, probe expectations, Last Result 101.
7. **Capture independence:** Probes/status/docs/truncate only. No new event types, no contracts DTO growth, no model/router bundling.
8. **Deps:** **No new direct deps on `ai-brains-cli`**. HTTP probe lives on **`LlamaCppProvider::probe_health`** in `ai-brains-models` (reuses existing `reqwest::Client`). No version pin bumps.

## 2. Live baseline (2026-08-11)

### 2.1 Operator dogfood (this machine)

| Item | Observed |
|------|----------|
| `%USERPROFILE%\.ai-brains\.env` | MODEL/EMBED URLs → `127.0.0.1:8081` / `:8083`; models gemma + nomic |
| `nightly-run.cmd` | Soft `curl` probes + `ai-brains --no-project-context nightly` (multi-import **on**) |
| `register-nightly-tasks.ps1` | Registers `AI-Brains-Nightly` daily 03:00 + `AI-Brains-Router` ONLOGON |
| `schtasks` AI-Brains-Nightly | **Ready**; next **2026-08-12 3:00:00 AM**; Last Result **101** |
| `schtasks /FO CSV` (live) | **Only 3 columns:** `TaskName,Next Run Time,Status` — **no Last Result column** |
| `Get-ScheduledTaskInfo` | `LastTaskResult = 101` (authoritative numeric source) |
| `schtasks /FO LIST /V` | Has `Last Result:` line (locale-sensitive labels) |
| Router HTTP | `:8081` and `:8083` `/health` + `/v1/models` → **200** when router up |
| `ai-brains nightly --status` | Schedule Yes + last run + multi-import; **no** model URLs / probe / Last Result |
| Last scheduled log | Panic mid embedding backfill → `nightly exit 101` |
| `ai-brains-cli` Cargo.toml | **No `reqwest`**; **no wiremock** in dev-deps |
| `ai-brains-models` | Has `reqwest` workspace + `wiremock` 0.6 dev-dep |

### 2.2 Root cause — exit 101 (hard F5)

```text
// crates/ai-brains-brain/src/embeddings.rs:69
let text = if content.len() > 4000 {
    &content[..4000]   // panics when byte 4000 is mid multi-byte UTF-8
} else {
    content
};
```

Live log (2026-08-11 ~03:16): panic → Task Scheduler **Last Result 101**. Status shows `Errors in last run: []` because the panic never wrote `last_nightly_errors`.

### 2.3 Root cause — random project_id (hard F13, AI2 M3)

```text
// nightly.rs:243-246
let project_id = std::env::var("AI_BRAINS_PROJECT_ID")
    .ok()
    .and_then(|s| ProjectId::from_str(&s).ok())
    .unwrap_or_default();  // ProjectId::default() == ProjectId::new() == Uuid::new_v4() RANDOM

// nightly.rs:248-252 — DEAD WARNING
if project_id == ProjectId::default() {  // compares two different random UUIDs → always false
    tracing::warn!("AI_BRAINS_PROJECT_ID not set...");
}
```

Same class as T231 F32. Brain already uses **nil UUID** as missing-project sentinel (`lib.rs` synth path). T229 uses that SOOT.

### 2.4 Product gaps (frozen)

| Gap | Detail |
|-----|--------|
| Status opacity | No MODEL/EMBED URLs; no probe; no Last Result |
| Probe only in cmd wrapper | Bare `ai-brains nightly` / SYSTEM wrapper do not probe in Rust |
| F6 CSV myth | Spec draft assumed Last Result in CSV col 5 — **live CSV has only 3 cols** |
| F2 dep myth | Spec draft said “reqwest, no new dep” in CLI — **CLI has no reqwest** |
| F4 gap-fill | T205 already merges global dotenv — **verify only** |
| Dual schedule paths | User multi-import vs SYSTEM `--skip-import` under-documented |
| UTF-8 panic | Kills overnight backfill (exit 101) |
| Random project | Missing env → phantom project + dead warning |

### 2.5 Touch map

| Site | Role |
|------|------|
| `ai-brains-brain/src/embeddings.rs` | **F5** `truncate_for_embed` via `floor_char_boundary` |
| `ai-brains-brain` unit tests | AC1–AC2 multi-byte / ASCII |
| `ai-brains-models` (`llama_cpp.rs` or sibling) | **F2** `probe_health(&self, timeout: Duration) -> ProbeStatus`; wiremock AC6 |
| `ai-brains-cli/src/commands/nightly.rs` | **F1/F6/F13** status lines, Last Result fetch, project resolve SOOT, call probe_health |
| `ai-brains-cli` unit tests | CSV/endpoint format pure units (no HTTP); F13 resolve units |
| `Docs/OPERATIONS.md` | **F3/F7** router.bat + dual schedule + Last Result 101 |
| `Docs/CAPABILITIES.md` | Nightly status endpoint/probe bullets |
| **`CHANGELOG.md` (repo root)** | **New** T229 row only — **not** `Docs/CHANGELOG.md` |
| Contracts / daemon API | **None** |
| Dep pins | **Zero version bumps**; **no new direct dep on CLI** (probe in models) |

## 3. Research (2026-08-11)

| Topic | Finding | Use in T229 |
|-------|---------|-------------|
| **`str::floor_char_boundary`** | Stable 1.91+ (repo pin ≥1.95) | **F5** |
| **llama.cpp server** | Prefer `GET /health`; fallback `/v1/models` | **F2** in provider |
| **reqwest** | Workspace 0.13; **models** depends; **CLI does not** | Probe **in models** only |
| **wiremock 0.6** | models dev-dep only | AC6 in models tests |
| **schtasks CSV (live Win)** | Header = TaskName, Next Run Time, Status **only** | Next-run stays CSV; **Last Result not in CSV** |
| **Get-ScheduledTaskInfo** | `LastTaskResult` numeric (101) | **F6 primary** Windows source |
| **LIST /V** | Has Last Result; labels locale-sensitive (T135 risk) | Soft fallback only if PS fails |
| **T205 gap-fill** | Global dotenv always merged before subcommands | F4 = verify only |
| **ProjectId::default** | = random v4 | F13 = nil UUID sentinel, not default() |

## 4. Findings (DoD)

| ID | Severity | Requirement |
|----|----------|-------------|
| **F5** | **Critical hard** | `truncate_for_embed(content, max)` = `&content[..content.floor_char_boundary(max.min(content.len()))]`. Never panic. Units CJK/emoji/smart-quote straddling 4000. |
| **F13** | **Hard** (AI2 M3) | Pure `resolve_nightly_project_id(env: Option<&str>) -> ProjectId`: missing/empty/invalid → **`ProjectId::from_uuid(Uuid::nil())`** (never `unwrap_or_default()` / random). Warn **iff** resolved is nil. Units AC13–AC14. |
| **F1** | Hard | Status prints completion + embedding **host:port** + model names; redact `user:pass@` if present; missing env → show documented defaults. |
| **F2** | Hard | Soft probe via **`LlamaCppProvider::probe_health(&self, timeout: Duration)`** (default caller **2s**). Sequence: GET `{base}/health` → if 404, GET `{base}/v1/models`. Map: 200→`ok`, connect fail→`down`, timeout→`timeout`, other→`error`. Call on **`--status`** and **before summarize (after multi-import)**. Non-fatal; status exit **0** when down. **No reqwest on CLI.** |
| **F6** | Hard | Windows Last Result: **primary** `powershell -NoProfile -Command "(Get-ScheduledTaskInfo -TaskName 'AI-Brains-Nightly').LastTaskResult"` (or equivalent COM). Status line `Last task result: N`. If unavailable → `unknown`. **Do not** assume CSV column 5. CSV path remains for **next run** only (cols 0–2). Non-Windows: omit. |
| **F6b** | Hard (AI1 M2) | Pure helpers for schedule display: quote-aware CSV split for next-run line **and** pure parse of Last Result string from PS stdout / LIST fallback. |
| **F3** | Hard docs | OPERATIONS: `c:\llm\router.bat` / AI-Brains-Router; ports; global dotenv; log path; Last Result 101 meaning. |
| **F4** | Hard verify-only | Confirm T205 global dotenv supplies model keys to schedule wrapper generation. Code change **only if broken**. |
| **F7** | Hard docs | Dual path table: user-principal multi-import ON vs SYSTEM `--skip-import` (T239 D12). |
| **F8** | Soft residual | Doctor model-port matrix — not DoD. |
| **F9** | Soft residual | Persist probe in sync_state — not DoD. |
| **F10** | Soft residual | Register Router from `nightly --schedule` — docs only. |
| **F11** | Soft residual | Router ONLOGON Last Result 267014 — out of scope. |
| **F12** | Soft residual | JSON nightly status — freeze human lines. |
| **F14** | Soft residual (AI2 L8) | 50ms sleep between embeddings — latency residual, not T229. |

## 5. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit (brain): multi-byte at ~4000 → valid UTF-8, len ≤ 4000, no panic |
| **AC2** | Unit (brain): short ASCII unchanged; long ASCII → exactly 4000 bytes |
| **AC3** | Unit (cli): `format_endpoint_line` host:port + strips `user:pass@` |
| **AC4** | Unit (cli): schedule status formatting with last_result `"101"` contains `101` |
| **AC5** | Unit (cli): empty/missing schedule data → `unknown` / Scheduled No, no panic |
| **AC6** | Unit/wiremock (**models**): probe_health 200→ok; refuse→down; timeout→timeout |
| **AC7** | Manual: `nightly --status` shows Completion/Embedding + probe + Last task result |
| **AC8** | Manual: router down → probe=down, status exit 0 |
| **AC9** | Unit (or manual): long multi-byte embed path does not panic |
| **AC10** | Docs: OPERATIONS router.bat, ports, dual schedule, log, Last Result 101 |
| **AC11** | CAPABILITIES nightly bullets mention endpoint/probe/Last Result |
| **AC12** | No contracts DTO change; **no version bumps**; **no new CLI direct dep**; full gate green |
| **AC13** | Unit: `resolve_nightly_project_id(None)` / empty / invalid → nil UUID (not random) |
| **AC14** | Unit: valid env UUID → that ProjectId; nil → warn path detectable (bool or equality) |
| **AC15** | Repo-root `CHANGELOG.md` has T229 row (not Docs/) |

## 6. Design notes

### 6.1 Truncate SOOT (F5) — reconciled snippet

```rust
const EMBED_TEXT_MAX_BYTES: usize = 4000;

pub(crate) fn truncate_for_embed(content: &str) -> &str {
    let end = content.floor_char_boundary(EMBED_TEXT_MAX_BYTES.min(content.len()));
    &content[..end]
}
// call site: let text = truncate_for_embed(content);
```

Use **`.min(content.len())` for clarity** (AI1 M1 / AI2 L1+O6). Single helper; `pub(crate)` for tests.

### 6.2 Status shape (human, additive)

```text
=== Nightly Status ===
Scheduled: Yes (next run: …)
Last task result: 101
Last nightly run: …
…
Completion: 127.0.0.1:8081  model=gemma-4-E4B-it-Q6_K.gguf  probe=ok
Embedding:  127.0.0.1:8083  model=nomic-embed-text-v1.5     probe=ok
Multi-import: …
======================
```

- Probe timeouts must not flip status exit code.
- Never print vault key or secrets.

### 6.3 Probe (F2) — **pinned: LlamaCppProvider (AI2 M1 option b)**

```rust
// ai-brains-models
pub enum ProbeStatus { Ok, Down, Timeout, Error }

impl LlamaCppProvider {
    /// Soft liveness probe. `timeout` is independent of completion/embedding timeouts
    /// (do NOT use the 120s LLM timeout).
    pub async fn probe_health(&self, timeout: Duration) -> ProbeStatus { … }
}
```

- Implementation uses **existing** `reqwest::Client` on the provider.
- Prefer `/health`; on 404 try `/v1/models`.
- Caller (nightly status + pre-summarize) passes `Duration::from_secs(2)`.
- **Tests in `ai-brains-models` with wiremock** (AC6). CLI does not add wiremock/reqwest.
- Status path: construct lightweight providers (or reuse URL-only probe helper on provider type) for both endpoints.

**Rejected:** (a) add reqwest to CLI (contradicts AC12 / grows CLI surface without need).  
**Rejected as primary:** (c) TCP-only `connect_timeout` — loses `/health` semantics; may use only as last-resort fallback inside provider if desired (soft, not required).

### 6.4 Probe phase (F2) — **pinned**

Probe **after multi-import, before summarize** (`nightly.rs` ~ after multi-import block, before `NightlyService::run_nightly`). Import is model-free; probe does not gate import; non-fatal for summarize path.

### 6.5 Last Result (F6) — **pinned, not CSV col 5**

| Source | Role |
|--------|------|
| `schtasks … /FO CSV /NH` | Next run only (fields 0–2). Keep T135 next-run parse; quote-safe split preferred. |
| `Get-ScheduledTaskInfo … LastTaskResult` | **Primary** Last Result (numeric). |
| `/FO LIST /V` English `Last Result:` | Soft fallback if PS fails; locale may break → `unknown`. |

### 6.6 Project resolve (F13)

```rust
pub(crate) fn resolve_nightly_project_id(env_val: Option<&str>) -> ProjectId {
    let Some(raw) = env_val.map(str::trim).filter(|s| !s.is_empty()) else {
        return ProjectId::from_uuid(uuid::Uuid::nil());
    };
    ProjectId::from_str(raw).unwrap_or_else(|_| ProjectId::from_uuid(uuid::Uuid::nil()))
}
```

Warn when result is nil. Do **not** change `NightlyService::run_nightly` signature this track (still takes `ProjectId`).

### 6.7 F4 schedule gap-fill

Verify-only expected. `main` merges global dotenv before schedule/run; wrapper reads process env. Fix only if empty when only global file has keys.

### 6.8 Capture independence

Status/probe/docs/truncate/project resolve only. No event appends for probes.

## 7. Non-goals

- Shipping or starting `router.bat` / llama-server from Rust  
- Multi-root path aliases / Ledgerful Phase2 (**T233**)  
- Changing SYSTEM `--skip-import` default  
- Doctor matrix expansion (F8)  
- Machine JSON status (F12)  
- Fixing AI-Brains-Router ONLOGON residual codes (F11)  
- Embedding inter-call sleep retune (F14)  
- Adding `reqwest` / `wiremock` to CLI crate  

## 8. Verification plan

```powershell
# Red → green
cargo nextest run -p ai-brains-brain --lib truncate_for_embed
cargo nextest run -p ai-brains-models -- probe_health
cargo nextest run -p ai-brains-cli --lib nightly

cargo clippy -p ai-brains-brain -p ai-brains-models -p ai-brains-cli --all-targets -- -D warnings

# Manual
ai-brains nightly --status
# Router down optional: probe=down, exit 0

# Full gate
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace ; cargo deny check ; cargo audit
ledgerful verify --scope full
```

## 9. Risk & series closeout

| Risk | Mitigation |
|------|------------|
| Probe hangs | Explicit 2s `Duration` param on `probe_health` (not 120s LLM timeout) |
| Status path needs async for probe | Status branch already in async `run`; call `.await` |
| Last Result locale | Prefer Get-ScheduledTaskInfo numeric, not localized LIST labels |
| CSV Last Result myth | Do not parse non-existent columns |
| Random project residual if F13 skipped | **Do not skip** — hard DoD |
| Operator Last Result stays 101 until next run | Docs: re-run nightly after F5 ship |

**Series:** T229 last open track in T217–T232.

## 10. Implement order (on go)

1. **Red→Green F5** truncate (brain)  
2. **Red→Green F13** project resolve (cli pure units)  
3. **Red→Green F1/F6/F6b** status format + Last Result fetch  
4. **Red→Green F2** `probe_health` in models + wiremock; wire status + pre-summarize  
5. **F4** verify gap-fill  
6. **Docs** F3/F7 OPERATIONS + CAPABILITIES + **repo-root** CHANGELOG  
7. Manual AC7–AC8 → review (cross-model for F5) → full gate → Complete  

## 11. Soft residuals (post-close)

| Residual | Note |
|----------|------|
| F8 doctor model ports | Later |
| F9 persist probe | Later |
| F10 schedule registers Router | Ops script remains |
| F11 Router ONLOGON 267014 | Ops |
| F12 JSON status | Later |
| F14 embed 50ms sleep | Perf residual |

## 12–14. (reserved)

## 15. AI fold-in disposition (2026-08-11)

### AI1 (T229 section) — agreed

| ID | Disposition |
|----|-------------|
| **M1** truncate_for_embed + floor_char_boundary | **F5 hard** (already; snippet reconciled) |
| **M2** quote-safe CSV + Last Result display | **F6/F6b hard** — CSV for next-run; Last Result **not** from CSV (see live truth) |
| **M3** endpoint format + credential redaction | **F1 hard** |
| **M4** soft probe 2s /health→/v1/models | **F2 hard** — home crate **models**, not inline CLI reqwest |
| **L1** dual schedule docs | **F7 hard** |
| **L2** OPERATIONS/CAPABILITIES/CHANGELOG | **F3 + AC10–AC11 + AC15** |
| **O1** pure unit tests CSV/endpoint | **Hard** via AC3–AC5 |

*Note:* AI1 AC matrix mapped into AC1–AC12; extended by AI2 (AC13–AC15).

### AI2 — agreed (primary blind-spot review)

| ID | Disposition |
|----|-------------|
| **M1** CLI has no reqwest; pin probe approach | **Hard** → **option (b)** `LlamaCppProvider::probe_health`; §6.3 rewritten; AC12 clarified |
| **M2** AC6 mock location | **Hard** → tests in **ai-brains-models** + wiremock; not CLI |
| **M3** random ProjectId + dead warning | **F13 hard** (nil UUID SOOT + warn fix) |
| **L1** §6.1 min(content.len()) | **Hard** reconciled snippet |
| **L2** CHANGELOG path | **Hard** → **repo-root** `CHANGELOG.md` |
| **L3** CSV Last Result index | **Superseded by live truth** — CSV lacks Last Result; F6 uses Get-ScheduledTaskInfo |
| **L4** pin probe phase | **Hard** → after multi-import, before summarize |
| **L5** probe timeout independent of 120s | **Hard** → explicit `Duration` param |
| **L6** F4 verify-only | **Hard** plan Phase 4 already; keep |
| **L7** dead warning note | **Absorbed into F13** |
| **L8** 50ms embed sleep | **F14 soft residual** |
| **L9** f32_vec_to_bytes untouched | **No-op** agree |
| **O1–O7** | Folded as above |
| **O8** cross-model for F5 | **Hard** plan Phase 6 — yes for production panic |

### Declined / not folded

| Item | Why |
|------|-----|
| AI1 assumption Last Result = CSV col 5 | **False on this Windows** — CSV is 3 columns only |
| AI2 option (a) add reqwest to CLI | Unnecessary if (b) lands |
| AI2 option (c) TCP-only as primary | Weaker honesty than `/health` |
| Fix `NightlyService::run_nightly` to `Option<ProjectId>` | Out of scope; nil UUID keeps signature stable |
| AI-review.md **T231** section (lines 1–177) | Wrong track; already shipped PR #138 — ignore for T229 |

### Note on AI-review.md structure

The file begins with a **T231** review (stale for this fold-in). **T229** content starts at the “Review track 229” / AI1 T229 / AI2 blocks. Only T229 findings were folded.
