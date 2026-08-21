# T275 Plan Review (opencode) — Discovery grants first-run (grant-wall + CLI bootstrap lock)

- **Reviewer:** opencode / deepseek-v4-flash
- **Review kind:** Plan audit only (review-track) — no folding, no implementation, no edits to spec/plan/conductor/deferred/product code.
- **Targets:** `spec.md` (338 lines) + `plan.md` (129 lines), cross-checked against live `src/` at HEAD `c576b58`.
- **Severity scale:** B (blocker) / M (major) / m (minor) / O (observation, low-info).
- **Verdict:** **Planned** — no blockers, no majors. All m findings are informational/deferrable; Phase-0 re-verify on **go** already covers the two that matter.

---

## Summary

The plan is unusually well-grounded. Every code location, line range, pin, PR claim, test name, and live-baseline signal I spot-checked against HEAD matches the plan (or differs by a line, counted as m). The 8/3 hole is real and reproduced: `briefing project --format human` on a 0-of-3-grants vault prints `## Decisions (current authority)` / `_None_` + `## Conclusions` / `_None_` after a `> **Denied:**` block — exactly the "denied looks like an empty vault" misread the track targets. The hermetic DoD (AC4/AC5) is a genuine gap today: `policy_bootstrap.rs` locks `policy check` + `source`/`review` list but **no** `briefing project` and **no** `evidence list` after CLI bootstrap. The plan's frozen decisions (F1–F34) are internally consistent and respect the declared isolation surface (T280/T263/T221, no auto-grant, no hint edits, renderer-only growth).

No B/M findings. Findings below are m only.

---

## Findings

