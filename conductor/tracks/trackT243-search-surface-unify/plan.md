# T243 Plan — Search surface unify

**Status:** 📋 **Planning** (plan-only until **go**)  
**Spec:** [spec.md](./spec.md) F0–F34 / AC1–AC15 + §12 AI fold-in  
**Category:** FEATURE / UX / CONTRACT  
**Ledger:** open on go — `ledgerful ledger start T243-search-surface-unify --category FEATURE`

---

## AI fold-in (2026-08-12) — `C:\dev\AI-review.md` AI1 + AI2

No Highs. Spec design affirmed. AI1 mediums restate F2/F3/F4/F5 (already planned). AI2 two mediums are **must-fold** before go.

### Disposition

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **AI1 M1–M2 / M4** | AI1 | **Agree** | Restate F2/F3/F4 — no design change |
| **AI1 M3** | AI1 | **Agree field; decline DTO rewrite** | Keep live `applied_scope: String` + existing fields; add `next_step` only (F5b/F5c). Ignore `scope: ScopeResolvedResponse` sketch and stale `briefings.rs:645` |
| **AI1 L1–L2 / O1** | AI1 | **Agree** | Already F12/F17/AC3 |
| **AI1 AC remap** | AI1 | **Decline** | Keep AC10–AC15 |
| **AI2 M1** | AI2 | **Agree hard** | F5c: 3 sites `new()` + `query.rs:94` + `query.rs:378` |
| **AI2 M2** | AI2 | **Agree hard** | F4b: append **before** `emit_json` (~87); extra stderr **after** `POLICY_DENIED_HINT` (~91) |
| **AI2 L1** | AI2 | **Agree** | F13 CLI-only constant + comment |
| **AI2 L2** | AI2 | **Agree** | F3 keep `Some(other) => other` |
| **AI2 L3** | AI2 | **Agree, refined** | F14: `SyncCommands::Query` **doc-comment** (no after_help on that variant) |
| **AI2 L4** | AI2 | **Agree** | F30: empty already live |
| **AI2 L5** | AI2 | **Agree problem; decline Decision hermetic** | F33 helper + AC9 unit |
| **AI2 L6–L8 / L10** | AI2 | **Agree** | Already F22/F23/F31/§11 |
| **AI2 L7** | AI2 | **Agree** | `is_none_or` OK; match fallback |
| **AI2 L9** | AI2 | **Agree** | AC2: `api_version` + `hits` |
| **AI2 L11** | AI2 | **Agree** | F24 daemon omit via `new()` |
| **AI2 L12** | AI2 | **Agree** | CHANGELOG Unreleased only |
| **AI2 O6** | AI2 | **Decline as DoD** | No Decision-ingest hermetic |

### Pins locked by fold-in

1. **F5c:** `next_step: None` at `briefings.rs` `new()`, `query.rs:94` deny, `query.rs:378` success. Desktop TS add `next_step?: string` only.  
2. **F4b:** `apply_progressive_search_hints` after line 85 backfill, **before** line 87 `emit_json`; extra `eprintln!(PROGRESSIVE_RECALL_FALLBACK)` after line 91.  
3. **F33:** pure helper carries AC8/AC9 units.  
4. **F34:** extend T221 `governed_first_run_deny_exit.rs` (deny + after-bootstrap `"x"`).  
5. **F3:** do not add `Some(_) => "json"` in `resolve_format`.  
6. **F14:** peer line on `SyncCommands::Query` doc-comment, not Sync parent.

---

## Preflight (plan time — 2026-08-12)

| Check | Result |
|-------|--------|
| `ai-brains search "test"` | clap exit **2** `unrecognized subcommand 'search'` |
| `query progressive "why was graph backend replaced?"` | exit **0**, `denied: false`, `results: []` (3 discovery grants live) |
| Same query via `recall` | Non-empty pin hits (T231 DECISION residuals) |
| `policy show` | 3 Read* grants on `test-alias` — progressive allowed; empty is **corpus** |
| `recall --help` | Peer `sync query`; format `json` or `pretty` only |
| `query progressive --help` | JSON ProgressiveQueryResponse; no corpus honesty |
| `recall --format text` | Code: `resolve_format` returns `"text"` → `_` → JSON |
| CAPABILITIES §15 | Two-command table; F8/F36 rows present |
| clap | Workspace **4.5**; lock **4.6.1**; crates.io **4.6.6** — **no bump** |
| `serde_json` | lock **1.0.150**; crates.io **1.0.151** — **no bump** |
| `is-terminal` | lock **0.4.17** — **no bump** |
| T204 Daily string | Exact inventory must stay (AC15) |
| `POLICY_DENIED_HINT` | Dual-site CLI + CP + daemon — **do not mutate** |
| Progressive ranking | Decisions + conclusions only; no Memory emit — **F7 leave** |
| Capture | Alias/hints/docs only |
| Live mutate / policy bootstrap | **Not run** in plan-only |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| Dual search + progressive first-run | deferred.md / series README | **DoD** F1, F4, F5, F14, F17 |
| T231 search noun | T231 F24 / O2 / review residual | **DoD F2** alias → **recall** (override T231 “→ sync”) |
| T231 recall text→pretty | T231 F22 / F8 | **DoD F3** |
| T231 invalid-env clap | T231 F36 | **Not absorbed** — F6 document |
| T231 F23 non-empty footer | T231 residual | **Soft F23** |
| T241 progressive ranking | T241 not-absorbed → T243 | **Honesty F4/F5/F7** — no ranking rewrite |
| T221 F14 authorized empty | T221 | **Keep** + `next_step` |
| T221 F18 daemon HTTP | T221 | **Soft F24** |
| T227 OutputFormat | T227 F34 | **Not absorbed** |
| Skill search one-liner | series pattern | **Soft F22** |
| clap 5 / engine merge / auto-grant | series non-goals | **Not absorbed** |

