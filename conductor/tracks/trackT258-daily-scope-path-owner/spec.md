# T258 — Daily Scope = path owner (no silent switch)

- **Track ID:** T258-DailyScopePathOwner
- **Status:** **Pending** (placeholder; plan-only until go)
- **Category:** FEATURE / UX / OPS
- **Owner:** —
- **Source:** Audit 2026-08-16 — non-working default identity; opportunity “rebind daily Scope to `3581317d`”
- **Depends on:** T240 whoami / mismatch warn / no silent rewrite (F2); T206 detect; T233 register-path
- **Blocks / feeds:** Honest scores for recall, preflight, briefing, query, memory. T259 leftover split is separate.
- **Absorbs:** Default Scope `test-alias` `441837f6` (591 mem) vs path owner `C:\dev\ai-brains` `3581317d` (2,673); shell leftover `7d97a456` visible in whoami; new pins landing in the sandbox
- **Not absorbed:** Auto-merge projects; silent `.env` write; splitting `7d97a456` (T259)

---

## 1. Objective

Make the **daily** project for this repo the path-alias owner of `C:\dev\ai-brains` / `C:\dev\AI-Brains` without violating T240 F2 (no silent auto-switch).

Two layers:

1. **Operator remediator (can ship as docs + `whoami` next-action):** rebind project `.env` `AI_BRAINS_PROJECT_ID=3581317d-601e-44f7-ab84-fde90aa12d3c`. No product write.
2. **Product (this track):** a first-class, confirmable remediator so operators do not have to hand-edit UUIDs. Candidate: `project adopt-path` / `project use --path-owner --yes` that prints the exact `.env` line or writes only with `--write-env --yes`.

## 2. Problem (live 2026-08-16)

`project whoami`:

| Signal | Value |
|--------|--------|
| effective / env | `441837f6` test-alias (591 pins, `*` in `project list`) |
| shell (pre-dotenv) | `7d97a456` (18,028 — leftover dump) |
| path alias / detect | `3581317d` `C:\dev\ai-brains` (2,673) |
| git slug | `AI-Brains` |
| mismatch | **true** |

T240 made this *visible*. It did not make daily commands *correct*. `preflight`, `recall`, `memory list`, `briefing`, `query progressive` all scoped to the 591-memory sandbox. Recent session pins from this machine also landed there, so the last two days of decisions are not on the path-owner project.

T240 F2 forbade silent rewrite. That is still right. The hole is **no first-class adopt path**.

## 3. Frozen intent (placeholder)

| ID | Intent |
|----|--------|
| **F0** | Plan-only until go. |
| **F1** | Keep T240 F2: never silently change effective PROJECT_ID. |
| **F2** | First-class remediator names the path owner and the exact rebind. `whoami` next-action is that remediator, not `whoami`. |
| **F3** | Writing `.env` requires `--yes` (and a dedicated flag). Default is print-only. |
| **F4** | Do not merge `441837f6` into `3581317d` in this track (compensating events / import = later if ever). |
| **F5** | Do not alias `7d97a456` as `AI-Brains` (T259 / T267). |

## 4. Verification sketch

- `project whoami` after adopt (print-only) still mismatch until `.env` actually changes.
- `--write-env --yes` hermetic tempdir: PROJECT_ID becomes path owner; no other keys touched.
- Without `--yes`, no file write.
- Capture independence: identity only.
