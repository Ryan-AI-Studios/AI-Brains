# T257 — Identity warning + JSON stdout hygiene

- **Track ID:** T257-WarningJsonStdoutHygiene
- **Status:** **Pending** (placeholder; plan-only until go)
- **Category:** UX / CONTRACTS-adjacent (CLI-local; no DTO bump required)
- **Owner:** —
- **Source:** Audit 2026-08-16 — friction #1; `scope resolve` **6/5**; `scope --format json` **7/6**; opportunity “warnings on stderr; JSON one object”
- **Depends on:** T240 mismatch warn; T249 scope TTY human; T255 nightly JSON
- **Absorbs:** Same identity warning on nearly every command; warning interleaved with JSON (`{` then warn then fields when streams merge); nightly/policy/scope `--format json` not pipe-safe under `2>&1`
- **Not absorbed:** Fixing the underlying mismatch (T258); leftover `7d97a456` (T259); format-default maze (T266)

---

## 1. Objective

1. Identity mismatch warning is **stderr-only**, **once per process** (T240 F3 already claimed once/process — live still feels like every command).
2. `--format json` stdout is a **single parseable object**. No warning text inside or before `{` on stdout.
3. Human commands may still show the warning, but not in the middle of a dry-run preview or a table.

## 2. Problem (live 2026-08-16)

Almost every non-destructive command printed:

```
Warning: project identity mismatch: daily Scope is '441837f6-…', but path is registered to '3581317d-…'. Run 'ai-brains project whoami'.
```

including `project whoami` itself, `nightly --schedule --dry-run` (between `[dry-run] Would execute:` and the schtasks line), and `--format json` surfaces.

`scope resolve` (default, non-TTY) emitted `{` then the warning then `"api_version"`. Merged stdout/stderr is how agents and PowerShell `2>&1` consume CLI. Copy-paste JSON is invalid.

`scope resolve --format json` had the warning *before* `{` and `"warnings": []` *inside* — the machine object denied the mismatch the human just saw.

## 3. Frozen intent (placeholder)

| ID | Intent |
|----|--------|
| **F0** | Plan-only until go. |
| **F1** | Mismatch warn → stderr only. Stdout JSON never contains the warning string. |
| **F2** | JSON objects that have a `warnings` array should include a structured mismatch entry when the warn fired (so `scope --format json` is honest). |
| **F3** | Keep T240 once/process. Re-verify live: one warn per `ai-brains` invocation, not once per nested helper. |
| **F4** | Dry-run / table / human blocks print the warn *after* the block or on stderr, never mid-block. |
| **F5** | No contracts crate DTO unless an existing CLI-local schema already has `warnings`. |

## 4. Verification sketch

- Hermetic JSON: `scope resolve --format json` stdout parses as one object.
- Nightly `--status --format json` stdout parses (warning, if any, stderr).
- `whoami --format json` (if present) same.
- Capture independence: warn path only.
