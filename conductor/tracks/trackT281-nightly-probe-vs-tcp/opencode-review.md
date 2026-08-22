# T281 — Nightly Completion timeout vs HTTP: one-line obvious — opencode plan review

- **Track:** T281-NightlyProbeVsTcp (OPS/UX/HONESTY) — **Planned** (F0 until go)
- **Absorbs:** `conductor/deferred.md:18` row via the absorption table at `deferred.md:387-406`
- **Head:** `550f3eb` (docs commit), 1 ahead of `origin/main`, clean tree
- **Scope of this file:** plan audit only. No implementation, no folding. Apply is `/fold-in T281`.

---

## Summary

The track removes a genuine double-truth trap. `format_probe_label_human` renders a
Completion probe timeout as the **human label** `timeout (750ms)` (`nightly_status.rs:11-16`),
but the Completion probe **blocks on the daemon TCP connect** (100 ms × 5 → `Open`/`Closed`,
`daemon.rs:749`) — not on the 750 ms HTTP `/health` budget. The label string `timeout (750ms)`
already implies the budget belongs to the Completion timeout even though it never applies to it.
The plan's F1 (`println!("  · HTTP /health 750ms ≠ daemon TCP timeout (100ms×5)")`, 31 chars,
U+2260) makes that visible on the exact line where the Completion probe prints, keyed off the
raw `completion_label == "timeout"` gate (`nightly.rs:195-203`), and **never** prints when `ok`.

The freeze discipline is exemplary: 750 ms is left untouched (T255 F18 freeze), the HTTP-probe
budget is *not* raised, no live `.env`, no `cargo install`, no schtasks mutation, and the
implementation starts a **BUGFIX** ledgerful TX on go. Design is clean: helpers stay in
`nightly_status.rs`, print stays in `nightly.rs`, `project.rs` (#1 hotspot) is not touched.

The BUGFIX TX carries two shifts under the honest "dual truth" framing (no extra pass,
`probe=skipped`):
- Completion view: `timeout (750ms)` → `timeout` (raw label passthrough)
- Embedding view: `skipped` stays `skipped` (no shift; `--quick` is a skip, not a failure)

Live-code anchoring is **thorough and verified**; research is **primary-source and
re-verified** (see Research below). Since the codebase is on Rust 2024, the dev-check script
(which uses `cargo fmt --check`) won't execute a formatter migration as part of the toolchain.

## Verdict: **Approved with 1 Medium guard (fold-in applies it)**

- **B = 0**
- **M = 1** (see M-1 — a call-site contract that must be encoded in the AC2 case list)
- **m = 2** (see m-1, m-2)
- **O = several** (see Observations — all non-blocking)

---

## Findings

### M-1 (Medium) — AC2 None-case list must include the human-wrapped `"timeout (750ms)"` label

**File:** `spec.md` AC2 / `plan.md` (F1 passthrough-None case list)

The spec's AC2 passthrough-None case list covers raw probe labels (`"skipped"`, `"ok"`,
`"down"`, `"error"`, `""`), but **omits the one string that the call site can realistically
produce by mistake**: `"timeout (750ms)"` — the output of `format_probe_label_human("timeout", 750)`
(`nightly_status.rs:384`).

If the F1 call site is mis-wired to pass the human-wrapped `completion_label`
`"timeout (750ms)"` instead of the raw `"timeout"`, the gate is a **silent no-op** — F1 never
prints — and only the observed-data AC10 would ever surface it. That is the exact class of
failure this honest-contrast track exists to prevent.

**Fix for fold-in:** add `#[case("timeout (750ms)")]` to the passthrough-None list, plus a
one-line AC note: *"human-wrapped label must yield None; call site passes the raw
`completion_label`."* The call site must keep using the raw `completion_label`.

### m-1 (minor) — Plan references `dev-check.ps1`; the real script is `scripts/dev-check.ps1`

The plan's Phase 4 verification text cites `dev-check.ps1`. There is no `dev-check.ps1` at
the repo root; the actual script is `scripts/dev-check.ps1`. Fix the path on fold-in.

### m-2 (minor) — AC-numbering drift between plan and in-file doc comments

The hermetic quick-skip test `tests/nightly_status.rs:75-112` carries doc-comment labels
"AC10"/"AC8" while the plan references that same test as AC7/AC8 (`plan.md`,
`--quick` skip section). Align the doc comment to the plan's AC numbers when extending the
test for AC7 (hermetic no-suffix-grow) and AC8 (probe=skipped).

---

## What looks solid

