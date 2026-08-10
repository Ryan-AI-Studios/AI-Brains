# T219 Plan — Preflight pretty readability

**Status:** 🔄 In Progress (implementation)  
**Category:** UX / BUGFIX  
**Depends:** T214 ✅, T032 ✅, T216 ✅, T220 ✅ (orthogonal), T180 ✅  
**Spec:** [spec.md](./spec.md) — includes AI fold-in **§14**

## Goal

1. **Fix newline collapse** in `trim_to_word_budget` (root cause of single-line pretty/JSON wall).  
2. **F2b truncation sentinel** `…` when over budget (JSON honesty for agents).  
3. **Human pretty path:** Scope header (T214 F24 / F6b CLI-only alias), display-only role strip (`&str` helper), section caps + **per-section** F31 next-step notices.  
4. **JSON:** T180 2-key compact; `text` gains newlines + F2b (no Scope/caps chrome).  
5. **Share** `strip_role_prefix` with `preview_line` **and** `content_has_tag` (prep T224).  
6. **Governed:** F1 applies; formatter must not mangle `##` (AC14).  
7. Zero new crates; marker selection unchanged; summary path unchanged.

## Absorbed deferred / audit / research / AI fold-in

| Source | Item | Handling |
|--------|------|----------|
| deferred.md T219 wall | quality 5 pretty body | This track |
| series README T219 | Preflight pretty readability P2 | This track |
| T214 F24 soft | Scope header on full preflight body | **F6 hard** |
| T220 F22/F23 | “not T219” residual | Closed by this plan |
| Placeholder draft | role strip, section budgets, JSON full | Expanded F1–F42 |
| Live dogfood 2026-08-09 | single-line body; ASSISTANT×29; CONSTRAINT jammed | F1 + F7 + F9 |
| word_budget.rs | `split_whitespace` + `join(" ")` | **F1 root fix** |
| clig.dev | human-first; say enough; next-step hints | **F31 per-section** |
| clap 4.6.6 / pin 4.5→4.6.1 | no bump | F16/F41 |
| T216 preview_line | case-sensitive role strip | F7/F8/AC9 |
| **AI1** | Architecture dual-path diagram + ACs | Affirmed |
| **AI1 #1** | Token invariance | F1 + AC16 |
| **AI1 #2** | Orphan headers | F37 + AC18 hard |
| **AI1 #3** | Mid-line strip | F39 |
| **AI1 #4** | Summary/governed isolation | F12/F40 |
| **AI2 M1** | JSON truncation sentinel | **F2b + AC15 hard** |
| **AI2 M2** | Governed `##` survival | **F14 + AC14 hard** |
| **AI2 M3** | `strip_role_prefix` → `&str` + dual converge | **F7/F8 hard** |
| **AI2 L2/O3** | +N wording honesty | **F31 hard** |
| **AI2 L3** | CRLF | F2 + AC17 |
| **AI2 L4** | No PrettyOpts for soft compact | F11/F29 |
| **AI2 L5** | Scope lookup CLI-only | F6b |
| **AI2 O1/O2/O4** | Docs + pure units + invariant | F3/F20/F19/AC16 |
| **AI2 L1** | truncate_turn cosmetic | Soft F38 |

**Not absorbed as DoD:** T224 full search strip; T228 recall Scope; `--compact` hard; pager; ledgerful-on-global; clap bump; marker policy change; `Scope: Repository:` vocabulary (AI1 draft rejected).

## Live dogfood freeze (2026-08-09)

| Command | Observed |
|---------|----------|
| `preflight --pretty -m 800` | ~1 physical line; headers jammed into CONSTRAINT/DECISION stream |
| Role noise | ~29× `ASSISTANT:`; Memory Index still prefixed |
| `preflight --format json` | 2 keys; `text` also newline-collapsed |
| `preflight --summary` | OK dual model (leave alone) |

## Research freeze (2026-08-09)