### m1 — Preflight HEAD/`origin/main` row is stale (self-correcting at go)
**Location:** `plan.md:15` (Preflight table) and `spec.md:36`.
**Plan claim:** HEAD `8cb1ce0` T274 `#189`, tree CLEAN, "In sync with `origin/main`".
**Live (2026-08-21):** HEAD = `c576b58` (`docs(conductor): plan T275 grant-wall and CLI bootstrap unlock` — the plan's own docs commit); `origin/main` = `8cb1ce0`; local is ahead by 1. Working tree is CLEAN (`ledgerful scan --impact` CLEAN; `git status --porcelain` empty). `git diff 8cb1ce0 HEAD -- crates/` and `-- Docs/` are **empty** → product `src/` is byte-identical to the claimed baseline. The dogfood numbers (3325 pinned, 0/0/0 in-context, grants 0 of 3, `would_issue`×3, `_None_` on deny) all still reproduce on the live binary.
- Severity: m. Functional drift is zero; only the literal HEAD/remote wording is stale. `plan.md` Phase-0 re-verify (`:40-48`) already re-checks HEAD; this finding is informational.

### m2 – `main.rs` Bootstrap variant line offset (off-by-4)
- **Plan:** spec.md §2.3 "clap Bootstrap … `main.rs` `PolicyCommands::Bootstrap` `:2207`".
- **Live:** enum variant at `main.rs:2211` (dispatch to `run_bootstrap` at `:4204`).
- Severity: m. Purely a citation drift; the variant exists exactly as described (optional `--scope`, `--dry-run`, no new flags).

### m3 – `empty_denied` call-site line drift (`project.rs:218` not `:217`)
- **Plan:** spec.md §2.3 "Denied project packet `project.rs` **`:216–228`**"; ledger search note `project.rs:217`.
- **Live:** the block is `project.rs:216–229`; `ProjectBriefingPacket::empty_denied(…)` is at `:218`, `denial_hint` set at `:224`. The range in the plan (216–228) is the correct functional span; the specific `:217` anchor is 1 line off.
- Severity: m.

### m4 – Grant-wall copy is markdown-only; JSON (default non-TTY) carries no "not empty" text
- **Location:** F2 / F3 / F29 design.
- The default output for a non-TTY `briefing project --format human` is markdown (via `classify_briefing_format` in `briefing.rs:221…`); the default for a script/CI (non-TTY, no `--format`) is **JSON**. The grant-wall const is emitted only in the markdown renderer. JSON consumers after this track still see `denied: true` + `decisions: []` + `denial_hint` (short SOOT, which says "bootstrap" but does not say "vault not empty").
- This is a **designed** limitation: F3 explicitly freezes the JSON surface and the hint string is T280-protected. `denied: true` is honest, so this is not a correctness hole — it is an agent-facing discoverability gap vs the human-facing fix.
- Severity: m (informational; do not change the frozen JSON per F3/F11 unless a follow-up track intentionally extends the denial_hint on deny).

### m5 – Preflight word-budget claim (F29/AC2) is analog, not verified
- **Plan:** AC2 + F29 assert the grant-wall line sits before `## Decisions` so "preflight word budget keeps it" (T227 F29 analog). I verified the order is achievable in `renderer.rs` (the denied branch appends `BRIEFING_DENIED_NEXT_STEP` then the grant-wall before the Decisions/Conclusions headings), but the *preflight* word-budget mechanics are in the CLI (`preflight.rs`), which the plan forbids touching. The order fix is plausible and low-risk; there is no hermetic test that proves the grant-wall survives the budget.
- Severity: m. Acceptable as designed; implement may add one assertion (word budget ≥ grant-wall length) at Phase-4 if cheap.

### m6 – `F16` "Domain in CLI" wording vs. actual owner crate
- **Plan:** F16 "Domain in CLI — Forbidden beyond renderer strings + hermetic."
- **Reality:** the renderer lives in the **control-plane** crate (`crates/ai-brains-control-plane/src/briefings/renderer.rs`), not the CLI. The intent (no policy-evaluator change, strings only) is correct and matches the touch map; the "in CLI" phrasing is a naming slip.
- Severity: m (wording only; no action).

---

## What looks solid (all verified live)

- **Line ranges / file counts** (plan vs `Measure-Object -Line` on HEAD): renderer.rs 497 ✓, project.rs 913 ✓, preflight.rs 2027 ✓, doctor.rs 1738 ✓, governed_common.rs 842 ✓, policy_cmd.rs 356 ✓, briefing.rs 277 ✓.
- **Renderer** `render_project_markdown` denied branch `:65–74`; `_None_` at `:93-94` / `:114-115`; empty-authority footer gated `!packet.denied` `:126-130`. Unit `render_project_markdown__denied__bootstrap_next_step_no_empty_authority` at `:457` (asserts the empty-authority footer is skipped on deny — the exact ancestor of AC1).
- **`run_bootstrap`** `policy_cmd.rs:234-358`: discovery trio `DISCOVERY_CAPS`, `active_grants` probe, `--dry-run` (`would_issue`), sort-by-capability, `register` only-when-missing — matches F7/F15/F30.
- **`cli_principal()`** `briefing.rs:198-212` (env `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` → Human else System `0xA1B2…`); `resolve_principal` `governed_common.rs:463-471` routes `--principal-id` → Human else `cli_principal()` (F31 trap correctly understood: System default when `--principal-id` omitted).
- **`project.rs:216-228`** — `empty_denied` + CP sets `BRIEFING_DENIED_DENIAL_HINT` (short SOOT). Matches "CP sets; contracts leave None."
- **Hint twins** frozen: CLI `POLICY_DENIED_HINT` `governed_common.rs:51`, CP `query.rs:93` const byte-equal; daemon twin referenced `services.rs` (not re-opened; plan says leave, F11/F23/F27 already hold). **T243 unit** `policy_denied_hint__wording__unchanged` exists (verified via test references).
- **Short/long SOOT** `:107-111` — show/preflight/JSON-denial_hint use short; doctor uses long (omit `--scope`); matches F9/F14 split.
- **Doctor `check_policy_grants`** `doctor.rs:657-708` — soft warn `<3`, skip on closed/no-scope; live `ai-brains doctor --summary` shows `policy_grants — discovery grants empty (0 of 3)` + warn. Matrix = 15 (ok=9,warn=4,skip=2 → 15 checks, AC14).
- **Preflight grants line** `preflight.rs:118` (short SOOT suffix) + `:911` (JSON `next_step`) — verified; live `preflight --summary` shows `discovery grants empty (0 of 3); …policy bootstrap…` at pinned 3325.
- **Gap claim (T210):** `policy_bootstrap.rs` tests (`policy_bootstrap__after__source_and_review_list_exit_0`, dangerous-cap deny, dry-run no-grant) do **not** call `briefing` or `evidence list`. Confirmed missing.
- **AC4/AC5 hermetic shape:** `briefing project --project-id <uuid>` exists (`briefing.rs:22`); `evidence list --local` exists (`evidence.rs:31/107`); `init --vault-path` pattern used by sibling tests (`governed_first_run_deny_exit.rs`, `progressive__after_system_bootstrap__exit_0_denied_false` at `:190`). F31 (omit `--principal-id`) is correctly specified to match `cli_principal()`.
- **Contracts:** `ProjectBriefingPacket` at `briefings.rs:155` with `denied: bool` (`:182`), `denial_hint: Option<String>` (`:188`), `decisions/`conclusions` `Vec<BriefingClaimDto>` (`:164/`:166`) — E1 shape `[]` on deny, no null, no new keys.
- **Pins (verified live, 2026-08-21):** `clap 4.6.1` (crates.io 4.6.6 — plan correct, no clap5), `serde_json 1.0.150` (crates.io 1.0.151), `chrono 0.4.44` (crates.io 0.4.45), `rusqlite 0.39.0` (crates.io 0.40.2), `uuid 1.23.1`. All "no bump" decisions confirmed.
- **`gh` PR audit:** `#189` — 0 comments, 0 reviews, 0 review-requests. `#188` Bugbot 2 Mediums → T284 (still on `14d42af`). No T285. Open PRs Dependabot-only. No Cursor leftover to mint.
- **Determinism/isolation:** F29 frozen const + order; F20 PowerShell `;`; F3 JSON freeze; F26 cross-model read-only on Phase-1 clean (FEATURE) — all consistent.
- **Ledger/ai-brains self-use during review:** `ledgerful doctor` (5 warns: legacy `.changeguard`, sig-pin, timings 24k rows, `:8081` unreachable — all pre-existing, none grants), `ledgerful ledger status --compact` **0 pending / 0 drift**, `ledgerful scan --impact` **CLEAN**, `ai-brains preflight --summary` matches the plan baseline exactly, `ai-brains recall` surfaces prior review-track dumps (no conflicting decision).

