# T266 — Format policy convergence

- **Track ID:** T266-FormatPolicyConvergence
- **Status:** **Pending** (placeholder; plan-only until go)
- **Category:** UX
- **Owner:** —
- **Source:** Audit 2026-08-16 — friction “format policy is a maze”; `project list-paths` **7/5**; `retention plan` default **6/5**
- **Depends on:** T248 retention TTY human; T249 scope TTY human; T246 graph; T255 nightly (pipes stay human); `format_resolve::resolve_human_json_format`
- **Absorbs:** Some commands default human, some JSON, some “pretty JSON”; list-paths/scan-roots always JSON; retention `auto` is JSON on this agent’s non-TTY (looks like “retention dumped JSON”); graph update default pretty JSON
- **Not absorbed:** Warning-on-stdout (T257); preflight JSON shape (T265); nightly pipes-stay-human (T255 closed)

---

## 1. Objective

One documented rule, implemented:

- **TTY:** human (table / labeled) unless `--format json`.
- **Pipe / non-TTY:** JSON unless the command is explicitly human-first (nightly status is the documented exception).
- **`--format human|pretty|text`:** always human, including non-TTY (agents can force pretty).

`list-paths` and `scan-roots` should obey that rule (they are operator inventory, not a frozen HTTP DTO).

## 2. Problem (live 2026-08-16)

Same session, same non-TTY:

| Command | What we got |
|---------|-------------|
| `doctor` / `memory list` / `nightly --status` | human |
| `scope resolve` (no `--format`) | JSON |
| `project list-paths` / `scan-roots` | JSON wall (126 lines) |
| `retention plan` | JSON (TTY human exists via `--format human`, default `auto` flipped) |
| `graph update` | pretty JSON |
| `recall` | JSON unless `--format pretty` |

T248/T249 already taught `auto`. The remaining surfaces never opted in. Operators cannot predict the next command.

## 3. Frozen intent (placeholder)

| ID | Intent |
|----|--------|
| **F0** | Plan-only until go. |
| **F1** | Adopt `resolve_human_json_format` (or sibling) on `list-paths`, `scan-roots`, and any other TTY-capable inventory still hard-JSON. |
| **F2** | CAPABILITIES OutputFormat table lists nightly as the human-pipe exception. |
| **F3** | JSON keys for list-paths/scan-roots stay frozen when `--format json`. |
| **F4** | No clap 5 multi-heading. |

## 4. Verification sketch

- Hermetic: `--format human` list-paths is a table; `--format json` parses.
- Docs table matches behavior.
