# T227 — Briefing format honesty + substance — Plan

**Status:** 🔄 In Progress  
**Category:** UX / FEATURE  
**Depends:** T152 · T202 · T210 · T221  
- [x] `ledgerful ledger start T227-briefing-format-substance` (on go)

## Goal

Stop silent **human→JSON** on `briefing project|personal`, keep T202 F9 TTY defaults, prove **granted substance** (decision + conclusion), make **empty allowed / empty personal / denied** honest with next-steps — no synthetic fill, no pin injection. Accept renderer flow-through into **governed preflight**.

## Absorbed deferred

| Item | Disposition |
|------|-------------|
| deferred.md “Briefing human→JSON; empty personal” | **DoD** |
| Series README T227 | **DoD** |
| T152-R1-08 empty personal | **Partial:** honesty + next-step |
| T202 unknown format soft | **Elevate briefing-only** |
| T221 residual T227 format | **Absorb** |

**Not absorbed:** pin→authority; deny exit 3; progressive human; typed constraints; #18; clap ValueEnum DoD; **OutputFormat::parse surface-wide** (F34 residual); --quiet.

## Research pins (2026-08-11)

| Pin | Evidence |
|-----|----------|
| Live human→JSON | `--format human` / `pretty` → JSON; md OK |
| Fall-through | emit only markdown/md |
| **No trim today** | `Some(f) => f` raw — `" markdown"` → JSON (AI2 M3) |
| F9 units OK | None+tty/non-tty tested |
| Soft deny exit 0 | T221 AC10 |
| **Shared renderer** | `preflight.rs:236` `render_project_markdown` + retag + word budget (AI2 M1) |
| Personal spacing | No blank before Denied (AI1 M3) |
| Warning kind doc | `stale \| disputed \| … \| other` — missing empty_* (AI2 M4) |
| OutputFormat residual | `_ => Json` on ~10 governed cmds (AI2 M2) |
| Deps | clap 4.5/4.6.1; is-terminal 0.4.17 — no bump |

## AI fold-in pins (hard)

| ID | Pin |
|----|-----|
| **AI1 M1 / F1–F3 / F28** | `BriefingFormatKind` + `classify_briefing_format` → `fail_usage` exit 2; zero stdout on unknown |
| **AI1 M2 / F27** | empty_authority **only** when `!denied` |
| **AI1 M3 / F30** | Personal blank line before Denied |
| **AI1 M4 / F14** | clap docs + after_help human examples |
| **AI2 M1 / F29** | Accept preflight flow-through; AC14 budget regression |
| **AI2 M2 / F34** | Note OutputFormat residual — do **not** fix in T227 |
| **AI2 M3 / F26** | **MUST** trim+lowercase; unit `" markdown"` |
| **AI2 M4 / F31** | Update `BriefingWarningDto.kind` doc |
| **AI2 L1 / F33** | AC6 seed grants for `cli_principal` System `0xA1_B2…` |
| **AI2 L2 / F16** | Enumerate CAPABILITIES (2 rows), OPERATIONS, CLI-EXIT-CODES note, CHANGELOG |
| **AI2 L3** | Rewrite existing 4 resolve unit tests to enum |
| **AI2 L4 / F32** | emit_output error type unifies with fail_usage |
| **AI2 O5 / F7** | Hermetic seed **decision + conclusion** |
| **AI1 L1 / L3 / O1–O2** | Folded into F26/F28/F7 |

**Soft:** L5 perf; L6 dogfood_compare check; O1 parse_or_fail residual; O2 ValueEnum residual; O3 help Usage soft; O4 PROTOCOL-COMPAT soft; O6 --quiet out.

See `spec.md` §15 full disposition.

## Frozen decision index

See `spec.md` §3 **F1–F36**. Hard summary:

1. Aliases → markdown; only `json` → JSON; unknown → exit 2 (F1–F3).  
2. **trim + lower** before classify (F26).  
3. TTY default + soft deny 0 unchanged (F4–F5).  
4. No pin inject (F6).  
5. AC6 dual seed + System principal (F7/F33).  
6. Empty notices only when allowed (F8/F9/F27).  
7. Denied bootstrap next-step (F10).  
8. Preflight accepts renderer flow-through (F29).  
9. Personal Denied spacing (F30).  
10. Contracts kind doc (F31).  
11. OutputFormat residual documented (F34).  