- **Freeze discipline.** 750 ms untouched (T255 F18 freeze), no HTTP-budget raise, no live
  `.env`, no `cargo install`, no schtasks mutation, BUGFIX TX starts only on `go`, and the
  plan honestly records the planning-side TX id (`b9b8c77d-3a92-476d-9887-1b7dfeed7fe2`).
- **Honest dual-truth framing.** Completion probes block on the daemon TCP connect
  (100 ms × 5), not on the HTTP `/health` 750 ms budget. The plan names this without trying
  to paper over the pre-existing label semantics.
- **F1 is exactly one line, always correct, never lying when `ok`.** The `== "timeout"` gate
  and the `≠` (U+2260) character are both backed by existing repo precedent
  (`main.rs:1638/:1648/:2262`, `tests/mapping_delta_smoke.rs:81`).
- **Red state is provable.** Grep confirms no existing `HTTP_VS_TCP` /
  `http_vs_tcp_contrast` / `completion_timeout_contrast` symbols, so AC1/AC2 tests are truly
  red at go.
- **TDD shape is the right, smallest slice.** AC1+AC2 (red) are the entire test surface; no
  oversized harness.
- **Hermetic coverage.** The `--quick`/`skipped` JSON and human snapshots run without a
  llama.cpp server (`nightly_status.rs:400-408`), and the `format_probe_label_human` cases
  are rstest `#[case]`-driven already (`nightly_status.rs:387-396`).
- **Research is primary-source and honest.** `#20684` (llama.cpp) is confirmed live — it is
  what `#20799` and `#20817` were built to fix; both are now merged/closed, which *strengthens*
  the plan's claim that `/health` has no special fast-path (dynamic threads were chosen instead).
- **Dependency pins are re-verified against `Cargo.lock`** — no bumps required (see Research).

---

## Deferred absorption (verified)

- `deferred.md:18` carries the T281 row; `deferred.md:387-406` is the T281 absorption table
  (base 2130 → 2131 → 2133; both bounds remapped). ✓
- **No open PR exists for this track.** Open PRs are Dependabot-only (no T285). ✓
- `conductor.md:228` T281 row = Pending (expected until go). ✓
- `conductor/ISSUES.md` does **not** exist (F23 verified); no issue to record. ✓

---

## Last-PR Cursor (plan-time comments)

- `last-PR` **#196**: comments `[]`, reviews `[]`, `api` `0`. Nothing to roll in. ✓

---

## Research / Tools

- **Re-verified live** (fetched, merged Mar 23 2026): llama.cpp PR **#20817**
  "server: use httplib dynamic threads" — the *fix* for #20684; thread says "Fix #20684",
  alternative to #20799 (which is now **closed**). HTTP threads are dynamically allocated
  (`ThreadPool(n_threads_http, 1024)` then `n_threads_http + 1024`); `/health` has **no**
  special handling in the merged version — which is exactly why the plan's "750 ms is the
  budget; the daemon TCP connect is the real gate" contrast is correct.
- **Re-verified primary sources**: k8s probes doc — `tcpSocket` succeeds on port-open, no
  HTTP status; `httpGet` requires 2xx/3xx. HTTP plan cited k8s semantics. ✓
- **Pins re-verified in `Cargo.lock`**: clap 4.6.1, serde_json 1.0.150, chrono 0.4.44,
  rusqlite 0.39.0, uuid 1.23.1, tokio 1.52.3, reqwest 0.13.4 — **no bumps**; no clap 5, no
  rusqlite 0.40. ✓

---

## Fold-in checklist (deliver to `/fold-in T281`)

1. **M-1** — add `#[case("timeout (750ms)")]` to AC2 passthrough-None list + AC note; call
   site must pass raw `completion_label`.
2. **m-1** — plan Phase 4: `scripts/dev-check.ps1` (not `dev-check.ps1`).
3. **m-2** — align `tests/nightly_status.rs:75-112` doc-comment labels to plan AC7/AC8.
4. **O-3** — decide whether to extend the AI-Brains skill for `nightly --status` (F19
   conditional is currently a no-op).

---

## Observations (non-blocking)

- **O-1** — F1's `timeout` arm is covered by the plan's unit tests only indirectly; AC10
  (observed data) is the sole end-to-end check for the printed line.
- **O-2** — `conductor.md:228` row is still Pending; commit at go will set Implemented.
- **O-3** — `.agents/skills/ai-brains/SKILL.md` has no `nightly --status` section, so the
  F19 conditional (render contrast when a guide is attached) is currently a no-op.
- **O-4** — no live `.env`; any budget env mutation is explicitly out of scope (T240 F2).
- **O-5** — HEAD is 1 ahead of `origin/main`; Phase 0 must reconcile/rebase before opening
  the BUGFIX TX at `go`.