---

## Phases

### Phase 0 — Plan freeze

- [x] Full spec + plan
- [x] Live dogfood + dep research (2026-08-12)
- [x] Roll T231 search/text residuals + T241 ranking-as-honesty
- [x] Fold `C:\dev\AI-review.md` AI1+AI2 → F4b/F5c/F33/F34 + §12
- [x] User **go** before production code

### Phase 1 — Red (TDD)

- [x] Unit: `resolve_format__explicit_text__returns_pretty` (AC3)
- [x] Unit: existing `resolve_format__no_explicit_not_tty__returns_json` still green (AC4)
- [x] Unit: `apply_progressive_search_hints` denied / empty / non-empty (F33; AC8/AC9)
- [x] Hermetic: `search --help` recognized (AC1)
- [x] Hermetic: `search --format json --no-bridge` has live `RecallResponse.results` array (AC2 / AI2 L9; not `hits`)
- [x] Hermetic: `recall --format text` pretty `Scope:` (AC5)
- [x] Extend T221 `progressive__deny__…` with `recall` in stdout hint + stderr (AC6/AC7 / F34)
- [x] Extend T221 `progressive__after_system_bootstrap__…` with `next_step` contains `recall` (AC8 / F34)
- [x] Regression: T221 deny still exit **3**; `POLICY_DENIED_HINT` source unchanged (AC12)
- [x] T204 Daily inventory test still passes (AC15)

### Phase 2 — Green (alias + text)

- [x] `#[command(visible_alias = "search")]` on `Recall`
- [x] Recall + `query progressive` help peers; **`SyncCommands::Query` doc-comment only** (F14 / AI2 L3) — not Sync parent
- [x] `help_ia` Start here line (F15) — do **not** rewrite Daily inventory
- [x] `resolve_format`: add `text`/`pretty` arm; **keep `Some(other) => other`** (F3 / AI2 L2)
- [x] `--format` help lists `json` \| `pretty` \| `text`

### Phase 3 — Green (progressive next-step + contracts)

- [x] Add `next_step` to `ProgressiveQueryResponse` (F5b)
- [x] F5c three sites, all `None`: `new()` ~447; `query.rs:94` deny; `query.rs:378` success
- [x] Desktop TS: optional `next_step?: string` only (F5c)
- [x] `PROGRESSIVE_RECALL_FALLBACK` in `governed_common.rs` after bootstrap SOOTs; comment CLI-only (F13)
- [x] `apply_progressive_search_hints` **after** line 85 backfill, **before** line 87 `emit_json` (F4b)
- [x] Extra stderr `eprintln!(PROGRESSIVE_RECALL_FALLBACK)` **after** line 91 `POLICY_DENIED_HINT` (F4b)
- [x] Expand Denied: **no** recall append (F20)
- [x] Golden stays valid (omit when None); `protocol_wire.rs:411` covered by `new()` (AC13)

### Phase 4 — Docs

- [x] CAPABILITIES §15 three-surface table + `search` row + text≡pretty (both recall + sync); F36 row stays
- [x] WORKFLOWS §5 progressive recipe + empty/deny honesty
- [x] CLI-EXIT-CODES: authorized-empty `next_step`; recall/search invalid env → 2; sync → 0 `project=(none)` (F6 / AI2 O12)
- [x] CHANGELOG **new** T243 under **Unreleased** only (AI2 L12)
- [x] Soft F22 skill one-liner if free

### Phase 5 — Live dogfood (go only)

- [x] `ai-brains search --help` — alias visible
- [x] `ai-brains search "<known pin>" --format pretty --limit 1` — hits
- [x] `ai-brains recall "<q>" --format text --limit 1` — `Scope:` chrome
- [x] Live progressive: no-grants deny exit 3 + `denial_hint` contains bootstrap **and** `recall`; after bootstrap, authorized empty `next_step` contains `recall` (F30)
- [x] Record exact commands + outputs below

