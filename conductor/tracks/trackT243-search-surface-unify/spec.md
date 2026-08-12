# T243 — Search surface unify

- **Track ID:** T243-SearchSurfaceUnify
- **Phase:** Post-install CLI effectiveness series (T240–T255) — P1
- **Status:** ✅ **Completed** (PR #153 `7a19d40`)
- **Category:** FEATURE / UX / CONTRACT (light, additive `next_step` only)
- **Owner:** Grok
- **Source:** Audit 2026-08-11 P1 dual search mental model; progressive first-run **E3**; T231 soft residuals (search noun; recall text→pretty; invalid-env clap); T241 feed “progressive ranking quality”
- **Depends on:** T231 decision table + empty next-step (PR #138); T221 deny exit **3** + `denial_hint` (PR #114); T241 grant discoverability (PR #151); T202 progressive project ceremony; T204 help IA
- **Blocks / feeds:** Operators know which of three search surfaces to run; progressive empty/deny no longer looks like “vault has no knowledge”
- **Absorbs:** deferred.md “Search dual model + progressive first-run”; T231 soft **search noun** + **recall text→pretty**; T241 leftover “progressive ranking” **as honesty/next-step only** (not a ranking rewrite)
- **Not absorbed (DoD):** Progressive ranking / corpus rewrite (T152 authority sort stays); merge `recall` + `sync query` + `query progressive` into one engine; `sync query --semantic`; JSON `scope` on recall; invalid-env clap converge (T231 F36 stays documented); clap 5 / ValueEnum; `is-terminal` → stdlib; T227 surface-wide `OutputFormat`; T221 F32 `--principal-id` on progressive; daemon/HTTP `QueryKnowledge` next-step; auto-init grants; MSI
- **Research date:** 2026-08-12 (live dogfood + code truth + clig.dev + clap 4.6.6 docs + crates.io pins + T221/T231/T241 residuals)
- **AI fold-in:** 2026-08-12 — `C:\dev\AI-review.md` AI1 + AI2. **No Highs.** AI1 M1–M4 restate already-planned F2/F3/F4/F5 (**agree** as design affirmation). **Decline AI1 M3 struct rewrite** (`scope: ScopeResolvedResponse` is not the live DTO; line 645 is stale — live is `briefings.rs:418`). **AI2 hard:** M1 enumerate **3** `next_step: None` sites; M2 pin F4 insert **before** `emit_json`. Lows L1–L4/L6–L12 agree as pins. L5 AC9 → **unit on helper** (not Decision-ingest hermetic). Disposition **§12**.
- **Ledger:** plan-only until go (`ledgerful ledger start T243-search-surface-unify --category FEATURE`)

## 1. Objective

1. **One operator story for three surfaces** — what to run when — without inventing a fourth retrieval engine.
2. **Make `search` do the daily thing** — top-level `ai-brains search` is a **visible alias of `recall`** (vault-first), not `sync query` and not progressive.
3. **Stop the progressive dead-end** — deny still exit **3** + bootstrap; **also** name ungoverned `recall`. Authorized empty stays exit **0** (T221 F14) but carries an in-band `next_step` pointing at `recall`.
4. **Close T231 F8 lie** — `recall --format text` becomes pretty (same as `sync query --format text`), not silent JSON.
5. **Capture independence** — display / alias / hints / docs / one additive optional DTO field. No event writes. No ranking algorithm change. No models required.

## 2. Live baseline (re-scan 2026-08-12)

### 2.1 Operator dogfood (this machine)

| Command | Observed | Gap |
|---------|----------|-----|
| `ai-brains search "test"` | clap **exit 2** `unrecognized subcommand 'search'` | First-run noun operators type |
| `recall --help` | Vault-first; peer `sync query`; format `'json' or 'pretty'` | No progressive peer; no `text` |
| `sync query --help` | Human vault+ledger; always pretty; agents → `recall` | No progressive peer |
| `query progressive --help` | “JSON ProgressiveQueryResponse”; project-id ceremony | **No corpus honesty** (sounds like vault search) |
| `query progressive "why was graph backend replaced?"` | **exit 0**, `denied: false`, `results: []` | Looks like “no knowledge”; `recall` on same string returns pins |
| `policy show` | 3 discovery grants on `test-alias` | Progressive is **allowed** — empty is corpus, not deny |
| `recall "search surface unify…"` | Hits T231 DECISION pins (text arm / search noun still residual) | Soft residuals never shipped |
| `recall --format text` | Code path: `resolve_format` returns `"text"` → `_` arm → **JSON** | T231 F8 documented lie |
| Invalid `AI_BRAINS_PROJECT_ID` | **recall** clap exit **2**; **sync** `project=(none)` exit **0** | T231 F36 — keep documented |
| CAPABILITIES §15 | Two-command table (recall vs sync) | Progressive missing |
| WORKFLOWS §5 | Find something = vault vs ledger | Progressive missing |
| Root `--help` Start here | `doctor` / `recall` / `scope resolve` | `search` not listed |

### 2.2 Root cause (frozen)

```text
// Three corpora, not three UIs on one index
recall / search     → vault memories (FTS ± semantic ± graph ± bridge)
sync query          → vault memories + Ledgerful ledger pane (lexical vault)
query progressive   → Approved decisions + Confirmed/Active conclusions
                      with evidence handles (T152). NOT vault FTS.

// T231 left these residuals on purpose
recall --format text → JSON via match `_`  (help omits text)
no top-level search
invalid-env clap vs manual — documented, not converged

// T221 F14 (keep)
grants + zero governed hits → denied:false, exit 0, results:[]
// T241 unblocked grants; empty is now the live effectiveness bug
```

`ProgressiveQueryHitDto.kind` comment lists `Memory` but `progressive_query` never emits Memory hits (decisions + conclusions only). Do **not** start emitting vault memories from progressive this track.

### 2.3 Touch map

| Site | Role |
|------|------|
| `ai-brains-cli/src/main.rs` | `#[command(visible_alias = "search")]` on `Recall`; help strings: recall + sync + progressive peers; `--format` help lists `text` |
| `ai-brains-cli/src/help_ia.rs` | Start here: one `search` alias line. **Do not** change T204 Daily inventory exact string |
| `ai-brains-cli/src/commands/recall.rs` | `resolve_format`: `"text"` → `"pretty"`; unit AC |
| `ai-brains-cli/src/commands/governed_query.rs` | Deny: append recall fallback to `denial_hint` + extra stderr line. Empty authorized: set `next_step`. Shared SOOT constant |
| `ai-brains-cli/src/commands/governed_common.rs` | `PROGRESSIVE_RECALL_FALLBACK` SOOT (do **not** mutate `POLICY_DENIED_HINT`) |
| `ai-brains-contracts/src/briefings.rs` | Additive `next_step: Option<String>` on `ProgressiveQueryResponse` (`default`, `skip_serializing_if`); `new()` leaves `None` |
| Hermetics | `search` alias; recall `--format text`; progressive deny append; progressive empty `next_step`; T221 deny exit 3 regression |
| Docs | CAPABILITIES §15 3-way table; WORKFLOWS §5; CLI-EXIT-CODES empty-vs-deny row; CHANGELOG **new** T243; skill one-liner **soft** |

### 2.4 Deps / research pins (2026-08-12)

| Pin | Evidence | Action |
|-----|----------|--------|
| `clap` workspace **4.5** → lock **4.6.1**; crates.io **4.6.6** (2026-08-06) | Cargo.lock / crates.io / GitHub releases | **No bump** |
| clap 5 | Not on crates.io; 4.6.x current major | **Out** (series non-goal) |
| `#[command(visible_alias = "search")]` | [docs.rs clap 4.6.6 `Command::visible_alias`](https://docs.rs/clap/4.6.6/clap/struct.Command.html#method.visible_alias); derive `command` attrs | **Use** — existing repo pattern is `visible_alias` on args (`claim`/`overwrite`); first **subcommand** visible alias |
| `serde_json` lock **1.0.150**; crates.io **1.0.151** | Cargo.lock / crates.io | **No bump** |
| `serde` lock **1.0.228** | Cargo.lock | **No bump** |
| `is-terminal` lock **0.4.17** | Cargo.lock | **No bump** |
| **Zero new crates** | F16 | — |
| [clig.dev](https://clig.dev/) Ease of discovery | Help, examples, suggest next command; “if you can guess what they meant, suggest it” | `search` alias + progressive next-step |
| clig — Return zero only on success | Deny stays exit **3**; authorized empty stays **0** | F5 / F12 |
| clig — stdout primary; stderr messaging | Packet stays stdout; extra recall line on stderr for deny | F4 |
| clig — Human-first output; `--json` for machines | `text` ≡ pretty (human); JSON path unchanged | F3 |
| clig — Don’t have a catch-all subcommand | `search` is an **explicit alias**, not “unknown verb → recall” | F2 |
| T221 F14 / F17 | Authorized empty unchanged; `denial_hint` already in-band | F5 additive `next_step` |
| T231 F2 / F3 / F24 | Keep dual recall vs sync; T231 said search→sync as residual — **override**: alias **recall** (daily vault-first) | F2 |
| T211 F3 / T152 | Leave progressive authority ranking alone | F7 |
| T241 F10 / F28 | No auto-init; deny-by-default stays | F18 |

## 3. Product decision (locked for plan)

| Option | Disposition |
|--------|-------------|
| **A** — Docs + help 3-way table (extend T231 A+C) | **Accept hard** |
| **B** — Top-level `search` → **`recall`** (`visible_alias`) | **Accept hard** (T231 residual now DoD). Not sync. Not a dispatcher. Not a third engine. |
| **C** — Progressive deny/empty name `recall` | **Accept hard** |
| **D** — `recall --format text` → pretty | **Accept hard** (close T231 F8) |
| **E** — Converge invalid-env clap | **Decline** (keep T231 F36 document-only) |
| **F** — Rewrite progressive ranking / add vault Memory hits | **Decline** (T152 corpus stays governed) |

**Why search → recall, not sync:** Daily inventory already lists `recall` as the human/agent vault path; TTY pretty already works; `sync query` is the specialized ledger pane. Operators typing `search` want vault memories. Sync remains the documented “plan vs shipped” command.

## 4. Frozen decisions (F0–F34)

| ID | Decision |
|----|----------|
| **F0 — Scope** | IA + alias + format honesty + progressive next-step. No FTS/RRF/T152 ranking change. No event writes. |
| **F1 — Three surfaces** | Keep three commands with three jobs. Document the **corpus**, not just the flags. |
| **F2 — `search` alias (hard)** | `Recall` variant: `#[command(visible_alias = "search")]`. Same argv, same handler, same exits. Help shows `recall [aliases: search]`. **Not** an alias of `sync` / `query progressive`. **Not** a new `Commands::Search` variant. |
| **F2b — Alias collision** | `evidence search` stays a **nested** noun (unchanged). Top-level `search` ≠ evidence search. |
| **F3 — text ≡ pretty (hard; AI2 L2)** | `resolve_format`: explicit `"text"` or `"pretty"` → `"pretty"`; `"json"` → `"json"`; **`Some(other) => other` pass-through** (do **not** add `Some(_) => "json"`). `None` → TTY pretty / non-TTY json (**do not regress** AC4). Caller `match` `_` arm (`recall.rs` ~284) still treats unknown as JSON. Help lists `json` \| `pretty` \| `text`. |
| **F4 — Progressive deny recall fallback (hard)** | When `denied`: keep T221 packet + exit **3** + `POLICY_DENIED:` + `POLICY_DENIED_HINT`. **Additionally:** (1) append `PROGRESSIVE_RECALL_FALLBACK` to `denial_hint` if it does not already contain `recall`; (2) one extra stderr line with the same fallback. **Do not** change shared `POLICY_DENIED_HINT` (lists/check/daemon dual-site stay). |
| **F4b — Insert order (AI2 M2 hard)** | In `run_progressive` **after** the existing `denial_hint.is_none()` backfill (`governed_query.rs` ~83–85) and **before** `emit_json` (~87): run `apply_progressive_search_hints` (append on deny / set `next_step` on authorized empty). After `emit_json`, in the deny stderr block (~88–91): keep `POLICY_DENIED:` + `POLICY_DENIED_HINT`, then **one extra** `eprintln!("{PROGRESSIVE_RECALL_FALLBACK}")`. Mutating after `emit_json` would leave stdout without the recall hint. |
| **F5 — Authorized empty `next_step` (hard)** | `!denied && results.is_empty()` → set `next_step = Some(PROGRESSIVE_RECALL_FALLBACK)` then emit JSON; exit **0**. Non-empty → `next_step` stays `None` (omit). Denied → `next_step` stays `None` (use `denial_hint`). T221 F14 **unchanged**. |
| **F5b — Contracts E1** | `ProgressiveQueryResponse.next_step: Option<String>` with `#[serde(default, skip_serializing_if = "Option::is_none")]`. CLI fills after Ok (T241 F7 pattern). Golden `progressive_query_response.json` stays valid (field omitted when None). `protocol_wire.rs:411` uses `new()` — covered by constructor. No daemon fill as DoD (F24). |
| **F5c — Struct-literal sites (AI2 M1 hard)** | Adding the field breaks every full literal. **Exactly 3 production sites**, all `next_step: None` (CP never fills): (1) `briefings.rs` `new()` body (~447–459); (2) `query.rs` **deny** literal (~94–106); (3) `query.rs` **success** literal (~378–390). Desktop TS `apps/desktop/src/lib/types/index.ts` `ProgressiveQueryResponse`: add optional `next_step?: string`. Pre-existing missing `denial_hint` on that TS type is **not** T243 DoD. |
| **F6 — Invalid-env (not converged)** | Reaffirm T231 F36 in the 3-way table. Do **not** add clap `env=` to sync. Do **not** remove clap env from recall. |
| **F7 — No ranking rewrite** | Do not change `progressive_query` sort, `matches_query`, valid-time windows, or emit Memory/vault hits. T241 “ranking quality” is **honesty**: empty governed ≠ empty vault. |
| **F8 — Capture independence** | Alias/hints/docs/DTO only. |
| **F9 — Dual keep** | Do not merge engines. Do not add `--semantic` to sync or progressive. |
| **F10 — TTY/json recall** | `resolve_format(None, …)` unchanged. |
| **F11 — Sync always-pretty** | Unchanged (T231 F33). |
| **F12 — Exit codes** | Deny **3**; missing project progressive **2**; authorized empty **0**; `search` success/fail same as `recall`; unknown top-level still **2** for other nouns. |
| **F13 — SOOT fallback (AI2 L1)** | `Ungoverned vault search: ai-brains recall "…"`. Must contain `recall`. No emoji. Generic `"…"`. Constant lives in `governed_common.rs` **next to** `POLICY_DENIED_HINT` / bootstrap SOOTs. Comment: `// CLI-only progressive→recall fallback (T243 F13). Not dual-site.` **No** daemon/CP twin. |
| **F14 — Help peers (AI2 L3)** | `recall --help`: keep sync peer; add one governed line. **`SyncCommands::Query` doc-comment** (main.rs ~1824–1826) — that variant has **no** `after_help` today; do **not** edit the `Sync` parent. `query` / `query progressive` after_help: corpus honesty + `recall` + `sync query`. |
| **F15 — help_ia Start here** | Add one line under Start here: `ai-brains search "what did we decide"  # alias of recall`. **Do not** change the T204 Daily inventory exact string (`Daily:     recall, preflight, …`). |
| **F16 — Zero dep bumps / zero new crates** | clap 4.5 workspace; no clap 5. |
| **F17 — Docs** | CAPABILITIES §15 becomes **three** surfaces (replace two-command intro). F8 row: **both** `text` ≡ pretty. F36 row stays. WORKFLOWS §5 adds progressive recipe + empty/deny honesty. CLI-EXIT-CODES: authorized empty may include `next_step`. CHANGELOG **new** T243 only. |
| **F18 — No auto-grant / no interactive** | Reaffirm T210/T241. |
| **F19 — Domain in CLI** | Forbidden beyond format map, alias attr, hint fill, docs. CP ranking untouched. |
| **F20 — Expand Denied** | Stay T221 (exit 3 + bootstrap hint). **Do not** append recall fallback (handle-specific, not a search miss). |
| **F21 — Briefing** | Soft deny exit 0 unchanged. No T243 briefing field work. |
| **F22 — Soft: skill one-liner** | Project skill “Find something” mentions `search` alias — residual if time-box. |
| **F23 — Soft: non-empty recall footer** | T231 F23 remains residual (noisy). |
| **F24 — Soft: daemon/HTTP next_step (AI2 L11)** | T221 F18 class. CLI packet is DoD. Daemon `QueryKnowledge` uses `new()` → `next_step` omitted by serde. **No daemon handler fill.** |
| **F25 — Review** | Primary required. Cross-model **hard** (FEATURE + contracts `next_step`). |
| **F26 — Test naming** | `function_or_feature__condition__expected_result`; hermetic temp vault. |
| **F27 — unwrap ban** | No new `unwrap`/`expect` in production. |
| **F28 — PowerShell** | `;` separators in gate scripts. |
| **F29 — Isolation / hermetics** | Alias + text tests: `tempdir` + `--no-project-context` where env-sensitive. Progressive deny/empty hermetics: **extend T221** (`governed_first_run_deny_exit.rs`) per F34. AC9 is the F33 unit, not a new Decision seed. |
| **F30 — Live dogfood (go; AI2 L4)** | Empty authorized path is **already live** (§2.1). Dogfood confirms `next_step` now appears; record the JSON snippet. Also: `search --help`; `search "…"` ≡ `recall`; `--format text` prints `Scope:`; no-grants deny still exit 3 and `denial_hint` contains `policy bootstrap` **and** `recall`. |
| **F31 — Parallel (AI2 L8)** | Touches recall format, clap Recall attr, progressive emit, contracts briefings, docs. Low conflict with T245 harness / T247 nightly if they avoid those files. Coordinate with T249 if it rewrites `help_ia`. |
| **F32 — Stop-before** | Ranking rewrite; catch-all unknown-verb→recall; auto-grants; clap 5; flipping authorized empty to exit 3; mutating `POLICY_DENIED_HINT` shared constant; AI1 M3 DTO rewrite. |
| **F33 — Pure hint helper (AI2 L5 refined)** | Extract `pub(crate) fn apply_progressive_search_hints(resp: &mut ProgressiveQueryResponse)` in `governed_query.rs` (or `governed_common.rs`). AC8/AC9 **units** on the helper: denied → `next_step` None + hint contains `recall`; authorized empty → `next_step` Some; authorized non-empty → both omit. **Do not** require a Decision-ingest hermetic for AC9. |
| **F34 — Extend T221 hermetics** | Reuse `governed_first_run_deny_exit.rs`: extend `progressive__deny__stderr_code_and_hint_stdout_denial_hint` with `recall` asserts (AC6/AC7); extend `progressive__after_system_bootstrap__exit_0_denied_false` (`query` is `"x"`) with `next_step` contains `recall` (AC8). Keep bootstrap asserts. |

## 5. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | `ai-brains search --help` exit 0; stdout contains vault-first recall help and/or `[aliases: search]` |
| **AC2** | Hermetic: `search "<q>" --format json --no-bridge` parses as live `RecallResponse` JSON (`results` array; not clap “unrecognized subcommand”). Spec draft said `api_version`+`hits` — live wire has neither; do not invent them |
| **AC3** | Unit: `resolve_format(Some("text"), false) == "pretty"` and `Some("text"), true == "pretty"` |
| **AC4** | Unit: `resolve_format(None, false) == "json"` (T231 AC11 regression) |
| **AC5** | Hermetic: `recall --format text --limit 1 --no-bridge` on a hit vault prints `Scope:` (pretty chrome), not a leading `{` JSON object |
| **AC6** | Hermetic no-grants: `query progressive "x"` exit **3**; stdout `denied: true`; `denial_hint` contains `policy bootstrap` **and** `recall` |
| **AC7** | Hermetic no-grants: stderr contains `POLICY_DENIED:` and `recall` (fallback line or appended hint) |
| **AC8** | Hermetic grants + zero governed hits: exit **0**; `denied: false`; `results` empty; `next_step` contains `recall`; `denial_hint` omitted |
| **AC9** | **Unit** on `apply_progressive_search_hints`: non-empty `results` + `denied: false` → `next_step` stays `None` (no Decision-ingest hermetic) |
| **AC10** | CAPABILITIES §15 three-surface table + `search` alias row + updated `text` row; WORKFLOWS §5 progressive; CHANGELOG T243 |
| **AC11** | Help: recall mentions progressive **or** governed; progressive after_help mentions `recall` and corpus (conclusions/decisions) |
| **AC12** | Shared `POLICY_DENIED_HINT` string **unchanged** (unit or source assert); evidence/source deny still uses old wording |
| **AC13** | Contracts: empty-success JSON omits `next_step` when `Some` not set; denied fixtures still parse; `new()` compiles with `next_step: None` |
| **AC14** | Full gate green; protocol_compat / golden updated if they pin `ProgressiveQueryResponse` |
| **AC15** | T204 Daily inventory exact string still passes (no `Daily:` rewrite) |

## 6. Risks

| Risk | Mitigation |
|------|------------|
| Operators think `search` is vault+ledger | Help + §15: alias of **recall**; ledger remains `sync query` |
| `evidence search` confusion | F2b; docs one-liner; no rename |
| Changing `POLICY_DENIED_HINT` breaks dual-site + many hermetics | **F4/F12** — append only in progressive CLI |
| Authorized empty `next_step` looks like a deny | Separate field; omit on deny; docs |
| Scripts parsed `recall --format text` as JSON | Rare (help never listed text). CHANGELOG notes the honesty fix. Do not keep silent JSON. |
| Catch-all `search` forever-blocks a future real Search command | Explicit `visible_alias` is a supported stable alias (clig: aliases must be explicit). A future dedicated noun would be a breaking deprecation track. |
| Ranking rewrite pressure | F7 + stop-before |
| Contracts golden drift | F5b + AC13/AC14 |

## 7. Non-goals

- Single command that does semantic + ledger + governed ranking
- Auto-widen `--global` on empty
- JSON `scope` on recall
- Invalid-env clap converge
- `sync query --semantic`
- Progressive Memory/vault fusion
- clap 5 / ValueEnum / `is-terminal` migrate
- T227 OutputFormat surface-wide
- T221 F32 `--principal-id` on progressive
- Daemon/HTTP 403 vs 200+denied
- MSI / packaging
- Nightly/router (T247/T255)
- Harness wiring (T245)

## 8. Verification plan

1. Preflight: `ledgerful doctor` + `ledger status` + `ledger start T243-search-surface-unify --category FEATURE` + `scan --impact`
2. Red: AC3/AC4 units; AC1/AC2 alias; AC5 text; AC6–AC8 progressive
3. Green: alias attr + `resolve_format` + progressive fill + contracts field
4. Docs AC10/AC11; AC12 hint freeze; AC15 T204
5. Live dogfood F30 (go)
6. Full gate + primary review + **hard** cross-model (F25)
7. Ledger commit + pin + deferred close + series README

## 9. Absorbed deferred index

| Source | Disposition |
|--------|-------------|
| deferred.md Search dual model + progressive first-run | **DoD** |
| Series README T243 | **Planning → Complete on ship** |
| T231 search noun (F24 / O2) | **Hard F2** (alias → recall, not sync) |
| T231 recall text→pretty (F22 / O1) | **Hard F3** |
| T231 invalid-env clap converge | **Not absorbed** — F6 document |
| T231 F23 non-empty footer | **Soft F23** |
| T241 “progressive ranking quality” | **Honesty only F4/F5/F7** — no ranking rewrite |
| AI-review.md AI1+AI2 (2026-08-12) | **F4b/F5c/F33/F34** + §12; AI1 DTO rewrite declined |
| T221 F14 authorized empty | **Keep** + additive `next_step` |
| T221 F18 daemon HTTP | **Soft F24** |
| T227 OutputFormat surface-wide | **Out** |
| T204 Daily inventory | **Do not rewrite** (AC15) |

## 10. Residual after T243 (expected soft)

| Item | Owner |
|------|-------|
| Skill one-liner (`search` alias) | Soft F22 |
| Non-empty recall ledger footer | Soft F23 |
| Daemon/HTTP progressive `next_step` | Soft F24 |
| Invalid-env clap/manual converge | Remains documented F6 |
| Distinct rich `text` renderer (not pretty) | Soft |
| `sync query --semantic` | Future if demanded |
| Progressive vault Memory hits / ranking rewrite | Separate product track |
| `is-terminal` → stdlib | Residual (T231 L8) |
| T227 F34 OutputFormat | Separate residual |

## 11. Implementer SOOT snippets (pins)

```rust
// F2 — main.rs Recall
/// Vault-first search (pretty on TTY, JSON when piped). Alias: `search`.
/// For vault + Ledgerful ledger: `sync query`. Governed conclusions/decisions: `query progressive`.
#[command(visible_alias = "search", display_order = 10)]
Recall { /* unchanged fields */ }

// F3 — recall.rs
fn resolve_format(explicit: Option<&str>, is_tty: bool) -> &str {
    match explicit {
        Some("text") | Some("pretty") => "pretty",
        Some("json") => "json",
        Some(other) => other,
        None => {
            if is_tty {
                "pretty"
            } else {
                "json"
            }
        }
    }
}

// F13 — governed_common.rs (new; do not edit POLICY_DENIED_HINT)
pub const PROGRESSIVE_RECALL_FALLBACK: &str =
    "Ungoverned vault search: ai-brains recall \"…\"";

// F4/F5 — governed_query.rs after Ok(resp)
if resp.denied {
    if resp
        .denial_hint
        .as_deref()
        .is_none_or(|h| !h.contains("recall"))
    {
        let base = resp
            .denial_hint
            .clone()
            .unwrap_or_else(|| POLICY_DENIED_HINT.to_string());
        resp.denial_hint = Some(format!("{base} {PROGRESSIVE_RECALL_FALLBACK}"));
    }
} else if resp.results.is_empty() {
    resp.next_step = Some(PROGRESSIVE_RECALL_FALLBACK.to_string());
}
```

Call `apply_progressive_search_hints(&mut resp)` **before** `emit_json`. `is_none_or` is fine on edition 2024 / Rust 1.85+ (AI2 L7); if clippy objects, use `match hint { None => true, Some(h) => !h.contains("recall") }`. **No** `unwrap` on `denial_hint`.

F4b order (live `governed_query.rs`):

```text
83-85  backfill denial_hint if None          // keep
       apply_progressive_search_hints(&mut resp)  // NEW — before emit
87     emit_json(&resp)?
88-91  if denied { POLICY_DENIED; POLICY_DENIED_HINT;
         eprintln!(PROGRESSIVE_RECALL_FALLBACK);  // NEW extra line
         return exit 3 }
```

F5c sites — all `next_step: None`:

```text
briefings.rs new()           ~447
query.rs deny literal        ~94
query.rs success literal     ~378
protocol_wire.rs:411         uses new() — no extra literal
```

## 12. AI fold-in disposition (2026-08-12)

Source: `C:\dev\AI-review.md` (AI1 + AI2). Independently re-checked live DTO, `query.rs` literals, `governed_query.rs` emit order, `SyncCommands::Query` (doc-comment only), T221 hermetics, golden fixture.

### AI1

| ID | Disposition | Pin |
|----|-------------|-----|
| **M1** `visible_alias = "search"` | **Agree** — already F2 | Phase 2 |
| **M2** `text` → pretty | **Agree** — already F3 | Phase 2 |
| **M3** add `next_step` | **Agree field; decline struct rewrite** | Live DTO is `applied_scope: String` + `applied_policy` / `query_trace_id` / `more_available` / summaries — **not** `scope: ScopeResolvedResponse`. Line **418** not 645. F5b/F5c. |
| **M4** wire fallback | **Agree** — already F4/F5; order → **F4b** | Phase 3 |
| **L1** keep `POLICY_DENIED_HINT` | **Agree** — already F4/F12 | AC12 |
| **L2** docs | **Agree** — already F17 | Phase 4 |
| **O1** resolve_format units | **Agree** — already AC3/AC4 | Phase 1 |
| **AC remap** (AI1 AC10=hint freeze, AC11=CI) | **Decline** | Keep our AC10–AC15 (docs/help/hint/gate/Daily) |

### AI2

| ID | Disposition | Pin |
|----|-------------|-----|
| **M1** 3 `next_step` literals | **Agree hard → F5c** | `new()` + `query.rs:94` + `query.rs:378` |
| **M2** append before `emit_json` | **Agree hard → F4b** | After line 85 backfill, before line 87 emit; extra stderr after line 91 |
| **L1** constant CLI-only | **Agree → F13** | `governed_common.rs`; no daemon twin |
| **L2** `Some(other) => other` | **Agree → F3** | Do not catch-all to json in `resolve_format` |
| **L3** sync peer site | **Agree, refined → F14** | `SyncCommands::Query` **doc-comment** (no `after_help` on that variant); not Sync parent |
| **L4** F30 empty already live | **Agree → F30** | Dogfood records `next_step` |
| **L5** AC9 fixture | **Agree problem; decline expensive hermetic** | **F33** unit helper; AC9 = non-empty → `next_step` None |
| **L6** F22 skill | **Agree** already soft | Phase 4 if free |
| **L7** `is_none_or` | **Agree** | Edition 2024; match fallback |
| **L8** T249 `help_ia` | **Agree** already F31 | Coordinate only |
| **L9** AC2 shape | **Agree → AC2** | `api_version` + `hits` array |
| **L10** F23 footer | **Agree** stays soft | — |
| **L11** daemon omit | **Agree → F24** | `new()` + serde skip |
| **L12** CHANGELOG Unreleased | **Agree → F17** | Additive only |
| **O1–O12** | **Agree as phase pins** except O6 Decision ingest | See plan fold-in table |

### Declined / out of scope

| Item | Why |
|------|-----|
| AI1 M3 rewritten DTO (`scope: ScopeResolvedResponse`) | Would drop live fields; wrong line numbers |
| AI1 remapped AC10/AC11 | Collides with existing AC12/AC14 |
| AI2 L5 Decision-ingest hermetic as DoD | Expensive; F33 unit is sufficient for “non-empty omits next_step” |
| Desktop backfill of missing `denial_hint` | Pre-existing T221 drift; F5c only adds `next_step?` |
