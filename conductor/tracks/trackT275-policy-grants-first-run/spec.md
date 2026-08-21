# T275 — Discovery grants must unlock briefing/progressive (or stop looking empty)

- **Track ID:** T275-PolicyGrantsFirstRun
- **Status:** **Planned** (Pending until **go**)
- **Category:** FEATURE / UX / GOVERNED
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — `briefing project` **8/3**, `query progressive` **8/3**, `evidence`/`source`/`review list` **6/3**, `policy check` **7/7**, `briefing personal` **5/7**. Placeholder minted with T274–T284 (`deabae7`).
- **Depends on:** T210 ✅ `policy bootstrap`; T221 ✅ deny exit 3 + progressive-after-bootstrap; T241 ✅ doctor `policy_grants`; T226 ✅ soft-resolve; T227 ✅ briefing format; T263 ✅ H1 granted-empty honesty (**H2 declined**)
- **Blocks / feeds:** After hermetic (or owner-confirmed) bootstrap, project briefing is not a grant wall. Denied human no longer reads as “the vault has no decisions.” Daily `recall` stays grant-independent (T274). Hint `--scope` omit is **T280**. Leftover rebind **T276**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “briefing/progressive/lists POLICY_DENIED (0 of 3 grants)”; T241 F21 skill one-liner (docs); T210 gap that CLI `policy bootstrap` is not locked to `briefing project` / `evidence list`
- **Not absorbed (DoD):** Auto-grant on `init` / first `preflight` (T210 F13 / T241 F10); `preflight --install-grants` (T241 F4/F20); T263 H2 pin→Approved; T280 hint omit-`--scope`; T276 leftover `7d97a456`; T284 #188 Work/samples; T279 Safety; clap 5 / rusqlite 0.40 / DTO keys; doctor 16th check; daemon IssueGrant
- **Research date:** 2026-08-21 (plan dogfood HEAD `8cb1ce0` T274 `#189`; product `src/` = T274)
- **AI fold-in:** none yet (plan-only)
- **Ledger:** planning DOCS TX `e13a4e01-3dd6-4adc-ae57-be75e7e98ba9`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`, rewrite `.env`, bootstrap the live operator vault unless the owner confirms at **go**, rebind leftover paths, live `retention apply --confirm`, or mutate schtasks. Do **not** grow hotspot `project.rs` / CLI `preflight.rs` / `doctor.rs` / `governed_common.rs`. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** change `POLICY_DENIED_HINT` (T243 AC12 / T280). Do **not** flip briefing deny to exit 3 (T210 F28).

---

## 1. Objective

1. **Denied briefing is a grant wall, not an empty vault.** `briefing project --format human` with `denied: true` must not print `## Decisions (current authority)` / `## Conclusions` as `_None_`. That copy trains agents that there are no decisions next to ~3k pins. Keep bootstrap as the primary next-step (T263: Repository deny still bootstraps). Add one grant-wall line that names `recall` / `search` for ungoverned pins.
2. **CLI bootstrap unlocks the governed read path.** Hermetic `policy bootstrap` (System principal — omit `--principal-id`, T221 F31 trap) then `briefing project` is `denied: false` (empty_authority + recall is OK) and `evidence list` exits **0** (items may be `[]`). T210 already locks `policy check` + `source`/`review` list; T221 already locks progressive `denied: false`; T263 already locks granted-empty via *in-process* `issue_grant`. This track locks the **operator command** to briefing + evidence.
3. **Do not silent-allow.** Deny-by-default stays. No auto-grant on `init` or first `preflight`. Mutation stays `policy bootstrap` (`--dry-run` first). Live operator vault: `--dry-run` only unless the owner confirms at go.
4. **North star.** Capture independence: ungoverned `recall` / `search` / preflight never require grants. Grants append `PrincipalRegistered` / `ScopeGrantIssued` only (existing T210 path). No hidden CoT. Agents that ignore doctor `0 of 3` still cannot misread Denied as “no decisions.”

