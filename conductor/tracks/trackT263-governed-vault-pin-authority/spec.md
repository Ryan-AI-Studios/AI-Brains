# T263 — Governed surface: honesty + vault-pin authority

- **Track ID:** T263-GovernedVaultPinAuthority
- **Status:** **Planned** (Pending in registry; plan-only until go)
- **Category:** FEATURE / UX
- **Owner:** —
- **Source:** Audit 2026-08-16 — governed empty; briefing **4/6** + **3/6**; `query progressive` **3/5**; expand **6/6**; trace **5/4**; evidence/source/review **3/5**; opportunity “connect governed to vault pins or stop advertising”
- **Depends on:** T152/T160/T203/T210/T221/T227/T241/T243 (surface exists; grants discoverable; progressive empty already names `recall`)
- **Blocks / feeds:** Daily “what did we decide” stays `recall` / `search`. Governed briefing/progressive stay *Approved / Active-Confirmed* only. Preflight leftover blender stays **T264**. Format maze stays **T266**. Harness/whoami/list next stays **T267**.
- **Absorbs:** Audit T263 row (3 grants / 0 authority); T227 empty_authority next that still says “seed an Approved decision”; Personal denied bootstrap-as-required; expand `Unknown` empty preview; evidence/source/review authorized-empty `items: []` with no remediator
- **Not absorbed:** Policy bootstrap mutation (T210/T241 closed); H2 live pin→DecisionProposed; silent Approve; `AI_BRAINS_GOVERNED_BRIEFING` on production preflight (T170); T167/T168 live migrate; Scope default (T258); leftover `--global` (T264); format maze (T266); T240 F2; T255 declines
- **Research date:** 2026-08-18 (HEAD `b2aae2d` T262 `#177`; live dogfood + crates.io + clig.dev + T167 classify + T180 P-CLI)
- **AI fold-in:** none yet (plan-track only)
- **Ledger:** planning DOCS TX on commit. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** run `policy bootstrap` on the live vault. Do **not** `migrate governed --confirm`. Do **not** enable `AI_BRAINS_GOVERNED_BRIEFING`. Do **not** `cargo install`. Do **not** write live `.env`. Do **not** reopen T240 F2 / T255 declines. Do **not** scrape `MemoryPinned` into briefing authority (T227 F3 stands).

---

## 1. Objective

The governed CLI must not look like the place to “ask what we decided” unless it can answer from **Approved decisions + Active/Confirmed conclusions**.

**Plan-time pick (frozen): H1 only.**

1. **H1 Honesty (DoD).** Help, CAPABILITIES, skill, empty briefing, authorized-empty progressive (already T243), authorized-empty discovery lists, and expand-unknown all name `recall` / `search` as the daily decision path. Progressive/briefing stay for *typed governed* rows only. Personal is optional continuity, not a required bootstrap.
2. **H2 Promotion (declined as DoD).** Do **not** classify live vault `DECISION:` / `CONSTRAINT:` pins into `DecisionProposed` / `ReviewItemOpened` in this track. T167 already under-promotes pins → **Evidence** (not decisions) and only on `migrate governed`. Auto-approve is forbidden. A dry-run proposal cannot fill `empty_authority`. There is no lossless pin→Approved mapping.

That advances the north star: capture stays grant-independent; the append-only log stays the SoT; pins remain `MemoryPinned` text; governed authority stays a separate, deny-by-default product. Two products, one binary — advertised honestly.

No models. No new crates. No clap 5. No live bootstrap / migrate as DoD.

---

