# T320 — Unified `ai-brains status` glance

- **Track ID:** T320-UnifiedStatus
- **Status:** **Planned** (Pending until **go**)
- **Category:** FEATURE / UX
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — opportunity (b). doctor / nightly / graph update / daemon status are each fast (0.2–1.3s) but four glances. Series README `README-T312-T324-CLI-DOGFOOD.md`.
- **Depends on:** T192/T249 doctor + `--summary`; T199/T297 daemon status (IPC, exit 0); T255/T269 nightly `--status` JSON + `--quick`; T213/T308 graph density; T204 `help_ia.rs` Daily group; T85 backend URLs (stay on `daemon status`)
- **Blocks / feeds:** Operator “is the vault healthy?” one command. Does **not** replace doctor / `nightly --status` / `daemon status` / `graph update`.
- **Absorbs:** Audit unified-status opportunity; four named sections; fail-open per section
- **Not absorbed (DoD):** Growing `doctor.rs`; 16th doctor check; `ai-brainsd --version` (T310 F15); T307; floor retune; minikube-style bitwise exits; HTTP `/health` 750 ms; daemon TCP retries; harness/device/replicate; T316–T318 / T321–T325
- **Research date:** 2026-08-29 (plan-write product HEAD `464edc2` T319 `#236` conductor note; T319 product `#235` `e03c49d`). Fold-in against `e15188e` (this plan’s own docs commit; ahead **1** of `origin/main` = `464edc2`). Snapshot — **re-verify at execute**.
- **AI fold-in:** 2026-08-29 `agy-review.md` + `opencode-review.md` (HEAD `e15188e`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** **Agree:** Agy m1 HEAD snapshot; Agy m2 `graph_density.rs` crate path; OpenCode O1 AC9 host-daemon; OpenCode O2 `status_next_line` reuse; OpenCode O3 scheduled mapper (live `next_run.is_some()`, not `found &&`). **Partial:** OpenCode m1 line counts — plan-write was **nonblank** (Measure-Object `-Line`); OpenCode counted **physical**; dual-count + crate paths. **Already:** Agy O1/O2/O3 F2 / F4 / F11. **Decline:** OpenCode m1 “pre-T317 / `src/graph.rs`” — 1606 is nonblank of live `commands/graph.rs`. Disposition **§13**.
- **Ledger:** planning DOCS TX `dcb67912-8fb7-4bbd-a354-68ba41857744`. Fold-in DOCS TX `a92f9b07-1894-42a1-8526-9f66fa9ed02d`. Series mint DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** implement until **go**. Do **not** `cargo install`. Do **not** grow hotspot `project.rs` / `sync.rs` / `governed_common.rs` / `doctor.rs` / `daemon.rs` `run_status` / `commands/graph.rs`. New sibling `commands/status.rs`. Dispatch **before** `AppContext` (doctor class). Do **not** print or commit `AI_BRAINS_KEY`. Do **not** start/stop the daemon, mutate schtasks, or `graph rebuild`.

---

## 1. Objective

1. **One glance.** `ai-brains status` prints four named sections: daemon Running/Stopped, doctor attention (`format_doctor_summary`), graph density one-liner, nightly last-run + scheduled. Operators stop running four commands to answer “is the vault healthy?”
2. **`--format json` is a compose envelope** (`schema_version: 1`, CLI-local). No contracts DTO. No required-key steal from `DoctorReport` / `NightlyStatusJson` / `GraphHealthOutput`.
3. **Fail-open per section.** A doctor/vault/schtasks error must not hide daemon (and vice versa). Glance never runs nightly 750 ms HTTP or daemon TCP retries.
4. **North star.** Capture independence: read-only compose of existing helpers. No new events. No models. No hidden CoT. Does not start the daemon or rebuild the graph.

This unblocks daily CLI: the audit’s remaining P1 after T312–T319 is a missing aggregator, not a missing probe.

---

## 2. Live baseline (re-scan 2026-08-29)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | Fold-in against plan-write `e15188e` `docs(conductor): plan T320 unified status glance`. Product `src/` = T319 `#236` `464edc2`. Tree **CLEAN** at plan-write. Branch `track/T320-unified-status`. `origin/main` = `464edc2` (ahead **1**). Plan-write snapshot was `464edc2` / ahead **0** (Agy m1). |
| PATH `ai-brains.exe` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **26,897,408** B; LastWriteTime **2026-08-27 8:21:55 PM**; `ai-brains 0.1.3`. **T263/T293 on PATH.** **T312 / T315 / T314 / T313 / T317 / T319 not.** T320 hole **is** (`unrecognized subcommand 'status'` clap exit **2**). **Do not `cargo install`.** Tests/manual AC use hermetic bin / `cargo run`. |
| `preflight --summary` (PATH) | Pinned **4563**. In-context **0/0/0**. Plan-write `Total Word Count: 699`; OpenCode re-scan **705** (PATH-behind T315 `Budget window words:`). **Not this DoD.** |
| `doctor --summary` | `status=degraded` `ok=12 warn=1 fail=0 skip=2`. **Only** `[warn] graph_density` sparse `E/N=0.416` (`nodes=64356 edges=26740 pinned=51267 memory_nodes=40394`). Exit **0**. `daemon_reachable` is **ok** even when down (message `"down"`) — **summary hides a Stopped daemon.** |
| `nightly --status --quick` | Scheduled **Yes** next **8/29/2026 3:00:00 AM**; Last task result **0**; Last nightly run **2026-08-28T07:08:30Z**; unsummarized **0**; `probe=skipped`; Router Ready + `last run: terminated`. Exit **0**. |
| `daemon status` | `Status: Running`; vault 169.0 MB; Memories 51267; LLM `:8081` Open; Embedding `:8083` Open; PID **3592**. Exit **0**. **No `--format`.** |
| `graph update --format human` | `status: sparse` `density: warn` `edge_node_ratio: 0.4155…` note keeps T308 lag nuance. **No remediator line** (graph-on Sparse). Exit **0**. |
| `ai-brains status --help` | clap `unrecognized subcommand 'status'` exit **2**. |
| Last GitHub PR | [#236](https://github.com/Ryan-AI-Studios/AI-Brains/pull/236) T319 conductor note. `mergedAt` **2026-08-29T01:32:14Z**. Issue/review/inline comments **[]**. PR body Cursor Bugbot is **overview / Low Risk** (docs-only; no defect). Product PR [#235](https://github.com/Ryan-AI-Studios/AI-Brains/pull/235) comments/reviews **[]**. **last-PR Cursor: N/A empty.** `#230` Bugbot already **T325**. Open PRs: **none**. **No T326.** |
| Ledger | 0 pending / 0 drift at scan. Hotspot **#1** `project.rs` (3.698) — **do not touch.** `sync.rs` **#2** (3.502). `governed_common.rs` **#3** (3.372). `daemon.rs` **#8** (1.956) — **do not grow `run_status`.** |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why four glances are still the hole

| Layer | Truth |
|-------|--------|
| No top-level `Commands::Status` | Nested only: `DaemonCommands::Status`, `HarnessCommands::Status`, `DeviceCommands::Status`, `ReplicateCommands::Status`. clap does **not** collide a new `Commands::Status` with those. |
| Doctor `--summary` is attention-only | Warn+fail. `daemon_reachable` is **always** `ok_msg` (`"up"` / `"down"`) so Stopped **never** appears in attention. Graph sparse **does**. Nightly last-run is **absent**. |
| Nightly `--status` is the model-port + schedule novel | 750 ms HTTP unless `--quick`. Router / multi-import / unsummarized are that command’s job. Glance needs last-run + scheduled only. |
| `graph update` is feature-gated | Feature-off `graph *` exit **2**. Doctor `graph_density` is SQL in `crates/ai-brains-cli/src/graph_density.rs` (`mod graph_density` in `main.rs:9`; **not** under `commands/`). Glance must work **graph-off**. |
| `daemon status` is IPC + TCP | T199 no AppContext. TCP 5-retry backoff is **not** a glance. `DaemonProbePolicy::Status` = 1×300 ms; doctor uses **Safety** 3×1000 ms via `probe_restore_daemon_busy`. Glance uses **Status** (same as `daemon status`). |
| Nightly `--status` uses `AppContext` | May migrate. Doctor / glance must stay `open_read_intent` (T192). Do **not** call `nightly::run`. |
| T204 Daily group already shipped | `help_ia.rs:11` exact: `Daily:     recall, preflight, doctor, project, pin, memory, context, stop-session, daemon`. Unit `:57–61` + `memory_list_inventory.rs:612–616` lock the string. Adding `status` **must** update those. |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| Commands enum | `main.rs:1626` | No top-level `Status`. Doctor `display_order = 12` `:1808`. Daemon `17`. Nightly `26`. |
| Doctor early dispatch | `main.rs:4454–4482` | **Before** AppContext. `Commands::Doctor { .. } => unreachable!` `:4551`. **Copy this class.** |
| `build_report` / `format_doctor_summary` | `doctor.rs:77` / `:932` | `pub` / `pub(crate)`. Summary first line `doctor: status={}  vault={}  ok={} warn={} fail={} skip={}`. **Reuse verbatim.** |
| 15-check matrix | `doctor.rs:1065` | Frozen. `daemon_reachable` index 4; `graph_density` index 10. |
| `DoctorOptions` | `doctor.rs:34` | Glance constructs this (`full: false`, `fail_on_degraded: false`, no kit). |
| `exit_code_for` | `doctor.rs:922` | **Do not call** from status. |
| Nightly status branch | `nightly.rs:40–222` | AppContext + optional 750 ms `LlamaCppProvider`. `--quick` sets `probe=skipped`. **Do not call `run`.** |
| `NightlyStatusJson` | `nightly_status.rs:88` | Full nightly JSON. Glance does **not** embed this object. |
| `fetch_schedule_snapshot` | `nightly.rs:1067` | **private**. `SchtasksListV` already `pub(crate)` `:911`. **F34:** `pub(crate)` snapshot + fetch (visibility only). |
| `get_last_nightly_run` | `query_store.rs:474` | `get_sync_state("last_nightly_run")`. Glance may `SELECT value FROM sync_state WHERE key = ?` on `open_read_intent` (no QueryStore / no AppContext). |
| `graph_health_report` | `commands/graph.rs:429` | **private**, feature-gated. **Do not import `commands/graph.rs`.** |
| `gather_density_snapshot` / `assess_graph_density` | `crates/ai-brains-cli/src/graph_density.rs` (`crate::graph_density`) | Shared by doctor + graph update. **Glance graph uses this.** Floors `MIN_EDGE_NODE_RATIO=0.50` `:14`. Agy m2: **not** `commands/graph_density.rs`. |
| `format_ratio` / `counts_suffix` | `graph_density.rs:121–136` | **private.** Glance one-liner may duplicate `{ratio:.3}` or `pub(crate)` `format_ratio` (one-word). Prefer a `pub(crate)` one-liner helper in `graph_density.rs` rather than growing doctor. |
| Daemon `run_status` | `daemon.rs:739` | IPC + vault section + TCP + PID + T297 tail. **Do not call.** Glance copies Running/Stopped **words** only. |
| `status_next_line` | `daemon.rs:697–701` | `pub(crate)`; Stopped → `Some("next: ai-brains daemon start")`. T249 unit `:861`. **F27:** glance human **calls this**; do not twin the literal. JSON `next_step` is prefix-less (`ai-brains daemon start`) — separate const; do not mangle the helper. |
| Probe policies | `daemon_probe.rs:12–34` | Status 1×300 ms; Safety 3×1000 ms. Glance = **Status**. |
| Format resolver | `format_resolve.rs:8` | `resolve_human_json_format` — **reuse** (T255 already does). |
| Help IA | `help_ia.rs:11` / tests `:57–61` | Daily string lock. |
| Line counts | **Nonblank** (plan-write `Measure-Object -Line`): doctor **1738**; nightly **1968**; nightly_status **834**; daemon **1232**; `commands/graph.rs` **1606**; `src/graph_density.rs` **668**; main **5402**; `governed_common` **1029**; project **1342**; sync **578**. **Physical** (OpenCode m1 `rg -c`): **1855 / 2128 / 895 / 1341 / 1731 / 732 / 5578 / 1133 / 1483 / 644**. Snapshot only — **F32 80-net is phase diff vs go HEAD**, not these numbers. |
| `mod.rs` | `commands/mod.rs` | No `status` module. |
| Contracts `DoctorReport` | `ai-brains-contracts` `doctor.rs` | schema_version 1; 15 checks. **Do not embed** in glance JSON. |

### 2.4 Dependency / standards research (2026-08-29)

| Pin | Workspace / lock | Action |
|-----|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** / crates.io **4.6.6** (2026-08-06) | **No bump.** New `Commands::Status` + `--format` tokens. clap 5 **forbidden** (`unstable-v5` exists; max stable 4.6.6). |
| `serde_json` | lock **1.0.150** | **No bump.** CLI-local struct. |
| `tokio` | workspace **1.53** / lock **1.53.1** | **No bump.** Async only for the IPC probe. |
| `rusqlite` | exact **0.40.2** | **No bump.** |
| `uuid` | ws `"1.13"` / lock **1.23.1** | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | Unchanged. |
| workspace version | **0.1.3** | **No bump.** |
| New crates | — | **Zero.** |

**CLI compose research (primary sources):**

| Source | What we take | What we decline |
|--------|----------------|-----------------|
| [clig.dev](https://clig.dev/) (fetched 2026-08-29) | Human-first + JSON `--json`/`--format json`; stdout machine / stderr messaging; examples in help; suggest next command; catch/rewrite errors; changing human output usually OK; keep JSON keys stable; “make it easy to see current state” (`git status` analog); timeout long probes | Fancy progress bars; minikube-like encoded exits; `--watch` |
| [minikube status](https://minikube.sigs.k8s.io/docs/commands/status/) | Named components (host/kubelet/apiserver); `--output json` | **Bitwise exit** (1+2+4) — conflicts with CLI-EXIT-CODES **0–7** freeze and T199 daemon status exit **0** both states |
| Docker Compose `ps` / git `status` | List independent parts; exit 0 is a report | N/A |
| kubectl | No generic `kubectl status` | Do not invent `get`/`describe` here |
| 12-factor CLI / T180 P-CLI | Additive optional JSON keys; pretty vs compact is a contract | Do not silently TTY-flip a surface that already defaulted JSON (this command is **new** → Family **A** auto is OK) |

N/A-if-skipped: Windows `schtasks` LIST /V already implemented (T255 F34). Reuse; do not re-research the English `Status:` parser.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — Command name** | Top-level `ai-brains status` = `Commands::Status { format }`. Does **not** rename or steal `daemon status` / `harness status` / `device status` / `replicate status`. No `visible_alias` (`health` is doctor). |
| **F2 — In-process** | Call existing helpers. **Never** `Command::new("ai-brains")` / PATH subprocess (PATH-behind would lie). |
| **F3 — Four sections** | Human + JSON always name **daemon**, **doctor**, **graph**, **nightly** (even when a section is `error`). Order frozen: daemon → doctor → graph → nightly → optional `next:`. |
| **F4 — Fail-open** | Catch section `Err` → that section `{ "error": "<display>" }` (skip other keys). Other sections still emit. Never `?` a section to abort the command (except missing vault_path — F35). |
| **F5 — Exit 0** | Successful emit exits **0** when doctor roll-up is fail, daemon Stopped, graph sparse, nightly never. Clap usage / unknown `--format` → **2**. Do **not** call `doctor::exit_code_for`. No `--fail-on-degraded`. |
| **F6 — Doctor 15 freeze** | Do not add a 16th check. Do not grow `doctor.rs` production. `full: false` (integrity stays skip). Reuse `build_report` + `format_doctor_summary`. |
| **F7 — No HTTP probes** | Do **not** construct `LlamaCppProvider` / `probe_health`. T255 750 ms budget stays on `nightly --status` only. No `--quick` on glance (nothing to skip). |
| **F8 — No daemon TCP** | Do not copy `run_status` TCP/PID/vault-size/T297 contrast. Glance daemon is IPC Running/Stopped only. |
| **F9 — Probe policy** | `probe_daemon_reachable(..., DaemonProbePolicy::Status)` (1×300 ms). **Not** doctor Safety `probe_restore_daemon_busy`. Pass the bool into `build_report` as `daemon_up`. |
| **F10 — Doctor section** | Human: print `format_doctor_summary` **verbatim** (already starts `doctor:`). JSON: slim extract — `status` (`ok`\|`degraded`\|`fail`), `ok`/`warn`/`fail`/`skip` counts, `vault_path`, `attention` (warn+fail `{name,severity,message,remediation?}`). **Not** full `DoctorReport.checks`. |
| **F11 — Graph section** | `gather_density_snapshot` + `assess_graph_density` in `status.rs` (or a `pub(crate)` one-liner in `crate::graph_density`). **Do not import `commands/graph.rs`.** Feature-off still runs SQL. Human: `graph: {status}  nodes={} edges={} E/N={:.3} pinned={}`. JSON: `status`, `density`, `edge_node_ratio`, `nodes`, `edges` (no `note` / `remediation` — those stay doctor attention / `graph update`). Tables missing → `error`, not clap 2. |
| **F12 — Nightly section** | Vault `last_nightly_run` (`sync_state`) + Windows `fetch_schedule_snapshot("AI-Brains-Nightly")`. **`scheduled` mapper identity (T255 Nightly JSON, live `nightly.rs:104`):** Windows `snap.next_run.is_some()` → `Some(bool)`; non-Windows `None`. Do **not** use Router `found` (T255 F34). `found=false` implies default snap / `next_run=None` so `scheduled=false`. `last_task_result` from snap. Human: `nightly: last={rfc3339\|never}  scheduled={Yes\|No\|unknown}  last_result={code\|n/a}`. **No** Router, multi-import, unsummarized, Completion/Embedding. Non-Windows: `scheduled` JSON `null`, human `unknown` / `n/a`. |
| **F13 — JSON envelope** | CLI-local in `status.rs`. `schema_version: 1`. Pretty `to_string_pretty`. **No** `ai-brains-contracts` type. Optional `error` / `next_step` / `remediation` use `skip_serializing_if`. |
| **F14 — Format default auto** | New surface → Family **A**: clap default `auto`; TTY human; pipe json. Tokens `auto\|pretty\|human\|text\|json\|markdown\|md` via `format_resolve::resolve_human_json_format`. `JSON` / `Pretty` → clap `InvalidValue` exit **2**. |
| **F15 — clap flags** | `--format` only. **No** `--json`, `--summary`, `--quick`, `--full`, `--fail-on-degraded`, `--kit-path`. |
| **F16 — display_order** | `12` (same as doctor; clap lists Doctor then Status alphabetically). |
| **F17 — Help IA** | Daily string becomes `Daily:     recall, preflight, doctor, status, project, pin, memory, context, stop-session, daemon`. Start-here additive `ai-brains status` after `doctor --summary`. Update `help_ia.rs` unit + `memory_list_inventory.rs` Daily assert. CAPABILITIES Daily line adds `status`. |
| **F18 — Capture independence** | Read-only. `open_read_intent` only. No events. No migrate. No models. |
| **F19 — Pins / crates** | No clap 5, no lock bumps, no new crates, workspace **0.1.3**. |
| **F20 — Standing declines** | T263 H2; T240 F2 silent `.env`; T308 floors; T307 Blocked; T310 F15 `--version`; csrf; density retune. |
| **F21 — T204** | Additive `status` in Daily only. Do **not** regroup Operator/Governed or restyle `--help`. |
| **F22 — PATH-behind** | Live PATH lacks T312–T319 **and** T320. Hermetic / `cargo run` SoT. Do not `cargo install`. |
| **F23 — Cross-model** | FEATURE (new operator JSON). After Phase-1 review clean, run read-only `codex-review`. |
| **F24 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F25 — Stop-before** | Even after go: do not `daemon start/stop`, do not `graph rebuild`, do not schtasks /change, do not write `.cmd`, unless the owner explicitly confirms that remediating action. |
| **F26 — after_help** | Required on `Commands::Status`: examples `ai-brains status` + `ai-brains status --format json`; “does not replace `doctor` / `nightly --status` / `daemon status` / `graph update`”; “never starts the daemon; never rebuilds the graph; no HTTP probes.” |
| **F27 — next:** | Human last line + JSON `next_step` **only** when daemon Stopped. Human = `daemon::status_next_line(false)` (`Some("next: ai-brains daemon start")` — T249 SoT; do **not** twin the literal). JSON `next_step` is the **prefix-less** string `ai-brains daemon start` (separate const; do not strip/`format!` the helper into both forms). Running → omit. Do **not** pile nightly/graph nexts. |
| **F28 — PowerShell** | `;` not `&&`. |
| **F29 — Isolation** | New `commands/status.rs` + `mod status`. Do not grow `project.rs` / `sync.rs` / `governed_common.rs` / `doctor.rs` / `daemon.rs` / `commands/graph.rs`. `nightly.rs` **visibility-only** (`pub(crate)` fetch). Importing `status_next_line` does **not** grow `daemon.rs`. `main.rs` clap + early dispatch + unreachable arm. `help_ia.rs` Daily/Start-here. |
| **F30 — Dispatch** | Handle `Commands::Status` **before** `AppContext::from_cli` (same block as doctor / daemon status). Match arm `unreachable!("status handled before AppContext")`. |
| **F31 — minikube exits** | **Decline** bitwise / non-zero for degraded. CLI-EXIT-CODES 0–7 freeze. Glance is a report (T199 class). |
| **F32 — Line-count 80-net** | `status.rs` new (no cap). Other production files: net &lt;80 **physical lines vs go HEAD** (not the §2.3 snapshot). `nightly.rs` visibility-only must stay ~0 net behavior. |
| **F33 — Format helper** | Reuse `format_resolve::resolve_human_json_format`. Do **not** fork a third resolver. |
| **F34 — schtasks reuse** | `pub(crate) fn fetch_schedule_snapshot` + `pub(crate) struct ScheduleSnapshot`. Do **not** reimplement LIST /V. Do **not** change nightly `--status` chrome. |
| **F35 — Vault path** | Required like doctor (`--vault-path` / `AI_BRAINS_VAULT_PATH`). Missing → same doctor error path (not a new clap required on the subcommand). Daemon-only without vault remains `daemon status`. |
| **F36 — Floors** | `MIN_EDGE_NODE_RATIO=0.50` frozen. Sparse glance is honest. No remediator on graph-on Sparse (T308). |
| **F37 — Nightly chrome freeze** | `=== Nightly Status ===` / Router / `--quick` / default human-pipes stay on `nightly --status`. Glance must **not** print that banner. |
| **F38 — Doctor summary freeze** | `format_doctor_summary` text + 15 names stay. Glance does not reformat attention lines. |
| **F39 — Daemon status freeze** | `run_status` human (TCP, PID, T297 `≠`, `next:`) unchanged. Glance is a **different command**. |
| **F40 — Graph update freeze** | Default pretty JSON / `--format human` labeled lines unchanged. Glance one-liner is not `graph update`. |
| **F41 — JSON pretty** | `to_string_pretty` (doctor / nightly-status class), not compact recall. |
| **F42 — Envelope keys** | Frozen §5.1. Adding a required key later is a new track. |
| **F43 — last-PR Cursor** | `#236` / `#235` empty. `#230` → **T325**. **No T326.** |
| **F44 — Two vault opens** | `build_report` opens internally; glance may `open_read_intent` once more for nightly SQL + density. Documented; do not grow doctor to return the conn. |
| **F45 — Tests** | Naming `function_or_feature__condition__expected_result`. No `unwrap`/`expect`/`panic` in production. No `std::env::set_var` in tests. Hermetic AC9 **must not** assert `daemon.state` (host IPC 300 ms — OpenCode O1); assert envelope structure + doctor/graph/nightly keys only. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: `resolve_human_json_format("auto", true) == "human"`; `("auto", false) == "json"` already green in `format_resolve` — **stay-green**. New: clap `status --format` uses that resolver (pipe default json). |
| **AC2** | Clap: `status --format xml` → `InvalidValue` exit **2**. `status --format JSON` → `InvalidValue` exit **2**. `daemon status` still parses (stay-green). |
| **AC3** | Unit: JSON envelope from a fixture contains **every** frozen key in §5.1: `schema_version`==1, `daemon`, `doctor`, `graph`, `nightly`. `next_step` omitted when Running. |
| **AC4** | Unit: doctor `Err` fixture → `doctor.error` nonempty; `daemon.state` still present. |
| **AC5** | Unit: graph tables-missing / gather fail → `graph.error`; other sections present. |
| **AC6** | Unit: nightly never + Windows snap with `next_run=None` → human `last=never` `scheduled=No`; JSON `last_run: null`, `scheduled: false`. Mapper identity: **`snap.next_run.is_some()`** (`nightly.rs:104`); `None` when `!cfg!(windows)`. `found=false` is a sufficient fixture (default snap has no next run) — do **not** re-derive `found &&`. |
| **AC7** | Unit: daemon Stopped → human last line **equals** `daemon::status_next_line(false)` (`Some("next: ai-brains daemon start")`); JSON `next_step` exact `ai-brains daemon start` (prefix-less const). Running → neither. Names must **not** collide with `daemon.rs` `status_next_line__stopped__daemon_start`. |
| **AC8** | Unit: `format_status_human` includes verbatim `format_doctor_summary` output and does **not** contain `=== Nightly Status ===` / `LLM backend` / `probe=`. |
| **AC9** | Hermetic: `ai-brains status --format json` on a temp vault exits **0**; stdout one JSON object; keys `schema_version`/`daemon`/`doctor`/`graph`/`nightly`. **Do not** assert `daemon.state` Running/Stopped (host IPC; OpenCode O1 / F45). **Do not** assert live E/N / live last_run. Assert `daemon` is an object with `state` **or** `error`. |
| **AC10** | Hermetic: default `status` on a TTY (or `--format human`) prints `daemon:` and `doctor:` lines. Piped `--format auto` is JSON (no `daemon: Running` human). |
| **AC11** | Manual (source bin): `cargo run -q -p ai-brains-cli -- status` shows daemon Running (this host), doctor degraded graph_density, graph sparse E/N ~0.42, nightly last 2026-08-28 (pass-with-observed-data). Exit 0. PATH-behind **not** a fail. |
| **AC12** | Docs: CAPABILITIES Daily + format-matrix row; PROTOCOL-COMPAT P-CLI **new** `status` row; OPERATIONS example; CLI-EXIT-CODES glance footnote (exit 0); CHANGELOG Unreleased; `after_help` F26. |
| **AC13** | Isolation: `git diff` product — `doctor.rs` / `commands/graph.rs` / `daemon.rs` `run_status` / `project.rs` / `sync.rs` / `governed_common.rs` / contracts **empty** of behavior. `nightly.rs` visibility-only. `daemon.rs` may show **zero** net if glance only **imports** `status_next_line`. |
| **AC14** | Stay-green: `health_check_order_names__fixed_matrix` len 15; T255 nightly heading unit; T249 `status_next_line__stopped__daemon_start`; T308 sparse omits remediation; T204 `root_after_long_help__contains_setup_and_stop_session` **updated** Daily string. |
| **AC15** | Graph-off: `cargo run -p ai-brains-cli -- status --format json` (no `--features graph`) exits **0** and includes `graph` (SQL or `error`) — **not** `FEATURE_UNAVAILABLE` exit 2. |
| **AC16** | Unit: human graph one-liner uses `E/N=` with three decimals (`0.416` class), not the raw f64 from `graph update`. JSON `edge_node_ratio` is the raw f64. |
| **AC17** | Help: `status --help` contains F26 examples and “does not replace”. Root `--help` Daily inventory contains `status` (AC14). |

---

## 5. Design notes

### 5.1 JSON shape (frozen, `schema_version: 1`)

```json
{
  "schema_version": 1,
  "daemon": { "state": "Running" },
  "doctor": {
    "status": "degraded",
    "vault_path": "C:\\dev\\ai-brains\\vault.db",
    "ok": 12,
    "warn": 1,
    "fail": 0,
    "skip": 2,
    "attention": [
      {
        "name": "graph_density",
        "severity": "warn",
        "message": "sparse: edge/node ratio below typed-lineage floor 0.5 (nodes=64356 edges=26740 E/N=0.416 pinned=51267 memory_nodes=40394)"
      }
    ]
  },
  "graph": {
    "status": "sparse",
    "density": "warn",
    "edge_node_ratio": 0.41550127416247123,
    "nodes": 64356,
    "edges": 26740
  },
  "nightly": {
    "last_run": "2026-08-28T07:08:30.025803200+00:00",
    "scheduled": true,
    "last_task_result": "0"
  }
}
```

- `next_step` omitted when daemon Running (not `null`).
- Section `error` (string) replaces that section’s other keys when the section failed to build.
- `attention[].remediation` skip when None (graph-on Sparse has none).
- `nightly.scheduled` is JSON `null` on non-Windows (F12 / T255 F32 class).
- `nightly.last_run` is JSON `null` when never (human prints `never`).
- Do **not** nest `DoctorReport`, `NightlyStatusJson`, or `GraphHealthOutput`.

### 5.2 Human layout (frozen)

```text
daemon: Running
doctor: status=degraded  vault=C:\dev\ai-brains\vault.db  ok=12 warn=1 fail=0 skip=2
attention:
  [warn] graph_density — sparse: edge/node ratio below typed-lineage floor 0.5 (nodes=64356 edges=26740 E/N=0.416 pinned=51267 memory_nodes=40394)
graph: sparse  nodes=64356 edges=26740 E/N=0.416 pinned=51267
nightly: last=2026-08-28T07:08:30.025803200+00:00  scheduled=Yes  last_result=0
```

When Stopped, append:

```text
next: ai-brains daemon start
```

No `===` banner (avoids grepping as nightly). Doctor block is `format_doctor_summary` including its trailing newline; glance concatenates sections with `\n`.

When doctor `No issues.`, still print graph + nightly (graph may still be `live` / `skip`).

### 5.3 Execution order

```text
1. IPC Status probe (300 ms) → daemon.state
2. build_report(DoctorOptions{ full:false, fail_on_degraded:false, … }, daemon_up)
3. open_read_intent → last_nightly_run SQL + gather_density_snapshot + assess
4. Windows fetch_schedule_snapshot("AI-Brains-Nightly")
5. emit human or JSON
```

Steps 2–4 independently fail-open (F4 / F44).

### 5.4 Why not subprocess / why not doctor-only

Subprocess would invoke PATH-behind 0.1.3 and lose T319 source. Doctor `--summary` already exists and **hides** Stopped daemon + **omits** nightly. Glance is the named-ok aggregator; doctor remains the 15-check deep dive.

### 5.5 Capture independence

No `MemoryPinned`, no nightly events, no probe persistence. Status is a query. Second `open_read_intent` is read-only. schtasks LIST /V is read-only (T255 F7 class).

---

## 6. Non-goals

- Replacing `doctor` / `nightly --status` / `daemon status` / `graph update`
- Doctor 16th check or `DoctorReport` growth
- HTTP `/health` 750 ms or daemon TCP retries on the glance
- `--fail-on-degraded` / `--full` / `--quick` / `--json` alias / `--watch`
- Starting the daemon; graph rebuild; schtasks mutate; `nightly-run.cmd`
- `ai-brainsd --version` (T310 F15)
- Harness / device / replicate / whoami / preflight in the envelope
- Unsummarized session count / Router / multi-import
- minikube bitwise exits
- clap 5 / pin bumps / new crates / contracts DTO
- T316 memory-list / T318 backup-list / T321 safety-sync / T322–T324 / T325 F8 recency
- T204 full regroup
- `cargo install` as planning or as Complete-blocking
- Printing `AI_BRAINS_KEY`

---

## 7. Verification plan

```powershell
# Red → green
cargo nextest run -p ai-brains-cli --lib status
cargo nextest run -p ai-brains-cli -E "test(status)"
cargo nextest run -p ai-brains-cli --lib help_ia
cargo clippy -p ai-brains-cli --all-targets -- -D warnings

# Manual (source bin — PATH is pre-T320)
cargo run -q -p ai-brains-cli -- status
cargo run -q -p ai-brains-cli -- status --format json
cargo run -q -p ai-brains-cli -- status --format human
# graph-off:
cargo run -q -p ai-brains-cli -- status --format json
# Do not daemon start/stop / graph rebuild / schtasks /change

# Stay-green
cargo nextest run -p ai-brains-cli -E "test(health_check_order_names) or test(status_next_line) or test(graph_health_output__sparse)"

# Full gate
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace ; cargo deny check ; cargo audit
ledgerful verify --scope full
```

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Operators think `status` replaced `daemon status` | F1/F26 after_help “does not replace”; glance omits TCP/PID |
| `status` vs nested `daemon status` clap confusion | AC2 stay-green parse; different parents |
| Doctor Safety vs glance Status probe disagreement | F9 documented; glance matches `daemon status` latency |
| Double vault open cost | F44; COUNT + one SELECT; no HTTP |
| Growing doctor.rs | F6/F29 sibling only |
| Feature-off exit 2 | F11/AC15 — no `graph.rs` import |
| T204 Daily unit red | F17/AC14 update both locks |
| Pipe default JSON surprises humans who grep | F14 Family A; `--format human` opt-in; after_help |
| minikube-style non-zero | F5/F31 |
| PATH-behind false fail | F22 `cargo run` |
| Live E/N / last_run volatility | AC9/AC11 observed-data; hermetic keys only |
| `nightly.rs` behavior drift | F34 visibility-only; F37 stay-green heading |

---

## 9. Deferred absorb/decline

| Item | Disposition |
|------|-------------|
| Audit opportunity (b) unified status | **Absorb** F1–F15 / AC3–AC12 |
| Placeholder compose vs subprocess | **Absorb** F2 in-process |
| Placeholder name vs `daemon status` | **Absorb** F1 |
| Placeholder fail-open / 750 ms | **Absorb** F4 / F7 |
| T310 F15 `ai-brainsd --version` | **Decline** F20 |
| T192 doctor 15 / T249 summary | **Affirm** F6 / F38 — reuse, do not replace |
| T199/T297 daemon status | **Affirm** F8 / F39 — do not call `run_status` |
| T255 nightly JSON / Router / 750 ms | **Affirm** F7 / F12 / F37 — last-run only |
| T308 Sparse remediator None / floors 0.50 | **Affirm** F36 |
| T204 Daily string lock | **Partial** F17 — additive `status` only |
| T263 H2 / T240 F2 / clap 5 | **Decline** F20 |
| T316 memory list / T318 backup / T321 safety | **Not stolen** |
| T322 / T323 / T324 | **Not stolen** |
| T325 F8 PreferRecency (`#230`) | **Not stolen** |
| T307 Blocked / T308 floor retune | **Not stolen** / **Decline** |
| last-PR Cursor `#236` / `#235` | **N/A empty** — no T326 (F43) |
| last-PR `#230` F8 recency | **T325** already Pending |
| minikube bitwise exit | **Decline** F31 |
| Doctor `--json` embed in glance | **Decline** F10 slim extract |
| Unsummarized / Router on glance | **Decline** F12 |
| `recovery_kit_event` doctor warn | **Decline** this series (doctor Q=9; still in 15 if warn) |
| DOCS TX | `dcb67912-8fb7-4bbd-a354-68ba41857744` |

---

## 10. Implement order (on go)

1. Phase 0 re-verify src + deferred + clap lock + last-PR + FEATURE TX  
2. Red AC2 clap + AC3 envelope keys + AC4/AC5 fail-open + AC7 next + AC8 no nightly banner  
3. Green `status.rs` compose + `main.rs` clap/dispatch + `mod.rs` + F34 nightly visibility + F17 help_ia  
4. Stay-green AC1/AC14/AC15 graph-off  
5. Docs AC12  
6. Manual AC11 (no live mutate) → review → cross-model F23 → full gate → Complete  

Suggested series order after this plan: **T320 go** (daily glance) or **T325** (F8 recency). Then T316 / T318 / T321. T307 stays Blocked.

---

## 11. Soft residuals (expected post-close)

| Residual | Note |
|----------|------|
| PATH until owner `cargo install` | F22 — not Complete-blocking |
| Live E/N ~0.42 still sparse | F36 honest; not a glance fail |
| Doctor Safety vs glance Status probe | F9 by design |
| Two vault opens | F44 — not a conn-sharing track |
| T204 still a long `--help` dump | F21 additive only |
| Glance omits unsummarized / Router / TCP | F12/F8 — use the dedicated commands |
| T325 F8 PreferRecency | Not stolen |
| `recovery_kit_event` warn when it fires | Still doctor’s 15; glance attention will show it |

---

## 12. Touch map (expected)

| Site | Change |
|------|--------|
| **new** `crates/ai-brains-cli/src/commands/status.rs` | Compose, JSON types, human join, units |
| `commands/mod.rs` | `pub mod status;` |
| `main.rs` | `Commands::Status { format }`; early dispatch; unreachable arm; `after_help` F26 |
| `help_ia.rs` | Daily + Start-here; update Daily unit |
| `tests/memory_list_inventory.rs` | Daily string assert |
| `nightly.rs` | `pub(crate)` `ScheduleSnapshot` + `fetch_schedule_snapshot` only |
| `crates/ai-brains-cli/src/graph_density.rs` | Optional `pub(crate)` one-liner / `format_ratio` — **or** format in `status.rs` |
| `Docs/CAPABILITIES.md` | Daily line + format-matrix row + honesty bullet |
| `Docs/PROTOCOL-COMPAT.md` | P-CLI `status` row (pretty JSON; keys §5.1) |
| `Docs/OPERATIONS.md` | Glance example; does not replace doctor |
| `Docs/CLI-EXIT-CODES.md` | `status` exit **0** footnote |
| `CHANGELOG.md` | T320 Unreleased |
| `conductor/conductor.md` | Planned → (on go) In Progress → Completed |
| doctor.rs / `commands/graph.rs` / daemon.rs `run_status` / contracts / retrieval | **No behavior** (`status_next_line` import only) |
| events / store / projector | **None** |

---

## 13. AI fold-in disposition (2026-08-29)

Source: `agy-review.md` + `opencode-review.md` (HEAD `e15188e`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.**

### Agy

| ID | Verdict | Action |
|----|---------|--------|
| **m1** HEAD `464edc2` vs `e15188e` | **Agree** | Snapshot fold-in `e15188e` / ahead **1**; product `src/` = `464edc2` (Agy m1 class) |
| **m2** `graph_density.rs` not under `commands/` | **Agree** | §2.3 / F11 / touch map: `crates/ai-brains-cli/src/graph_density.rs` via `main.rs:9` `mod graph_density` |
| **O1** in-process compose | **Already** | F2 / F7 / F8 |
| **O2** fail-open | **Already** | F4 / AC4 / AC5 |
| **O3** graph-off SQL | **Already** | F11 / AC15 |

### OpenCode

| ID | Verdict | Action |
|----|---------|--------|
| **m1** line-count row stale / `src/graph.rs` | **Partial** | Dual-count §2.3: plan-write was **nonblank** (`Measure-Object -Line`); OpenCode **physical**. Crate paths. **Decline** “pre-T317 / `src/graph.rs`” — **1606** is nonblank of live `commands/graph.rs` (phys **1731**). F32 restated: 80-net vs **go HEAD**. |
| **O1** AC9 host IPC daemon | **Agree** | AC9 / F45 — do not assert `daemon.state`; object with `state` or `error` |
| **O2** reuse `status_next_line` | **Agree** | F27 / AC7 — human = helper; JSON prefix-less const; unique test names |
| **O3** AC6 scheduled mapper | **Agree (live mapper)** | F12 / AC6 — **`snap.next_run.is_some()`** (`nightly.rs:104`), **not** Router `found &&`. `None` when `!cfg!(windows)` |

### Pins locked by fold-in

1. **§2.3 paths:** `crate::graph_density` = `src/graph_density.rs`; graph health = `commands/graph.rs:429`.
2. **F32:** 80-net is phase diff vs **go HEAD**, not the snapshot.
3. **F27/AC7:** human `next:` = `daemon::status_next_line(false)`; JSON `next_step` prefix-less.
4. **F12/AC6:** Nightly `scheduled` = `next_run.is_some()` (T255 Nightly JSON).
5. **AC9/F45:** hermetic must not assert host `daemon.state`.
6. **last-PR Cursor:** `#236` / `#235` empty — **no T326**. `#230` → **T325**.

**Planning + fold-in 2026-08-29.** Still **plan-only until go**.