| Topic | Note |
|-------|------|
| Root cause | `trim_to_word_budget` space-join |
| Fix locus | retrieval word_budget (JSON + pretty + governed + truncate_turn) |
| F2b | Over-budget append `\n…` (content words ≤ max) |
| Display polish | CLI-only human_mode; const caps; no PrettyOpts required |
| Role strip | `fn strip_role_prefix(line: &str) -> &str`; both memory callers |
| Caps | safety 8 / turns 6 / sessions 3 / index 15 + F31 wording |
| Scope | `format_scope_line` CLI-only (F6b) |
| Governed | preserve `##`; no caps v1 (AC14) |
| clap | 4.5 workspace; 4.6.1 resolved; 4.6.6 latest — no bump |
| is-terminal | 0.4.17 soft migrate residual |

## Implementation sketch (on go)

### Phase 0 — Ledger

```powershell
ledgerful ledger start T219-preflight-pretty-readability --category UX --message "Preflight pretty: newline budget + F2b + Scope + role strip + section caps"
```

### Phase 1 — Red/Green word_budget (F1/F2b)

- [x] Unit tests: AC1 preserve; AC2 structure truncate; AC15 sentinel; AC16 invariant; AC17 CRLF; empty / zero max
- [x] Confirm current impl fails preserve test
- [x] Implement newline-preserving trim + F2b (spec §9)
- [x] Soft F38: `truncate_turn` shape if free
- [x] Run retrieval preflight tests

### Phase 2 — Shared role strip (F7/F8/M3)

- [x] `pub(crate) fn strip_role_prefix(line: &str) -> &str`
- [x] Wire **both** `preview_line` and `content_has_tag`
- [x] Unit AC9

### Phase 3 — Pretty body formatter (F9/F10/F31/F37)

- [x] `format_preflight_pretty_body` using module `const` caps (no PrettyOpts flag plumbing)
- [x] Only `---` headers; orphan omit (AC18); F31 per-section notices (AC6)
- [x] Governed-style `##` pure unit (AC14)
- [x] Role strip on index/session lines

### Phase 4 — Wire CLI human path (F6/F6b)

- [x] Alias via `get_project_by_id` mirror `print_summary`
- [x] Human/pretty: `Scope:` + blank + pretty body
- [x] JSON: raw post-F1 `context.text` only
- [x] Summary: no change

### Phase 5 — Hermetic + protocol

- [x] `tests/preflight_pretty_readability.rs` AC3–AC5/AC7 (+ AC14 if hermetic preferred)
- [x] protocol_compat T180 keys green

### Phase 6 — Docs + gate

- [x] CAPABILITIES (pretty + F2b + governed F1 note + F31)
- [x] CHANGELOG
- [x] Soft skill one-liner if free
- [ ] Full gate + ledgerful verify
- [x] Manual AC13 (debug `ai-brains`: multi-line Scope, no ASSISTANT index, JSON 2-key + newlines)
- [x] `review.md`; internal CLEAN; Codex R1 product clean (P1 process/P2 governance/P3 sentinel addressed)
- [ ] conductor + deferred closeout; ledger commit (after CI merge)

## Task checklist

### Spec / planning

- [x] Expand placeholder → full spec (F1–F42, AC1–AC18)
- [x] Plan with deferred rollup + research freeze
- [x] AI fold-in §14 (M1–M3 hard + L/O)
- [x] Pin DECISION for planning intent
- [x] User **go** before production edits

### Implementation

- [x] Phase 1 word_budget + F2b
- [x] Phase 2 strip helper converge
- [x] Phase 3 pretty formatter
- [x] Phase 4 CLI wire
- [x] Phase 5 hermetic
- [ ] Phase 6 docs + full gate + review + ledger commit (docs/review done; gate+closeout pending)

## Manual test script (post-go)

```powershell
ai-brains preflight --pretty -m 800
# Expect: multi-line; Scope:; no wall of ASSISTANT:; section headers spaced; +N more if dense

ai-brains preflight --format json | ConvertFrom-Json | Select-Object -ExpandProperty text
# Expect: newlines in text; only text+word_count keys

ai-brains preflight --summary
# Expect: unchanged dual summary
```

## Out of scope (reconfirm)

T224 full search strip · T228 recall Scope · marker ranking · ledgerful global · clap bump · contracts growth · `--compact` hard DoD