## 2. Live baseline (2026-08-18)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `b2aae2d` — T262 Completed (`#177`). Tree **CLEAN**. `main` == `origin/main`. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` version **0.1.1**. **Do not `cargo install`.** Progressive deny on daily Scope printed packet + `POLICY_DENIED` (stderr mixed in capture); T221 contract is exit **3** — treat PATH-behind as out of scope. |
| `preflight --summary` | Scope path owner `3581317d` (`C:\dev\ai-brains`); **2952** pinned. `discovery grants empty (0 of 3); next: policy bootstrap`. T241 works on the **daily** Scope. |
| `policy show` (daily) | `grants: []` + `next_step` bootstrap. |
| `doctor --summary` | `policy_grants` **warn** `discovery grants empty (0 of 3)`. Other warns: backup_recent, recovery_kit, graph_density (T262 residual / T213 floor). |
| `briefing project --format human` (daily) | **Denied** ReadDecisions/ReadConclusions + T227/T241 bootstrap next. Exit **0** (soft deny). |
| `query progressive "why was graph backend replaced?"` (daily) | `denied: true` + `denial_hint` bootstrap + recall fallback (T243). |
| `evidence` / `source` / `review` list (daily) | **POLICY_DENIED** (0 grants). |
| `policy show --scope Repository:441837f6-5c55-d075-0000-000000000000` | **3 grants** (ReadEvidence / ReadConclusions / ReadDecisions) on leftover `test-alias`. This is the **2026-08-16 hole**. |
| `briefing project --project-id 441837f6-… --format human` | **Allowed.** Decisions _None_; Conclusions _None_; `_No current authority_`; **`next: seed an Approved decision and Active/Confirmed conclusion (propose + approve/activate)`**; warning `empty_authority`. Ledgerful degraded. **This is the product hole.** |
| `query progressive … --project-id 441837f6-…` | `denied: false`, `results: []`, `next_step`: `Ungoverned vault search: ai-brains recall "…"` (T243 already honest). |
| `evidence list --scope Repository:441837f6-…` | `items: []` (authorized empty). No remediator. |
| `source list --scope Repository:441837f6-…` | `items: []`. |
| `query expand 00000000-…` | `kind: Unknown`, `preview: ""`, exit **0**. |
| `query trace 00000000-…` | literal stdout `null`, exit **0** (`governed_query.rs` F31). |
| `briefing personal --format human` | Scope `Personal:a1b2a1b2-a1b2-a1b2-a1b2-a1b2a1b2a1b2` (System principal UUID mapped to `UserId`). Denied + **bootstrap** next. Personal is unused on this machine. |
| `memory list` (daily) | Recent pins include T70 symbol stubs + session text. Marker `DECISION:` pins exist in the vault (preflight in-context scrape + audit); default list is recency, not marker-first. |
| Last GitHub PR | [#177](https://github.com/Ryan-AI-Studios/AI-Brains/pull/177) T262. `gh pr view --comments`, `/reviews`, `/comments` all **empty**. No open PR on `main`. **last-PR Cursor: N/A.** |
| Ledgerful | `doctor` ready (legacy `.changeguard` / sig-pin / timings / :8081 unreachable; :8083 ok). 0 pending 0 drift. Hotspot **#1** `project.rs` (3.884) — **do not touch.** `#9` `briefings/personal.rs` (2.083) — prefer `renderer.rs`. |
| ai-brains recall | Lexical/semantic hits are recent session + symbols. No prior “H1 only / no live pin promotion” pin. |

### 2.2 Why this still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| 3 grants / 0 authority | T241 unblocked *grants*. It did not create *authority*. After bootstrap, briefing still says “seed an Approved decision” while hundreds of `DECISION:` pins live in the same vault. Agents follow the wrong product. **DoD = H1.** |
| Daily Scope 0 grants | T258 moved daily Scope to `3581317d`. T241 correctly warns. **Not T263 DoD** — do not steal bootstrap; do not run it on live. |
| H2 pin→proposal | T167 `classify_legacy` maps `MemoryPinned` → `ImportActionKind::Evidence` (`REASON_LEGACY_PIN`), **not** Decision. Decision rows come from typed `DecisionRecorded`. Filling briefing requires **Approved + Active/Confirmed**. Dry-run proposals do not. Auto-approve is forbidden. T170 stop-before live migrate. T227 F3: never scrape pins into authority. **Decline H2.** |
| Expand `preview: ""` | `kind: Unknown` + empty preview looks like a broken payload. Field exists — fill it. **DoD.** |
| Trace `null` | Documented T152 F31 + OPERATIONS empty-success. Wrapping as `{trace:null}` is **P-CLI breaking**. T180 compact-key freeze is preflight-specific; this scalar is still a frozen empty-success. **Decline wrap; document.** |
| Personal bootstrap next | Personal briefing is a shipped T152 surface. Default scope id **is** the System principal UUID (`briefing.rs` `UserId::from_uuid(principal.id)`). It is **not** the project vault. Leading with `policy bootstrap --scope Personal:a1b2…` trains operators to grant a product they do not use. **DoD = unused/optional + recall.** |
| Help tip still names `query progressive` | `help_ia.rs` Start-here already says `recall "what did we decide"`. The **Tip** line still exemplifies `query progressive`. **DoD.** |
| T243 progressive empty | Already names recall. **Leave.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Empty-authority next | `control-plane/src/briefings/renderer.rs` `BRIEFING_EMPTY_AUTHORITY_NEXT_STEP` | “seed an Approved decision…”. Shared with governed preflight (T227 F29). |
| Empty-authority notice | same `BRIEFING_EMPTY_AUTHORITY_NOTICE` | Keep. |
| Denied next | `BRIEFING_DENIED_NEXT_STEP` / `BRIEFING_DENIED_DENIAL_HINT` | **Keep for Repository deny** (T241). Specialize **Personal** deny. |
| Packet warning | `briefings/project.rs` kind `empty_authority` only when `!denied` | Keep kind; do not inject pins. |
| Personal default user | `cli/src/commands/briefing.rs` `run_personal` | `UserId::from_uuid(principal.id)` → `Personal:a1b2a1b2-…`. |
| Progressive hints | `governed_query.rs` `apply_progressive_search_hints` | Authorized empty → `PROGRESSIVE_RECALL_FALLBACK`. |
| Recall fallback SOOT | `governed_common.rs` `PROGRESSIVE_RECALL_FALLBACK` | `Ungoverned vault search: ai-brains recall "…"`. **Reuse.** |
| Trace null | `governed_query.rs` `run_trace` F31 | `println!("null")` on `None`. |
| Expand Unknown | `HandlePreviewDto.preview` | Empty string today. CLI adds `applied_scope`. |
| Evidence list | `EvidenceListResponse` (`items`, `more_available`, `warnings`) | **No** `next_step`. E1 `items: []`. |
| Source list | `SourceListResponse` | Same shape + `sources` alias. |
| Review list | `ReviewQueueResponse` via `review.rs` `run_list_local` | Authorized empty = empty `items`. |
| Help Start-here | `help_ia.rs` `ROOT_AFTER_LONG_HELP` | Already `recall` / `search`. Tip still progressive. |
| Query after_help | `main.rs` ~992 / ~1295 | Already “not vault FTS. Vault-first: recall / search.” |
| T167 pins | `legacy_import.rs` ~682 | `ImportActionKind::Evidence` + `REASON_LEGACY_PIN`. |
| T167 decisions | same ~1024 | Typed decision events → Decision + Review. Not pin text. |
| T170 D21 | OPERATIONS + deferred #44 | Never use `preflight --summary` as governed authority. |
| Hotspots | `project.rs` #1; `personal.rs` #9 | Do not touch `project.rs`. Prefer `renderer.rs` over `personal.rs`. |

### 2.4 Dependency / standards research (2026-08-18)

| Pin | Workspace / lock | Action |
|-----|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** / crates.io **4.6.6** (docs.rs 2026-08-11) | **No bump.** clap **5** not current. Snapshot — re-verify at execute. |
| `serde_json` | workspace **1.0** / lock **1.0.150** / crates.io **1.0.151** | **No bump.** |
| `thiserror` | workspace **2.0** / crates.io **2.0.20** | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | Unchanged. |
| workspace version | **0.1.1** | **No bump.** |
| New crates | — | **Zero.** |
| [clig.dev](https://clig.dev/) (fetched 2026-08-18) | First-run setup then real work; suggest the next command; conversation after setup | After grants exist, next command is **`recall`**, not “seed Approved”. Bootstrap stays the next command only while grants are empty (T241). |
| T180 P-CLI | `PROTOCOL-COMPAT.md` §5 / §9.3 | Freeze stable keys. Additive optional keys ignored by N−1. Scalar `null` empty-success is documented; do not replace with an object. |
| T227 F3 | Completed spec | **Do not** scrape legacy `MemoryPinned` into briefing authority. Dual-model honesty. |
| T167 L4 | under-promote | Pins → Evidence. Not a lossless decision mapping. |
| T170 D1 / D21 | stop-before live; never `--summary` for governed | Affirm. |

Training data is not a pin. Re-verify clap/serde_json at execute.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — H1 only** | This track is honesty. H2 live promotion is **out**. |
| **F2 — empty_authority next** | Change `BRIEFING_EMPTY_AUTHORITY_NEXT_STEP` so the **primary** remediator is `ai-brains recall` / `search`. Must contain `recall`. Must **not** lead with “seed an Approved decision”. Optional second sentence may mention propose+approve for *typed* authority only. |
| **F3 — Dual-model stands** | T227 F3: briefing authority sections stay Approved / Active-Confirmed only. **Never** inject pin text, pin counts, or `MemoryPinned` rows into `decisions[]` / `conclusions[]`. `empty_authority` kind stays. Notice string may stay. |
| **F4 — Personal deny next** | When `briefing personal` is `denied`, next-step / `denial_hint` must say Personal continuity is **optional / unused** and name `recall` for daily decisions. Must **not** lead with `policy bootstrap --scope Personal:…`. Allowed-empty `empty_continuity` next may stay (T227 F9) or add a recall clause — must not invent continuity (#18 stays deferred). |
| **F5 — Repository deny unchanged** | Project briefing / progressive / list **POLICY_DENIED** keep T221/T241 bootstrap. Exit codes unchanged (briefing soft **0**; progressive/expand Denied **3**; lists **3**). |
| **F6 — Trace `null` frozen** | `query trace` missing/unauthorized stays stdout `null` + exit **0**. Do **not** wrap. Strengthen clap `after_help` + CAPABILITIES/OPERATIONS one-liner. |
| **F7 — Expand Unknown preview** | When `kind == "Unknown"`, `preview` is a non-empty SOOT (e.g. `Handle not found.`). Exit **0** stays. No new JSON keys. `applied_scope` stays. |
| **F8 — Authorized-empty lists** | CLI `evidence list` / `source list` / `review list` when **not** denied and `items` empty: emit additive `next_step` = `PROGRESSIVE_RECALL_FALLBACK` (or that string + “vault pins are not governed evidence”). Prefer CLI overlay on the serialized value (T243 pattern) so daemon/HTTP DTOs need not change. If a field is added to a contracts DTO, it is `Option` + `skip_serializing_if` (T180 additive). Denied lists stay exit **3** with existing hint. |
| **F9 — Progressive empty** | Leave T243 `apply_progressive_search_hints`. No second SOOT. |
| **F10 — Help / docs / skill** | `ROOT_AFTER_LONG_HELP` Tip example becomes `recall` (not `query progressive`). CAPABILITIES §15 + WORKFLOWS “Find something”: one dual-model row — “what did we decide” → `recall`/`search`; briefing/progressive = Approved/Active only. Skill Phase 2 / governed section: same one-liner. Query `after_help` already honest — keep. |
| **F11 — Decline H2** | No live `classify_legacy` / `decision propose` from pins. No dry-run promoter CLI. No auto-approve. No `migrate governed` against the live vault. Promotion path remains T168 on an explicit dest. |
| **F12 — Decline GOVERNED_BRIEFING** | Do not set `AI_BRAINS_GOVERNED_BRIEFING` on production preflight. T170 D21: `--summary` is not governed authority. |
| **F13 — Decline live bootstrap** | Do not run `policy bootstrap` on the operator vault as implement/manual DoD. Hermetic tests may bootstrap **temp** vaults. |
| **F14 — Decline daily-Scope grant fill** | 0-of-3 on `3581317d` stays T241. T263 owns the **granted-empty** hole (`441837f6` live proof). |
| **F15 — Pins / crates** | No clap 5, no lock bumps, no new crates, workspace **0.1.1**. |
| **F16 — Contracts** | Expand preview = existing string. List `next_step` = CLI additive (or optional DTO). Trace scalar unchanged. PROTOCOL-COMPAT: one honesty bullet if a DTO key is added; otherwise CAPABILITIES/OPERATIONS only. CHANGELOG T263 row. |
| **F17 — Capture independence** | Copy + empty-state only. No models, embeddings, or graph required. No new events. |
| **F18 — Tests** | Naming `function_or_feature__condition__expected_result`. Units for renderer next-step + Personal deny; expand Unknown preview; list overlay; help tip. Update T227 renderer units that lock the old seed-Approved string. Hermetic: granted-empty briefing names `recall`; expand Unknown preview non-empty; list empty has `next_step`; denied project still bootstrap. No `unwrap`/`expect`/`panic` in production. |
| **F19 — Cross-model** | FEATURE (operator remediator + optional additive JSON). After Phase-1 review clean, run read-only `codex-review`. |
| **F20 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F21 — PATH-behind** | Do not `cargo install` unless the user asks. Tests/manual AC use `cargo run` / hermetic bin. |
| **F22 — Stop-before** | Even after go: no live bootstrap, no live migrate, no `.env` rewrite, no `nightly` mutate, no T240 F2 silent Scope switch. |
| **F23 — Hotspots** | Do not edit `project.rs`. Prefer `renderer.rs` + `governed_query.rs` + `governed_common.rs` + list emit helpers. Touch `personal.rs` only if renderer cannot specialize Personal deny. |
| **F24 — Soft vault pin count** | Optional later: SQL COUNT of pinned `DECISION:`/`CONSTRAINT:` prefixes on empty_authority. **Not DoD.** Must not enter authority arrays. |
| **F25 — Soft daemon/HTTP list next** | Same class as T243 F24. CLI is DoD. |
| **F26 — Soft wrap trace** | Only if a later track owns a P-CLI version bump. Not T263. |
| **F27 — `#18` continuity fill** | Stays deferred. No synthetic personal summary. |
| **F28 — T266 / T264 / T267** | Format maze, leftover `--global`, harness/whoami/list next — **not** this track. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: `BRIEFING_EMPTY_AUTHORITY_NEXT_STEP` contains `recall` and does **not** start with `seed an Approved`. Renderer allowed-empty markdown contains the new constant and **not** the old lead-in. |
| **AC2** | Unit: denied **project** markdown still contains `policy bootstrap` and does **not** contain `empty_authority` (T227 AC7 stands). |
| **AC3** | Unit: denied **personal** markdown names `recall` and does **not** lead with `policy bootstrap --scope Personal`. Exit still **0**. |
| **AC4** | Hermetic: granted-empty project briefing (`--format human` / markdown) prints `empty_authority` + recall next; JSON warning kind still `empty_authority`; `decisions`/`conclusions` stay `[]`. |
| **AC5** | Hermetic or unit: `query expand` unknown UUID → `kind: Unknown`, `preview` non-empty SOOT, exit **0**. |
| **AC6** | Hermetic: `query trace` unknown UUID → stdout is the JSON token `null` (byte-equal after trim), exit **0**. |
| **AC7** | Hermetic: authorized-empty `evidence list` / `source list` / `review list` JSON includes `next_step` containing `recall`; `items` stays `[]`. |
| **AC8** | Hermetic: denied list (no grants) still exit **3** + existing bootstrap hint; **no** `empty_authority` / no authorized-empty `next_step`. |
| **AC9** | Unit: `ROOT_AFTER_LONG_HELP` Tip does not exemplify `query progressive`; Start-here still has `recall "what did we decide"`. |
| **AC10** | Unit: T243 `apply_progressive_search_hints` authorized-empty still sets `next_step` with `recall` (regression). |
| **AC11** | Docs: CAPABILITIES §15 dual-model row + WORKFLOWS + skill one-liner + CHANGELOG T263. OPERATIONS empty-vs-deny mentions granted-empty → recall. |
| **AC12** | No new crate. No clap 5. No `unwrap`/`expect`/`panic` in production. `cargo clippy -p ai-brains-cli -p ai-brains-control-plane --all-targets -- -D warnings` clean on go. |
| **AC13** | Manual (source/hermetic bin, not PATH): leftover granted Scope briefing names recall; daily Scope deny still bootstrap; expand/trace as AC5/AC6. |

---

## 5. Design notes

### 5.1 Two products

| Product | Corpus | How you fill it |
|---------|--------|-----------------|
| Vault pins | `MemoryPinned` text (`DECISION:` / `CONSTRAINT:`) | `ai-brains pin` / harness ingest |
| Governed authority | Approved decisions + Active/Confirmed conclusions | `decision propose` + review/approve; or T168 `migrate governed` on a dest |

Briefing and `query progressive` read **only** the second table. T241 grants unlock the read. They do not copy the first table into the second.

### 5.2 Shared renderer

`render_project_markdown` is consumed by CLI briefing **and** governed preflight (T227 F29). Changing `BRIEFING_EMPTY_AUTHORITY_NEXT_STEP` flows into both. That is intended: preflight must not keep “seed Approved” after T263.

Word-budget: deny next-step stays **before** `## Decisions`. Empty-authority next stays after the notice (current order). New string must stay short enough to survive T250 line-cap when it appears in preflight pretty.

### 5.3 Personal UUID

`Personal:a1b2a1b2-…` is not a mystery fixture. It is `cli_principal()` System id mapped to `UserId`. Do not remint. Do not treat it as the project Scope. Honesty copy may say “Personal scope defaults to the CLI System principal; it is not this repo.”

### 5.4 List `next_step`

Shared helper next to `apply_progressive_search_hints` (e.g. `apply_authorized_empty_list_next(value: &mut Value)`): if object has `items` array empty and no `denied`/error envelope, set `next_step` if absent. Call from evidence/source/review emit paths only.

### 5.5 Why H2 cannot close AC4

`empty_authority` fires when **both** authority sections are empty. T167 pin actions are Evidence. Evidence lists are not briefing decisions. A proposal is not Approved. The only way H2 fills briefing is silent Approve — forbidden.

---

## 6. Non-goals

- Live `policy bootstrap` / grant admin / revoke UI
- Live `migrate governed` / `classify_legacy` apply
- Auto-approve / inject pins into briefing
- Enable `AI_BRAINS_GOVERNED_BRIEFING` on preflight
- Wrap `query trace` `null`
- Vault pin COUNT overlay (F24 soft)
- Daemon/HTTP list `next_step` (F25 soft)
- `#18` personal continuity fill
- T264 leftover `--global` / T266 format / T267 harness next
- clap 5 / new crates / `project.rs` edits
- T240 F2 silent Scope switch; T255 declined bag

---

## 7. Verification plan (TDD)

Failing tests first (names):

1. `briefing_empty_authority_next_step__contains_recall_not_seed_approved`
2. `render_project_markdown__allowed_empty__names_recall`
3. `render_personal_markdown__denied__names_recall_not_personal_bootstrap` (new)
4. `expand_unknown__preview_nonempty`
5. `apply_authorized_empty_list_next__empty_items__sets_recall`
6. `root_after_long_help__tip_names_recall_not_progressive`
7. Hermetic: `briefing_format_substance` granted-empty / `governed_first_run_deny_exit` deny regression / new `governed_vault_pin_honesty` for expand + lists

Then green: renderer constants, expand preview SOOT, list overlay, help tip, docs.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| T227 units lock old next-step string | Update those units in the same red/green; keep deny AC2. |
| Preflight word budget drops new next | Keep string short; deny path order unchanged. |
| Agents still run progressive first | Help tip + skill + CAPABILITIES. Cannot fix unread docs. |
| Operator expects H2 | Spec §5.5 + CHANGELOG “pins are not authority.” |
| Contracts ripple | Prefer CLI overlay; no required DTO. |
| `personal.rs` hotspot | Renderer-only if possible. |
| PATH-behind | Hermetic/source bin for AC13. |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| Audit T263 (3 grants / 0 authority) | **Absorb** F1–F10 / AC1–AC13 |
| T227 empty_authority “seed Approved” | **Absorb** F2 / AC1 / AC4 |
| T227 F3 no pin inject | **Affirm** F3 |
| T227 F34 OutputFormat surface-wide | **Decline → T266** |
| T152-R1-08 / #18 continuity fill | **Decline** F27 |
| T241 F20 install-grants | **Decline** (bootstrap exists; not T263) |
| T241 F21 skill one-liner | **Partial** F10 skill dual-model |
| T241 F22 soft-resolve hermetic | **Decline** |
| T221 F32 `--principal-id` | **Decline** |
| T221 F18 daemon/HTTP progressive | **Decline** |
| T221 F36 trace `applied_policy` | **Decline** |
| T243 F24 daemon `next_step` | **Decline** F25 (same class for lists) |
| T210 full grant admin | **Decline** |
| T167/T168 pin classify | **Decline H2** F11; point at migrate dest |
| T170 live enable / `--summary` as authority | **Affirm** F12 |
| T262 closeout “T263 governed” | **This track** |
| T264 leftover `--global` | **Decline → T264** |
| T266 format maze | **Decline → T266** |
| T267 harness/whoami/list next | **Decline → T267** |
| T240 F2 / T255 declines | **Decline** (standing) |
| last-PR Cursor #177 | **N/A** — comments/reviews/inline empty; no leftover to mint |
| MSI / R-CI-BRANCH / anyhow allowlist | **N/A** — not this surface |
| cargo audit `anyhow` #5 | **N/A** |

---

## 10. Implement order (on go)

1. Phase 0 re-verify constants + leftover granted briefing + clap pins + deferred rescan.
2. Red: AC1–AC10 tests (expect fail on next-step / preview / tip).
3. Green: renderer + Personal deny + expand preview + list overlay + help tip.
4. Docs: CAPABILITIES / WORKFLOWS / OPERATIONS / skill / CHANGELOG.
5. Targeted clippy/nextest on cli + control-plane + hermetics.
6. Review loop + FEATURE codex-review.
7. Manual AC13 on source/hermetic bin.
8. Closeout: conductor Completed only after implement-track publish (not this pass).

---

## 11. Soft residuals

| Residual | Disposition |
|----------|-------------|
| Vault marker-pin COUNT on empty_authority | F24 |
| Daemon/HTTP list `next_step` | F25 |
| Wrap trace `null` | F26 |
| `#18` continuity | F27 |
| PATH `ai-brains` until `cargo install` | F21 |
| Daily Scope 0 grants until operator bootstrap | F14 / T241 |
| H2 / migrate live | F11 — never silent |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-control-plane/src/briefings/renderer.rs` | F2 / F4 constants + units |
| `crates/ai-brains-cli/src/commands/governed_query.rs` | F7 expand preview (or CP expand_handle) |
| `crates/ai-brains-cli/src/commands/governed_common.rs` | F8 helper; reuse fallback |
| `crates/ai-brains-cli/src/commands/evidence.rs` / `source.rs` / `review.rs` | F8 overlay at emit |
| `crates/ai-brains-cli/src/help_ia.rs` | F10 tip |
| `crates/ai-brains-cli/tests/*` | AC4–AC10 hermetics |
| `Docs/CAPABILITIES.md` / `WORKFLOWS.md` / `OPERATIONS.md` / `CHANGELOG.md` | F10 / AC11 |
| `.claude/skills/ai-brains/SKILL.md` + onboarding recall blurb if it still oversells progressive | F10 |
| `conductor/conductor.md` / `deferred.md` / `README-T256-T271-CLI-AUDIT.md` | Planned row |

Do **not** touch: `project.rs`, `legacy_import.rs`, `policy bootstrap`, graph, nightly, `.env`.
