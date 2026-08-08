# T220 — Preflight summary JSON honesty

- **Status:** 📋 Placeholder (plan-only until go)
- **Source:** Audit — `preflight --summary --format json` still prints human summary (flag lie)
- **Scores:** usefulness **4** · quality **3**
- **Category:** BUGFIX / CONTRACT
- **Depends on:** T214 dual counts; T180 freeze caution

## Objective

`--summary` + `--format json` emits a **machine object** with Scope + vault counts (+ in-context markers), or **exit 2** if unsupported — never silent human fallback.

## Draft decisions

| F1 | JSON shape CLI-local (prefer no T180 freeze unless contracts) |
| F2 | Keys: `api_version`, `scope`, `project_id`, `pinned`, `active_sessions`, `projects?`, `in_context_*`, `word_count` |
| F3 | Human summary remains default |
| F4 | Hermetic asserts JSON parse + no `--- AI-Brains Preflight Summary ---` |

Closes T214 soft residual F11/F24 summary JSON DTO.
