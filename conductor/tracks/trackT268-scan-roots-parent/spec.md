# T268 — scan-roots default parent / `--root`

- **Track ID:** T268-ScanRootsParent
- **Status:** **Pending** (placeholder; plan-only until go)
- **Category:** UX
- **Owner:** —
- **Source:** Audit 2026-08-16 — `project scan-roots` **4/5**
- **Depends on:** T254 scan-roots dry-run
- **Absorbs:** cwd-only scan from `C:\dev\AI-Brains` finds itself and suggests `register-path` for a path already owned by `3581317d`; operator roots live under `C:\dev`
- **Not absorbed:** Actually registering (write); leftover split (T259); format human (T266)

---

## 1. Objective

`scan-roots` must discover **sibling** ledgerful repos the operator cares about, not only “the repo I am already in.”

## 2. Problem (live 2026-08-16)

```
scan_root: C:\dev\AI-Brains
roots: [ { path: C:\dev\AI-Brains, registered_project_id: 3581317d, suggested: register-path 3581317d C:\dev\AI-Brains } ]
```

Help: “Discover immediate child directories that contain `.ledgerful`.” From inside a repo there are no such children. The useful scan is **parent** `C:\dev` (immediate children: ledgerful, stl, coordinator, …). Effectiveness 4.

## 3. Frozen intent (placeholder)

| ID | Intent |
|----|--------|
| **F0** | Plan-only until go. |
| **F1** | Add `--root <dir>` (default remains cwd for scripts). |
| **F2** | Default *human* next-step when cwd is a git worktree: also print “try `--root <parent>`” if zero *unregistered* children. |
| **F3** | Do not suggest `register-path` for a path already registered to that project. |
| **F4** | Still dry-run; never writes. |

## 4. Verification sketch

- Hermetic: already-registered root not in `suggested`.
- `--root` parent lists children with `.ledgerful`.