#### F30 record (2026-08-12, debug binary, project `test-alias`)

```text
ai-brains search --help
  → Usage: ai-brains.exe recall [OPTIONS] <QUERY>
    About: Alias: `search`. Vault-first. Peer sync query + query progressive.

ai-brains search "search surface unify" --format pretty --limit 1 --no-bridge
  → Scope: project=test-alias (…)
    hit (vault memory)

ai-brains recall "search surface unify" --format text --limit 1 --no-bridge
  → Scope: project=test-alias (…)  (pretty chrome, not leading `{`)

# grants were empty on this machine at go; deny path first:
ai-brains query progressive "why was graph backend replaced?"
  → exit 3, denied:true
    denial_hint: "…policy bootstrap… Ungoverned vault search: ai-brains recall \"…\""
    stderr: POLICY_DENIED: … / POLICY_DENIED_HINT / PROGRESSIVE_RECALL_FALLBACK
    next_step omitted

ai-brains policy bootstrap --dry-run ; ai-brains policy bootstrap
  → 3 Read* issued

ai-brains query progressive "x"
  → exit 0, denied:false, results:[]
    "next_step": "Ungoverned vault search: ai-brains recall \"…\""
    denial_hint omitted
```

### Phase 6 — Gate + close

- [ ] Full CI gate (`fmt` / `clippy -D warnings` / nextest / deny / audit)
- [ ] `ledgerful verify --scope fast` then full as required
- [ ] Primary review + **hard** cross-model (F25)
- [ ] `conductor.md` T243 → Completed; deferred.md strike; series README
- [ ] `ledgerful ledger commit`
- [ ] Pin decisions (alias → recall; text≡pretty; ranking not rewritten)

---

## Implementation notes

### Search alias (F2)

```rust
#[command(visible_alias = "search", display_order = 10)]
Recall { /* unchanged */ }
```

clap 4.6.6: `Command::visible_alias` — same matches, listed in help. Do **not** add `Commands::Search`.

### Format map (F3 / AI2 L2)

```rust
Some("text") | Some("pretty") => "pretty",
Some("json") => "json",
Some(other) => other, // pass-through; caller `_` arm → JSON
None => { if is_tty { "pretty" } else { "json" } }
```

Do **not** add `Some(_) => "json"` inside `resolve_format`.

### Progressive fill order (F4b / AI2 M2)

Live `governed_query.rs` today: backfill None (~83–85) → `emit_json` (~87) → stderr CODE + `POLICY_DENIED_HINT` (~88–91).

Insert:

1. After ~85 backfill: `apply_progressive_search_hints(&mut resp)` (deny append **or** empty `next_step`).
2. Then `emit_json` (packet now carries the hint / `next_step`).
3. After ~91 `eprintln!(POLICY_DENIED_HINT)`: `eprintln!("{PROGRESSIVE_RECALL_FALLBACK}")`.

Never set both `denial_hint` recall-append and `next_step` on the same packet. Never mutate `resp` after `emit_json`.

### Contracts E1 (F5b)

| State | `denied` | `denial_hint` | `next_step` |
|-------|----------|---------------|-------------|
| Policy wall | true | bootstrap + recall fallback | omit |
| Authorized empty | false | omit | recall fallback |
| Authorized hits | false | omit | omit |

### Docs table (F17) — ship in CAPABILITIES §15

| Intent | Command |
|--------|---------|
| Human, vault only | `recall "…"` / `search "…"` (TTY pretty) |
| Agent / pipe | `recall "…"` JSON (or `search`) |
| Human, vault + ledger | `sync query "…" --format pretty` |
| Governed conclusions/decisions | `query progressive "…"` (needs discovery grants) |
| Embeddings / hybrid | `recall "…" --semantic` |
| Invalid `AI_BRAINS_PROJECT_ID` | recall/search clap **2**; sync `project=(none)` **0** |
| `text` format | recall **and** sync: `text` ≡ pretty |

---

## Risks / stop-before

- Stop if product owner wants `search` → `sync query` instead of recall — re-open F2.
- Stop if product owner wants progressive to search vault memories — that is a new track (F7).
- Do not flip authorized empty to exit 3 (T221 F14).
- Do not edit `POLICY_DENIED_HINT` / daemon twin.
- Do not add clap `env=` to sync (F6).

## Done when

- AC1–AC15 green
- Live F30 recorded
- Docs 3-way table shipped
- Gate green; reviews clean or residual soft only (cap)
- deferred dual-search + T231 search/text closed

## Next after ship

- Soft F22 skill; F23 footer; F24 daemon next_step
- Series: **T245** harness wiring or **T247** nightly status (README order)
- Invalid-env converge only if operators still trip (separate)
