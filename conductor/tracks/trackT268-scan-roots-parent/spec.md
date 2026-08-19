# T268 — scan-roots parent / `--root`

- **Track ID:** T268-ScanRootsParent
- **Status:** **Pending** (requirements written; **Planned**, not Placeholder). Plan-only until go.
- **Category:** UX
- **Owner:** —
- **Source:** Audit 2026-08-16 — `project scan-roots` **4/5**. Live re-scan 2026-08-19.
- **Depends on:** T254 ✅ scan-roots dry-run; T266 ✅ `--format` tokens on ScanRoots
- **Blocks / feeds:** Operators can discover sibling `.ledgerful` roots without inventing a new default. **T273** minted here from last-PR Cursor (not this DoD).
- **Absorbs:** deferred.md “`scan-roots` cwd-only (4/5)”; placeholder F1–F4; already-registered `suggested` is a no-op `register-path`
- **Not absorbed (DoD):** Auto-register / `--apply`; leftover rebind (T259); default scan root = `C:\dev` (T254 F21 stands); T266 format maze; T269/T270/T272; last-PR #183 dash-query (**T273**); clap 5 / pin bumps; contracts DTO; `project.rs` hotspot growth
- **Research date:** 2026-08-19 (source HEAD `a4ac170`; fold-in against `d00fb17`)
- **AI fold-in:** 2026-08-19 `agy-review.md` + `opencode-review.md`. No Blockers/Majors. **Agree:** git spawn fail-open (F22); volume-root/UNC/case (F21/AC12); pure `parent_scan_hint` (F28); Windows hint separators (F29); empty-scan hint is intentional (F2). **Already covered:** empty `--root` copy; `—` glyph; after_help `--root`; `suggested: ""` not null; clap XOR. **Decline:** “`--root` + PATH silently ignores” (XOR already F20); JSON `parent_hint` key (F10); mint T273 again (already minted). Disposition **§13**.
- **Ledger:** planning DOCS TX `7cccacdb-e7fb-41e4-b073-ea4cfb3b3e1a`. Fold-in DOCS TX `52dc7831-9393-4b30-ac82-099bfbf2d435`. Implement starts a FEATURE TX on **go**.
- **Isolation:** Do **not** reopen T254 F20–F23 scan bounds (`.ledgerful` only, include marked scan root, immediate children, cap 200, one-shot HashMap). Do **not** write events, `.env`, or repo-local `.ledgerful`. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** `cargo install`.

---

## 1. Objective

1. **Operators can name the directory to scan.** `project scan-roots --root <dir>` is a named XOR of the existing positional path. Default stays **cwd** (scripts and T254 hermetics keep working).
2. **From inside a git worktree, the human path points at the sibling parent.** When the implicit-cwd scan has **zero unregistered** hits, print `next: ai-brains project scan-roots --root <parent-of-toplevel>` (not a drive root).
3. **Already-registered hits are not a remediator.** `suggested` is empty when `registered_project_id` is set. The row still lists the owner. Operators who need to move a bind use `unregister-path` / `rebind-path` (T254/T259).
4. **Stay dry-run.** Never append events. Never write `.env`. Never auto-register.
5. **Capture independence.** Filesystem + vault alias lookup only. No models, embeddings, or graph.

---

