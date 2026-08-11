# T227 — Briefing format honesty + substance

- **Track ID:** T227-BriefingFormatSubstance
- **Phase:** Post-audit CLI quality series (T217–T232) — P2 polish after T226
- **Status:** 🔄 **In Progress**
- **Depends on:** T152 briefings ✅; T202 F7–F9 deny + TTY markdown default ✅; T210 bootstrap ✅; T221 soft briefing deny exit 0 kept ✅
- **Blocks / feeds:** Operators who type `--format human` (preflight/recall vocabulary) stop getting silent JSON; empty allowed briefings stop looking like broken product; next honesty **T228** non-empty recall Scope
- **Category:** UX / FEATURE (light) / DOCS
- **Source:** Non-destructive CLI audit 2026-08-05 — briefing usefulness **4–5** · quality **5–6** (“human→JSON”; empty personal; thin project substance without grants/claims)
- **Deferred absorbed:** deferred.md “Briefing human→JSON; empty personal”; series README T227; T152-R1-08 **partial** (honest empty + next-step — **no** synthetic continuity fill); T202 soft “unknown `--format`” residual **elevated** for briefing only
- **Not absorbed:** #18 session-synthesis continuity fill; inject legacy MemoryPinned / preflight pins into briefing authority; flip briefing soft deny to exit 3 (T221 F7 keep); progressive human pretty (T202 F28); clap 5 / clap ValueEnum DoD; MSI; fix `OutputFormat::parse` silent-JSON for all governed commands (residual); `--quiet` footer suppress
- **Research date:** 2026-08-11 (live dogfood + code truth + clig.dev / clap pins)
- **AI fold-in:** 2026-08-11 — AI1 **M1–M4 hard**; **L1–L3** hard; **O1–O2** hard. AI2 **M1–M4 hard** (preflight share, OutputFormat residual, trim pin, warning-kind doc); **L1–L4/L6** hard; **L5** soft; **O3/O5** elevate DoD; **O1/O2/O4/O6** soft residual. Disposition **§15**.
- **Ledger:** plan-only until go (`ledgerful ledger start` on go)

## 1. Objective

1. **Format honesty:** Explicit human-facing format strings (`human`, `pretty`, `text`, `markdown`, `md`) emit **markdown**, not silent JSON. Unknown formats **fail usage (exit 2)** instead of silently defaulting to JSON.
2. **TTY contract unchanged (T202 F9):** omit `--format` → markdown on TTY, JSON on non-TTY; explicit `--format json` always wins (dogfood/scripts safe).
3. **Granted substance (honest):** With discovery grants + seeded governed **Approved** decision **and** **Active/Confirmed** conclusion, project briefing surfaces claim text in markdown **and** JSON authority sections. **Do not** scrape legacy pin memories into briefing (dual-model honesty: preflight pins ≠ epistemic authority).
4. **Empty personal honesty (T152-R1-08 partial):** When Personal is allowed but continuity/preferences are empty, packet + markdown say so with a **next-step** (no synthetic summary). Denied personal keeps soft exit **0** + `kind=denied`.
5. **Empty allowed project honesty:** When grants allow (`!denied`) but decisions/conclusions are empty, surface a scannable empty reason + next-step — **never** when soft-denied (AI1 M2 / F27).
6. **Capture independence:** no models, embeddings, or graph required for format or empty honesty.
7. **Zero new production crates; no clap pin bump.**

## 2. Live baseline (re-scan 2026-08-11)

### 2.1 Operator dogfood (this machine)

| Command | Observed |
|---------|----------|
| `briefing project` (piped / agent non-TTY) | Pretty JSON packet; `denied: true`; decisions/conclusions `[]` |
| `briefing project --format markdown` | `# Project Briefing` + `> **Denied:** ReadDecisions/ReadConclusions…` |
| `briefing project --format human` | **JSON** (silent fall-through) ← audit surprise |
| `briefing project --format pretty` | **JSON** (same) |
| `briefing project --format md` | Markdown OK |
| `briefing personal` (non-TTY) | JSON denied; `continuity.summary: ""`; preferences `[]` |
| `briefing personal --format markdown` | `# Personal Continuity Briefing` + Denied **without blank line** after Scope; Preferences/Continuity `_None_` — **no next-step** |
| Preflight summary | 51 in-context decisions / 9 constraints (legacy pins) — **orthogonal** to briefing authority |
| Help | Documents `json` or `markdown` only; examples always `--format json` |