## Task checklist

### 0. Preflight (on go)

- [x] `ledgerful doctor` + `ledgerful ledger status --compact`
- [x] `ledgerful ledger start T227-briefing-format-substance --category FEATURE --message "briefing format honesty + substance"`
- [x] `ledgerful scan --impact` (include `briefing.rs`, `renderer.rs`, `preflight.rs`, contracts briefings)
- [x] Confirm clean ledger / tree

### 1. Red — format honesty (AI1 M1, AI2 M3/L3)

- [x] **Rewrite** 4 existing `resolve_briefing_format` unit tests → `classify_briefing_format` + `BriefingFormatKind`
- [x] Unit: human/pretty/text/md/markdown → Markdown; json → Json; None+tty/non-tty
- [x] Unit AC5b: `" markdown"` / `"HUMAN"` → Markdown
- [x] Unit: banana → Err with accepted list
- [x] Hermetic AC1–AC4: human/pretty → `# Project`; banana exit 2 + empty stdout
- [x] Hermetic AC11: help lists aliases

### 2. Green — format wire (AI1 M1/M4, AI2 L4)

- [x] Implement `BriefingFormatKind` + `classify_briefing_format` (trim+lower **required**)
- [x] `emit_output` → Result with `fail_usage` on Err (**error type** F32)
- [x] `main.rs` help + after_help (project + personal)
- [x] Soft AC11: Usage optional `--format`

### 3. Red — substance + empty honesty (AI1 M2, AI2 O5/L1)

- [x] Hermetic AC6: seed grants for **System cli_principal** + Approved decision **+** Active/Confirmed conclusion → both in md+JSON
- [x] Unit AC7: empty_authority only when !denied; denied must not get empty_authority
- [x] Unit AC8–AC9/AC9b: empty personal; denied next-step; personal blank line
- [x] Hermetic AC10: soft deny exit 0
- [x] AC14: preflight governed path still matches Denied/_None_/denied OR new next-step token within budget

### 4. Green — renderer / warnings / contracts (AI1 M2/M3, AI2 M1/M4)

- [x] `render_project_markdown` footers (empty + deny next-step); keep next-step near top of trailer so budget keeps it
- [x] `render_personal_markdown` blank line + footers
- [x] Optional CP warnings empty_* **only when !denied**
- [x] F31: `BriefingWarningDto.kind` doc + soft round-trip
- [x] F35: check dogfood_compare fixture if needed
- [x] Confirm preflight_governed tests still pass (F29)

### 5. Docs (AI2 L2)

- [x] CAPABILITIES: format table (~L84) + Briefing format / Denied rows (~L271–272) — aliases + dual model + empty honesty
- [x] OPERATIONS: format line + examples
- [x] CLI-EXIT-CODES: note unknown `--format` → exit 2 for briefing (soft adjacent to soft-deny row)
- [x] CHANGELOG minor
- [x] Soft: PROTOCOL-COMPAT additive kinds; skill one-liner

### 6. Review + gate

- [x] Primary review vs spec (CLEAN; P3 test tighten fixed)
- [x] Cross-model Codex R1 (product PASS; process P2 = closeout after merge)
- [x] Full gate + `ledgerful verify --scope full` (2548 + deny/audit)
- [x] Manual dogfood evidence (help aliases; banana exit 2)
- [ ] conductor ✅; deferred strike; series README; ledger commit; pin if non-obvious (post-merge closeout)

## Manual dogfood (record on implement)

```powershell
ai-brains briefing project --format human
ai-brains briefing project --format pretty
ai-brains briefing project --format banana
ai-brains briefing personal --format human
ai-brains policy bootstrap
# seed decision+conclusion if live proof beyond hermetic
ai-brains briefing project --format markdown
ai-brains preflight --pretty -m 800
```

## Out of scope checklist

- [ ] #18 continuity synthesis  
- [ ] MemoryPinned injection  
- [ ] Briefing deny → exit 3  
- [ ] OutputFormat::parse surface-wide  
- [ ] ValueEnum / --quiet  

## Residual after close

- F34 OutputFormat silent-JSON; #18; typed constraints; is-terminal; T228 Scope; soft parse_or_fail track  
