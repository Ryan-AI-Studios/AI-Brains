# Track review: T268-ScanRootsParent

**Harness:** Antigravity (`agy`)  
**Track:** `conductor/tracks/trackT268-scan-roots-parent`  
**Date:** 2026-08-19  
**HEAD:** `d00fb17`  

---

## Summary

Track T268 addresses the `scan-roots` discovery UX (`project scan-roots` rated 4/5 in audit) by:
1. Adding a named `--root <DIR>` flag that is a clean clap XOR (`conflicts_with = "path"`) with the existing positional `[PATH]` argument while preserving the implicit **cwd** default.
2. Emitting an actionable human remediator hint (`next: ai-brains project scan-roots --root <parent-of-toplevel>`) when an operator runs an implicit-cwd scan inside a git worktree and finds zero unregistered hits.
3. Suppressing redundant `register-path` remediator suggestions in the `suggested` column/field for rows that are already registered to a project (human prints `—`, JSON `suggested` is `""`).
4. Maintaining strict dry-run behavior and capture independence (filesystem traversal + `list_path_aliases` lookup only; no event log mutation, no `.env` write, no auto-register).

The plan and specification are well-scoped, rigorously verified against the live codebase, and adhere to all engineering invariants.

---

## Findings (B/M/m/O)

### Blockers (B)
*None.*

### Major (M)
*None.*

### Minor (m)
- **m1: Pure volume-root predicate for `parent_scan_hint` (AC12 / F21):** On Windows, `Path::new(r"C:\dev").parent()` is `Some(Path::new(r"C:\"))`, whose parent is `None`. On Unix, `Path::new("/home/user").parent()` is `Some(Path::new("/home"))`, whose parent is `Some(Path::new("/"))` (itself having parent `None`). A helper implementation like `toplevel.parent().filter(|p| p.parent().is_some())` handles both cleanly without live filesystem access. Ensure the unit test suite in `project_paths.rs` tests Windows drive root (`C:\`), Windows UNC root (`\\server\share`), and Unix root (`/`) inputs.
- **m2: Error copy on empty `--root ""` (AC11 / F19):** When `--root ""` is passed, dispatch forwards `Some("")` to `scan_roots`, which invokes `fail_usage("scan-roots path is empty; pass a directory or omit to use the current directory")` (exit code 2). This error copy remains accurate and unified for both positional and named `--root`.
- **m3: Consistency of empty `suggested` placeholder (F3 / AC4):** In human tabular output, registered rows will display `—` (Unicode U+2014 em-dash) in the `suggested` column. This matches `registered_to` fallback glyph at `project_paths.rs:309`.

### Opportunities (O)
- **O1: Pure helper design for `parent_scan_hint`:** Structuring `parent_scan_hint(is_implicit_cwd: bool, unregistered_count: usize, git_toplevel: Option<&Path>) -> Option<PathBuf>` as a pure function allows comprehensive unit testing of all decision matrix branches without spawning git subprocesses in unit tests.
- **O2: After-help documentation update (AC13):** Updating `ProjectCommands::ScanRoots` `after_help` in `main.rs` to include `ai-brains project scan-roots --root C:\dev` alongside the existing positional example will ensure discoverability in `ai-brains project scan-roots --help`.

---

## What Looks Solid

1. **Clap XOR Architecture:** Using `#[arg(long, value_name = "DIR", conflicts_with = "path")]` directly mirrors the tested `ProjectCommands::Resolve` pattern (`alias` vs `alias_positional`), providing automatic clap usage error handling (exit 2) when both are supplied.
2. **Git Toplevel vs CWD Parent:** Correctly utilizing `collect_git_identity(scan_root).toplevel.parent()` instead of `cwd.parent()` prevents erroneous hints when invoked from subdirectories inside a repo.
3. **Capture Independence & Dry-Run Integrity:** No event appending, no `.env` alteration, no dependencies on models/embeddings/graph.
4. **Hotspot Restraint:** Rank #1 hotspot `project.rs` is untouched except for reusing `collect_git_identity`; all new logic and helpers remain isolated in `project_paths.rs` and `main.rs`.
5. **Backwards Compatibility:** Positional argument remains functional; default remains cwd; JSON envelope keys (`api_version`, `scan_root`, `truncated`, `roots`) remain unchanged.

---

## Deferred Fold-In Table

| Deferred Item | Spec/Plan Disposition | Assessment |
|---------------|------------------------|------------|
| `scan-roots` cwd-only (4/5) | Absorbed into DoD (F1–F3 / AC1–AC7) | Fully addressed with `--root` XOR and human parent hint |
| Already-registered `suggested` | Absorbed into DoD (F3 / AC4) | Suppresses redundant register-path commands |
| T254 F21 cwd default | Affirmed in F15 | Correctly rejects changing global default to `C:\dev` |
| T254 F20–F23 scan bounds | Affirmed in F5 | Bounds (.ledgerful only, 200 cap) preserved |
| Leftover `7d97a456` root ownership | Declined in F12 | Correctly left to operator `rebind-path` (T259) |
| Auto-register / `--apply` | Declined in F13 | Preserves dry-run safety invariant |
| Recurse / cap increase | Declined in F14 | Out of scope; immediate children only |

---

## Last-PR Cursor Comments

- **Scanned PR:** [#183](https://github.com/Ryan-AI-Studios/AI-Brains/pull/183) (merged 2026-08-19, T271 `sync query ledger pane`).
- **Cursor Bugbot Comment:** Medium finding on `crates/ai-brains-cli/src/commands/sync_query_ledger.rs:157` regarding raw query strings starting with dashes (e.g. `--limit`, `--days`) being parsed as Ledgerful flags rather than query positionals.
- **Disposition:** Spec §2.1 and §9 correctly identified that this issue is specific to `sync query` command execution and does not belong in T268. A dedicated track **T273** (`trackT273-sync-query-ledger-dash-flags`) was minted and registered in `conductor/conductor.md`.

---

## Research / Tools Notes

- **`clap`:** Verified locked at `4.6.1` (`clap_builder` 4.6.0, `clap_derive` 4.6.1). `conflicts_with` derive macro attribute functions as expected with positional optional fields.
- **`serde_json`:** Verified locked at `1.0.150`. Serializing `suggested: ""` preserves key presence for existing JSON consumers.
- **Toolchain / Rust:** Verified `1.95.0` (Edition 2024), workspace `0.1.1`.
- **`ledgerful` / `ai-brains`:**
  - `ai-brains preflight --summary`: Project `3581317d`, 3,130 pinned memories, 2 active sessions.
  - `ledgerful ledger status --compact`: 0 pending, 0 unaudited drift.
  - `ledgerful search scan_roots`: Identified callsites in `project_paths.rs` and `project_path_aliases.rs`.

---

## Verdict: Planned

The plan is approved as **Planned** with minor suggestions for test coverage (`parent_scan_hint` root edge cases). Implementation should proceed under TDD once the user issues `/implement-track`.