### 2.2 Root cause (frozen)

```text
// crates/ai-brains-cli/src/commands/briefing.rs emit_output:
let fmt = resolve_briefing_format(format, stdout().is_terminal());
// resolve returns raw Some(f) — NO trim (AI2 M3); emit only checks markdown|md case-insensitively
if fmt.eq_ignore_ascii_case("markdown") || fmt.eq_ignore_ascii_case("md") {
    println!("{}", markdown());
} else {
    println!("{}", json()?);  // human | pretty | text | " markdown" | garbage → JSON
}
```

| Gap | Detail |
|-----|--------|
| Alias hole | Governed `OutputFormat::parse` maps `human\|text\|pretty` → Human; briefing only accepts `markdown\|md` |
| No trim | `Some(f) => f` raw; `" markdown"` falls through to JSON today |
| Unknown format | Silent JSON (T202 F1 soft residual) |
| Substance model | Approved decisions + Active/Confirmed conclusions; constraints = CONSTRAINT/INVARIANT scrape. **Not** MemoryPinned |
| Empty personal | Continuity always `""` until #18; `_None_` no next-step; personal Denied lacks blank line (AI1 M3) |
| Soft deny | Exit 0 intentional (T221); markdown Denied has no bootstrap next-step |
| **Shared renderer** | `ai-brains-retrieval/src/preflight.rs:236` calls `render_project_markdown` then retags header + `trim_to_word_budget` — **T227 footers flow into governed preflight** (AI2 M1) |

### 2.3 Touch map

| Site | Role |
|------|------|
| `ai-brains-cli/src/commands/briefing.rs` | **F1–F3:** `BriefingFormatKind` + `classify_briefing_format` (trim+lower); `emit_output` → `fail_usage` on Err; rewrite 4 unit tests |
| Soft: `governed_common.rs` | Reuse `fail_usage` only — **do not** change `OutputFormat::parse` in T227 (residual) |
| `ai-brains-control-plane/src/briefings/renderer.rs` | Empty/deny next-step footers; personal blank line before Denied; **shared with preflight** |
| Soft: `briefings/project.rs` / `personal.rs` | Structured `warnings[]` kinds `empty_authority` / `empty_continuity` **only when `!denied`** |
| `ai-brains-contracts/src/briefings.rs` | **F31:** extend `BriefingWarningDto.kind` doc comment with new kinds |
| **`ai-brains-retrieval/src/preflight.rs:236`** | **F29:** affected consumer of `render_project_markdown` — accept flow-through; regression within budget |
| `ai-brains-cli/src/main.rs` | clap help + after_help (human examples) for Project + Personal |
| Hermetic | `tests/briefing_format_substance.rs` — format + AC6 dual seed + soft deny |
| Unit | classify (incl. `" markdown"`); renderer empty/deny/spacing; soft contracts kind round-trip |
| Docs | CAPABILITIES (format table L84 + briefing L271–272); OPERATIONS L199+examples; CLI-EXIT-CODES note unknown→2; CHANGELOG minor; soft PROTOCOL-COMPAT |
| Soft residual | T152-R1-08 typed constraints; #18 synthesis; `OutputFormat::parse` surface-wide |

### 2.4 Deps / pins (researched 2026-08-11)

| Item | Workspace / note |
|------|------------------|
| clap | Workspace **`4.5`** (resolved **4.6.1**); crates.io latest **4.6.6** — **no bump** DoD |
| is-terminal | **0.4** (crates.io **0.4.17**); soft residual → `std::io::IsTerminal` — not DoD |
| serde_json | Existing pretty packet emit — additive warnings only |
| Zero new crates | No pager, table, or CLI framework swap |

