# T322 — `decision in-force --as-of`

- **Track ID:** T322-DecisionAsOf
- **Status:** **Planned** (Pending until **go**)
- **Category:** FEATURE
- **Owner:** Grok
- **Source:** T311 residual **R2** (`--as-of` was a written non-goal). Series README `README-T312-T324-CLI-DOGFOOD.md`. Not the 2026-08-27 CLI audit hole (T311 shipped after that audit’s “propose-only” note).
- **Depends on:** T311 ✅ `decision in-force`; T150 `superseded_by` + `valid_from`/`valid_until`; T160 propose/approve/supersede; `decision_valid_at(row, at)` already parameterized
- **Blocks / feeds:** Time-travel “what was in force at instant T?” Default **now** stays T311. Does **not** steal T323 conclusion in-force / T324 empty TERM / T325 / T326.
- **Absorbs:** T311 R2. T311 R4 `approved_at` **column declined** — hop-stop on superseded/revoked `updated_at` is sufficient (proof §2.2). Event payload already has `approved_at`; projector does not persist it.
- **Not absorbed (DoD):** T311 R1 daemon `ListInForce`; R3 sibling Approved (F7 earliest-root freeze); R5 conclusion (T323); R7 empty TERM (T324); H2; FTS `decision list`; projector `valid_until` close; new `approved_at` column; date-only `--as-of`; `--from`/`--to` range; clap 5
- **Research date:** 2026-08-29 (plan-write product HEAD `0eef80b` T321 `#243`). Snapshot — **re-verify at execute**.
- **Ledger:** planning DOCS TX `d8e6e556-cfb8-4cd6-84cc-3f5b1599532c`. Series mint DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** implement until **go**. Do **not** grow `governed_common.rs` (#3) / `project.rs` / `briefings/personal.rs` (#9) / decision **projector**. Extend CP `in_force.rs` + CLI clap. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** `cargo install`. Do **not** propose/approve/supersede on the **live** operator vault as proof.

---

## 1. Objective

1. **Point-in-time ruling.** `ai-brains decision in-force <TERM> --as-of <RFC3339>` returns the node that was the chain tip at that instant (or honest none). At the exact supersede/revoke instant the hop **has** happened (closed-open, SQL Server `AS OF`).
2. **Default remains now.** Omit `--as-of` → today’s T311 walk + `decision_valid_at(now)`. JSON keys `term`/`scope`/`ruling`/`chain` **byte-identical** to T311 when the flag is absent (`as_of` omitted via `skip_serializing_if`).
3. **Stay a governed read.** `ReadDecisions`. No new events. No pin→Approved. No daemon DTO.
4. **North star.** Capture independence: projection query over the existing supersede chain. No models, no graph, no embeddings.

This unblocks: T311 answers only “what governs **now**.” After supersession the prior Approved is `state=Superseded` and dropped. Operators (and audits) need “what governed on date D?” without a schema tourist `approved_at` column.

---

## 2. Live baseline (re-scan 2026-08-29)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | Plan-write `0eef80b` `feat(cli): T321 safety sync write honesty (#243)`. Tree **DIRTY** at plan start: uncommitted T321 conductor Completed + implement residuals (`conductor.md` / `deferred.md`) — absorbed into this DOCS commit, not product. Branch `track/T322-decision-as-of`. `origin/main` = `0eef80b`. |
| PATH `ai-brains.exe` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **26,897,408** B; LastWriteTime **2026-08-27 8:21:55 PM**; `ai-brains 0.1.3`. **T311 on PATH** (`decision in-force` exists). T312–T321 **not**. T322 hole **is** on PATH **and** source (no `--as-of`). **Do not `cargo install`.** |
| `preflight --summary` (PATH) | Pinned **4630**. In-context **0/0/0**. `Total Word Count: 728` (PATH-behind T315 `Budget window words:`). **Not this DoD.** |
| PATH `decision in-force --help` | `<TERM>`; `--scope`; `--format` default **json**; tokens `auto\|pretty\|human\|text\|json\|markdown\|md`. **No** `--as-of`. after_help two examples, neither as-of. |
| PATH `decision in-force workspace_id --format json` | `{"term":"workspace_id","scope":"Repository:3581317d-…","ruling":null,"chain":[]}` exit **0**. Live vault has no in-force Approved for that term. |
| PATH `decision in-force workspace_id --as-of 2026-01-01T00:00:00Z` | clap **unexpected argument** exit **2**. Hole confirmed. |
| Last GitHub PR | [#243](https://github.com/Ryan-AI-Studios/AI-Brains/pull/243) T321. `mergedAt` **2026-08-29T15:07:18Z**. Issue/review/inline comments **[]**. Open PRs: **none**. `#237` Bugbot already **T326**. `#230` already **T325**. **No T327 from Cursor.** |
| Ledger | 0 pending / 0 drift before this DOCS TX. |
| Hotspots | `project.rs` **#1** (3.640) — **do not touch.** `sync.rs` **#2**. `governed_common.rs` **#3**. `forget.rs` **#5**. CLI `preflight.rs` **#7**. `briefings/personal.rs` **#9**. `in_force.rs` / `decision.rs` **not** top 10. |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why `updated_at` is enough (R4 proof — no `approved_at` column)

| Layer | Truth |
|-------|--------|
| Event already has `approved_at` | `DecisionApprovedPayload.approved_at` (`payload.rs:424–430`). `approve_decision` writes `approved_at: clock.now()` (`decisions.rs:192–197`). |
| Projector does **not** persist it | `DecisionApproved` SQL: `state='Approved', approver, proposal_event_id COALESCE, updated_at=envelope.occurred_at`. **No** `approved_at` column (`decision.rs:83–96`). |
| Supersede **overwrites** `updated_at` | `DecisionSuperseded`: `state='Superseded', superseded_by, updated_at=occurred_at`. Does **not** close `valid_until` (`:97–108`). |
| Revoke same overwrite | `DecisionRevoked`: `state='Revoked', updated_at=occurred_at`. No `superseded_by`. |
| Propose default `valid_from` | If payload `valid_from` is None, projector stores **proposal** `occurred_at` (`:18–24`). CLI propose always sends `valid_from: None` (`decision.rs:129`). |
| `decision_valid_at(row, at)` | `valid_from.map(\|vf\| vf <= at).unwrap_or(true) && valid_until.map(\|u\| u > at).unwrap_or(true)` (`briefings/project.rs:695–698`). **Already takes `at`.** T311 passes `clock.now()`. Open `valid_until` ⇒ valid forever after `valid_from`. |
| T311 walk | Follow `superseded_by` to the **tip**, then F9: tip is ruling iff `state==Approved && decision_valid_at(now)`. As-of cannot be “run F9 at T on the tip” — the tip may not have been the tip at T. |
| Hop timestamp | On a **Superseded** row, `updated_at` **is** the `DecisionSuperseded` occurred_at. That is the transaction-time hop. Take the hop iff `as_of >= current.updated_at`. Stop (keep current) iff `as_of < current.updated_at`. |
| Approved tip | `updated_at` is the approval occurred_at (nothing after approve except supersede/revoke, which change state). For as-of on a still-Approved node: require `updated_at <= as_of` so a proposed-not-yet-approved tip is not a ruling. T311 **now** path does **not** add this check (stay-green). |
| Closing `valid_until` on supersede | **Decline.** D2.`valid_from` is proposal time, often **before** D1’s supersede `occurred_at` → overlapping valid windows. Hop-stop does not need the projector change. Existing rows would also need replay. |
| Propose→approve gap on a **superseded** node | Approval time is **lost** (overwritten by supersede). Hop-stop treats the superseded node as the ruling for any `as_of` in `[valid_from, hop_at)`. That includes a few milliseconds (tests) or minutes (slow approve) while it was still `Proposed`. **Soft residual** — not Complete-blocking. Column would close that gap; not required for the operator date question. |

**Pick:** transaction-time **AS OF** on the supersede/revoke chain (hop-stop). Not valid-time SQL (`conclusions_valid_at` style) — decision `valid_until` is never closed, so that SQL would keep every open-ended row.

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| Resolver | `control-plane/src/in_force.rs:52–96` | 4-arg `resolve_in_force(query, clock, scope_key, term)`. Walk `:148–189` then F9 `:76–81`. **Keep 4-arg as now-wrapper.** Add `resolve_in_force_at(..., as_of: Option<OffsetDateTime>)`. |
| JSON | `InForceResponse` `:23–28` | `term`, `scope`, `ruling`, `chain`. Ruling `:32–39` includes `updated_at` not `approved_at`. **Additive optional `as_of: Option<String>` + `skip_serializing_if`. Do not skip `ruling`.** |
| Valid-time helper | `briefings/project.rs:695` `pub(crate)` | **Call with `at`.** Do **not** edit `project.rs`. |
| CLI handler | `commands/decision.rs:219–275` | `InForceOptions` has `term/scope/format/principal_id`. Add `as_of: Option<String>`. `run_in_force` calls 4-arg today `:271`. |
| clap | `main.rs:2925–2944` `DecisionCommands::InForce` | Positional `TERM`; `--scope`; `--format` value_parser T311 F3; `--principal-id`. **Add `--as-of`.** Dispatch `:5002–5015`. |
| after_help | `:2927` | Two examples. **Additive** `--as-of 2026-01-15T00:00:00Z`. |
| Parent after_help | `:2889` | Propose + in-force. **Do not require** parent change. |
| Hermetics | `control-plane/tests/in_force.rs` AC1–AC7; `cli/tests/decision_in_force.rs` AC8–AC10 | **Stay-green** 4-arg / help-without-asserting-absent-as-of. New as-of tests named below. |
| Policy | `ReadDecisions` + `fail_api` exit **3** | **Freeze** T311 F10. |
| Empty term | `fail_usage` exit **2** | **Freeze.** T324 is PowerShell `""` dropping the argv slot — **not stolen.** |
| Clock | `SystemClock` / `Clock` trait | as-of **Some** uses the flag, not `clock.now()`. **None** uses `clock.now()` (T311 F11). Tests: read `get_decision(d1).updated_at` after supersede; `before = hop_at - Duration::NANOSECOND`. **No sleep.** No new FakeClock. |
| `supersede_decision` | `decisions.rs:204` `_clock` unused | Event `occurred_at` from `build_event` (wall). Tests must not assume injected clock on the hop. |
| `help_ia.rs:13` | Governed already names `decision` | **Freeze.** |
| Line counts | `in_force.rs` **204** physical; `decision.rs` **299**; CP tests **267**; CLI tests **252**. Snapshot — **F22 80-net is phase diff vs go HEAD**. |
| Contracts | none | No DTO. PROTOCOL-COMPAT has **no** in-force row. **Do not add** (local JSON, not daemon). |

### 2.4 Dependency / standards research (2026-08-29) — snapshot, re-verify at execute

| Pin | Workspace / lock | crates.io / docs | Action |
|-----|------------------|------------------|--------|
| clap | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6**; docs.rs 4.6 `value_parser` accepts `Fn(&str) -> Result<T, E>` ([TypedValueParser](https://docs.rs/clap/4.6.1/clap/builder/trait.TypedValueParser.html)) | **No bump.** Add `--as-of` with Rfc3339 parser. clap **5** forbidden. |
| time | workspace **0.3** (`parsing`+`formatting`) / lock **0.3.47** | crates.io **0.3.55**; docs.rs 0.3.47 `Rfc3339` example `1985-04-12T23:20:50.52Z`. Date-only **fails**. | **No bump.** `OffsetDateTime::parse(s, &Rfc3339)`. |
| serde / serde_json | workspace 1.0 / lock **1.0.150** | — | **No bump.** `skip_serializing_if = "Option::is_none"` on additive `as_of`. |
| rusqlite | **0.40.2** / SQLCipher **4.14.0 community** | — | **No bump.** No new SQL. |
| uuid | workspace **1.13** / lock **1.23.1** | — | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged. |
| workspace version | **0.1.3** | — | **No bump.** |
| New crates | — | — | **Zero.** |

**As-of research (primary sources):**

- SQL Server temporal `FOR SYSTEM_TIME AS OF` = `ValidFrom <= t AND ValidTo > t` (closed-open). At `t = ValidTo` the old row is **out**. ([Microsoft Learn — Temporal tables](https://learn.microsoft.com/en-us/sql/relational-databases/tables/temporal/overview), updated 2026-08-18.)
- Wikipedia / XTDB: valid-time vs transaction-time. T150 `valid_from`/`valid_until` is valid-time but **supersede does not close** `valid_until`. The chain hop is transaction-time. **Fit:** hop-stop on superseded/revoked `updated_at`, same closed-open (`as_of >= hop_at` takes the hop).
- CP `conclusions_valid_at` (`adapters.rs:851–873`) already documents “Does NOT use recorded_at / occurred_at (bitemporal: domain valid time).” **Do not reuse** that SQL for decisions — it would skip `Superseded` rows we need as hop parents.

N/A: Windows schtasks / SQLCipher pin change / clap 5.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — Default now** | Omit `--as-of` → T311 behavior exactly. Keep `resolve_in_force` 4-arg as a wrapper that calls `resolve_in_force_at(..., None)`. T311 tests **do not** have to change call sites. |
| **F2 — CLI flag** | `--as-of <RFC3339>` optional on `DecisionCommands::InForce` only. `value_parser` = Rfc3339 (`time` crate). Invalid / date-only / empty → clap `InvalidValue` exit **2**. No `--from`/`--to`. No `--term` (T324). No new format tokens. |
| **F3 — T311 format freeze** | Default **json**. `value_parser` tokens unchanged. Unknown `--format` stay-green AC8. |
| **F4 — JSON keys** | Always emit T311 F4: `term`, `scope`, `ruling` (never skip), `chain`. **Additive** `as_of` (RFC3339 string) when the flag was set; `skip_serializing_if = "Option::is_none"`. Ruling object keys **frozen** (`decision_id`, `title`, `statement`, `state`, `approver`, `updated_at`). Do **not** add `approved_at` to JSON. |
| **F5 — Hop-stop** | Walk `superseded_by` as T311 F8 (cap 32, cycle error, scope equality, broken id stop). **Before taking a hop**, if `as_of` is `Some(t)` and `t < current.updated_at`, **break** (keep current; do not hop). `as_of is None` → take all hops (T311). Closed-open: `t >= current.updated_at` takes the hop. |
| **F6 — Ruling at as-of** | Let `at = as_of.unwrap_or(now)`. After walk: **None-path (T311 F9):** `Approved && decision_valid_at(now)` only. **Some-path:** (a) `Approved && decision_valid_at(at) && updated_at <= at` → ruling; (b) `Superseded && decision_valid_at(at)` (stopped because hop is in the future) → ruling; (c) `Revoked && updated_at > at && decision_valid_at(at)` → ruling; else none. Revoked-root empty-chain special case **only** on the None-path (T311 F7 stay-green). Some-path may return a ruling with empty chain (revoked-before-revoke / stopped at root). |
| **F7 — Chain is the walk prefix** | `chain` is hops **taken**, not the full current-day chain. as-of that stops at D1 while D1→D2 exists now → `chain=[]`, ruling D1. as-of after hop → chain D1→D2 (same as now if D2 is tip). |
| **F8 — T311 F-list freeze** | Scope F5, term match F6, earliest-root F7, hop cap/cycle F8, policy F10, empty unknown F12 (`ruling: null`, no `next_step` key, human `format_authorized_empty_next`), local-only F13, H2 F14. Do not reopen. |
| **F9 — Decline `approved_at` column** | R4 proof §2.2. Do **not** edit `projections/decision.rs`. Do **not** add a migration. Event payload field stays unprojected. |
| **F10 — Decline projector `valid_until` close** | Overlap with successor `valid_from` (proposal). Not needed for hop-stop. |
| **F11 — Decline daemon / DTO / HTTP** | T311 F13. Soft residual R1 stands. |
| **F12 — Human** | None-path chrome freeze (`Term:` / `Scope:` / `Ruling:` or F12 none + next). Some-path: additive `As of: <rfc3339>` line (canonical `Rfc3339` format) when flag set. Do **not** change none-copy. |
| **F13 — Decline extra CLI** | No `--quiet` / `--format` new tokens / `--confirm` / `--global` / `--as-memory`. No `decision show` / `decision list`. |
| **F14 — Isolation** | Touch `in_force.rs` + `lib.rs` export + CLI `decision.rs` + `main.rs` InForce clap/dispatch/after_help + CP/CLI tests + CHANGELOG + CAPABILITIES/OPERATIONS one-liners. Do **not** grow `governed_common.rs` / `project.rs` / `help_ia.rs` / projector / daemon-api / contracts. |
| **F15 — Capture independence** | Projection read only. No new events. |
| **F16 — unwrap** | Production: no `unwrap`/`expect`/`panic`. Rfc3339 format `map_err` into `ControlPlaneError::Query`. |
| **F17 — Test naming** | `function_or_feature__condition__expected_result`. No `test_` prefix. |
| **F18 — FEATURE** | Implement-track starts FEATURE TX + cross-model (in-force is FEATURE). |
| **F19 — `ISSUES.md`** | Does not exist. Residuals → `deferred.md`. |
| **F20 — PowerShell `;`** | Not `&&`. |
| **F21 — F36-class** | N/A (no JSON envelope finder). |
| **F22 — 80-net** | Production net vs **go HEAD**. Test blocks in `main.rs` clap may inflate physical; F32 note. |
| **F23 — after_help required** | InForce after_help **must** name `--as-of` + a full RFC3339 example. |
| **F24 — Stay-green T311** | AC1–AC7 CP; CLI help lists TERM/scope/format; `--format nope` exit 2; empty term exit 2; deny exit 3; JSON `ruling` key. 4-arg `resolve_in_force` unchanged. |
| **F25 — last-PR** | `#243` empty. `#237` → T326. `#230` → T325. **No T327.** |
| **F26 — Decline peers** | T323 / T324 / T325 / T326 / T307 Blocked / T308 floors / H2 / clap 5 / T240 F2. |
| **F27 — PATH-behind** | T312–T321 not on PATH. T322 hole **is** on PATH. Hermetic/`cargo run` SoT. PATH install not Complete-blocking. |
| **F28 — No live vault writes** | Hermetic CP tests + clap hermetics prove DoD. Do **not** `decision propose` / approve / supersede on the operator vault. |
| **F29 — Parser location** | `pub(crate) fn parse_as_of_rfc3339(s: &str) -> Result<String, String>` in `commands/decision.rs` (CLI). clap `value_parser` points at it. CP `resolve_in_force_at` takes `Option<OffsetDateTime>` already parsed. **Copy-not-share** with any future conclusion as-of (T323). |
| **F30 — Strict RFC3339** | Full timestamp with offset (`…Z` or `…+00:00`). Date-only `2026-01-01` is InvalidValue. Do not invent a date-only helper. |
| **F31 — Examples keep TERM first** | `decision in-force workspace_id --as-of 2026-01-15T00:00:00Z`. `--as-of` before TERM is legal clap (flag takes a value) but a missing/invalid timestamp is exit 2 — not a fail. |
| **F32 — `main.rs` physical** | Additive clap field + dispatch + after_help only. Do not relocate `DecisionCommands`. |
| **F33 — Docs honesty** | CAPABILITIES Family C row names `--as-of`. OPERATIONS one example. CHANGELOG Unreleased Added. |
| **F34 — help_ia freeze** | Governed list already has `decision`. |
| **F35 — No timeout crate** | N/A. |
| **F36 — Clock on None only** | `as_of Some` must **not** call `clock.now()` for F9 (tests inject wall hop via projection `updated_at`). |

---

## 4. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | `decision_in_force_help__after_help__names_as_of` — `--help` lists `--as-of` and an RFC3339 example containing `T` and `Z` or a numeric offset. Must **fail** today (no flag). |
| **AC2** | `decision_in_force_clap__default__as_of_absent` — help / clap struct has `--as-of` optional (not required). Green-on-arrival for “optional” once the flag exists; red today on missing `--as-of` in help (same proof as AC1 is fine — do not require a second help spawn if AC1 already asserts optional-not-required via clap `[OPTIONS]`). Prefer a dedicated clap parse: `in-force workspace_id` (no `--as-of`) still parses. Hermetic. |
| **AC3** | `resolve_in_force_at__as_of_before_supersede__prior_approved` — D1 Approved then superseded by D2; `as_of = d1.updated_at - 1ns` → ruling **D1**, `state=in_force`, `chain=[]`. |
| **AC4** | `resolve_in_force_at__as_of_at_supersede__successor` — same fixture; `as_of = d1.updated_at` → ruling **D2**, chain len 1 D1→D2. Closed-open. |
| **AC5** | `resolve_in_force_at__as_of_before_valid_from__none` — `as_of` far in the past (`1970-01-01T00:00:00Z`) → `ruling=None` (valid_from is proposal/now). |
| **AC6** | `parse_as_of_rfc3339__date_only__err` — `"2026-01-01"` errors (no spawn). `parse_as_of_rfc3339__zulu__ok` — `"2026-01-15T00:00:00Z"` ok. |
| **AC7** | `decision_in_force__as_of_invalid__clap_exit_2` — `--as-of not-a-date` and `--as-of 2026-01-01` → clap exit **2**, `invalid value`. |
| **AC8** | Stay-green T311 CLI: `decision_in_force__help__lists_term_scope_format`; `decision_in_force__format_nope__clap_exit_2`; empty term exit 2; deny exit 3. |
| **AC9** | Stay-green T311 CP: AC1 superseded-root current D2; AC2 successor-term empty chain; AC3 revoked-root none (**None-path**); AC4 unknown; AC5 empty term; AC6 other scope; AC7 cycle. 4-arg `resolve_in_force` still compiles. |
| **AC10** | Stay-green JSON: omit `--as-of` → object has `term`/`scope`/`ruling`/`chain` and **does not** contain key `as_of`. With `--as-of` (hermetic or CP `to_value`) key `as_of` is the RFC3339 string. |
| **AC11** | `resolve_in_force_at__revoked_as_of_before_revoke__prior_approved` — D1 approved then revoked; `as_of = d1.updated_at - 1ns` → ruling D1; 4-arg / `None` still none + empty chain (AC3 stay-green). |
| **AC12** | Targeted: `cargo clippy -p ai-brains-control-plane -p ai-brains-cli --all-targets -- -D warnings`; nextest those packages (plus new tests). Full workspace gate on implement-track publish, not plan. |
| **AC13** | Docs: CAPABILITIES `decision in-force` row names `--as-of`; OPERATIONS one example; CHANGELOG Unreleased Added. Grep, not a docs-file hermetic. |
| **AC14** | Manual (on go, after green): `cargo run -p ai-brains-cli -- decision in-force --help` lists `--as-of`. `decision in-force workspace_id --as-of 2020-01-01T00:00:00Z --format json` → `ruling: null` on **this** live vault (pass-with-observed-data). **Do not** propose to the operator vault. |
| **AC15** | `resolve_in_force_at__none__matches_four_arg` — same fixture as T311 AC1; `resolve_in_force_at(..., None)` equals `resolve_in_force(...)` (ruling D2). |

---

## 5. Design notes

### 5.1 Algorithm (`resolve_in_force_at`)

1. Reject empty trim (T311).
2. `list_decisions(Some(scope_key), None)`; `select_root` unchanged.
3. `walk_chain(..., as_of)`: T311 loop, plus F5 hop-stop **before** cycle/get_decision when `as_of` is Some.
4. `at = as_of.unwrap_or(clock.now()?)`.
5. Apply F6 ruling rules. Map row → `InForceRuling` (F4 keys).
6. Set `response.as_of = as_of.map(|t| t.format(Rfc3339))`.

`resolve_in_force` = `resolve_in_force_at(..., None)`.

### 5.2 CLI

`parse_as_of_rfc3339` in `decision.rs`. clap:

```text
#[arg(long = "as-of", value_name = "RFC3339", value_parser = commands::decision::parse_as_of_rfc3339)]
as_of: Option<String>,
```

`run_in_force` parses the (already-validated) string to `OffsetDateTime` with `Rfc3339` and `map_err` into `fail_usage` only if the invariant breaks (should not). Prefer `expect` **forbidden** — use `fail_usage` on reparse err.

### 5.3 Tests without sleep

After `supersede_decision` / `revoke_decision`, `ports.query.get_decision(d1)` → `updated_at`. `before = hop_at - time::Duration::NANOSECOND`. Do **not** `sleep`. Do **not** invent FakeClock this track (`supersede` ignores `Clock`).

---

## 6. Non-goals

- `approved_at` projection column / migration / rebuild
- Closing `valid_until` on `DecisionSuperseded`
- Conclusion in-force (T323)
- PowerShell empty TERM (T324)
- Daemon `ListInForce` / contracts DTO / PROTOCOL-COMPAT row
- `decision list` / `decision show` / FTS
- Date-only `--as-of` / `--from`/`--to` range
- pin→Approved (H2)
- clap 5 / pin bumps / growing `governed_common.rs` / `project.rs`
- Live vault propose/approve as Complete-blocking AC
- T325 / T326 / T307

---

## 7. Verification plan

**Red first (TDD):** AC1 help must **fail** on today’s tree (no `--as-of`). AC3 must **fail** (`resolve_in_force_at` missing). AC6 date-only parser missing. Then green hop-stop + clap. Then AC4/AC5/AC7/AC11/AC15. Stay-green AC8–AC10/AC9.

**Manual (on go, after green):** AC14. Record JSON. Do not require live Approved chain.

Do **not** require full workspace nextest to finish the **plan**.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Same-nanosecond propose/approve/supersede flakes | AC3/AC4 use **stored** `updated_at ± 1ns`, not wall capture around the call |
| Signature break T311 tests | F1 4-arg wrapper |
| JSON key churn | F4 skip_serializing_if; AC10 forbids `as_of` when omitted |
| Projector tourism | F9/F10 |
| H2 creep | F8 / F14 |
| Growing hotspots | F14; do not touch `governed_common.rs` #3 / `project.rs` #1 |
| Date-only surprise | F30 / AC6 / AC7 |
| Cycle + as-of | Hop-stop may avoid the cyclic hop; AC7 stay-green is **None-path** only |
| Propose→approve gap on superseded rows | §11 residual; decline column |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-29 (T321 implement residuals through T142). Overlapping **open** rows:

| Item | Disposition |
|------|-------------|
| T311 R2 `--as-of` | **Absorb** F1–F7 / AC1–AC7 / AC11 / AC14 / AC15 |
| T311 R4 `approved_at` column | **Decline** F9 — hop-stop uses superseded/revoked `updated_at`; event field stays unprojected |
| T311 R1 daemon `ListInForce` | **Decline** F11 — no consumer |
| T311 R3 sibling Approved | **Decline** — T311 F7 earliest-root freeze |
| T311 R5 conclusion in-force | **Not stolen** T323 |
| T311 R7 PowerShell empty TERM | **Not stolen** T324 |
| T321 implement residuals (PATH / tracing stdout / F35) | **Not stolen** — T321 Completed |
| T325 F8 PreferRecency (`#230`) | **Not stolen** |
| T326 `PinnedCountFailed` fake `pinned=0` (`#237`) | **Not stolen** |
| T307 Blocked / T308 floors / H2 / clap 5 / T240 F2 | **Not stolen** / **Decline** |
| last-PR Cursor `#243` | **N/A empty** — comments/reviews/issue `[]`. **No T327.** |
| last-PR `#237` / `#230` | **T326** / **T325** already Pending |
| `ISSUES.md` | **Does not exist** |
| T321 uncommitted conductor Completed + residuals | **Absorb into this DOCS commit** (working-tree truth; not product) |

---

## 10. Implement order (on go)

1. Phase 0: re-read `in_force.rs` walk + F9, clap `InForce`, `decision_valid_at`, projector supersede arm, T311 tests; lock clap **4.6.1** / time **0.3.47**; FEATURE TX. **Do not install.** **Do not** live-propose.
2. Red AC1 / AC3 / AC6 (must fail).
3. Green `resolve_in_force_at` + 4-arg wrapper + F5 hop-stop + F6 Some-path. AC3–AC5 / AC11 / AC15.
4. CLI `--as-of` + `parse_as_of_rfc3339` + after_help + human `As of:` + JSON skip. AC1 / AC2 / AC7 / AC10.
5. Stay-green AC8 / AC9.
6. CHANGELOG + CAPABILITIES/OPERATIONS (AC13).
7. Targeted clippy/nextest AC12. Implement-track full gate before publish.

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| Propose→approve gap on a superseded/revoked node | Approval `updated_at` overwritten; hop-stop uses `[valid_from, hop_at)`. Column declined F9 |
| PATH until owner `cargo install` | F27 — hermetic/`cargo run` SoT |
| Live vault `workspace_id` ruling null | Honesty; AC14 pass-with-observed-data |
| Daemon `ListInForce` | F11 / T311 R1 |
| Date-only UX | F30 by design |
| `supersede_decision` ignores `Clock` | Wall `build_event`; tests use stored `updated_at` |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-control-plane/src/in_force.rs` | `resolve_in_force_at`; hop-stop; additive `as_of` JSON; 4-arg wrapper |
| `crates/ai-brains-control-plane/src/lib.rs` | `pub use` `resolve_in_force_at` |
| `crates/ai-brains-control-plane/tests/in_force.rs` | AC3–AC5 / AC11 / AC15 (existing AC1–AC7 stay) |
| `crates/ai-brains-cli/src/commands/decision.rs` | `parse_as_of_rfc3339`; `InForceOptions.as_of`; `run_in_force` passes `Option<OffsetDateTime>`; human `As of:` |
| `crates/ai-brains-cli/src/main.rs` | `InForce` `--as-of` + dispatch + after_help |
| `crates/ai-brains-cli/tests/decision_in_force.rs` | AC1 / AC2 / AC7 / AC10 |
| `CHANGELOG.md` | Unreleased Added |
| `Docs/CAPABILITIES.md` | Family C row `--as-of` |
| `Docs/OPERATIONS.md` | One example |

Do **not** touch: `governed_common.rs`, `briefings/project.rs`, `projections/decision.rs`, `help_ia.rs`, daemon-api, contracts, retrieval, graph, INSTALL.md.