---

## Deferred / fold-in notes (not part of this audit)

- T276 leftover rebind, T280 hint omit-`--scope`, T284 Work/samples, T258 adopt-path identity, T263 H2 — all correctly routed to their owning tracks by F25/F11/F12. `ISSUES.md` does not exist (verified) → deferrals stay in `deferred.md`; no new debt file minted by this plan.
- The plan's own absorb/decline table (spec §9) is complete; nothing was missed in the scan I did of `deferred.md` rows touching grants/bootstrap/briefing.

---

## Research & tools notes

- crates.io API queried for clap/rusqlite/serde_json/chrono (single-line JSON) — matches plan pins.
- `gh` used for PR comment/review counts on `#189`.
- `ledgerful` (status + scan --impact) and `ai-brains` (preflight + recall) used per AGENTS.md — no blockers surfaced.
- No live vault bootstrap was performed (`--dry-run` policy only). No product source changes. No edits to `conductor/` beyond this review file.

---

## Verdict

**Planned** — ready for **go** (implement) with F0 gate. All findings are m/ and non-blocking; the two with shelf-life (m1 HEAD wording, m4 JSON-agent exposure) are already accounted for by the plan's own Phase-0 re-verify and F3 freeze. Proceed to `/fold-in T275` if the owner is satisfied.