This unblocks the daily product: T210/T241 made bootstrap *possible* and *discoverable*; T263 made granted-empty *honest*; the 2026-08-21 audit still scores 8/3 because the live Scope has **0 of 3** grants and Denied human looks empty.

---

## 2. Live baseline (re-scan 2026-08-21)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `8cb1ce0` — T274 squash `#189` (`feat(retrieval): T274 pins beat harness session dumps`). Tree **CLEAN**. In sync with `origin/main`. Product `src/` = T274. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-21 05:55**, 25 368 576 bytes, **0.1.1**. **T270** on PATH (before T274 11:52Z merge). Grants surfaces are T241-era — **PATH-behind for T274 ranking, not for this hole.** **Do not `cargo install`.** |
| Source debug | `target\debug\ai-brains.exe` mtime **2026-08-21 07:46**. Tests/manual AC use `cargo run` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **3325**. In-context **0/0/0**. **`discovery grants empty (0 of 3)`** + short SOOT bootstrap. Capture independence holds. |
| `policy bootstrap --dry-run` | `registered: already`. Three **`would_issue`**: ReadConclusions, ReadDecisions, ReadEvidence. Principal `a1b2a1b2-…` (well-known System). Scope `Repository:3581317d-…`. **Path exists; daily never applied.** |
| `policy show` | `grants: []` + `next_step` short SOOT (no `--scope`). Exit 0. |
| `policy check --capability ReadEvidence` | Exit **3** `POLICY_DENIED` + `details.hint` still `bootstrap --scope …` (**T280**). |
| `doctor --summary` | `policy_grants` **warn** `discovery grants empty (0 of 3)` + **long** SOOT (omit `--scope` when authoritative). Also backup_recent / recovery_kit_event / graph_density. status=degraded. Matrix 15. |
| `briefing project --format human` | `> **Denied:** ReadDecisions/ReadConclusions…` then `BRIEFING_DENIED_NEXT_STEP` (`bootstrap --scope …`). Then **`## Decisions (current authority)` `_None_`** and **`## Conclusions` `_None_`**. Exit **0**. **This is the 8/3 hole.** |
| `briefing project --format json` | `denied: true`, `decisions: []`, `conclusions: []`, `denial_hint` = short SOOT (no `--scope`). JSON is parseable; human `_None_` is the lie. |
| `evidence` / `source` / `review` list `--format json` | Exit **3** `POLICY_DENIED` + bootstrap `--scope …` hint. |
| `query progressive "what did we decide"` | Exit **3**, `denied: true`, `denial_hint` bootstrap + T243 recall fallback. Packet honest. Still dead until grants. |
| `briefing personal --format human` | Denied + T263 Personal recall next (not Personal bootstrap). Unused on this machine. |
| Last GitHub PR | [#189](https://github.com/Ryan-AI-Studios/AI-Brains/pull/189) T274 (2026-08-21). `gh pr view --comments`, `/reviews`, `/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, actions). **No leftover to mint.** Prior #188 Bugbot Mediums remain **T284**. |
| Identity / doctor | Summary Scope `3581317d`. ledgerful doctor 5 warn (legacy `.changeguard` / sig-pin / timings / :8081). **0 pending / 0 drift.** Hotspot **#1** `project.rs` (3.990). `governed_common.rs` **#5** (2.651). CLI `preflight.rs` **#7**. `personal.rs` **#6**. |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why this still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| Daily 0 of 3 grants | T210 mutation + T241 discoverability shipped. Doctor/preflight/`policy show` already name bootstrap. Operators and planning agents are **forbidden** to live-bootstrap (T263/T274 isolation). The remediator is printed, not executed. **DoD is not silent grant** — it is (1) Denied human cannot look empty, (2) hermetic CLI bootstrap → briefing/evidence unlock. Live apply is owner-confirm at go. |
| Denied `_None_` | T227 F8 / renderer F8: empty_authority footer is **suppressed** when denied (unit `render_project_markdown__denied__bootstrap_next_step_no_empty_authority`). The **section bodies still print `_None_`**. Agents read Decisions `_None_` as “no decisions.” **DoD.** |
| T210 after-bootstrap tests | `policy_bootstrap.rs` AC3 checks + AC4 **source + review** list. **No** `briefing project`. **No** `evidence list`. T221 AC3 = progressive after System bootstrap. T263 AC4 = granted-empty briefing after **in-process** `issue_grant` (not CLI bootstrap). Operator path untested. **DoD.** |
| Auto-grant / `--install-grants` | T210 F13 / T241 F4/F10/F20. Least privilege + deny-default (Entra / Orca 2026 / OSO 2025): start from zero, add Read* only via explicit bootstrap. Putting a flag on preflight does not put mutation on the *default* path unless we auto-run it. **Decline as DoD.** |
| Hint `--scope …` | Doctor long SOOT omits `--scope`; deny `details.hint` and markdown next still use ellipsis. **T280.** Do not steal. |
| T263 H2 | Pins are not Approved. Filling briefing authority requires propose+approve. **Decline.** |
| Briefing deny exit 0 | T210 F28 / T221 F7. JSON already has `denied: true`. Do **not** flip to exit 3. |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Bootstrap SOOT | `policy_cmd.rs` `run_bootstrap` **`:234–358`** (file **356**) | Discovery trio `DISCOVERY_CAPS`; `active_grants` + `get_principal`; `--dry-run`; CLI-local JSON `api_version: "1"`. **Call only — do not reimplement.** |
| Principal | `briefing.rs` `cli_principal()` **`:198–212`** | Env UUID → Human; else System `0xA1B2…`. Bootstrap default is the same via `resolve_principal(None)`. T210 hermetic `--principal-id bbbb…` ≠ briefing — **F31 trap; omit `--principal-id` in T275 hermetic.** |
| Denied project packet | `control-plane/.../briefings/project.rs` **`:216–228`** | `empty_denied` + `BRIEFING_DENIED_DENIAL_HINT` (short SOOT). |
| Human render | `renderer.rs` **`:56–130`** (**497** lines) | Denied blockquote + `BRIEFING_DENIED_NEXT_STEP` (`--scope …`). Then **unconditional** `_None_` for empty decisions/conclusions (`:93–115`). Empty-authority footer gated `!packet.denied` (`:125–130`). **Grant-wall consts + `_None_` branch live here.** |
| Personal deny | `renderer.rs` `BRIEFING_PERSONAL_DENIED_*` | T263 F4 — recall, not Personal bootstrap. **Do not restyle as project bootstrap.** Personal `_None_` is not the audit 8/3; optional same hidden placeholder if the helper is shared. |
| Progressive deny | `governed_query.rs` + `query.rs` | Exit **3** + `POLICY_DENIED_HINT` + T243 recall fallback. **Leave.** T221 AC3 already locks after-bootstrap. |
| Lists deny | T203 / T263 AC8 | Exit **3** + bootstrap hint; no authorized-empty `next_step`. **Leave.** Evidence after CLI bootstrap is the new lock. |
| Doctor `policy_grants` | `doctor.rs` `check_policy_grants` **`:657–708`** (**1738** lines) | 15-check matrix. Warn `<3`. **Do not grow. Do not add `--fix`.** |
| Preflight grants line | CLI `preflight.rs` **`:118` / `:911`** (**2027**, hotspot **#7**) | Post-hoc short SOOT. **Do not grow. No `--install-grants`.** |
| Dual-site hint | `governed_common.rs` `POLICY_DENIED_HINT` **`:51`** (hotspot **#5**) + daemon `services.rs` **`:989`** + `query.rs` **`:93`** | T243 unit `policy_denied_hint__wording__unchanged`. **T280 owns omit-scope. Do not edit.** |
| Short / long SOOT | `POLICY_BOOTSTRAP_SOOT_SHORT` / `_LONG` `:107–111` | Show / preflight / JSON denial_hint vs doctor. |
| Contracts | `ProjectBriefingPacket` `denied` / `denial_hint` / `decisions` | E1: denied JSON keeps `decisions: []` (not null). **No new keys.** |
| `project.rs` | hotspot **#1** (3.990) | **Do not touch.** |
| clap Bootstrap | `main.rs` `PolicyCommands::Bootstrap` **`:2207`** | `--scope` optional, `--dry-run`/`-n`, `principal_id` env, format default json. **No new flags.** |

### 2.4 Dependency / standards research (2026-08-21)

**Snapshot — re-verify at execute.**

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** (2026-08-06). GitHub latest tag **v4.6.6**. **No clap 5.** | **No bump.** No new flags. |
| `serde_json` | lock **1.0.150** | — | **No bump.** JSON keys frozen. |
| `chrono` | workspace **0.4** / lock **0.4.44** | crates.io **0.4.45** (Dependabot #62) | **No bump.** |
| `rusqlite` | lock **0.39.0** | crates.io **0.40.2** (Dependabot #61; T213 L4) | **No bump.** Grants use existing CP projections. |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged |
| nextest | **0.9.140** | — | Unchanged |
| workspace | **0.1.1** | — | **No bump** |
| New crates | — | — | **Zero.** No OSO/Cedar/OPA. Existing `DefaultPolicyEvaluator`. |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| First-run setup then real work; suggest next command; dry-run before state change; change state → tell the user | [clig.dev](https://clig.dev/) (current) | Keep `--dry-run` then `policy bootstrap`. Denied human must say what happened (grant wall) and what to run next (bootstrap). Do not silent-allow. |
| Ease of discovery vs remember-and-type | clig.dev § Ease of discovery | Doctor/preflight already discover. Remaining failure is **misread empty** + untested operator path, not a missing mutation API. |
| Default-deny; start identities from zero; add narrow allows | [Microsoft Entra least privilege](https://learn.microsoft.com/en-us/entra/id-governance/scenarios/least-privileged) (2025-04); [Orca PoLP 2026-07](https://orca.security/resources/blog/cloud-least-privilege-principles-best-practices/); [OSO RBAC 2025](https://www.osohq.com/learn/rbac-best-practices) | **No auto-init grant.** Bootstrap issues exactly ReadEvidence / ReadConclusions / ReadDecisions (T210 F2). Never Approve/Erase/Export/Propose. |
| Over-permissioning for convenience | OSO 2025 “new user got admin because the last person had it” | Decline `preflight --install-grants` auto and any standing write/approve grant. |
| clap 4 current | [docs.rs/clap/4.6.6](https://docs.rs/clap/4.6.6/clap/) tutorial; crates.io 4.6.6 | Reuse existing Bootstrap parser. `after_help` already names omit-`--scope`. |
| Dual-site string freeze | Live `policy_denied_hint__wording__unchanged` | Hint rewrite = **T280**. |

**N/A:** SQLCipher page crypto, schtasks, T180 2-key DTO, Windows service, llama.cpp `/health`, FTS5 ranking (T274 closed).

**Could not verify:** live briefing after a *confirmed* operator bootstrap (stop-before). Hermetic CLI bootstrap is the proof. T241 F25 live sequence was never applied to this vault.

**ledgerful / ai-brains:** `preflight --summary` 0 of 3 grants @ **3325** pins; `policy bootstrap --dry-run` `would_issue` ×3; `ledgerful ledger status --compact` 0 pending / 0 drift; `search "run_bootstrap"` → `policy_cmd.rs:234`; `search "empty_denied"` → `project.rs:217` + contracts; `ask "what calls issue_grant"` includes `run_bootstrap` + T263 `seed_discovery_grants`. Recall lexical still surfaces T263 review-track Objective dumps (PATH-behind T274). Semantic: no hits above threshold.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `e13a4e01`. Implement starts a **FEATURE** TX. |
| **F1 — Grant-wall human (required)** | When `packet.denied` on **project** markdown: do **not** emit `_None_` under Decisions or Conclusions. Emit a hidden-until-grants placeholder (const). Keep `> **Denied:**` + `BRIEFING_DENIED_NEXT_STEP` (bootstrap primary). |
| **F2 — Grant-wall sentence** | One additional line, **≤140** chars, after the bootstrap next-step (T263 F29 budget). Must contain `recall` (and `search` if it still fits). Must **not** replace bootstrap. Suggested const `BRIEFING_DENIED_GRANT_WALL`: `This is a grant wall, not an empty vault. Pins remain via \`ai-brains recall\` / \`search\`.` |
| **F3 — JSON E1 freeze** | Denied JSON keeps `denied: true`, `decisions: []`, `conclusions: []` (not null, not omitted). `denial_hint` stays T241 short SOOT. **No** new DTO keys (`grant_wall`, `is_empty`, …). |
| **F4 — Soft deny exit** | Briefing denied stays exit **0** (T210 F28). Progressive / lists stay exit **3**. |
| **F5 — CLI bootstrap → briefing** | Hermetic: `init` → `policy bootstrap --scope Repository:<uuid> --format json` **without** `--principal-id` (System = `cli_principal`) → `briefing project --project-id <uuid> --format json` has `denied: false`. Human has **no** `**Denied:**`. empty_authority + recall is OK (T263 AC4 already). |
| **F6 — CLI bootstrap → evidence** | Same vault: `evidence list --scope Repository:<uuid> --format json --local` (or project-context equivalent) exits **0**; `items` may be `[]`. T210 AC4 source/review stay green. |
| **F7 — Do not reimplement bootstrap** | Call existing `run_bootstrap` / CP `register_principal` + `issue_grant`. Discovery trio + `Privacy::LocalOnly` + idempotent `active_grants` **unchanged**. |
| **F8 — No auto-grant** | Reaffirm T210 F13 / T241 F10. `init` does not issue grants. First `preflight` / `doctor` / `briefing` does not append grant events. |
| **F9 — No `--install-grants` / doctor `--fix`** | Reaffirm T241 F4/F20. Mutation stays `policy bootstrap`. Soft residual only. |
| **F10 — Live vault stop-before** | Plan-only: `--dry-run` only. On **go**, do **not** bootstrap the operator vault unless the owner confirms in the go prompt. Hermetic is sufficient DoD. |
| **F11 — T280 isolation** | Do **not** edit `POLICY_DENIED_HINT`, daemon twin, `query.rs` twin, or `BRIEFING_DENIED_NEXT_STEP` `--scope …` wording. Grant-wall is a **new** const. |
| **F12 — T263 isolation** | Granted-empty next stays `BRIEFING_EMPTY_AUTHORITY_NEXT_STEP` (recall). Personal deny stays recall, not Personal bootstrap. **H2 declined.** Do not scrape pins into authority. |
| **F13 — T221 isolation** | Progressive deny exit 3 + T243 recall fallback stay. After-bootstrap progressive AC3 stays green — do not retune. |
| **F14 — Capture independence** | Recall / search / ungoverned preflight never require grants. Grant-wall copy must not imply capture is blocked. |
| **F15 — Least privilege** | Bootstrap still refuses Propose*/Approve*/Export/Erase. No `policy grant` admin (T210 F24). |
| **F16 — Domain in CLI** | Forbidden beyond renderer strings + hermetic. No evaluator change. |
| **F17 — Pins / crates** | No clap 5, no rusqlite 0.40, no chrono 0.4.45, no new crates, workspace **0.1.1**. |
| **F18 — PATH** | Do not `cargo install` unless the user asks. |
| **F19 — Contracts** | No new required keys. N−1 ignore extras still holds. `empty_denied` still leaves `denial_hint: None`; CP sets it. |
| **F20 — Tests** | Naming `function_or_feature__condition__expected_result`. No `unwrap`/`expect`/`panic` in production. Hermetic `tempfile::tempdir`. T210 `bbbb` principal **forbidden** on briefing AC (F31). |
| **F21 — File growth** | Grant-wall consts + `_None_` branch in **`renderer.rs`**. New hermetic tests in `policy_bootstrap.rs` (or thin sibling). Renderer unit next to existing denied test. **Do not** grow `project.rs`, CLI `preflight.rs`, `doctor.rs`, `governed_common.rs`, `sync.rs`. |
| **F22 — Existing tests stay green** | T210 AC1–AC5; T221 progressive deny + after-bootstrap; T241 doctor/show/check; T263 granted-empty + Personal deny + list AC8; T227 substance; `render_project_markdown__denied__bootstrap_next_step_no_empty_authority` (bootstrap next still precedes Decisions). |
| **F23 — Docs** | CAPABILITIES governed cold-start: Denied = grant wall; bootstrap then briefing; pins via recall. OPERATIONS sequence already exists — add grant-wall sentence. CHANGELOG minor. INSTALL post-init bootstrap stays. Skill one-liner (T241 F21 absorb) if the in-repo skill mentions briefing. |
| **F24 — last-PR Cursor** | #189 comments/reviews **empty** → N/A. #188 two Mediums stay **T284**. **No T285.** Open HEAD PR: none (Dependabot remotes). |
| **F25 — Decline peers** | T276 leftover; T277 backup; T278 graph; T279 Safety; T280 hint; T281 nightly probe; T282 context leftover; T283 list cwd-first; T284 Work/samples; T240 F2; T255 750 ms; T263 H2; T266 JSON freeze. |
| **F26 — Cross-model** | FEATURE. After Phase-1 clean, run read-only `codex-review` (governed honesty + grant path). |
| **F27 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F28 — PowerShell** | `;` not `&&`. |
| **F29 — Determinism** | Placeholder const frozen; renderer line order: Denied → bootstrap next → grant-wall → (optional blank) → Decisions hidden → Conclusions hidden. Grant-wall before Decisions so preflight word budget keeps it (T227 F29 analog). |
| **F30 — Partial grants** | 1 or 2 of 3: doctor still warns (T241 F31). Briefing may still deny if ReadDecisions **and** ReadConclusions missing (`project.rs` both-false). Bootstrap idempotent fills. **Not** a new evaluator. |
| **F31 — Principal trap** | T275 hermetic briefing/evidence **must** use System default (omit `--principal-id`), matching `cli_principal()`. Do not copy T210 `bbbbbbbb-…` Human unless also passing the same `--principal-id` / env to briefing (not DoD). |
| **F32 — Personal `_None_`** | Not audit 8/3. If the hidden-placeholder helper is shared, personal denied may use it; Personal next-step stays T263 recall. **Not required** to change Personal if a separate branch is cleaner. |
| **F33 — No interactive prompt** | Reaffirm T210 F34. |
| **F34 — Event sourcing** | No raw grant SQL. No revoke in DoD. No compensating “delete grant” — out of scope. |

---

## 4. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | Denied project markdown does **not** contain `_None_` under Decisions/Conclusions; contains grant-wall const with `recall`; still contains `**Denied:**` + `policy bootstrap` | Renderer unit (required red) |
| **AC2** | Grant-wall line is ≤140 chars, one line, after bootstrap next, before `## Decisions` | Unit |
| **AC3** | Denied JSON: `denied: true`, `decisions`/`conclusions` are `[]` (not null), `denial_hint` contains `policy bootstrap`; exit 0 | Existing T221 AC10 + lock (or thin hermetic) |
| **AC4** | CLI System `policy bootstrap` then `briefing project` JSON `denied: false`; human has no `**Denied:**` | Hermetic (required) |
| **AC5** | Same vault: `evidence list` JSON exit **0** (`items` array; empty OK) | Hermetic |
| **AC6** | Allowed empty (T263) still emits `_None_` **or** empty_authority notice — grant-wall is **denied-only** | Unit + T263 AC4 stays green |
| **AC7** | Progressive deny still exit **3** + bootstrap (T221); after System bootstrap progressive `denied: false` (T221 AC3) | Regression |
| **AC8** | Denied evidence/source/review still exit **3** (T263 AC8) until bootstrap | Regression |
| **AC9** | Dangerous caps still denied after discovery bootstrap (T210 AC) | Regression |
| **AC10** | Capture independence: recall without grants still works (no new grant requirement) | Review / existing smoke |
| **AC11** | Docs + CHANGELOG; CAPABILITIES grant-wall sentence | Grep |
| **AC12** | No production `unwrap`/`expect`/`panic`; no clap/rusqlite bump; no DTO keys | Review / lock diff |
| **AC13** | `POLICY_DENIED_HINT` byte-equal to T243 unit (T280 not stolen) | Existing unit |
| **AC14** | Doctor matrix still **15**; `policy_grants` still warn on 0 of 3 | Existing doctor unit |
| **AC15** | Live operator vault: this planning pass ran **`--dry-run` only**. Implement live bootstrap only if owner confirms | Manual |

---

## 5. Design notes

### 5.1 Renderer branch (F1/F2)

Today:

```text
Denied blockquote
BRIEFING_DENIED_NEXT_STEP
## Decisions
_None_
## Conclusions
_None_
# empty_authority footer skipped because denied
```

Target:

```text
Denied blockquote
BRIEFING_DENIED_NEXT_STEP
BRIEFING_DENIED_GRANT_WALL
## Decisions (current authority)
_(hidden until discovery grants)_
## Conclusions (current authority)
_(hidden until discovery grants)_
```

Placeholder const name frozen in implement (`BRIEFING_DENIED_HIDDEN` or inline the F2 wall line only — **must not** use `_None_`). Prefer a dedicated hidden const so AC6 allowed-empty `_None_` stays.

Do **not** omit the Decisions/Conclusions headings (preflight/agents scan headings). Replace the body only.

### 5.2 Why not auto-bootstrap

T151 deny-default + T210 F13 + Entra/Orca default-deny. A single-user Windows vault still must opt in: grants are durable event-log writes (LocalOnly Read* is small, but silent write on `preflight` would surprise). Discoverability already exists (T241). T275 makes misread impossible and locks the operator command.

### 5.3 Hermetic shape (F5/F6/F31)

Copy T221 `progressive__after_system_bootstrap__exit_0_denied_false`: `--no-project-context`, `--vault-path`, `init`, `policy bootstrap --scope Repository:<uuid> --format json` (no `--principal-id`), then briefing with `AI_BRAINS_PROJECT_ID` or `--project-id` matching that uuid. Evidence list: `--scope` + `--local` like T210 AC4.

### 5.4 `search` / daemon / sync

Ungoverned. No change. Daemon `POLICY_DENIED_HINT` twin stays (T280).

---

## 6. Non-goals

- Auto-grant on `init` / first preflight / doctor.
- `preflight --install-grants` or `doctor --fix` (T241 F20).
- Flip briefing deny to exit 3.
- T263 H2 pin→Approved / `migrate governed`.
- T280 omit-`--scope` hint unification.
- T276 leftover rebind; T284 Work/samples; T279 Safety.
- Full grant admin / revoke CLI / daemon IssueGrant.
- New DTO keys; clap 5; rusqlite 0.40; new crates.
- Growing `project.rs` / CLI `preflight.rs` / `doctor.rs` / `governed_common.rs`.
- `cargo install`; live `.env`; live vault bootstrap without owner confirm.

---

## 7. Verification plan (TDD)

**Phase 1 red (required before green):** AC1 (renderer `_None_` / grant-wall). AC4/AC5 may already be green as *locks* — still write them first; if they pass on HEAD, keep as regression and do not weaken.

Then green: renderer consts + denied `_None_` branch.

**Stay green:** AC6–AC14, T210, T221, T241, T263, T227.

Targeted: `cargo nextest run -p ai-brains-control-plane --lib` renderer filter + `-p ai-brains-cli --test policy_bootstrap` + `--test governed_first_run_deny_exit` + `--test governed_vault_pin_honesty` + `cargo clippy -p ai-brains-control-plane -p ai-brains-cli --all-targets -- -D warnings`.

Full workspace gate only at implement closeout — **not** a plan gate.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Agents still never run bootstrap | Grant-wall + recall line; live apply is owner-confirm; not silent allow |
| Grant-wall steals T280 | New const only; F11 |
| F5 already green | Still required AC; T210 never asserted briefing |
| T210 `bbbb` principal copied | F31 / AC4 omit `--principal-id` |
| Preflight word budget drops grant-wall | F29 order before Decisions (T227 F29) |
| Allowed-empty loses `_None_` | AC6; denied-only branch |
| Hotspot growth | F21 renderer only |
| Owner later wants live grants | F10 — confirm at go; not this planning pass |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| briefing/progressive/lists POLICY_DENIED (0 of 3) | **Absorb** F1–F6 / AC1–AC5 |
| `policy bootstrap --dry-run` would_issue ×3 but daily still deny | **Partial** — hermetic unlock DoD; live apply **F10** owner-confirm |
| T241 F21 skill one-liner | **Absorb** F23 docs |
| T241 F20 `preflight --install-grants` | **Decline F9** (reaffirm T241 F4) |
| T241 F22 bootstrap success soft-resolve hermetic | **Partial** — AC4 uses explicit `--scope` (T221 shape); omit-scope success stays T226 soft |
| T210 residual full admin / revoke | **Decline F15** |
| T210 auto-init | **Decline F8** |
| T263 H1 granted-empty | **Affirm F12** — do not restyle |
| T263 H2 pin→Approved | **Decline F12 / F25** |
| T263 daily 0 of 3 “do not live-bootstrap” | **Lift F10** — this track may, only if owner confirms at go |
| Preflight Safety = Objective | **Decline F25 → T279** |
| leftover `7d97a456` | **Decline F25 → T276** |
| deny hint `--scope …` vs doctor omit | **Decline F11 → T280** |
| #188 Work / apply samples | **Decline F24 → T284** |
| last-PR Cursor #189 | **N/A** — comments/reviews empty |
| T240 F2 / T255 750 ms / clap 5 / rusqlite 0.40 / DTO | **Decline F17 / F25** |
| Identity mismatch `7d97a456` vs `fcb8a40f` | **Not this track** — T258 adopt-path; leftover data T276 |
| Historical CE wipe, MSI, `anyhow` allowlist, archive `.changeguard` | **Decline** — not grants |

---

## 10. Implement order (on go)

1. Phase 0 re-verify renderer `:93–130`, `run_bootstrap`, `cli_principal`, T210/T221/T263 tests, deferred.md, #189 still empty.
2. Red AC1 renderer; add AC4/AC5 hermetic (lock if already green).
3. Green F1/F2 consts + denied body branch in `renderer.rs`.
4. Docs F23. Do not touch hint twins.
5. Targeted nextest + clippy. Review log. Cross-model F26.
6. FEATURE TX commit. implement-track Phase 6 publish.

---

## 11. Soft residuals

| Residual | Disposition |
|----------|-------------|
| `preflight --install-grants` | F9 / T241 F20 — not DoD |
| Live operator bootstrap | F10 — owner confirm |
| T280 omit-scope on deny hint | Peer |
| Personal denied `_None_` | F32 optional |
| Daemon IssueGrant | T210 F25 |
| `policy grant` / revoke | T210 F24/F26 |
| PATH until `cargo install` | F18 |
| T226 omit-scope bootstrap success hermetic | T241 F22 / T226 O1 |
| Doctor 16th / `--fix` | Decline |

---

## 12. Touch map

| Area | Files (indicative) |
|------|-------------------|
| Renderer | `crates/ai-brains-control-plane/src/briefings/renderer.rs` — consts + denied `_None_` branch + unit AC1/AC2/AC6 |
| Hermetic | `crates/ai-brains-cli/tests/policy_bootstrap.rs` — AC4/AC5 (System principal) |
| Docs | `Docs/CAPABILITIES.md`, `Docs/OPERATIONS.md`, `CHANGELOG.md` |
| Conductor | this spec/plan; `conductor.md`; `deferred.md`; series README |
| **Do not touch** | `project.rs`, CLI `preflight.rs`, `doctor.rs`, `governed_common.rs` `POLICY_DENIED_HINT`, `services.rs` hint twin, `query.rs` hint twin, `DefaultPolicyEvaluator`, clap Bootstrap flags |

---

## 13. AI fold-in

None yet. Plan-review files land as `agy-review.md` / `opencode-review.md` after `/review-track`.