### 2.5 Online / product research

| Finding | Application |
|---------|-------------|
| [clig.dev](https://clig.dev/) — human-readable paramount; TTY heuristic | Keep F4; fix human alias; refuse silent unknown |
| clig — consistency across subcommands | Aliases match preflight/recall vocabulary; note OutputFormat residual widens gap intentionally for T227 scope |
| clig — suggest next step | F8/F9/F10 next-step footers |
| clig — changing human output OK; machine stay stable | Markdown footers free; JSON keys frozen except additive `warnings[]` |
| T202 F9 / T221 | Defaults + soft deny exit 0 unchanged |
| T152 / T170 D21 | No pin inject; briefing JSON remains governed authority probe |

## 3. Frozen decisions (F1–F36)

| ID | Decision |
|----|----------|
| **F1 — Human aliases → markdown (hard / AI1 M1)** | Explicit `--format` (case-insensitive, **trim**): `human`, `pretty`, `text`, `markdown`, `md` → **markdown**. Never JSON. |
| **F2 — JSON explicit only (hard)** | Only known JSON token is `json`. Non-TTY + omit still JSON (F4). |
| **F3 — Unknown format → exit 2 (hard / AI1 M1)** | Unrecognized → `fail_usage` **stderr** + exit **2**; **zero stdout** (no JSON pollution). Message lists accepted values. |
| **F4 — TTY default unchanged (hard)** | `None` + TTY → markdown; `None` + non-TTY → json. |
| **F5 — No soft-deny exit flip (hard)** | `denied=true` → process exit **0** (T221). |
| **F6 — No pin injection (hard)** | Do not copy MemoryPinned into briefing authority. Document dual model. |
| **F7 — Granted substance proof (hard / AI2 O5)** | Hermetic: seed grants for **`cli_principal()` System UUID** (AI2 L1) + **Approved decision AND Active/Confirmed conclusion** → markdown + JSON include both statements. |
| **F8 — Empty allowed project honesty (hard / AI1 M2)** | When **`!denied`** and decisions+conclusions empty: markdown empty-authority notice + next-step; prefer `warnings[]` `kind: "empty_authority"`. **Never** when `denied==true`. |
| **F9 — Empty personal honesty (hard)** | When **`!denied`** and continuity empty: next-step footer; prefer `warnings[]` `kind: "empty_continuity"`. **No synthetic summary.** |
| **F10 — Denied markdown next-step (hard)** | When `denied=true`, after Denied blockquote: one bootstrap next-step line (mirror `POLICY_DENIED_HINT` / bootstrap SOOT if free). |
| **F11 — Capture independence (hard)** | Offline without embed/completion. |
| **F12 — Zero new crates / no pin bumps (hard)** | clap / is-terminal stay. |
| **F13 — Contracts additive (hard)** | Warnings kinds only; no required new top-level packet fields. |
| **F14 — Help honesty (hard / AI1 M4)** | Doc string: `human, pretty, text, markdown, md, or json (default: markdown on TTY, json otherwise)`. after_help: human **and** json examples for project + personal. |
| **F15 — TDD (hard)** | Red → green for all ACs. |
| **F16 — Docs (hard / AI2 L2)** | Enumerate sites: CAPABILITIES format table + Briefing rows; OPERATIONS format + examples; CLI-EXIT-CODES unknown→2 note (soft row); CHANGELOG minor. Soft: PROTOCOL-COMPAT additive kinds (O4). |
| **F17 — Determinism (hard)** | Stable next-step strings. |
| **F18 — Parallel-friendly (soft)** | Low conflict with T228 if T228 avoids briefing/renderer. |
| **F19 — is-terminal migrate (soft residual)** | Not DoD. |
| **F20 — ValueEnum (soft residual / AI2 O2)** | Not DoD; string classify OK. |
| **F21 — #18 continuity (out)** | Synthesis stays deferred. |
| **F22 — Typed constraints (out)** | Substring scrape stays. |
| **F23 — Progressive human (out)** | T202 F28 residual. |
| **F24 — Review (hard)** | Primary required; cross-model soft if F1+F7+F9 land. |
| **F25 — fail_usage SOOT (hard)** | Existing `governed_common::fail_usage`. |
| **F26 — Case / trim (hard / AI2 M3)** | **Current code returns raw `Some(f)` without trim** — rewrite **MUST** `trim().to_ascii_lowercase()` before match (parity T220 + `OutputFormat::parse`). Unit: `" markdown"` → Markdown. |
| **F27 — Empty only when allowed (hard / AI1 M2)** | Elevate: empty_authority / empty_continuity notices **only when `!packet.denied`**. Partial-grant section warnings unchanged. |
| **F28 — BriefingFormatKind enum (hard / AI1 O1)** | `pub(crate) enum BriefingFormatKind { Markdown, Json }` — type-safe routing; rewrite existing 4 string unit tests to assert enum (AI2 L3). |
| **F29 — Preflight flow-through (hard / AI2 M1)** | Accept: `render_project_markdown` changes appear in governed preflight. **Do not** split renderer unless budget truncates footers unrecoverably. Add regression: governed preflight empty-authority or deny path retains next-step token within default budget (or place next-step early enough that budget keeps it). Note dual “bootstrap” line may appear in both surfaces — acceptable consistency. |
| **F30 — Personal Denied spacing (hard / AI1 M3)** | `render_personal_markdown`: blank line before `> **Denied:**` (parity with project renderer) and before next-step line. |
| **F31 — Warning kind contract doc (hard / AI2 M4)** | Update `BriefingWarningDto.kind` doc at `briefings.rs:102` to include `empty_authority \| empty_continuity`. Soft: round-trip assert in contracts tests. |
| **F32 — emit_output error type (hard / AI2 L4)** | `emit_output` must return `Result<(), Box<dyn Error>>` (or call site maps `fail_usage`) so unknown-format path is not forced through `serde_json::Error`. |
| **F33 — AC6 principal SOOT (hard / AI2 L1)** | Seed grants for existing `cli_principal()` System sentinel `0xA1_B2…` (same as preflight); reuse T210/T221 bootstrap helpers — do not invent a new principal. |
| **F34 — OutputFormat residual (hard note / AI2 M2)** | Document in §11: silent-JSON-on-unknown remains for non-briefing governed commands via `OutputFormat::parse` `_ => Json`. T227 F3 is **briefing-only**. Future track may add `parse_or_fail`. |
| **F35 — dogfood_compare / perf (soft / AI2 L5–L6)** | Sanity-check `dogfood_compare` fixture if empty_authority appears on allowed-empty packets; perf harness bypasses CLI format (content only if CP warnings added). |
| **F36 — No --quiet in T227 (soft residual / AI2 O6)** | Footer suppress flag out of scope. |

## 4. Acceptance criteria

| ID | Criterion | Proof |
|----|-----------|-------|
| **AC1** | `--format human` → markdown (`# Project Briefing` / `# Personal`) | Hermetic project + personal |
| **AC2** | `--format pretty` and `--format text` → markdown | Unit + hermetic |
| **AC3** | `markdown` / `md` → md; `json` → JSON | Unit + hermetic |
| **AC4** | `--format banana` → exit **2**, stderr accepted list, **no** stdout JSON | Hermetic |
| **AC5** | TTY omit → markdown; non-TTY omit → json | Unit classify |
| **AC5b** | `--format " markdown"` (leading/trailing space) → markdown (F26) | Unit |
| **AC6** | Granted + seeded **decision + conclusion** → both statements in md + JSON (F7/F33) | Hermetic |
| **AC7** | Allowed empty project (`!denied`) → empty_authority next-step; **denied** packet must **not** also emit empty_authority (F27) | Unit renderer + soft hermetic |
| **AC8** | Allowed empty personal → empty_continuity next-step; no invented summary | Unit/hermetic |
| **AC9** | Denied markdown includes bootstrap next-step (F10) | Unit renderer |
| **AC9b** | Personal denied markdown has blank line before Denied (F30) | Unit |
| **AC10** | Soft deny exit still **0** without grants | Hermetic (T221 lock) |
| **AC11** | Help lists aliases + human example; soft: Usage shows optional `--format` | Hermetic help |
| **AC12** | CAPABILITIES dual-model + aliases; OPERATIONS; CHANGELOG minor; kind doc F31 | Diff |
| **AC13** | No pin-injection: raw memories alone do not invent briefing decisions | Soft unit / doc |
| **AC14** | Governed preflight still OK after renderer change; next-step token survives default budget when empty/deny (F29) | Unit or hermetic preflight_governed |
| **AC15** | Full gate green; ledger clean | Gate |

## 5. Non-goals

- Session continuity synthesis (#18)
- MemoryPinned → briefing authority bridge
- Flip briefing deny to exit 3
- Progressive/expand human pretty format
- Fix `OutputFormat::parse` for all governed commands (residual F34)
- clap 5, ValueEnum DoD, MSI, notarization
- Changing DefaultPolicyEvaluator matrix
- Auto-bootstrap on briefing deny
- Growing packet with pin sections
- `--quiet` / footer suppress flag

## 6. Verification plan

### 6.1 Targeted (during work)

```powershell
cargo nextest run -p ai-brains-cli --test briefing_format_substance
cargo nextest run -p ai-brains-cli briefing
cargo nextest run -p ai-brains-control-plane briefing
cargo nextest run -p ai-brains-retrieval preflight_governed
cargo clippy -p ai-brains-cli -p ai-brains-control-plane -p ai-brains-retrieval --all-targets -- -D warnings
```

### 6.2 Manual dogfood (on implement)

```powershell
ai-brains briefing project --format human
ai-brains briefing project --format pretty
ai-brains briefing project --format banana   # expect exit 2
ai-brains briefing personal --format human
# After policy bootstrap + decision/conclusion seed:
ai-brains briefing project --format markdown
ai-brains briefing project --format json
# Soft: governed preflight body still scannable with footers
ai-brains preflight --pretty -m 800
```

### 6.3 Full gate (before closeout)

```powershell
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit ; ledgerful verify --scope full
```

## 7. Risks

| Risk | Mitigation |
|------|-------------|
| Scripts rely on silent JSON for bad formats | F3 intentional; CHANGELOG minor |
| Operators expect pins in briefing | F6 dual-model docs + empty next-step |
| Empty + Denied double noise | F27 hard gate on `!denied` |
| Preflight budget truncates footer | F29 place next-step early or assert within budget |
| Hermetic principal mismatch | F33 reuse `cli_principal` System |
| Shared renderer surprises | F29 accept + AC14 regression |
| dogfood_compare fixture drift | F35 sanity check |

## 8. Suggested implement order

1. Red: classify units (aliases, trim, unknown Err); rewrite old resolve tests.  
2. Green: `BriefingFormatKind` + classify + emit/`fail_usage` + help.  
3. Red: renderer spacing/empty/deny; AC6 dual seed; AC14 preflight.  
4. Green: renderer + optional CP warnings (`!denied` only) + contracts kind doc.  
5. Docs (enumerated sites) + dogfood_compare sanity.  
6. Gate + review.

## 9. Wire sketch (pin on go)

```rust
// briefing.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BriefingFormatKind { Markdown, Json }

fn classify_briefing_format(explicit: Option<&str>, is_tty: bool) -> Result<BriefingFormatKind, String> {
    match explicit.map(|s| s.trim().to_ascii_lowercase()).as_deref() {
        None => Ok(if is_tty { BriefingFormatKind::Markdown } else { BriefingFormatKind::Json }),
        Some("json") => Ok(BriefingFormatKind::Json),
        Some("markdown") | Some("md") | Some("human") | Some("pretty") | Some("text") => {
            Ok(BriefingFormatKind::Markdown)
        }
        Some(other) => Err(format!(
            "unknown --format '{other}' (accepted: human, pretty, text, markdown, md, json)"
        )),
    }
}

fn emit_output(...) -> Result<(), Box<dyn std::error::Error>> {
    match classify_briefing_format(format, std::io::stdout().is_terminal()) {
        Ok(BriefingFormatKind::Markdown) => { println!("{}", markdown()); Ok(()) }
        Ok(BriefingFormatKind::Json) => { println!("{}", json()?); Ok(()) }
        Err(msg) => fail_usage(msg).map_err(|e| e.into()), // or map via GovernedCliError
    }
}
```

Empty notices (renderer and/or CP):

```rust
if !packet.denied && packet.decisions.is_empty() && packet.conclusions.is_empty() {
    // empty_authority warning + markdown next-step only
}
```

## 10. Deferred roll-up (this plan)

| Item | Disposition |
|------|-------------|
| Briefing human→JSON; empty personal | **DoD** |
| T152-R1-08 empty personal | **Partial:** honesty + next-step; synthesis out |
| T202 unknown format soft | **Elevate briefing-only** (F3) |
| OutputFormat silent-JSON surface | **Residual F34** (not fixed) |
| T152-R1-08 typed constraints / #18 | **Out** |
| ValueEnum / is-terminal / --quiet | **Soft residual** |

## 11. Residual after T227 (expected)

- **OutputFormat::parse** silent-JSON-on-unknown for source/review/policy/decision/conclusion/evidence/retention/erasure/scope (AI2 M2 / F34) — future `parse_or_fail` track  
- #18 personal continuity synthesis  
- Typed constraint projection  
- clap ValueEnum + `std::io::IsTerminal`  
- Progressive human pretty (T202 F28)  
- T228 non-empty recall Scope  
- Soft: PROTOCOL-COMPAT kind list; `--quiet` footer suppress; shared `parse_or_fail` (AI2 O1)

## 12. AI fold-in disposition (2026-08-11)

### AI1

| ID | Disposition |
|----|-------------|
| **M1** classify + fail_usage | **Accept hard** → F1–F3, F25, F28, wire sketch |
| **M2** empty_authority only when !denied | **Accept hard** → F8, F27 elevate, AC7 |
| **M3** personal Denied blank line | **Accept hard** → F30, AC9b |
| **M4** clap help + after_help | **Accept hard** → F14 |
| **L1** trim/lowercase | **Accept hard** → F26, AC5b |
| **L2** dual-model CAPABILITIES | **Accept hard** → F16 |
| **L3** expand unit tests | **Accept hard** → F28, Phase 1 |
| **O1** BriefingFormatKind | **Accept hard** → F28 |
| **O2** hermetic grant + decision | **Accept hard** → F7 (extend with conclusion per AI2 O5) |

### AI2

| ID | Disposition |
|----|-------------|
| **M1** preflight shares renderer | **Accept hard** → F29 flow-through (option a) + AC14; touch map preflight.rs |
| **M2** OutputFormat silent-JSON residual | **Accept hard as residual note** → F34, §11 — **not** fixed in T227 |
| **M3** trim missing today | **Accept hard** → F26 pin + unit |
| **M4** BriefingWarningDto.kind doc | **Accept hard** → F31 |
| **L1** AC6 System principal | **Accept hard** → F33 |
| **L2** enumerate 4 doc sites | **Accept hard** → F16 |
| **L3** rewrite 4 existing unit tests | **Accept hard** → F28 |
| **L4** emit_output error type | **Accept hard** → F32 |
| **L5** perf harness | **Soft** → F35 |
| **L6** dogfood_compare fixture | **Accept soft-hard** → F35 check on implement |
| **O1** shared parse_or_fail | **Soft residual** → F34 future track |
| **O2** ValueEnum elevate | **Decline DoD** → F20 soft residual |
| **O3** help Usage --format optional | **Soft elevate** → AC11 soft |
| **O4** PROTOCOL-COMPAT | **Soft** → F16 soft |
| **O5** seed decision **and** conclusion | **Accept hard** → F7, AC6 |
| **O6** --quiet footers | **Decline** → F36 residual |

**Rejected / out:** pin injection, deny exit 3, surface-wide OutputFormat fix as T227 DoD, ValueEnum as DoD.

---

**Plan-only.** Say **go** to implement.
