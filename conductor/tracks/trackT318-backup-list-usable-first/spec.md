# T318 — `backup list` usable-first (collapse residual fleet)

- **Track ID:** T318-BackupListUsableFirst
- **Status:** **Planned** (Pending until **go**)
- **Category:** UX / OPS
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — `backup list` 6/**6**. Series README `README-T312-T324-CLI-DOGFOOD.md`. T244 already **sorts** usable-first; Default **still prints every residual row**. T225 already quiet-verify counts + first 5 FAIL; mixed fleet with ≥1 OK still dumps five long FAIL reasons.
- **Depends on:** T295 ✅ ≥1 usable encrypted backup (`vault-2026-08-24T10-01-54.db.bak`); T244 ✅ class + CLI sort + F6 summary; T225 ✅ quiet verify; T209 honesty / `ListMode`; T277 engine freeze
- **Blocks / feeds:** Daily recoverability skim. Doctor `backup_recent` remediator stays T277 F8 (`ai-brains backup create`).
- **Absorbs:** Audit list residual noise (T295 solved **existence**; T244 solved **class + sort**; this is **Default emit collapse** + F6 stderr→stdout + mixed-verify summary). T316 Windows-first stderr analog (F6 `eprintln!` looks like an error).
- **Not absorbed (DoD):** Transcode/rekey legacy `.bak`; default `--keep 10`; class-aware prune (T244 F18); verify `--quiet` / JSON `summary` / `VerifyError` (T225/T244 F17); growing `doctor.rs`; T277 create engine; T325/T326
- **Research date:** 2026-08-29 (plan-write product HEAD `ed2f5f8` T316 Completed note `#240`; T316 product `#239` `4d1a53e`). Fold-in against `93a788a` (this plan’s own docs commit; ahead **1** of `origin/main` = `ed2f5f8`). Snapshot — **re-verify at execute**.
- **AI fold-in:** 2026-08-29 `agy-review.md` + `opencode-review.md` (HEAD `93a788a`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** **Agree:** Agy m1 HEAD snapshot; OpenCode m1 Default-mode flip census (F31); OpenCode m2 AC5 split (mixed-quiet vs all-residual quiet + named dual-flag); OpenCode O1 mixed trailer helper in `verify_report.rs` (F9); OpenCode O2 named empty-list hermetic (AC6). **Already:** Agy m2 F4/AC3/AC6 empty vs residuals-only; Agy O1/O2/O3 F1/F2/F5. Disposition **§13**.
- **Ledger:** planning DOCS TX `156b2a03-b5aa-4905-b840-d14fb182aa90`. Fold-in DOCS TX `5f4aace2-b78d-4757-961f-12bc2366f5b3`. Series mint DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** implement until **go**. Do **not** live `backup create` / prune / restore. Do **not** `cargo install`. Do **not** grow hotspot `project.rs` / `sync.rs` / `forget.rs` production / `doctor.rs` / brain `backup.rs` production. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Default human list shows recoverability, not the graveyard.** Table rows are **usable only** (`Readable | PreT109`). The 1 good backup must not sit under 22 residual lines. Residual classes collapse to one **stdout** footer (existing F6 SOOT `not recoverable under current key`) unless `--verbose`.
2. **F6 is not an error.** Move the residual summary off stderr (PowerShell ErrorRecord / `$Error`). Quiet omits the footer. Verbose keeps the full table + existing per-file WARN detail.
3. **`verify` Default is a summary when recoverability already exists.** Mixed fleet (`ok >= 1`) prints counts + a FAIL-count trailer pointing at `--verbose` — **no** first-5 `FAIL —` dump. Zero-OK keeps T225 first-5 + create nudge (operator still needs *why*). Exit **1** on any FAIL stays (T295/T225).
4. **North star.** Capture independence: list/verify **presentation**. No transcode, no prune, no doctor-string rewrite. Operators who run `backup list` must see the usable snapshot first (and only), then a copy-paste pointer to `--verbose` / `backup verify`.

This unblocks daily CLI: T244 sorted the good row to the top; the table is still a 24-line wall. T225 quieted verify INFO; mixed-success still prints five legacy-plain essays.

---

## 2. Live baseline (re-scan 2026-08-29)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | Fold-in against plan-write `93a788a` `docs(conductor): plan T318 backup list usable-first (collapse residuals, mixed-verify summary)`. Product `src/` = T316 `#240` `ed2f5f8`. Tree **CLEAN** at fold-in. Branch `track/T318-backup-list-usable-first`. `origin/main` = `ed2f5f8` (ahead **1**). Plan-write snapshot was `ed2f5f8` / ahead **0** (Agy m1). |
| PATH `ai-brains.exe` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **26,897,408** B; LastWriteTime **2026-08-27 8:21:55 PM**; `ai-brains 0.1.3`. **T209/T225/T244/T277/T295 on PATH.** **T316/T320/T319/T317 not.** List+verify hole **is** on PATH **and** source (`backup.rs` unchanged by T316). **Do not `cargo install`.** Tests/manual AC use hermetic bin / `cargo run`. |
| `preflight --summary` (PATH) | Pinned **4601**. In-context **0/0/0**. `Total Word Count: 802` (PATH-behind T315 `Budget window words:`). **Not this DoD.** |
| PATH `backup list` | stdout **24** lines = header + **23** files. First data row **usable**: `vault-2026-08-24T10-01-54.db.bak` `2026-08-24 10:01:54` `C:\dev\AI-Brains\vault.db` `0.1.2` `151977984`. Then **22** residual rows (`(unreadable key)` / `(legacy plain)` / `(no core tables)`). stderr F6: `22 backup(s) not recoverable under current key (legacy plain / incomplete / key / corrupt): use --verbose or ai-brains backup verify`. Exit **0**. |
| PATH `backup list --quiet` | Same table including residuals; **no** F6. |
| PATH `backup list --verbose` | Per-file `WARN Backup is legacy plaintext…` flood (T209 Verbose) **then** the full table. |
| PATH `backup verify` | `Verified 23 backups: 1 OK, 22 FAIL.` + **5** `filename: FAIL — {reason}` + `… and 17 more FAIL (use --verbose for full list).` Exit **1**. Create nudge **absent** (ok≥1, T225/T295 F14). First FAIL is T244 exhibit `vault-2026-08-12T15-50-06.db.bak` (`file is not a database`). |
| `backup list --help` | `--quiet` / `--verbose` only. **No** after_help. No usable-only sentence. |
| `backup verify --help` | `[PATH]` / `--full` / `--format json\|pretty` / `--verbose`. Default pretty (not auto TTY-flip). |
| Last GitHub PR | [#240](https://github.com/Ryan-AI-Studios/AI-Brains/pull/240) T316 conductor note. `mergedAt` **2026-08-29T05:10:01Z**. Issue comments **[]**. Inline comments **[]**. Reviews **[]**. Bugbot **overview only** (docs, low risk, no defect). **last-PR product** [#239](https://github.com/Ryan-AI-Studios/AI-Brains/pull/239) T316 list preview. `mergedAt` **2026-08-29T05:09:03Z**. Comments/reviews **[]**. Bugbot overview only (display-only, no leftover). `#237` Bugbot already **T326**. `#230` already **T325**. Open PRs: **none**. **No T327.** |
| Ledger | 0 pending / 0 drift at scan. Hotspot **#1** `project.rs` (3.665) — **do not touch.** `sync.rs` **#2**. `governed_common.rs` **#3**. `forget.rs` **#5** (2.832) — **do not grow production.** `session_chrome.rs` **#6**. `backup.rs` **not** in top 10 — edit CLI emit only. `doctor.rs` **1738** nonblank — **do not grow.** |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why list+verify are still the hole

| Layer | Truth |
|-------|--------|
| T244 F7 sort shipped | `list_sort_key` (`backup.rs:156–161`) + `run_list` `:171–172` usable-first. Live first row **is** the T295 Readable file. **Sort is not the remaining DoD.** |
| Default still prints every class | `run_list` `:179–220` loops **all** `backups` into the table, then F6 stderr when `residual_count >= 1` (`:222–228`). Quiet = same table, no F6. Verbose = omit F6 (comment “per-file detail already emitted”) — table still all rows. |
| F6 stderr is T244 by design | Same Windows-first conflict as T316 F36: native stderr → ErrorRecord / `$Error` ([about_Redirection](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_redirection)). Captured dogfood: stdout table + stderr one-liner. Repo precedent T267/T249/T299/T317: collapsing extras / next-steps on **stdout**. T316 **dropped** nonempty forget stderr. T318 **moves** F6 (the count is the product). |
| `--verbose` help vs emit | Help: “Per-file detail for non-readable backups.” Live Verbose = brain `ListMode::Verbose` tracing WARNs (T209) + full table. **Keep that.** Default must stop duplicating the table of residuals. |
| T225 verify quiet shipped | Counts + `VERIFY_FAIL_PREVIEW_CAP=5` + trailer + nudge-when-ok==0. Mixed **1 OK / 22 FAIL** still dumps 5 long reasons. Placeholder “verify Default is a summary” = **supersede mixed preview only**. Zero-OK diagnosis **stays** T225. |
| Brain vs CLI | `list_backups` stays timestamp-desc for doctor `find_map` newest usable (T244 F7 / AI2 M3). Filter is **CLI emit**. Do **not** edit `crates/ai-brains-brain/src/backup.rs` production. |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| `run_list` | `commands/backup.rs:163–229` | Sort + print **all** rows + F6 stderr. **Filter Default/Quiet table to usable; footer stdout.** |
| `list_sort_key` | `:156–161` | T244 F7. **Do not change.** |
| `backup_class_token` | `:145–154` | Tokens freeze. Verbose table still uses them. |
| `is_usable_class` / `residual_for_summary` | `brain/backup.rs:32–40` | Readable\|PreT109 vs complement. **Import; do not edit.** |
| `ListMode::from_flags` | `brain/backup.rs:51–61` | Quiet wins. **Freeze.** |
| clap `BackupCommands::List` | `main.rs:3626–3634` | `--quiet` / `--verbose`. **No new flag.** after_help **none** — add one sentence. |
| clap `BackupCommands::Verify` | `main.rs:3635–3648` | `path` / `--full` / `--format` / `--verbose`. **No new flag.** |
| Dispatch | `main.rs:5273–5282` | `ListMode::from_flags`; `run_verify(…, *verbose)`. |
| `run_verify` human default | `backup.rs:377–405` | Always `format_fail_preview` when not verbose/json. **Gate: ok==0 keep; ok>=1 counts + trailer only.** |
| `VERIFY_FAIL_PREVIEW_CAP` | `verify_report.rs:7` | **5.** Freeze. Zero-OK path only after this track. |
| `format_verify_counts` | `verify_report.rs:13–16` | `Verified {n} backup(s): {ok} OK, {fail} FAIL.` Freeze. |
| `should_emit_create_nudge` | `verify_report.rs:48–49` | `ok==0 && total>=1`. Freeze. |
| JSON `VerifyOutput` | `backup.rs:250–267` | `results[]` / `status` / optional `message`. **No new keys.** |
| Hermetic list | `tests/backup_list_honesty.rs` | AC1 token+stderr summary; AC3 ≤1 summary on stderr; AC4 verbose; AC5 quiet; mixed usable-first **requires residual rows in table** (`:500–546`) — **update**. |
| T277 mixed list | `tests/backup_recoverable.rs:228–245` | stderr `not recoverable` — **flip to stdout**. Mixed create still lists residual row (`:150–154`) — **update Default**. |
| Smoke verify mixed | `tests/smoke.rs:1359–1441` `backup_verify_all__mixed__reports_per_file` | Requires `FAIL —` preview on mixed ok≥1 — **flip**. |
| Smoke zero-OK | `smoke.rs:1527–1583` | 5 `FAIL —` + trailer + nudge. **Stay-green.** |
| Smoke verbose mixed | `smoke.rs:1444–1525` | Full stream, no `Verified `. **Stay-green.** |
| T198 empty | `empty_states_exit_hygiene.rs` | `No backups found.` / `No backups to verify.` **Stay-green.** |
| CAPABILITIES | `Docs/CAPABILITIES.md:538` | stderr summary + usable-first sort + first 5 FAIL. **Rewrite emit sentences.** |
| OPERATIONS | `Docs/OPERATIONS.md:781–783` | Same. |
| Line counts | CLI `backup.rs` **847** nonblank / **936** physical; `verify_report.rs` **139** nonblank; brain `backup.rs` **1254**; `doctor.rs` **1738**. Snapshot — **F22 80-net is phase diff vs go HEAD**. |
| Contracts | none | CLI-local verify JSON. PROTOCOL-COMPAT N/A. |

### 2.4 Dependency / standards research (2026-08-29)

| Pin | Workspace / lock | Action |
|-----|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** / crates.io **4.6.6** (2026-08-06; [docs.rs/clap/4.6.6](https://docs.rs/clap/4.6.6/clap/struct.Arg.html)) | **No bump.** No new flag. clap 5 **forbidden**. |
| `serde_json` | lock **1.0.150** | **No bump.** Verify JSON values only (no new keys). |
| `rusqlite` | exact **0.40.2** | **No bump.** No SQL change. |
| `uuid` | ws `"1.13"` / lock **1.23.1** | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | Unchanged. |
| workspace version | **0.1.3** | **No bump.** |
| New crates | — | **Zero.** |

**CLI list/verify research (primary sources):**

| Source | What we take | What we decline |
|--------|----------------|-----------------|
| [clig.dev Output](https://clig.dev/#output) (fetched 2026-08-29) | Human-first; “Changing output for humans is usually OK”; suggest next command; stdout = data; stderr = messages/errors; “saying (just) enough”; `--verbose` for detail; default the right thing | Keeping F6 on stderr **because** clig says messaging→stderr — Windows-first PowerShell treats native stderr as the error stream; this repo already moved collapsing extras to stdout (T317 `+N more RECALLS`, T299 `next:`) |
| [about_Redirection](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_redirection) (Learn) | Native stderr is stream 2; Windows PowerShell 5.1 / ISE wrap native stderr as ErrorRecord | Teaching operators to `2>$null` as the product fix |
| [restic check](https://restic.readthedocs.io/en/stable/045_working_with_repos.html) / `restic-check(1)` (Debian testing, 2026-06-25) | Default is a health **summary** (`no errors were found` / error count); `-v` for detail; exit 1 on any error | Importing restic’s cache / `--read-data` surface; flipping mixed-fleet verify to exit 0 |
| T209/T225/T244 live | Classify + Quiet/Verbose + counts + first-5 zero-OK + usable-first **sort** | Printing residual **rows** on Default; first-5 FAIL when `ok>=1` |
| T316 F9 | Nonempty success must not look like a PowerShell error | Dropping the residual **count** (unlike F36, the count is the product) |

N/A-if-skipped: SQLCipher create path, schtasks, llama.cpp `/health`, FTS5, clap `num_args` (no new flags).

**Could not verify:** exact 22-way class histogram without vault SQL (do not print `AI_BRAINS_KEY`). Dogfood showed mix of KeyMismatch / LegacyPlain / Incomplete. Hermetic mixed fixtures are SoT. Manual live N may drift (T295 F38 class) — record Phase 0 N on go.

**ledgerful / ai-brains:** `preflight --summary` Pinned **4601** / in-context 0/0/0 / words **802** (PATH). `recall "backup list usable-first residual not recoverable"` lexical hits T277 review dumps (PATH dump-first). `ledgerful ledger status --compact` 0 pending / 0 drift; `search "residual_for_summary"` → `brain/backup.rs:38` + `cli/commands/backup.rs:181`; `scan --impact` CLEAN at `ed2f5f8`; hotspots as §2.1.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — Default table = usable only** | `run_list` Default and Quiet print table rows where `is_usable_class(info.class)` only. Residual classes never appear as table rows on Default/Quiet. Sort remains T244 F7 (usable band first — after filter the band is the whole table). |
| **F2 — Residual footer on stdout** | When `mode == ListMode::Default && residual_count >= 1`, print the existing F6 sentence on **stdout** after the table (or after the no-usable line). SOOT substring `not recoverable under current key` **stays**. Count still `residual_for_summary`. **Do not** `eprintln!` that sentence. Quiet: no footer. Verbose: no footer (rows + WARNs are the detail). |
| **F3 — Quiet / Verbose freeze** | `ListMode::from_flags` quiet-wins **stays**. Quiet = usable rows, no footer, no F6. Verbose = **all** rows usable-first + existing T209 per-file WARN detail. Dual `--quiet --verbose` → Quiet. |
| **F4 — Residuals-only / empty** | Zero discovered files: `No backups found.` (T198). Residual files but zero usable: print `No usable backups.` (no residual tokens in the table) + Default footer. Do **not** reuse `No backups found.` for a non-empty residual fleet. |
| **F5 — Verify mixed vs zero-OK** | Human default, not `--verbose`, not JSON: **(a)** `ok >= 1`: `format_verify_counts` + if `fail >= 1` one trailer `{fail} FAIL (use --verbose for per-file).` — **no** `format_fail_preview` / no `FAIL —` lines. **(b)** `ok == 0`: T225 first-5 + overflow trailer + create nudge **unchanged**. `--verbose` full stream (T225 F4/L1) **unchanged**. JSON full `results[]` **unchanged**. Exit **1** on any FAIL **unchanged**. Single-file PATH verify that FAILs is `ok==0` → still shows that `FAIL —` (T138 stay-green). |
| **F6 — T244 sort / brain freeze** | `list_sort_key` + brain `list_backups` timestamp-desc **unchanged**. Doctor `check_backup_recent` still `find_map` newest usable on brain order. Filter is CLI emit only. |
| **F7 — Classify freeze** | `BackupReadClass` / `is_usable_class` / `residual_for_summary` / `backup_class_token` **unchanged**. Do **not** edit `crates/ai-brains-brain/src/backup.rs` production. |
| **F8 — JSON / flags** | No list JSON (none today). Verify JSON keys freeze. **No** new clap flag (`--usable-only` / `--no-residuals` / verify `--quiet`). |
| **F9 — T225 helpers** | `VERIFY_FAIL_PREVIEW_CAP=5`, `format_verify_counts`, `should_emit_create_nudge`, `format_create_nudge` freeze. Mixed-success trailer **lives in** `verify_report.rs` as `format_mixed_fail_trailer(fail) -> String` SOOT `{fail} FAIL (use --verbose for per-file).` with unit `format_mixed_trailer__contains_verbose_and_count` (OpenCode O1). Do **not** reuse the T225 overflow string `… and {more} more FAIL (use --verbose for full list).` |
| **F10 — T277 / doctor / keep-10** | Create engine, remediator string `ai-brains backup create`, keep-10, 15-check matrix **frozen**. **Do not grow `doctor.rs`.** |
| **F11 — Isolation** | Edit CLI `commands/backup.rs` emit + optional `verify_report.rs` trailer + `main.rs` List `after_help` + hermetics + docs. **Do not** grow `project.rs` / `sync.rs` / `forget.rs` production / `doctor.rs` / `governed_common.rs` / brain `backup.rs` production / `session_chrome.rs`. |
| **F12 — No transcode / prune / create** | Do not rekey, `sqlcipher_export` residuals, live prune, live create, restore, or NIST Purge. |
| **F13 — T244/T225 F17–F18** | Decline verify `--quiet` flag, JSON `summary` field, structured `VerifyError`, class-aware prune, `backups/legacy/` archive helper. |
| **F14 — Capture independence** | Presentation only. No events, models, embeddings, graph, ledgerful. |
| **F15 — Pins / crates** | No clap 5, no lock bumps, no new crates, workspace **0.1.3**. |
| **F16 — Standing declines** | T263 H2; T240 F2; T308 floors; T307 Blocked; csrf; KIND bump. |
| **F17 — Tests** | Naming `function_or_feature__condition__expected_result`. No `unwrap`/`expect`/`panic` in production. `tempfile::tempdir` per hermetic. |
| **F18 — Cross-model** | UX display change is FEATURE. After Phase-1 clean, run read-only `codex-review`. |
| **F19 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F20 — PowerShell** | `;` not `&&`. |
| **F21 — T209 Corrupt WARN** | Short-garbage per-file `tracing::warn!` under Verbose / `RUST_LOG=warn` **stays**. Default table omits `(corrupt)` rows (F1); AC2 updates to Verbose token **or** tracing WARN, not Default table. |
| **F22 — Line-count 80-net** | CLI `backup.rs` production net **&lt;80 physical lines vs go HEAD**. Test blocks may exceed. Snapshot §2.3 is not the cap. |
| **F23 — after_help** | One additive sentence on `BackupCommands::List`: default lists usable encrypted backups only; residual count is a stdout footer; `--verbose` lists every class; `--quiet` usable-only without the footer. |
| **F24 — Stay-green** | `list_sort_key` units; T198 empty **verify**; T225 zero-OK 5-FAIL + nudge; T225 verbose mixed stream; T225 JSON full results; T295 create after_help; T277 mixed doctor `backup_recent` Ok; verify exit 1 any FAIL; quiet-wins. List empty is **AC6** (named hermetic — T198 does not cover list). |
| **F25 — last-PR Cursor** | `#240` / `#239` Bugbot overview only (no defect). `#237` → **T326**. `#230` → **T325**. **No T327.** |
| **F26 — Decline peers** | T321 safety sync; T322–T324; T325 F8 recency; T326 pin-count; T316 Completed; T307 Blocked. |
| **F27 — PATH-behind** | T316 chrome-skip not on PATH — **not this hole**. Hermetic / `cargo run` SoT. Do not `cargo install`. |
| **F28 — Dual-truth** | T244 **sort** already usable-first; T318 **emit** collapses residuals. after_help names usable-only default vs `--verbose` all classes. |
| **F29 — Family** | List is human-only (no `--format`). Verify default **pretty** (explicit `--format json`; not T266 auto TTY-flip). |
| **F30 — T316 analog** | F6 count **moves** to stdout (do not drop). Unlike F36, the sentence is the recoverability remaining-count. |
| **F31 — Default-mode flip census** | Same-commit updates (not “unrelated failures”) for every Default-mode residual-token / stderr-SOOT assert in `backup_list_honesty.rs`: `__plain_unset_rust_log__legacy_plain_no_per_file_warn` (`:82`), `__two_plain__at_most_one_summary` (`:164` — summary lines move to **stdout**), `__large_key_mismatch__summary_not_warn_flood` (`:336`), `__incomplete__token_and_residual_summary` (`:394`), `__incomplete_default_rust_log_warn__no_per_file_warn` (`:430`), plus mixed AC1/AC2 (`:500`) and `backup_recoverable.rs` list (`:150` / `:229`). Verbose stay-greens do **not** flip table tokens. (OpenCode m1) |

---

## 4. Acceptance criteria

| ID | Criterion | Proof |
|----|-----------|--------|
| **AC1** | Mixed usable+residual Default: stdout data rows contain the Readable filename and **do not** contain residual tokens `(legacy plain)` / `(unreadable key)` / `(no core tables)` / `(corrupt)` | Hermetic update of `backup_list_honesty__mixed_usable_and_residual__usable_first` |
| **AC2** | Same fixture: stdout contains `not recoverable under current key` and the residual **count**; stderr does **not** contain that SOOT | Hermetic (flips T244/T277 stderr asserts) |
| **AC3** | All-residual Default (plain only): stdout contains `No usable backups.` + footer count; stdout table has no `(legacy plain)` row | New hermetic `backup_list__all_residual__no_usable_and_footer` **and** F31 census flips (T209 AC1 `:82`, two_plain `:164`, key-mismatch `:336`, incomplete `:394`/`:430`) |
| **AC4** | `--verbose` still prints residual tokens + omits footer SOOT | Stay-green/update `backup_list_honesty__verbose_plain__per_file_detail` |
| **AC5** | `--quiet` **mixed**: usable row present; no residual tokens; no footer SOOT on stdout **or** stderr | New/updated `backup_list_honesty__quiet_mixed__usable_row_no_footer` — mixed fixture (readable create like `:505–512`) + `--quiet`. Do **not** add a readable backup to the all-plain quiet tests (OpenCode m2) |
| **AC20** | `--quiet` **all-residual** (existing plain-only fixtures): `No usable backups.`; no footer on stdout **or** stderr; dual `--quiet --verbose` quiet wins (no per-file WARN) | Update `backup_list_honesty__quiet__no_summary` (`:240`) **and** `backup_list_honesty__quiet_and_verbose__quiet_wins` (`:263`) |
| **AC6** | Empty backups dir (no `vault-*.db.bak`): stdout `No backups found.`; exit 0; no footer SOOT | New hermetic `backup_list__empty__no_backups_found_exit_0` (T198 covers **verify** empty only — OpenCode O2) |
| **AC7** | `list_sort_key` units unchanged | Stay-green `list_sort_tests` |
| **AC8** | Verify mixed 1 OK + 1 FAIL: counts contain `1 OK` and `1 FAIL`; **no** `FAIL —`; trailer contains `--verbose`; exit **1**; **no** create nudge | Flip `backup_verify_all__mixed__reports_per_file` + stay-green `backup_verify__mixed_ok_and_key_mismatch__one_ok_exit_1_no_nudge` + unit `format_mixed_trailer__contains_verbose_and_count` |
| **AC9** | Verify 6-FAIL zero-OK: exactly 5 `FAIL —` + trailer `--verbose` + create nudge + `0 OK` `6 FAIL` | Stay-green `backup_verify__multi_fail__preview_cap_trailer_and_nudge` |
| **AC10** | Verify `--verbose` mixed: no `Verified `; 1 OK line + 1 FAIL line | Stay-green `backup_verify_all__mixed__verbose_per_file_stream` |
| **AC11** | Verify `--format json` still full `results[]` (verbose ignored) | Stay-green T225 JSON |
| **AC12** | Empty verify human + JSON | Stay-green T198 |
| **AC13** | `backup create --help` T295 after_help unchanged | Stay-green `backup_create_help__after_help__mentions_no_prune_default_dir` |
| **AC14** | `backup list --help` after_help names usable-only default, stdout residual footer, `--verbose` all classes | New hermetic `backup_list_help__after_help__names_usable_only_and_verbose` |
| **AC15** | Docs: CAPABILITIES §11 emit sentences; OPERATIONS list/verify lines; CHANGELOG T318 | File grep |
| **AC16** | `doctor.rs` / brain `backup.rs` / `project.rs` / `forget.rs` production empty of behavior diff | `git diff -- crates/...` name-only |
| **AC17** | Manual `cargo run -p ai-brains-cli -- backup list`: usable row(s) only + stdout footer with live residual count; `backup verify`: `1 OK, 22 FAIL` (or Phase 0 N) **without** first-5 `FAIL —`; exit 1. Pass-with-observed-data if N drifted | Recorded stdout/stderr |
| **AC18** | List exit 0 any mix; verify any FAIL → 1; unknown list flag clap exit 2 | Stay-green + clap |
| **AC19** | T277 mixed create still first-row Readable + doctor `backup_recent` Ok; Default list **omits** residual table row but **keeps** footer count | Update `backup_recoverable.rs` list asserts; doctor stay-green |

---

## 5. Design notes

### 5.1 `run_list` emit

After existing `sort_by_key(list_sort_key)`:

1. Count `residual_count` with `residual_for_summary` over **all** infos (unchanged).
2. `usable`: infos where `is_usable_class`.
3. Empty `backups` → `No backups found.` return (today).
4. **Verbose:** print header + **all** sorted infos (today’s loop). No footer.
5. **Default / Quiet:** if `usable` empty → `println!("No usable backups.")`; else header + usable rows only.
6. **Default** + `residual_count >= 1` → `println!` the current F6 sentence (stdout). Quiet skips.

No `unwrap`/`expect`/`panic`. Keep `saturating_add`.

Suggested footer (word-wrap OK; AC2 locks substring + count):

```text
{n} backup(s) not recoverable under current key (legacy plain / incomplete / key / corrupt): use --verbose or ai-brains backup verify
```

### 5.2 `run_verify` mixed gate

In the human-default arm (`backup.rs:377`), after counts:

- if `ok == 0`: existing `format_fail_preview` + nudge.
- else if `fail >= 1`: `println!("{}", format_mixed_fail_trailer(fail));` (no preview lines, no T225 overflow trailer — this line *is* the trailer). Helper in `verify_report.rs` (F9 / OpenCode O1).
- nudge predicate unchanged (`ok==0` only).

Do not change JSON / verbose arms.

### 5.3 Dual-truth

Human Default list **omits residual rows**. `--verbose` **shows them**. Doctor still sees every file via brain `list_backups`. Verify JSON still lists every result. after_help one sentence.

---

## 6. Non-goals

Rekey / transcode / `sqlcipher_export` of residual `.bak`. Changing keep-10. Class-aware prune / `backups/legacy/` (T244 F18). verify `--quiet` / JSON `summary` / `VerifyError` (T244 F17). Growing `doctor.rs`. Editing brain `backup.rs` production. T321 safety sync. T322–T324. T325 F8 recency. T326 pin-count. clap 5. Pin→Approved (H2). Silent `.env`. Floor retune. Live create/prune as this track. New list `--format json`.

---

## 7. Verification plan (TDD)

**Red first** (must fail on today’s tree):

- Hermetic mixed Default still contains residual tokens in stdout table (AC1) — today it does.
- Hermetic mixed footer on **stdout** / absent on stderr (AC2) — today F6 is stderr.
- Hermetic all-residual `No usable backups.` (AC3) — today prints `(legacy plain)` row.
- F31 census Default-mode token/stderr asserts (must fail or be updated same commit).
- Hermetic AC5 mixed-quiet usable row (today quiet fixtures are all-plain).
- Hermetic AC6 `backup_list__empty__no_backups_found_exit_0` (today untested).
- Hermetic AC14 after_help — today’s List help has no after_help.
- Hermetic AC20 all-residual quiet + dual-flag.
- Smoke mixed verify `FAIL —` **absent** (AC8) + `format_mixed_trailer__contains_verbose_and_count`.

**Green:** filter Default/Quiet rows; `println!` F6; `format_mixed_fail_trailer`; gate verify preview on `ok==0`; List after_help.

**Stay-green:** AC4/AC7/AC9–AC13/AC16/AC18/AC19-doctor.

**Manual AC17:** `cargo run`; PATH-behind T316 not a fail. Live N recorded.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Operators think residuals vanished | Footer SOOT + `--verbose` + after_help; doctor still ages usable only |
| Quiet noisier than Default if we kept residual rows | F3 Quiet also usable-only |
| Zero-OK verify loses diagnosis | F5b keeps T225 first-5 + nudge |
| Mixed verify scripts grepping `FAIL —` | Breaking **human** output is OK (clig); JSON `results[]` unchanged; CAPABILITIES/CHANGELOG |
| T209 Default-mode token/stderr bitrot | F31 census — same-commit flips of `:82/:164/:336/:394/:430` plus mixed/recoverable |
| Doctor sort break | F6/F7 brain untouched (AC16) |
| 80-net | Filter is a few lines in `run_list`; trailer one `if`; tests in existing files |

---

## 9. Deferred absorb/decline

| Item | Disposition |
|------|-------------|
| Audit `backup list` 6/6 residual fleet noise | **Absorb** F1–F5 / AC1–AC5 / AC17 |
| deferred.md `backup list` residual noise → T318 Pending | **Absorb** this plan |
| T244 F7 usable-first **sort** | **Affirm freeze** F6; **extend emit** F1 |
| T244 F6 stderr summary | **Supersede stream** F2 (stdout); SOOT/count stay |
| T225 first-5 FAIL default | **Partial** F5 — keep zero-OK; **supersede mixed** |
| T225/T244 F17 verify `--quiet` / JSON summary / VerifyError | **Decline** F13 |
| T244 F18 class-aware prune / archive | **Decline** F13 |
| T209 L3/L4 real wrong-key fixture / PreT109 unit | **Decline** (soft; not this DoD) |
| T277 engine / doctor remediator / keep-10 | **Affirm freeze** F10 |
| T295 ≥1 usable + verify exit 1 mixed | **Affirm** F5/F24 / AC8/AC19 |
| T316 F36 stderr analog | **Absorb pattern** F2/F30 (move, don’t drop count) |
| T318 listed as “not stolen” on T316–T320 | **This track** |
| T321 / T322–T324 | **Not stolen** F26 |
| T325 F8 PreferRecency (`#230`) | **Not stolen** |
| T326 `PinnedCountFailed` (`#237`) | **Not stolen** |
| T307 Blocked / T308 floors / H2 / clap 5 / T240 F2 | **Not stolen** / **Decline** F16 |
| last-PR Cursor `#240` / `#239` | **N/A empty** (Bugbot overview, no defect) |
| last-PR `#237` pin-count | **T326** already Pending |
| last-PR `#230` F8 recency | **T325** already Pending |
| `ISSUES.md` | **Does not exist** F19 |
| DOCS TX (plan) | `156b2a03-b5aa-4905-b840-d14fb182aa90` |
| Agy m1 HEAD `ed2f5f8` vs `93a788a` | **Folded** snapshot `93a788a` / ahead **1** |
| Agy m2 empty vs residuals-only | **Already** F4 / AC3 / AC6 |
| OpenCode m1 Default-mode flip census | **Folded** F31 / AC3 |
| OpenCode m2 AC5 quiet fixtures all-plain | **Folded** AC5 mixed-quiet new fixture; AC20 all-residual quiet + dual-flag named |
| OpenCode O1 two trailer formats | **Folded** F9 `format_mixed_fail_trailer` + unit |
| OpenCode O2 list empty untested | **Folded** AC6 named hermetic |
| Agy O1/O2/O3 usable-only / stdout / mixed verify | **Already** F1 / F2 / F5 |
| DOCS TX (fold-in) | `5f4aace2-b78d-4757-961f-12bc2366f5b3` |

---

## 10. Implement order (on go)

1. Phase 0 re-read `run_list` / F6 / `run_verify` default arm / `verify_report.rs` / T209+T244 hermetics / **F31 census** (`:82/:164/:336/:394/:430`) / T277 mixed list / smoke mixed verify; rescan deferred; FEATURE TX; record live N.
2. Red hermetics AC1–AC6/AC8/AC14/AC20 + `format_mixed_trailer__contains_verbose_and_count` (must fail).
3. Green: usable-only emit + stdout footer + `format_mixed_fail_trailer` + verify mixed gate + List after_help.
4. Stay-green AC7/AC9–AC13/AC16/AC18; F31 same-commit flips; doctor mixed Ok.
5. Docs CAPABILITIES / OPERATIONS / CHANGELOG.
6. Manual AC17; targeted clippy/nextest; FEATURE cross-model; full gate; publish (implement-track Phase 6). Never `git push origin main`.

Suggested series order after this plan: **T318 go** (daily DR skim) or **T325** (F8 recency) or **T326** (glance pin-count). Then T321. T307 stays Blocked.

---

## 11. Soft residuals

| Residual | Note |
|----------|------|
| Live 22 residual `.bak` still KeyMismatch / plain / Incomplete | F12 — expected; verify exit 1 |
| PATH until owner `cargo install` | F27 — T318 hole is already on PATH; T316 is not |
| T209 L3 real wrong-key SQLCipher fixture | Still soft |
| verify JSON `summary` / `--quiet` | F13 declined |
| Class-aware prune | T244 F18 declined |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/backup.rs` | Default/Quiet usable-only rows; F6 `println!`; `run_verify` mixed gate calls `format_mixed_fail_trailer` |
| `crates/ai-brains-cli/src/verify_report.rs` | **Required** `format_mixed_fail_trailer` + unit `format_mixed_trailer__contains_verbose_and_count` |
| `crates/ai-brains-cli/src/main.rs` | `BackupCommands::List` after_help one sentence only |
| `crates/ai-brains-cli/tests/backup_list_honesty.rs` | AC1–AC5/AC20/F31 census; AC6 empty-list; AC14 after_help |
| `crates/ai-brains-cli/tests/backup_recoverable.rs` | Mixed Default omits residual row; footer stdout; doctor stay-green |
| `crates/ai-brains-cli/tests/smoke.rs` | Flip mixed default `FAIL —` assert (AC8) |
| `Docs/CAPABILITIES.md` | §11 emit sentences |
| `Docs/OPERATIONS.md` | List/verify one-liners |
| `CHANGELOG.md` | Unreleased |
| `conductor/conductor.md` | T318 Planned |
| `conductor/deferred.md` | This plan |
| **Do not touch** | `doctor.rs`, brain `backup.rs` production, `project.rs`, `sync.rs`, `forget.rs` production, `status.rs` (T326), contracts, retrieval |

---

## 13. AI fold-in disposition (2026-08-29)

Source: `agy-review.md` + `opencode-review.md` (HEAD `93a788a`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.**

### Agy

| ID | Verdict | Action |
|----|---------|--------|
| **m1** HEAD `ed2f5f8` vs `93a788a` | **Agree** | Snapshot `93a788a` / ahead **1** of `origin/main` `ed2f5f8` |
| **m2** empty dir vs residuals-only | **Already** | F4 / AC3 / AC6 |
| **O1** usable-only table | **Already** | F1 / AC1 |
| **O2** F6 stdout | **Already** | F2 / F30 / AC2 |
| **O3** mixed verify summary | **Already** | F5 / AC8 / AC9 |

### OpenCode

| ID | Verdict | Action |
|----|---------|--------|
| **m1** Default-mode flip set under-enumerated | **Agree** | F31 census; Phase 0 names `:82/:164/:336/:394/:430` + recoverable |
| **m2** AC5 “usable row present” on all-plain quiet fixtures | **Agree** | AC5 = mixed-quiet **new** fixture; AC20 = existing all-plain quiet + dual-flag named |
| **O1** two trailer SOOTs | **Agree** | F9 `format_mixed_fail_trailer` in `verify_report.rs` + unit (do not merge with T225 overflow string) |
| **O2** `No backups found.` untested | **Agree** | AC6 named hermetic `backup_list__empty__no_backups_found_exit_0` |

### Pins locked by fold-in

1. **F31:** same-commit flip of every Default-mode residual-token / stderr-SOOT assert listed.
2. **AC5 ≠ AC20:** mixed quiet needs a readable backup; all-residual quiet stays plain-only (`No usable backups.`).
3. **F9:** mixed trailer helper in `verify_report.rs`; T225 overflow trailer frozen.
4. **AC6:** list empty is a real hermetic, not “stay-green T198”.
5. **last-PR:** `#240` / `#239` N/A empty; `#237` → T326; `#230` → T325; **no T327**.

Plan-write HEAD `ed2f5f8`. Fold-in against `93a788a` (ahead **1**). last-PR `#240` empty / `#239` empty / `#237` → T326 / `#230` → T325 / no T327. Still **plan-only until go**.
