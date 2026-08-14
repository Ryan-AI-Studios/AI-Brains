# T248 — Retention plan human summary

- **Track ID:** T248-RetentionPlanHuman
- **Status:** ✅ **Completed** (2026-08-14 PR #161 `c633781`)
- **Category:** UX / FEATURE
- **Owner:** Grok
- **Source:** CLI audit 2026-08-11 P2 — `retention plan` **E7/Q7** JSON-only / empty classes thin
- **Depends on:** T166 class-based retention (plan/apply + `RetentionPlanReport`); T165 CE wipe honesty; T201/T227 format-exit patterns; T246 TTY/`auto` presentation SOOT
- **Blocks / feeds:** Operators can read the class matrix on a TTY without parsing JSON. T249 (scope/daemon/doctor) and T250 (preflight density) stay separate. Apply mutation stays T166.
- **Absorbs:** deferred.md “Retention plan human”; placeholder F1–F3; T166 §6.2 “json/human” human half (engine already shipped); README `retention plan` **7/7**
- **Not absorbed (DoD):** T166 planner/apply/daemon/scope/confirm; nightly CE; desktop retention UI; doctor retention check; HTTP `/v1/retention`; clap 5; contracts DTO / `api_version` bump; `OutputFormat::parse` silent-JSON (T227 F34); T249/T250
- **Research date:** 2026-08-14 (live dogfood + T166/T246 SOOT + CLIG + ISO 27001 A.8.10 + crates.io pins)
- **AI fold-in:** 2026-08-14 `C:\dev\AI-review.md` **T248** AI1 + AI2. No Highs. **Agree hard:** AI2 M1 `memory_legacy` zero-row is **`skip`** (not `soft_forget`; OPERATIONS overclaim → F14); AI2 L1 `next:` last after Errors; AI2 L2 pinned Totals line; AI2 L3 TempEnv on hermetics; AI2 L4 `--format` case-sensitive; AI2 L6 HORIZON **36**; AI2 L7 `format: String`. **Decline:** AI1 M1 `Some(other) => other` passthrough; AI1 remapped ACs. Disposition **§14**.
- **Ledger:** plan-only until go (`ledgerful ledger start T248-retention-plan-human --category FEATURE`)
- **Isolation:** Do **not** rewrite `plan_retention` / `build_report` / apply orchestration. Do **not** run live `retention apply --confirm`. Do **not** change JSON keys or honesty warning **strings**. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Make `retention plan` human on a TTY.** Default `auto`: scannable summary when stdout is a TTY; existing pretty JSON when piped or `--format json`.
2. **Show the schedule even when nothing is due.** Empty vault / zero candidates must print **Nothing to dispose.** plus the full 9-class horizon matrix — not a title, a zero totals line, and four raw warning dumps.
3. **Keep machine JSON stable.** `RetentionPlanReport` keys, `api_version: "1"`, empty `classes: []`, and honesty warning strings stay. Pretty fills missing class rows locally.
4. **Do not make apply quieter or sneakier.** `retention apply` stays default JSON (dangerous / scripted). Opt-in `--format human` reuses the same formatter with an apply title. Confirm / daemon / `--scope` gates unchanged.
5. **Stay capture-independent.** Presentation + existing read-only `plan_retention`. No models, no graph, no new events, no new crates, no pin bumps.

---

## 2. Live baseline (re-scan 2026-08-14)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| `retention plan` (no flags) | Pretty JSON wall. `classes: []`. `totals.candidates=0`. Horizons present for all 9 classes. Four honesty warnings. `errors_count=0`. Exit **0**. |
| `retention plan --format human` | Two data lines then four `!` warnings. **No** horizons. **No** class rows. **No** “nothing to dispose.” `samples={:?}` would be Debug if any class existed. |
| `generated_at` | RFC3339 with Windows nanos (`2026-08-14T03:12:41.076921700+00:00`) — unreadable in the human title |
| Apply | Still `--confirm` + JSON default. Not dogfooded (mutating). |
| Nightly | Separate one-line `eprintln!` totals; not this surface |
| Desktop | “Honest unavailable — class-based retention plan UI” (T172). Stays unavailable. |
| Doctor | No retention-plan check |

### 2.2 Why the audit scored 7/7

| Layer | Truth |
|-------|--------|
| Default | clap `default_value = "json"` on Plan **and** Apply. TTY operators always get JSON. |
| Human path | Exists (`OutputFormat::Human` / `Markdown` → `emit_report`) but is a debug dump: title + totals + `samples={:?}` + raw warning constants. |
| Empty classes | `build_report` only emits buckets that have candidates. Empty vault → `classes: []`. Human then shows **no matrix**, so the schedule is invisible unless you read JSON `horizons`. |
| Horizons | On the DTO; **never printed** on the human path. |
| Parser | `OutputFormat::parse` maps unknown/`None` → **Json** (governed silent-JSON). T227 F34 residual — **do not** change the shared parser. |

T166 closed the **engine** (class matrix, dry-run, apply, CE reuse, honesty strings). T248 closes **plan presentation**.

### 2.3 Code truth

| Site | Role |
|------|------|
| `commands/retention.rs` `run_plan` | `plan_retention` + `emit_report` |
| `emit_report` | Json → `emit_json` (`to_string_pretty`). Human/Markdown → title, totals, Debug samples, raw warnings |
| `RetentionCommands::Plan` | `--format` `Option<String>` default `"json"` — no `value_parser` |
| `RetentionCommands::Apply` | same format flag; `--confirm` / `--dry-run` / `--scope` |
| `governed_common::OutputFormat::parse` | `human\|text\|pretty` → Human; `md\|markdown` → Markdown; **else Json** |
| `RetentionPlanReport` | contracts `api_version="1"`; horizons map; sparse `classes`; totals; cascade; warnings; optional `errors` |
| `CANONICAL_CLASSES` | 9 ids; longest `orphaned_envelope` / `decision_approved` = **18** |
| Horizon labels | numeric days as decimal strings (`"90"`); policy text `revoked_superseded+30d_cooldown` (**32**), `none_auto`, `skip_apply` |
| `plan_retention` / `build_report` | Read-only; omits zero-count classes. Empty() test asserts `classes.is_empty()` |
| Nightly | `plan_retention` + totals log only — **do not restyle** this track |
| Desktop / HTTP | No retention route. Contract exists without HTTP (T161 residual honesty) |

### 2.4 Event / apply honesty (do not “fix” here)

Apply remains confirm-gated, CE-first via daemon + explicit `--scope`, projection-only without daemon, `RetentionApplied` audit, nightly CE **intent-only**. Pretty must not imply that reading a plan disposes anything. Pretty must not label projection delete as CE (R3). Sample cells stay truncated ids (R4).

---

## 3. Research (2026-08-14)

| Topic | Finding | Use in T248 |
|-------|---------|-------------|
| **[CLIG — Output](https://clig.dev/)** | Humans first; TTY heuristic; `--json` for structure; human output may evolve; scripts opt into `--json` | Plan default **`auto`**: TTY human, pipe JSON. Apply stays JSON. |
| **PROTOCOL-COMPAT §5** | Compact↔pretty without a flag is breaking. Retention is **not** in the inventory today. `emit_json` is already **pretty** JSON | Add a §5 row. JSON **keys** frozen. Pretty-print style of JSON stays (`emit_json`). Document TTY/pipe split. |
| **T246 SOOT** | `resolve_*_format` + clap `value_parser`; apply-like surfaces (`graph update`) do **not** TTY-switch | Copy for plan `auto`; apply `auto` ≡ json |
| **T227** | Unknown format exit **2**; do not silently emit JSON. Surface-wide `OutputFormat::parse` change is F34 leftover | Local resolver + clap reject. **Do not** edit `OutputFormat::parse` |
| **T166 R1/R3/R4** | Dry-run first; legacy ≠ CE; no plaintext in reports | Human restates those; does not change strings on the JSON path |
| **ISO/IEC 27001:2022 A.8.10** | Information deletion: appropriate method, **record** what would be / was deleted | Human plan is the pre-delete review record (counts/class/mechanism). Apply audit stays `RetentionApplied`. Not a compliance product. |
| **ISO 27001 A.5.33** | Protection of records / retention schedule | Print the schedule (horizons) even when counts are zero |
| **GDPR Art. 5(1)(e) practice** | Storage limitation **by category**; document periods + purpose | Category matrix already T166; T248 **presents** it. Not legal advice. |
| **restic `forget --dry-run`** | Prints the **policy**, then what would be removed; does not delete | Empty pretty still prints the class/horizon policy |
| **clap** | workspace **4.5** / lock **4.6.1** / crates.io **4.6.6** (fetched 2026-08-14). clap **5 not released** | `value_parser`; **no bump** |
| **serde_json** | lock **1.0.150** / crates.io **1.0.151** | **No bump**; keep `emit_json` pretty |
| **chrono** | lock **0.4.44** / crates.io **0.4.45** | Reuse for human timestamp; **no bump** |
| **is-terminal** | lock **0.4.17**. Crate docs prefer `std::io::IsTerminal` since 1.70 | Keep crate (T246 majority SOOT). Migration remains **T214 F24** |
| **comfy-table / tabled** | New crates | **Forbidden**. Hand-roll columns |
| **Desktop** | T172 honest-unavailable | **Do not** wire a retention screen |
| **rustc** | toolchain **1.95.0** | Unchanged |

---

## 4. Findings (DoD)

| ID | Severity | Requirement |
|----|----------|-------------|
| **F1** | Hard | `retention plan` `--format: String` (not clap enum; not `Option`) default **`auto`**, `value_parser = ["auto","pretty","human","text","json","markdown","md"]`. Signature: `pub(crate) fn resolve_retention_format(explicit: &str, is_tty: bool) -> &'static str` (graph SOOT; **not** `Option<&str>`). Resolve: `pretty`\|`human`\|`text`\|`markdown`\|`md` → human; `json` → existing pretty JSON; `auto` → TTY human else JSON. Apply calls the same helper with **`is_tty: false`** so `auto` never TTY-switches (F4). Probe `std::io::stdout().is_terminal()` via **`is_terminal::IsTerminal`**. Invalid token → clap usage **exit 2** (case-sensitive: `JSON`/`Pretty` fail). **No `other` passthrough** (AI1 M1 declined — clap already rejected unknowns). Does **not** call `OutputFormat::parse`. Ripple: `PlanOptions`/`ApplyOptions.format: String` + `main.rs` dispatch (AI2 L7). |
| **F2** | Hard | Human layout, **this order is normative** (AI2 L1 — F11 follows F2, not the reverse): (1) title `Retention plan (dry-run)` or `Retention apply` + `generated YYYY-MM-DD HH:MM UTC`; (2) empty → `Nothing to dispose.` / non-empty → `Work` table (non-zero classes only — matrix below still lists them; intended, not a dupe bug); (3) `Class matrix` for **all** `CANONICAL_CLASSES` (plus any extra report classes after); (4) Totals **exactly** `Totals  candidates={n} ce_wipe={n} projection_delete={n} skip={n} held={n}` (two spaces after `Totals`, single spaces between fields — AI2 L2); (5) `Honesty` short labels; (6) cascade line only if `parents_marked_for_resynthesis > 0`; (7) `Errors:` if any; (8) `next:` last, only on **plan** when `would_ce_wipe + would_projection_delete > 0`. Columns: **CLASS 18**, **HORIZON 36** (AI2 L6: `revoked_superseded+36500d_cooldown` is 34), **MECHANISM 18**, **COUNT 5**. Work table: CLASS 18 / COUNT 5 / MECHANISM 18 / SAMPLES rest. Format `{:<18} {:<36} {:<18} {:>5}`. No `comfy-table`. |
| **F3** | Hard | Zero candidates (empty `classes` **or** `totals.candidates == 0`): print `Nothing to dispose.` then the full matrix (zeros). Exit **0**. Do **not** print an apply `next:`. JSON empty stays `{ classes: [], totals: {… zeros}, horizons: {9 keys} }` — **do not** inject zero buckets into JSON (`RetentionPlanReport::empty` + CP `retention_plan__empty_vault__zero_counts` stay green). |
| **F4** | Hard | Apply `--format` same token set, default **`json`**. Omitted / `json` / `auto` / pipe = existing `emit_json`. **`auto` does not TTY-switch on apply** (dangerous + current docs/scripts). `--format human` (and pretty/text/md) uses the same formatter with the apply title. Confirm / dry-run XOR / daemon / `--scope` / `RetentionApplied` **unchanged**. |
| **F5** | Hard | JSON **keys frozen**: `api_version`, `generated_at`, `mode`, `horizons`, `classes[]` (`class`, `candidate_count`, `mechanism`, `sample_ids`, `notes`), `totals` (`candidates`, `would_ce_wipe`, `would_projection_delete`, `would_skip`, `would_held`), `cascade.parents_marked_for_resynthesis`, `warnings`, `errors_count`, optional `errors`. `api_version` stays `"1"`. **No** required new keys. Prefer **zero** additive keys. Warning **strings** unchanged. `emit_json` stays `to_string_pretty` (not compact). |
| **F6** | Hard | Pretty **Class matrix** always shows 9 canonical rows even when JSON `classes` is empty. Horizon text comes from **`report.horizons`** (env overrides must show, e.g. `AI_BRAINS_RETENTION_RAW_TURN_DAYS=45` → `45d`). Numeric-only labels get a `d` suffix for display. Policy labels stay as stored (`revoked_superseded+30d_cooldown`, `none_auto`, `skip_apply`) — do not rewrite JSON horizon values. Missing horizon key → `—`. |
| **F7** | Hard | When a class has a report bucket, pretty MECHANISM is the bucket’s `mechanism`. When filled as a zero row, MECHANISM is the **T166 v1 policy default** (display only): `raw_turn`/`query_trace`/`review_trace`/`decision_approved` → `projection_delete`; `evidence`/`secret`/`orphaned_envelope` → `ce_wipe`; **`memory_legacy` → `skip`** (AI2 M1 — v1 **none auto**, stream-A never scanned; `soft_forget` is future policy; `held` is only if a pinned subject were classified). `unclassified` → `skip`. Do not invent a tenth class. Extra non-canonical classes in the report print after the nine with their bucket mechanism. F14 **must** rewrite the OPERATIONS `memory_legacy` row (today `pinned → held` overclaims — T166 review already said so). |
| **F8** | Hard | Sample cells: join with `", "` — **never** `{:?}`. Already-truncated ids from the DTO. Empty samples → `—`. Do not print bodies, notes blobs, or full unwraps. Notes stay off the default human matrix (JSON `notes` unchanged). Soft F17 if someone wants `--verbose` notes. |
| **F9** | Hard | Human `generated_at`: parse RFC3339 via existing `chrono`; print `YYYY-MM-DD HH:MM UTC`. On parse failure, strip a fractional-second suffix (cut at `.` before tz) rather than showing nanos. JSON `generated_at` stays the planner’s RFC3339. |
| **F10** | Hard | Honesty block maps **known** constants to short labels; unknown warning strings print as-is (forward-compat): `legacy projection delete is not cryptographic erasure` → `projection delete ≠ CE`; `not NIST Purge/Destroy; not physical media sanitization (TRUNCATE is not Purge)` → `not NIST Purge/Destroy`; `stream_a_and_stream_b_independent_until_subject_join` → `stream A and B independent until subject join`; `erasure ticket and soft forget are not cryptographic erasure` → `ticket / soft forget ≠ CE`; `pre-erase backups, exports, and offline copies remain decryptable if restored` → `pre-erase backups remain decryptable`. Apply-only prefixes `command_id=` and `ce_pending=` print **as-is** (AI2 L5 — they carry ids/counts; do not invent shorts). JSON `warnings` **verbatim**. |
| **F11** | Hard | Plan `next:` copy is **last** (after Errors — AI2 L1 / F2 item 8). If `would_ce_wipe > 0` → `next: ai-brains retention apply --confirm --scope Repository:<uuid>`; else if `would_projection_delete > 0` → `next: ai-brains retention apply --confirm`; else omit. **Never** invent a scope UUID. Apply human omits `next:`. |
| **F12** | Hard | Zero new crates; **no version pin bumps**; no CLI reqwest; no contracts DTO change; no HTTP route; no desktop screen; no doctor check; no nightly `eprintln!` restyle; capture-independent. clap 5 is **not released** (max 4.6.6). |
| **F13** | Hard stop-before | Do **not** run live `retention apply --confirm`. Do not retune horizons. Do not enable nightly CE. Do not rewrite `class_based_retention.rs` candidate collection. |
| **F14** | Hard docs | CAPABILITIES OutputFormat table: plan TTY human / pipe JSON; apply default JSON + opt-in human. PROTOCOL-COMPAT §5 new row: TTY/pipe split; JSON **keys** unchanged; JSON stays `to_string_pretty`; **human path is not a wire contract** (AI2 B3); **`--format` tokens are case-sensitive** (`JSON` exit 2 — AI2 L4; pre-T248 `OutputFormat::parse` lowercased). OPERATIONS: TTY vs `--format json` examples **and** rewrite `memory_legacy` mechanism to `skip` (v1 none auto; no stream-A scan; pinned-held only if a subject were classified). Skill one-liner. Repo-root `CHANGELOG.md` T248 row only. after_help on `RetentionCommands` + Plan: one TTY example + one `--format json`. |
| **F15** | Hard verify | CP `class_based_retention` suite stays green (empty vault `classes` empty). CLI apply unit tests (`production_apply_requires_*`, `resolve_retention_apply_scope__*`) stay green. Protocol `API_VERSION == "1"` stays green. |
| **F16** | Soft residual | `--verbose` notes / untruncated samples; JSON zero-count buckets; doctor `retention_plan` check; nightly human one-liner; desktop UI; HTTP list; `OutputFormat::parse` surface-wide (T227 F34); is-terminal → std (T214 F24); shared `resolve_*_format` helper (AI2 B2); apply-warning shorts for `command_id=` / `ce_pending=` |
| **F17** | Soft residual | T166 engine leftovers (cascade-on-partial-CE honesty, double-plan TOCTOU, brain `days_from_env` overflow on legacy nightly) — **not** presentation |
| **F18** | Soft | Color / pager / markdown tables / `comfy-table` |

---

## 5. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: `resolve_retention_format` — `auto`+TTY → human; `auto`+pipe → json; `pretty`/`human`/`text`/`markdown`/`md` → human (TTY or not); `json` → json. Apply helper: `auto` → json even when TTY. |
| **AC2** | Unit: `format_retention_pretty` on `RetentionPlanReport::empty` contains `Nothing to dispose.`, all 9 class ids, a `90d` (or config) horizon, the **exact** line `Totals  candidates=0 ce_wipe=0 projection_delete=0 skip=0 held=0`, short honesty labels, `memory_legacy` mechanism `skip` (not `soft_forget` / not `held`), and does **not** contain `next: ai-brains retention apply`. |
| **AC3** | Unit: fixture with `raw_turn` count 2 / `projection_delete` / two sample ids — `Work` table, comma-joined samples (no `[` Debug), `next: ai-brains retention apply --confirm`, no `--scope`. |
| **AC4** | Unit: fixture `would_ce_wipe=1` — `next:` includes `--scope Repository:<uuid>` and `--confirm`. |
| **AC5** | Unit: custom `horizons["raw_turn"]="45"` → pretty shows `45d` (not hardcoded 90). |
| **AC6** | Unit: `generated_at` with nanos → human line has `YYYY-MM-DD HH:MM UTC` and no `.076`. |
| **AC7** | Unit: known warning constants map to F10 shorts; an unknown warning string is echoed. |
| **AC8** | Hermetic: `retention plan --format json` parses as JSON with keys `api_version`/`horizons`/`classes`/`totals`/`warnings`; `classes` may be `[]`; no new required keys. |
| **AC9** | Hermetic: `retention plan --format pretty` (or `human`) contains `Nothing to dispose` (empty fixture vault) **and** `raw_turn` **and** `Class matrix`. |
| **AC10** | Hermetic: `retention plan --format xml` (or `yaml`) exit **2** (clap usage). Zero stdout JSON. |
| **AC11** | Hermetic: `retention apply` without `--confirm` still `INVALID_PAYLOAD` (exit 6 class) — format change must not swallow the refuse. |
| **AC12** | Hermetic/unit: `retention apply --confirm --format json` default path still pretty-JSON parseable when it emits a report (projection-only fixture if cheap); `--format human` title contains `Retention apply`. Do **not** require a live CE wipe. |
| **AC13** | Live (on go): TTY `ai-brains retention plan` is human (matrix visible). Piped `ai-brains retention plan` is JSON (`classes` / `horizons` keys). Do **not** apply. |
| **AC14** | Docs: CAPABILITIES OutputFormat row + OPERATIONS + PROTOCOL-COMPAT §5 + CHANGELOG T248 |
| **AC15** | Existing `cargo nextest run -p ai-brains-control-plane class_based_retention` + CLI retention unit module stay green |

---

## 6. Non-goals

- Changing T166 classification, horizons, apply order, daemon/scope gates, or `RetentionApplied`
- Nightly auto-CE or restyling the nightly totals `eprintln!`
- Desktop retention UI / HTTP `/v1/retention`
- Doctor `--summary` (T249) or a new `retention_plan` doctor check
- Filling JSON `classes` with zero buckets
- clap 5 / serde_json / chrono / is-terminal bumps
- `comfy-table`, color, pager
- T249 / T250 / T251 presentation
- Printing vault keys or sample **bodies**

---

## 7. Capture independence / contracts / exits

| Topic | Rule |
|-------|------|
| Capture | Presentation + existing read-only `plan_retention`. No new events on plan. Apply events unchanged. No models/graph |
| Contracts | **No** `ai-brains-contracts` field/API_VERSION change. CLI-local formatters only |
| Exits | Plan empty success **0**; bad `--format` **2**; apply refuse without confirm stays **6** `INVALID_PAYLOAD`; vault errors unchanged |
| Privacy | Samples are already truncated ids (R4). Do not print keys or turn bodies |

---

## 8. File touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/main.rs` | Plan default `auto` + `value_parser`; Apply default `json` + same parser; after_help examples |
| `crates/ai-brains-cli/src/commands/retention.rs` | `resolve_retention_format` + `format_retention_pretty` + wire `emit_report`; keep apply gates |
| `crates/ai-brains-cli/tests/` | hermetic pretty/json + clap reject + apply refuse |
| `Docs/CAPABILITIES.md` | OutputFormat row + short Operator note |
| `Docs/PROTOCOL-COMPAT.md` | §5 inventory row |
| `Docs/OPERATIONS.md` | TTY vs `--format json` examples |
| `.agents/skills/ai-brains/SKILL.md` | one-liner |
| `CHANGELOG.md` | T248 row |
| `conductor/*` | status / deferred / README |

**Do not touch:** `class_based_retention.rs` planner, `contracts/retention.rs` DTO, `nightly.rs` dry-run log, desktop `ErasureScreen`, T243/T245/T246/T247 product files.

---

## 9. Verification plan

```powershell
# Units
cargo nextest run -p ai-brains-cli retention
cargo nextest run -p ai-brains-control-plane class_based_retention
cargo clippy -p ai-brains-cli --all-targets -- -D warnings

# Hermetic
cargo nextest run -p ai-brains-cli --test retention_plan_human

# Live on go (do not apply)
ai-brains retention plan
ai-brains retention plan | ConvertFrom-Json | Select-Object api_version, mode
ai-brains retention plan --format json
ai-brains retention plan --format pretty
ai-brains retention plan --format xml   # expect exit 2

# Full gate
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace ; cargo deny check ; cargo audit
ledgerful verify --scope full
```

---

## 10. Risk / review

- **Category:** FEATURE / UX (not SECURITY). Cross-model still useful: JSON key freeze + apply-gate non-regression + R4 samples.
- **Highest regression:** injecting zero class buckets into JSON; TTY-switching apply; changing honesty warning strings; silent `OutputFormat::parse` edit that flips other governed commands; running apply on the live vault.
- **Cap deferred mediums:** ≤3; presentation softs go to F16–F18 / deferred.md.

---

## 11. Suggested implement order (locked)

1. Pure `resolve_retention_format` + `format_retention_pretty` (Red → Green units, no vault)
2. Wire `run_plan` / `emit_report`; clap `auto` / `value_parser`
3. Apply `--format human` title only (gates untouched)
4. Hermetic + apply-refuse regression
5. Docs

---

## 12. Placeholder disposition

| Draft | Disposition |
|-------|-------------|
| F1 Human summary lines for totals + honesty warnings | **Absorbed** F2 / F10 (plus horizons + matrix) |
| F2 JSON shape frozen when `--format json` | **Absorbed** F5 |
| F3 Zero candidates still explains “nothing to dispose” + horizons | **Absorbed** F3 / F6 |
| AC1 Human + json hermetic | **Absorbed** AC8 / AC9 |
| AC2 Empty candidates non-blank human | **Absorbed** AC2 / AC9 |
| AC3 CAPABILITIES | **Absorbed** AC14 |

---

## 13. Deferred fold-in

| Item | Source | Disposition |
|------|--------|-------------|
| Retention plan human / JSON-only | deferred.md T248 / README E7/Q7 | **DoD** F1–F11 / AC1–AC13 |
| Placeholder F1–F3 | spec draft | All absorbed, refined (matrix + shorts + next-step) |
| T166 §6.2 json/human | T166 CLI table | Human half **DoD**; engine already shipped |
| T166 R4 no plaintext | T166 | **F8** samples join; no bodies |
| ISO A.8.10 / A.5.33 schedule visibility | research | **F3/F6** print policy when empty |
| Desktop retention UI | T172 / OPERATIONS | **Not absorbed** F12 / F16 |
| Doctor retention check | audit leftover | **Not absorbed** (T249-class) F16 |
| Nightly dry-run restyle | T166 F-004 | **Not absorbed** — totals `eprintln!` stays |
| T166 cascade / TOCTOU / brain overflow | T166 review residuals | **F17 soft** |
| T227 F34 OutputFormat silent-JSON | T227 residual | **Not absorbed** — local resolver only |
| T249 / T250 | peer placeholders | **Not absorbed** |

---

## 14. AI fold-in disposition (2026-08-14) — `C:\dev\AI-review.md` AI1 + AI2

No Highs. AI1 restates planned work (resolver, 9-class matrix, timestamp, sample join, docs, units). AI2 one medium is a real honesty bug in F7. AI1’s `other` passthrough is the same decline as T246.

### Disposition

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **AI1 M1** | AI1 | **Agree dispatch / decline passthrough** | F1 2-arg `&str` resolver; apply passes `is_tty: false`; **no** `Some(other) => other`; **no** `Option<&str>` |
| **AI1 M2** | AI1 | **Agree** (already F2/F6/F7) | 9-class matrix + policy defaults |
| **AI1 M3** | AI1 | **Agree** (already F9) | chrono RFC3339 → `YYYY-MM-DD HH:MM UTC` |
| **AI1 M4** | AI1 | **Agree** (already F8) | `join(", ")`; never `{:?}` |
| **AI1 L1 / O1** | AI1 | **Agree** | Already F14 / AC1–AC7 |
| **AI1 remapped ACs** | AI1 | **Decline** | Keep AC1–AC15 |
| **AI2 M1** | AI2 | **Agree hard** | F7 `memory_legacy` zero-row **`skip`**; F14 OPERATIONS rewrite (do **not** adopt `held` — T166 never scans; OPERATIONS already overclaims) |
| **AI2 L1** | AI2 | **Agree hard** | F2 order normative; `next:` last after Errors; F11 follows F2 |
| **AI2 L2** | AI2 | **Agree hard** | Totals exact string; AC2 asserts it |
| **AI2 L3** | AI2 | **Agree hard** | Phase 2 hermetics: `TempEnv` clear `AI_BRAINS_RETENTION_*` |
| **AI2 L4** | AI2 | **Agree** | F14 PROTOCOL-COMPAT case-sensitive tokens |
| **AI2 L5** | AI2 | **Agree as-is** | F10: `command_id=` / `ce_pending=` print verbatim; shorts → F16 |
| **AI2 L6** | AI2 | **Agree hard** | HORIZON width **36** (max cooldown label 34) |
| **AI2 L7** | AI2 | **Agree hard** | Phase 2: `format: String` on options + dispatch |
| **AI2 B1** | AI2 | **Affirm** | Work + matrix overlap is intended (F2) |
| **AI2 B2** | AI2 | **Decline as DoD** | Shared helper → F16 |
| **AI2 B3** | AI2 | **Agree** | F14: human path is not a wire contract |

### Pins locked by fold-in

1. **F7:** `memory_legacy` → `skip`; OPERATIONS row must match (not `soft_forget`, not `held`).
2. **F2/F11:** layout order; `next:` last; Totals exact `Totals  candidates=N ce_wipe=N projection_delete=N skip=N held=N`.
3. **F2:** HORIZON width **36**.
4. **F1:** clap reject; `format: String`; apply `is_tty: false`; no passthrough.
5. **F14:** case-sensitive `--format`; human not a wire contract.
6. **Hermetics:** `TempEnv` isolate `AI_BRAINS_RETENTION_*`.