## 2. Live baseline (re-scan 2026-08-19)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `a4ac170` — T271 squash-merged (#183). Tree CLEAN at plan start. `main` == `origin/main`. |
| PATH `ai-brains` | `0.1.1`. `project scan-roots --format human` works (T254+T266 present). **Do not** treat PATH-behind as T268 DoD. |
| `scan-roots` (implicit cwd = `C:\dev\AI-Brains`) | One row: `C:\dev\AI-Brains` registered to `3581317d-…`, disk `ok`, **suggested `register-path 3581317d … C:\dev\AI-Brains`** (already owned). No parent hint. |
| `scan-roots C:\dev` | 17 `.ledgerful` hits (scan root + children). `C:\dev` itself unregistered (`—`) with a `register-path` suggestion. AI-Brains + most siblings already registered; leftover `7d97a456` still owns several `C:\dev\*` roots (T259 operator rebind — **not** this track). |
| `--root` | **Does not exist.** Positional `[PATH]` already does what the 2026-08-16 stub called `--root`. |
| `--format` | T266 `auto` / `pretty` / `human` / `json` / … on `ScanRoots`. Pipes default JSON. **Do not** retune. |
| Last GitHub PR | [#183](https://github.com/Ryan-AI-Studios/AI-Brains/pull/183) T271 (2026-08-19). **Cursor Bugbot Medium:** dash-leading `sync query` strings parsed as Ledgerful flags. Still true at `sync_query_ledger.rs:157`. **Does not fit T268.** Minted **T273**. Open PRs on this HEAD: none (Dependabot remotes only). |
| Identity / doctor | Scope `3581317d`; discovery grants 0 of 3; ledgerful doctor leftover `.changeguard` / sig-pin / timings. Do not “fix” here. |

### 2.2 Why the residual still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| cwd-only *default* (audit 4/5) | Help already documents `scan-roots C:\dev`. Operators inside a repo still run the default and only see themselves. **DoD is a named `--root` + human parent hint**, not a new default. |
| `suggested` on already-registered rows | Live cwd scan tells the operator to re-run `register-path` for a path F21 already owns (conflict / no-op). **DoD F3.** |
| Flip default to `C:\dev` | T254 F21 / open Q4: “`C:\dev` is this machine’s habit, not a product default.” **Decline F15.** |
| Auto-register the 17 roots | T254 F23 / F33. **Decline F13.** |
| Leftover `7d97a456` owning sibling roots | T259 `rebind-path`. Scan must not invent a remediator. **Decline F12.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Clap | `main.rs` `ProjectCommands::ScanRoots` ~2523 | `path: Option<String>` positional; `format: String` T266 parser. **No** `--root`. |
| Dispatch | `main.rs` ~4595 | `scan_roots(&ctx, path.as_deref(), format)` |
| Impl | `commands/project_paths.rs` `scan_roots` | cwd if `None`; empty path → `fail_usage`; include marked root; cap 200; HashMap owners |
| Suggested | `scan_rows_for_hits` ~271–274 | **Always** `register-path {owner-or-placeholder} {display}` |
| Human | `emit_scan_human` | Always prints `suggested` column. Empty hits: `No .ledgerful roots found.` — **no** parent next-step |
| JSON | `ScanRootsJson` | Frozen keys: `api_version`, `scan_root`, `truncated`, `roots[{path, registered_project_id, exists, suggested}]` |
| Git | `project.rs` `collect_git_identity` | `rev-parse --show-toplevel` + origin slug. `GitIdentity.toplevel: Option<PathBuf>` |
| Tests | `tests/project_path_aliases.rs` | AC10–AC12 + already-registered **asserts owner, not suggested**; clap format units in `main.rs` |
| Hotspot | `project.rs` | Rank **#1**. New clap field + hint stay in `main.rs` / `project_paths.rs`. |
| XOR peer | `ProjectCommands::Resolve` `--alias` `conflicts_with = "alias_positional"` | Copy this pattern (T108). |
| Contracts / daemon | none | No path-alias DTO. |

### 2.4 Dependency / standards research (2026-08-19)

**Snapshot — re-verify at execute.**

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** (2026-08-06). **No clap 5.** | **No bump.** Add `--root` + `conflicts_with = "path"`. |
| `serde_json` | lock **1.0.150** | 1.0.x | **No bump.** Empty-string `suggested` keeps the key. |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged |
| nextest | **0.9.140** | — | Unchanged |
| workspace | **0.1.1** | — | **No bump** |
| `camino` | desktop/tauri only (T254 F26) | — | **Do not add** |
| New crates | — | — | **Zero** |

**clap (docs.rs 4.6.1 `Arg`, 2026-08-19):** positional is the default; `.long("root")` makes an option. `conflicts_with` is the documented XOR (pacman example + T108 live). Both `--root DIR` and positional `DIR` must not be set together — clap `ArgumentConflict` → exit **2**.

**`--` end-of-options:** clap 4 treats `--` as the POSIX terminator. That is the natural fix for last-PR #183 (`ledgerful ledger search --json -- <query>`). **Not this track** — see T273.

**N/A:** Windows schtasks, SQLCipher, PROTOCOL-COMPAT DTO — scan-roots does not touch them.

**ledgerful / ai-brains used:** `preflight --summary`; `recall` (lexical thin — session reviews, not a T254 pin); `ledgerful search scan_roots` (hits `project_paths.rs:198` + hermetics); `ledgerful ask` (semantic index dim 384≠768 — continued without graph ask); `ledgerful ledger status` clean; doctor 4 warn (legacy `.changeguard`, sig-pin, sig-version, timings).

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — `--root` XOR positional** | Add `#[arg(long, value_name = "DIR", conflicts_with = "path")] root: Option<String>` on `ScanRoots`. Resolve `root.as_deref().or(path.as_deref())` into today’s `scan_roots` `path` argument. Default remains **cwd** when both absent. |
| **F2 — Human parent hint** | Only when **all** of: (a) implicit cwd (both `path` and `root` absent); (b) **zero** hits with `registered_project_id.is_none()` — **including zero total hits** (intentional: a git worktree with no `.ledgerful` still gets the parent remediator); (c) best-effort git toplevel is `Some` (F22); (d) `toplevel.parent()` exists and is **not** a volume/share root (F21). Print after the table (or after `No .ledgerful roots found.`): `next: ai-brains project scan-roots --root <parent>`. Parent is **`toplevel.parent()`**, not `cwd.parent()`. |
| **F3 — No re-register suggestion** | If `registered_project_id` is `Some`, JSON `suggested` is `""` (**not** `null` — `ScanRootRow.suggested` is `String`; existing tests use `as_str().unwrap_or("")`). Human suggested column is `—` (U+2014, same glyph as `registered_to` at `project_paths.rs:309`). Any owner counts. Unregistered hits keep `register-path <project-id-or-alias> <path>`. Rows stay listed. |
| **F4 — Dry-run freeze** | Never append events. Never write `.env`. No `--apply` / `--from-scan` write flag. T254 F3/F23 stand. |
| **F5 — Scan bounds freeze** | T254 F20–F23 unchanged: `.ledgerful` only (not `.changeguard`); include marked scan root; immediate children; cap 200; one-shot HashMap; exact `normalize_for_location_compare`. |
| **F6 — Format freeze** | T266 `value_parser` + `is_json_output` stay. Do not flip `auto` / pipe JSON. |
| **F7 — Hotspot** | Logic stays in `project_paths.rs`. `project.rs` is not grown. Import `collect_git_identity` for F2 only. |
| **F8 — Capture independence** | No models / embeddings / graph. Scan is filesystem + `list_path_aliases`. |
| **F9 — Pins / crates** | No clap 5, no lock bumps, no camino, workspace **0.1.1**. |
| **F10 — Contracts** | No `ai-brains-contracts` DTO. JSON **keys** stay T254 F22. Value of `suggested` may be empty (F3). **Do not** add `next_step` / `hint` keys (F2 is human-only). |
| **F11 — Help / docs** | `ScanRoots` `after_help`: `--root` example + “already-registered suggested is empty” + keep `scan-roots C:\dev`. Additive CAPABILITIES / OPERATIONS / root CHANGELOG. |
| **F12 — Decline leftover rebind** | Do not mention leftover UUIDs. Do not call `rebind-path` from scan. T259 stands. |
| **F13 — Decline auto-register** | F4. |
| **F14 — Decline recurse / cap / marker** | F5. |
| **F15 — Decline default=parent** | T254 F21 stands. Scripts that run from a repo must keep scanning that repo unless they pass a path / `--root`. |
| **F16 — PATH-behind** | Do not `cargo install`. Tests/manual AC use hermetic / `cargo run`. |
| **F17 — last-PR #183** | Dash-leading `sync query` → Ledgerful flags. **Mint T273.** Do not absorb. |
| **F18 — Peer placeholders** | T269 nightly/router, T270 retention classify, T272 Safety skip: **do not steal**. |
| **F19 — Empty `--root`** | Dispatch forwards `Some("")` into today’s `scan_roots` empty-path arm. Same `fail_usage` copy as positional: `scan-roots path is empty; pass a directory or omit to use the current directory`. Exit **2**. |
| **F20 — Both set** | clap `conflicts_with = "path"` → exit **2**. No silent ignore. No manual `if` in `scan_roots`. |
| **F21 — Volume/share-root parent** | No F2 hint when parent is: Unix `/`; Windows drive root `X:\` / `X:` (**case-insensitive** `c:\` == `C:\`); UNC share root `\\server\share` (exactly two components after `\\` / `//`). `Path::parent().is_some()` alone is **not** enough for UNC. Predicate lives on the pure helper (F28). |
| **F22 — Git spawn fail-open** | Human implicit-cwd only. `collect_git_identity(scan_root).unwrap_or_default()` — same as `identity_warn.rs:100` and `doctor.rs:742`. Git missing / spawn `Err` / non-repo → `toplevel: None` → no hint; **scan still exits 0**. JSON implicit-cwd does **not** spawn git. |
| **F23 — Tests** | Naming `function_or_feature__condition__expected_result`. Hermetics in `project_path_aliases.rs`. Clap units in `main.rs` beside T266 ScanRoots cases. No `unwrap`/`expect`/`panic` in production. |
| **F24 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F25 — Cross-model** | `suggested` empty is a CLI contract change (scripts that always exec `suggested` would no-op). After Phase-1 clean, run read-only `codex-review`. |
| **F26 — Exit codes** | Success (including empty + hint) **0**. Usage (empty `--root`, clap conflict, bad `--format`) **2**. T254 F35 otherwise. |
| **F27 — T240 F2 / T255 bag** | Do not reopen. |
| **F28 — Pure hint helper** | `parent_scan_hint(implicit_cwd: bool, unregistered_count: usize, git_toplevel: Option<&Path>) -> Option<PathBuf>` in `project_paths.rs`. No filesystem I/O, no git spawn. Units cover the full decision matrix. Caller supplies toplevel after F22. |
| **F29 — Hint display separators** | Printed `--root` path uses native separators on Windows (`/` → `\`) so git’s `C:/dev` toplevel does not print `C:/dev` in the remediator. **Do not** run the hint through `normalize_for_location_compare` (compare key: lowercases, UNC rewrite). |
| **F30 — Dispatch** | `root.as_deref().or(path.as_deref())`. Implicit cwd ⇔ both `None`. Empty string still hits F19. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Clap: `project scan-roots --root X PATH` → `ArgumentConflict` class, exit **2** |
| **AC2** | Hermetic: `--root <tree>` and positional `<tree>` produce the same JSON `roots` (same paths, same owners) |
| **AC3** | Hermetic: no path / no `--root` still scans process cwd (existing “scan the tree we `current_dir` into” pattern, or a DirGuard unit). Default is **not** parent |
| **AC4** | Hermetic: already-registered hit → JSON `suggested == ""` and `registered_project_id` is the owner. Human suggested column is `—` (no `register-path`) |
| **AC5** | Existing `scan_roots__ledgerful_child_hits_plain_misses` stays green: unregistered `suggested` contains `register-path` |
| **AC6** | Hermetic (temp git repo + registered self-hit, implicit cwd): human stdout contains `next: ai-brains project scan-roots --root` and the parent path. JSON of the same tree has **no** `next_step` key |
| **AC7** | Hermetic: explicit `--root` / positional on that same git repo → **no** `next:` parent line |
| **AC8** | JSON object keys remain `api_version`, `scan_root`, `truncated`, `roots` only (no new envelope keys) |
| **AC9** | `scan-roots` still does not create aliases (existing `scan_roots__never_writes_events`) |
| **AC10** | T254/T266 scan hermetics stay green: changeguard-only miss; grandchild miss; marked scan-root included; `--format pretty` / `json` / `JSON` clap units |
| **AC11** | Clap / usage: `--root` with empty value (if expressible) or empty positional still exit **2** with today’s empty-path copy class |
| **AC12** | Unit (no disk): `parent_scan_hint` → `None` when `implicit_cwd` is false, `unregistered_count > 0`, toplevel `None`, or parent is `/`, `C:\`, `c:\`, `C:`, `\\server\share`. Toplevel `C:\dev\AI-Brains` → `Some` whose display (F29) is `C:\dev` |
| **AC13** | Docs: after_help names `--root` (example `scan-roots --root C:\dev` **and** positional `C:\dev`); CAPABILITIES/OPERATIONS/CHANGELOG additive. `project.rs` line count does not grow by scan logic |
| **AC14** | No production `unwrap`/`expect`/`panic`. No pin bumps. No contracts DTO |
| **AC15** | Manual (source bin): implicit cwd in this repo prints parent `next:` to `C:\dev`; `scan-roots --root C:\dev --format human` lists siblings; already-registered AI-Brains row has `—` suggested. Exit 0. **Do not** register or rebind |
| **AC16** | Unit: `parent_scan_hint(true, 0, None)` → `None` (F22 fail-open: git `Err` mapped to default). Hermetic optional: implicit-cwd human still exit **0** when `GIT` is missing from PATH (or skip with reason if CI always has git) |
| **AC17** | Unit: `parent_scan_hint(true, 0, Some(repo))` with zero hits (vacuous unregistered) still returns the parent — F2 empty-scan is intentional |

---

## 5. Design notes

### 5.1 Why `--root` if positional exists

The 2026-08-16 stub assumed there was no way to name a parent. T254 already shipped `scan-roots [PATH]`. Operators still do not discover it from the default cwd run. `--root` is the documented remediator string F2 can print (`next: … --root <parent>`) without looking like a second positional. Scripts keep the positional.

### 5.2 Parent of *toplevel*, not cwd

`C:\dev\AI-Brains\crates` as cwd: `cwd.parent()` is the repo. Sibling roots live under `C:\dev` = `toplevel.parent()`. F2 uses that.

### 5.3 Empty `suggested` vs omit key

T254 AC and agents already expect `suggested` on every row. Empty string keeps the key (F10) and is not a valid command. Human `—` matches the unregistered `registered_to` glyph.

### 5.4 Capture independence

No new events. F2 git spawn is a `rev-parse` already used by detect/whoami/footer — human implicit-cwd only, fail-open (F22).

### 5.5 Hint helper (fold-in)

```text
parent_scan_hint(implicit_cwd, unregistered_count, git_toplevel) -> Option<PathBuf>
```

No I/O. Volume/share-root predicate is inside the helper (F21). Display `\` rewrite is a thin wrapper used only when printing F2.

---

## 6. Non-goals

- Changing the default scan directory to parent / `C:\dev`
- Auto-register, `--apply`, writing `.env`
- Leftover split / `rebind-path` (T259)
- T266 format retune; T269 nightly/router; T270 classify; T272 Safety skip
- T273 dash-query argv (minted, not stolen)
- Recurse, cap change, `.changeguard` as a hit
- New JSON keys (`next_step`, `hint`)
- Growing `project.rs`; clap 5; pin bumps; contracts DTO
- T240 F2 / T255 declined bag
- `cargo install`

---

## 7. Verification plan

```powershell
# Red → green
cargo nextest run -p ai-brains-cli --lib project_paths
cargo nextest run -p ai-brains-cli -E "test(scan_roots)"
cargo clippy -p ai-brains-cli --all-targets -- -D warnings

# Manual (source bin)
cargo run -q -p ai-brains-cli -- project scan-roots --format human
cargo run -q -p ai-brains-cli -- project scan-roots --root C:\dev --format human
# Do not register-path / rebind-path / write .env

# Full gate (before finalize)
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace ; cargo deny check ; cargo audit
ledgerful verify --scope full
```

TDD: land failing AC4/AC6/AC1 tests first (Red), then `--root` + empty suggested + hint (Green).

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Flipping default to parent breaks scripts | **F15** cwd default |
| Scripts exec `suggested` on registered rows | **F3** empty string — they no-op instead of conflict |
| `cwd.parent()` hints the repo | **F2** `toplevel.parent()` |
| Drive-root `--root C:\` | **F21** no hint |
| JSON key growth | **F10** human-only hint |
| Growing hotspot `project.rs` | **F7** |
| Git spawn on every JSON scan | **F22** human implicit-cwd only; spawn `Err` → no hint |
| Git missing aborts the scan | **F22** `unwrap_or_default()` |
| UNC `\\server\share` hinted as parent | **F21** share-root predicate |
| Hint prints `C:/dev` | **F29** display `\` only; not compare-normalize |
| Stealing T273 / T272 | **F17/F18** |
| PATH-behind false AC | **F16** hermetic / `cargo run` |

---

## 9. Deferred absorb / decline + last-PR Cursor

Entire `conductor/deferred.md` scanned 2026-08-19. Closed/strikethrough rows stay closed.

| Item | Source | Disposition |
|------|--------|-------------|
| `scan-roots` cwd-only (4/5) | deferred post-T255 audit map | **Absorb** F1–F3 / AC1–AC7 |
| T254 F21 default = cwd | T254 open Q4 | **Affirm** F15 |
| T254 F20–F23 bounds | T254 | **Affirm** F5 |
| T254 F12 closeout (TTY auto hermetic, etc.) | T254 residuals | **Decline** — not parent/`suggested` |
| T233-F44 `ledgerful endpoints` | T254 decline | **Decline** F14 |
| Leftover `7d97a456` roots | T259 | **Decline** F12 — operator `rebind-path` |
| T267 list footer leftover | T267 | **Closed** — not scan |
| T266 format maze | T266 | **Closed** — **Decline** retune F6 |
| T269 nightly / T270 classify / T272 Safety | peers | **Decline** F18 |
| last-PR Cursor #183 dash-query Medium | PR [#183](https://github.com/Ryan-AI-Studios/AI-Brains/pull/183) `cursor[bot]` inline on `sync_query_ledger.rs:157` | **Mint T273** — still true today; fits no Pending placeholder |
| T272 #179 Safety skip | already Pending | **Do not steal** F18 |
| last-PR issue comments / other reviews on #183 | `gh` | Empty except Bugbot review + that one inline |
| Open PR on HEAD | none | N/A |
| Dependabot open PRs | #72… | Upsell/deps — not findings |
| MSI / R-CI-BRANCH / T196 SIGTERM | packaging / admin | **Not related** (one line: not scan-roots) |
| T210 grants / T241 0-of-3 | policy | **Not related** |
| PATH `cargo install` | series soft | **Decline** F16 |
| T240 F2 / T255 bag | standing | **Decline** F27 |
| T118 `RUST_LOG=""` / Cozo INFO / backup WARN | closed series | **Not related** |
| `ISSUES.md` | — | **Does not exist** |

---

## 10. Implement order (on go)

1. Red→Green F3/AC4/AC5 empty `suggested` for registered (pure `scan_rows_for_hits` + hermetic)  
2. Red→Green F1/F19/F20/F30/AC1/AC2/AC11 `--root` clap XOR + dispatch `root.or(path)`  
3. Red→Green F2/F21/F22/F28/F29/AC6/AC7/AC12/AC16/AC17 `parent_scan_hint` + fail-open git + human-only emit  
4. Docs F11 / AC13  
5. Manual AC15 (no live mutate) → review → `codex-review` (F25) → full gate → Complete  

---

## 11. Soft residuals (post-close)

| Residual | Note |
|----------|------|
| PATH reinstall | F16 — operator |
| T259 leftover sibling owners | Operator `rebind-path` |
| JSON `next_step` | Declined F10 — reopen only if agents need it |
| Default=parent product flag | Declined F15 |
| T269 / T270 / T272 / T273 | Peers |
| T254 F12 TTY-auto hermetic | Still soft |

---

## 12. Touch map (expected)

| Site | Change |
|------|--------|
| `commands/project_paths.rs` | empty suggested; `parent_scan_hint`; human `next:`; units |
| `main.rs` | `ScanRoots.root`; XOR; dispatch `root.or(path)`; after_help; clap units |
| `tests/project_path_aliases.rs` | AC2/AC4/AC6/AC7 hermetics |
| `Docs/CAPABILITIES.md` / `Docs/OPERATIONS.md` / `CHANGELOG.md` | Additive `--root` + empty suggested |
| `conductor/deferred.md` / `conductor.md` | This plan + T273 mint |
| events / store / daemon / contracts / `project.rs` scan body | **No** |

---

## 13. AI fold-in (2026-08-19)

Sources: `agy-review.md` (HEAD `d00fb17`) + `opencode-review.md` (stated HEAD `a4ac170`; plan files were already `d00fb17`). No Blockers. No Majors. Verdict both: **Planned**.

### agy

| ID | Verdict | Action |
|----|---------|--------|
| **m1** volume-root tests (`C:\`, UNC, `/`) | **Agree** | F21 / AC12 |
| **m2** empty `--root ""` copy | **Already covered** | F19 / AC11 — pin the live `fail_usage` string |
| **m3** human `—` == `registered_to` glyph | **Already covered** | F3 |
| **O1** pure `parent_scan_hint(...)` | **Agree** | F28 |
| **O2** after_help `--root C:\dev` | **Already covered** | F11 / AC13 |
| last-PR #183 / T273 | **Already covered** | F17 — minted at plan time |

### opencode

| ID | Verdict | Action |
|----|---------|--------|
| **m1** git spawn `Err` fails the scan | **Agree** | F22 `unwrap_or_default()`; AC16 |
| **m2** drive-letter case + UNC share root | **Agree** | F21 / AC12 (same bag as agy m1) |
| **m3** git `C:/dev` in the hint | **Partial** | F29 display `\` rewrite. **Decline** `normalize_for_location_compare` for the printed hint (compare key lowercases / UNC-rewrites) |
| **m4** `suggested: ""` not null; `root.or(path)` | **Already covered** | F3 / F1 / F30 — extra pin that JSON is `""` not `null` |
| **O** zero total hits vacuously triggers F2 | **Agree** | F2 (b) + AC17 — intentional |
| **note** `--root X PATH` silently ignores | **Decline as stated** | F20 clap XOR already exit **2**; AC1. Re-trigger only if clap `conflicts_with` is dropped |
| “add JSON `parent_hint`” (looks-solid AC5) | **Decline** | F10 / AC8 — no new envelope keys |
| mint T273 during fold-in | **Already covered** | T273 exists at `conductor.md` T273; do not remint |
| deferred “T254 F12 = volume-root” | **Decline misread** | T254 F12 is list-paths first-path-only / TTY hermetic, not scan parent |

### Pins locked by fold-in

1. **F22:** `collect_git_identity(...).unwrap_or_default()`; scan must not fail because git is missing.
2. **F21:** volume/share-root = `/` + case-insensitive `X:\`/`X:` + UNC `\\server\share`. Not `parent().is_some()` alone.
3. **F28:** `parent_scan_hint(implicit_cwd, unregistered_count, git_toplevel) -> Option<PathBuf>` is pure.
4. **F29:** Windows hint print uses `\`; not `normalize_for_location_compare`.
5. **F2 (b):** zero total hits still hints — AC17.
6. **F3:** JSON `suggested` is `""`, never `null`.
7. **F10:** no JSON `next_step` / `parent_hint` key.

---

**Planning + fold-in 2026-08-19.** Still **plan-only until go**.
