# Deferred Follow-Ups

Tracks deferred from T142. Append-only; strike through when promoted to a real track.

### T323 fold-in (2026-08-29) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m2 HEAD `766a6c8` vs `0ead377` | **Folded** snapshot `0ead377` / ahead **1** |
| Agy m1 / m3 / O1 / O2 | **Already** F35 / F1 / F29 / F9 |
| OpenCode m1 Proposed upsert resets state | **Decline** — `ON CONFLICT` omits `state`/`superseded_by` (`projector :43–52`) |
| OpenCode m1 fixture pick | **Folded** F37 `EventBuilder` single `ConclusionSuperseded` self-hop |
| OpenCode m2 linear-scan passes suite | **Decline** — AC1 `chain.len()==1` already fails a non-walker |
| OpenCode m2 three-hop | **Folded** AC17 |
| OpenCode O1 status comment | **Decline** as required — F4 T311 mirror |
| OpenCode O2 AC16 live null | **Folded** expected-empty sentence |
| last-PR `#244` Cursor | **Affirm** N/A empty — no T327 |
| Agy/OpenCode B / M | None filed |
| DOCS TX | `853b18d9-ee2e-4ed9-afe3-01962bab0430` |

### T323 full plan (2026-08-29) — conclusion in-force walker (no `--as-of`)

| Item | Disposition |
|------|-------------|
| T311 R5 conclusion in-force | **Absorb** F1–F12 / AC1–AC13 / AC16 — **walker** (`correct_conclusion` + projector `superseded_by`) |
| Placeholder “decline if no chain” | **Superseded** — chain exists in live src |
| T322 `--as-of` copy | **Decline** F30 — residual §11 |
| T311 R1 daemon `ListInForce` | **Decline** F13 |
| T311 R7 PowerShell empty TERM | **Not stolen** T324 |
| T322 implement residuals | **Not stolen** — T322 Completed `#244` |
| T325 F8 PreferRecency (`#230`) | **Not stolen** |
| T326 `PinnedCountFailed` fake `pinned=0` (`#237`) | **Not stolen** |
| T307 Blocked / T308 floors / H2 / clap 5 / T240 F2 | **Not stolen** / **Decline** |
| last-PR Cursor `#244` | **N/A empty** (no defect) |
| last-PR `#237` / `#230` | **T326** / **T325** already Pending — **no T327** |
| `ISSUES.md` | **Does not exist** |
| DOCS TX | `61b188d1-fd07-48e6-9bec-bdce0d197c60` |

### T322 implement residuals (2026-08-29) — non-easy lows

| Item | Disposition |
|------|-------------|
| Propose→approve gap on superseded/revoked node (`updated_at` overwritten; hop uses `[valid_from, hop_at)`) | **Deferred** §11 / F9 — column declined |
| PATH `ai-brains` until owner `cargo install` (post-T322 needs install for `--as-of`) | **Deferred** F27 — hermetic/`cargo run` SoT |
| Live vault `workspace_id` ruling null honesty (AC14 pass-with-observed-data) | **By design** — no live propose |
| Daemon `ListInForce` / contracts DTO | **Decline** F11 / T311 R1 |
| Date-only `--as-of` UX | **Decline** F30 |
| `supersede_decision` ignores injected `Clock` (wall `build_event`) | **Deferred** soft — tests use stored `updated_at` |
| ~~F6(b) Superseded without `updated_at > at` on orphan/broken successor~~ | **Fixed** Codex P2 — guard + hermetic `broken_successor_after_hop` |
| AC16 distinct wall-hop assert (`d1_hop < d2_hop`) | **Deferred** T322-R5 — fails closed if same-tick; unlikely |
| Exact revoke-boundary hermetic (`as_of == revoke updated_at`) | **Deferred** soft P3 — closed-open covered by supersede AC4 |
| T323 / T324 / T325 / T326 / T307 | **Not stolen** |
| FEATURE TX / PR | `331ce060` / `#244` → `766a6c8` |

### T322 fold-in (2026-08-29) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m2 HEAD `0eef80b` vs `7867d56` | **Folded** snapshot `7867d56` / ahead **1** |
| Agy m1 AC3/AC11 same-tick `valid_from` | **Folded** F37 explicit `2020-01-01T00:00:00Z` |
| Agy m3 / O1 unwrap + wrapper | **Already** F16/F29 / F1 |
| Agy O2 omit `as_of` key | **Folded** AC10 (a) |
| OpenCode m1 F6 Proposed-as-ruling | **Already** §2.2 / F6 residual; no column |
| OpenCode m2 AC10 CLI cannot prove key | **Partial** — CLI unknown+`--as-of` **does**; CP `to_value` on AC3/AC4; **decline** negative-only |
| OpenCode m2b flag-before-TERM | **Folded** AC7 |
| OpenCode O1 AC5 `valid_from` None | **Decline** — payload/projector store Some(proposal) |
| OpenCode O2 three-chain prefix | **Folded** AC16 |
| last-PR `#243` Cursor | **Affirm** N/A empty — no T327 |
| Agy/OpenCode B / M | None filed |
| DOCS TX | `418e2547-d972-4457-a1cb-c927b5f41f37` |

### T322 full plan (2026-08-29) — `decision in-force --as-of` (hop-stop; no column)

| Item | Disposition |
|------|-------------|
| T311 R2 `--as-of` | **Absorb** F1–F7 / AC1–AC7 / AC11 / AC14 / AC15 |
| T311 R4 `approved_at` column | **Decline** F9 — superseded/revoked `updated_at` is the hop timestamp; event `approved_at` stays unprojected |
| Projector `valid_until` close on supersede | **Decline** F10 — overlap with successor `valid_from` |
| T311 R1 daemon `ListInForce` | **Decline** F11 |
| T311 R3 sibling Approved | **Decline** — T311 F7 freeze |
| T311 R5 conclusion in-force | **Not stolen** T323 |
| T311 R7 PowerShell empty TERM | **Not stolen** T324 |
| Date-only `--as-of` / `--from`/`--to` | **Decline** F30 / F2 |
| T321 implement residuals | **Not stolen** |
| T325 F8 PreferRecency (`#230`) | **Not stolen** |
| T326 `PinnedCountFailed` fake `pinned=0` (`#237`) | **Not stolen** |
| T307 Blocked / T308 floors / H2 / clap 5 / T240 F2 | **Not stolen** / **Decline** |
| last-PR Cursor `#243` | **N/A empty** (no defect) |
| last-PR `#237` / `#230` | **T326** / **T325** already Pending — **no T327** |
| `ISSUES.md` | **Does not exist** |
| DOCS TX | `d8e6e556-cfb8-4cd6-84cc-3f5b1599532c` |

### T321 implement residuals (2026-08-29) — non-easy lows

| Item | Disposition |
|------|-------------|
| PATH `ai-brains` until owner `cargo install` (pre-T321 emit on PATH) | **Deferred** F27 — hermetic/`cargo run` SoT; PATH-behind not Complete-blocking |
| In-context hotspots 0 on **PATH** until install (source envelope parse is SoT) | **Deferred** soft — expected until PATH catch-up |
| `tracing::warn!` JSON→text fallback still on **stdout** under default fmt subscriber | **Deferred** F4 / OpenCode m1 — subscriber freeze; silence `--log-format off` / `RUST_LOG` |
| T279 F35 unbounded `ledgerful hotspots` wait | **Decline** F35 — no timeout crate this track |
| `displayScore` not shown (raw `score` only) | **By design** F5 — not a defect |
| T318 PATH install / live residuals / PreT109 hermetic / recall timeout under load | **Carry** prior soft residuals — not stolen |
| T322–T324 / T325 F8 PreferRecency / T326 pin-count | **Not stolen** |
| T307 Blocked | **Not stolen** |
| FEATURE TX | `3fadf62c-976d-4257-8a20-08960683292e` |
| PR | [#243](https://github.com/Ryan-AI-Studios/AI-Brains/pull/243) squash `0eef80b` |

### T321 full plan (2026-08-29) — `safety sync` write honesty (banner-only; JSON envelope)

| Item | Disposition |
|------|-------------|
| Audit `safety sync` 5/5 write surprise + chatter | **Absorb** F1–F6 / AC1–AC4 / AC7 / AC14 |
| Placeholder dry-run-by-default vs banner | **Pick banner** F1 — default stays write; T279 remediator is already `--dry-run` |
| T279 remediator `safety sync --dry-run` / SAFETY_EMPTY / GLOB | **Affirm freeze** F8 |
| T279 F21 no live pin as proof | **Affirm** F12 |
| T279 F29 CLI vs retrieval parse drift | **Partial** F7/F29 copy-not-share envelope (`files[]`); cap differs |
| T279 F35 unbounded `ledgerful hotspots` wait | **Decline** F35 |
| Live `hotspots --json` `{schemaVersion:1, files[]}` (dogfood; CLI text-fallback; retrieval empty) | **Absorb** F7 / AC5 / AC6 — restores T279 live inject + raw `score` |
| `WORKFLOWS.md` JSON `LedgerEntry` lie | **Absorb** F10 / AC13 |
| `antigravity-rule.md` session-start write | **Absorb** F33 — `--dry-run` or `preflight` |
| T316 stderr analog | **Analog only** F30 — banner on stdout |
| T322 / T323 / T324 | **Not stolen** |
| T325 F8 PreferRecency (`#230`) | **Not stolen** |
| T326 `PinnedCountFailed` fake `pinned=0` (`#237`) | **Not stolen** |
| T307 Blocked / T308 floors / H2 / clap 5 / T240 F2 | **Not stolen** / **Decline** |
| last-PR Cursor `#242` / `#241` | **N/A empty** (no defect) |
| last-PR `#237` / `#230` | **T326** / **T325** already Pending — **no T327** |
| `ISSUES.md` | **Does not exist** |
| DOCS TX | `956c8463-c577-44cf-a614-169d77117446` |

### T321 fold-in (2026-08-29) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m3 HEAD `16edc3f` vs `42df225` | **Folded** snapshot `42df225` / ahead **1** |
| Agy m1 AC6 spawn hazard | **Folded** F7/AC6 `parse_ledgerful_hotspots_json` (no spawn) |
| Agy m2 score vs displayScore false-pass | **Folded** AC5/AC6 fixture both fields distinct |
| Agy m4 / OpenCode O2 serde required freq/complexity | **Folded** F7 `#[serde(default)]` on CLI those two fields |
| Agy O1 Value one-pass | **Decline** as required — impl detail; AC5/AC6 SoT |
| Agy O2 docs-file hermetics | **Decline** as DoD — AC13 grep |
| OpenCode m1 F4 tracing warn “(not stdout)” | **Folded** F4 reword; subscriber freeze; `--log-format off` residual |
| OpenCode m2 write-path unproven | **Partial** F5/AC17 `format_detail_row`; **decline** tempdir write (F12) |
| OpenCode O1 `--limit 0` empty | **Folded** AC15 |
| OpenCode O3 F29 “unbounded”/“20” | **Folded** operator-set `--limit` (default 5), no inject cap |
| last-PR `#242` / `#241` Cursor | **Affirm** N/A empty — no T327 |
| Agy/OpenCode B / M | None filed |
| DOCS TX | `573fb6ba-01f8-4ccb-b40d-3d0d3e6d58f2` |



### T318 implement residuals (2026-08-29) — non-easy lows

| Item | Disposition |
|------|-------------|
| PATH `ai-brains` until owner `cargo install` (pre-T318 emit on PATH) | **Deferred** F27 — hermetic/`cargo run` SoT; PATH-behind not Complete-blocking |
| Live ~22 residual KeyMismatch/plain/Incomplete fleet | **Deferred** F12 — expected; verify exit 1; no transcode this track |
| T209 L3 real wrong-key SQLCipher fixture | **Deferred** soft / declined DoD |
| verify JSON `summary` / verify `--quiet` / class-aware prune | **Decline** F13 |
| PreT109 Default visibility dedicated hermetic (CX-P2-2) | **Deferred** low-info — `is_usable_class` already includes PreT109; readable mixed hermetic covers usable band |
| Recall hermetic 120s timeouts under heavy parallel load | **Deferred** low-info — unrelated; `NEXTEST_TEST_THREADS=2` full workspace PASS; same as T316 residual |
| T325 F8 PreferRecency / T326 pin-count / T321–T324 | **Not stolen** |
| FEATURE TX | `93fbf235-8dc2-40d8-add1-9ac9bfc2643b` |

### T318 fold-in (2026-08-29) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 HEAD `ed2f5f8` vs `93a788a` | **Folded** snapshot `93a788a` / ahead **1** |
| Agy m2 empty vs residuals-only | **Already** F4 / AC3 / AC6 |
| Agy O1/O2/O3 usable-only / stdout F6 / mixed verify | **Already** F1 / F2 / F5 |
| OpenCode m1 Default-mode flip census | **Folded** F31 — `:82/:164/:336/:394/:430` + recoverable same commit as AC1 |
| OpenCode m2 AC5 all-plain quiet cannot show usable row | **Folded** AC5 mixed-quiet new fixture; AC20 all-residual quiet + dual-flag named |
| OpenCode O1 two trailer formats | **Folded** F9 `format_mixed_fail_trailer` + unit (do not merge T225 overflow string) |
| OpenCode O2 list `No backups found.` untested | **Folded** AC6 `backup_list__empty__no_backups_found_exit_0` |
| last-PR `#240` / `#239` Cursor | **Affirm** N/A empty — no T327 |
| Agy/OpenCode B / M | None filed |
| DOCS TX | `5f4aace2-b78d-4757-961f-12bc2366f5b3` |

### T318 full plan (2026-08-29) — backup list usable-only Default + mixed-verify summary

| Item | Disposition |
|------|-------------|
| ~~Audit `backup list` 6/6 residual fleet noise~~ | **Resolved** T318 — usable-only Default + stdout F6 + mixed-verify summary |
| T244 F7 CLI usable-first sort / brain timestamp-desc | **Affirm freeze** F6/F7 — do not edit brain `backup.rs` production |
| T244 F6 stderr residual summary | **Supersede stream** F2 — same SOOT on **stdout** (T316 Windows-first analog; do not drop the count) |
| T225 first-5 FAIL on default verify | **Partial** F5 — keep zero-OK + nudge; **supersede mixed** (`ok>=1` counts + `--verbose` trailer, no `FAIL —`) |
| T225/T244 F17 verify `--quiet` / JSON `summary` / `VerifyError` | **Decline** F13 |
| T244 F18 class-aware prune / `backups/legacy/` | **Decline** F13 |
| T209 L3/L4 wrong-key fixture / dedicated PreT109 unit | **Decline** (soft; not this DoD) |
| T277 create engine / doctor remediator `ai-brains backup create` / keep-10 | **Affirm freeze** F10 — do not grow `doctor.rs` |
| T295 ≥1 usable + mixed verify exit 1 + no nudge | **Affirm** F5/F24 / AC8/AC19 |
| T316 F36 stderr drop | **Analog only** F30 — F6 **moves** (count is the product) |
| T321 / T322–T324 | **Not stolen** |
| T325 F8 PreferRecency (`#230`) | **Not stolen** |
| T326 `PinnedCountFailed` fake `pinned=0` (`#237`) | **Not stolen** |
| T307 Blocked / T308 floors / H2 / clap 5 / T240 F2 | **Not stolen** / **Decline** |
| last-PR Cursor `#240` / `#239` | **N/A empty** (Bugbot overview, no defect) |
| last-PR `#237` / `#230` | **T326** / **T325** already Pending — **no T327** |
| `ISSUES.md` | **Does not exist** |
| DOCS TX | `156b2a03-b5aa-4905-b840-d14fb182aa90` |


### T316 implement residuals (2026-08-29) — non-easy lows

| Item | Disposition |
|------|-------------|
| PATH `ai-brains` until owner `cargo install` | **Deferred** F31 — hermetic/`cargo run` SoT; PATH-behind not Complete-blocking |
| T287 R1-1 live GLOB+retain empty → recency first page | **Deferred** F27 — preview DoD landed; ORDER not this track |
| Agent prefix set may grow (`Now let me check whether…`) | **Deferred** F2/F33 — closed set; extend only with evidence |
| JSON preview values change for chrome rows | **By design** F7 — not a key change; not a residual bug |
| OpenCode O2 briefing/graph inherit hermetics | **Decline** F6/F14 — helper units are inherit SoT |
| T325 F8 PreferRecency / T318 / T321–T324 | **Not stolen** |
| T326 `PinnedCountFailed` fake `pinned=0` | **Not stolen** — Pending placeholder |
| T319 `memory show <id>` | **Decline** F13 |
| FEATURE TX | `50c73816-3152-499e-bee9-1b5aeb7b0aec` |
| Recall hermetic 120s timeouts under heavy parallel load | **Deferred** low-info — unrelated to T316; serial re-run 10/10 PASS; do not broaden-fix |

### T316 fold-in (2026-08-29) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 HEAD `d1c3bd3` vs `120bbfa` | **Folded** snapshot `120bbfa` / ahead **1** |
| Agy m2 all-chrome fallback | **Already** F5 / AC5 |
| Agy O1/O2/O3 F36 / skip walk / authority | **Already** F9 / F1–F2 / F3 |
| OpenCode m1 F3 walk-stop underspecified | **Folded** F1/F3 first-non-chrome; **AC19** fence-then-Decision |
| OpenCode m2 AC14 after_help unnamed | **Folded** AC14 `memory_list_help__after_help__names_chrome_skip_and_no_forget_hint` |
| OpenCode O1 `classify_pin_kind("")` → Other | **Folded** F3 |
| OpenCode O2 briefing/graph inherit smoke | **Partial** F6 helper units; **decline** extra hermetics (F14) |
| OpenCode O3 T326 line citations | **Already** T326 Phase 0 re-cite |
| last-PR `#238` / `#237` Cursor | **Affirm** N/A empty / **T326** — no T327 |
| Agy/OpenCode B / M | None filed |
| DOCS TX | `69e50ba1-5c35-49d4-abb3-56f1ff6419c6` |

### T316 full plan (2026-08-29) — memory list chrome-skip preview + drop F36 stderr

| Item | Disposition |
|------|-------------|
| Audit `memory list` 6/6 raw first-line previews + F36 stderr-as-error | **Absorb** F1–F10 / AC1–AC9 / AC17 |
| T216 F36 stderr next-step | **Supersede runtime** F9; after_help/docs stay |
| T287 ORDER / JSON recency / envelope `preview_line` | **Affirm** F8 / F7; **extend** skip after envelope F1 |
| T287 R1-1 live GLOB+retain empty → recency first page | **Partial** F27 — preview still DoD; ORDER not |
| T299 empty `Pinned: N` + `next:` | **Affirm** F11; **update** nonempty F36 hermetic AC9 |
| T319 no `memory show` | **Decline** F13 |
| T318 / T321 / T322–T324 | **Not stolen** |
| T325 F8 PreferRecency (`#230`) | **Not stolen** |
| T307 Blocked / T308 floors / H2 / clap 5 / T240 F2 | **Not stolen** / **Decline** |
| last-PR Cursor `#238` | **N/A empty** |
| last-PR Cursor `#237` Bugbot `PinnedCountFailed` fake `pinned=0` | **Mint T326** — still true `status.rs:329–340` + `graph.rs:445–458`; doctor skip is SOOT |
| last-PR `#230` F8 recency | **T325** already Pending |
| DOCS TX | `66b597f7-faf9-4f3e-bb06-6af72811bdc6` |

### T326 mint (2026-08-29) — `#237` Cursor leftover (placeholder)

| Item | Disposition |
|------|-------------|
| `#237` Bugbot medium: `PinnedCountFailed` invents `pinned=0` then assesses (`status.rs:329–340`) | **T326** Pending placeholder |
| `graph.rs:445–458` same fake 0 | **Absorb into T326** (same hole) |
| Doctor skip `:901` | **SOOT** — do not grow doctor |
| T316 list preview | **Not stolen** (this mint) |
| T320 four-section compose / floors 0.50 | **Affirm freeze** until `/plan-track T326` |

### T320 implement residuals (2026-08-29) — non-easy lows

| Item | Disposition |
|------|-------------|
| PATH `ai-brains` until owner `cargo install` | **Deferred** F22 — hermetic/`cargo run` SoT; PATH-behind not Complete-blocking |
| Live E/N ~0.42 still sparse | **Deferred** F36 honesty — no floor retune / rebuild on glance |
| Doctor Safety vs glance Status probe | **Deferred** F9 by design — Status 1×300 ms matches `daemon status` |
| Two vault opens (`build_report` + glance `open_read_intent`) | **Deferred** F44 — not a conn-sharing track; do not grow doctor |
| F32 other-file physical net ~82 incl. clap test blocks in `main.rs` | **Deferred** — production-only estimate ~60 under 80; test blocks inflate file total |
| T325 F8 PreferRecency / T316 / T318 / T321–T324 | **Not stolen** (T316 Planned 2026-08-29; T326 minted from `#237`) |
| FEATURE TX | `a700986c-41d5-41b6-b417-6cac9153be0e` |

### T320 fold-in (2026-08-29) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 HEAD `464edc2` vs `e15188e` | **Folded** snapshot `e15188e` / ahead **1** |
| Agy m2 `graph_density.rs` not under `commands/` | **Folded** §2.3 / F11 — `src/graph_density.rs` via `main.rs:9` |
| Agy O1/O2/O3 in-process / fail-open / graph-off | **Already** F2 / F4 / F11 |
| OpenCode m1 line counts 1738 vs 1855 | **Partial** dual-count (nonblank vs physical); F32 go-HEAD. **Decline** “pre-T317 / `src/graph.rs`” |
| OpenCode O1 AC9 host daemon IPC | **Folded** AC9 / F45 — do not assert `daemon.state` |
| OpenCode O2 `status_next_line` reuse | **Folded** F27 / AC7 — human = helper; JSON prefix-less |
| OpenCode O3 scheduled mapper | **Folded** F12 / AC6 — live `next_run.is_some()` (`nightly.rs:104`), not Router `found &&` |
| last-PR `#236` / `#235` Cursor | **Affirm** N/A empty — no T326 |
| Agy/OpenCode B / M | None filed |
| DOCS TX | `a92f9b07-1894-42a1-8526-9f66fa9ed02d` |

### T320 full plan (2026-08-29) — unified `ai-brains status` glance

| Item | Disposition |
|------|-------------|
| Audit opportunity (b) no single `status` | **Absorb** F1–F15 / AC3–AC12 — in-process compose; fail-open |
| Placeholder name vs `daemon status` | **Absorb** F1 — top-level `Commands::Status`; nested unchanged |
| Placeholder compose vs subprocess | **Absorb** F2 — never PATH subprocess |
| Placeholder 750 ms fail-open | **Absorb** F4 / F7 — no HTTP; Status IPC 300 ms |
| T192/T249 doctor 15 + `--summary` | **Affirm** F6 / F10 / F38 — reuse; do not replace |
| T199/T297 `daemon status` | **Affirm** F8 / F39 — do not call `run_status` |
| T255 nightly JSON / Router / 750 ms | **Affirm** F12 / F37 — last-run + schtasks only |
| T308 Sparse remediator None / floors 0.50 | **Affirm** F36 |
| T204 Daily string lock | **Partial** F17 — additive `status` only |
| T310 F15 `ai-brainsd --version` | **Decline** F20 |
| minikube bitwise exit | **Decline** F31 |
| Unsummarized / Router / TCP on glance | **Decline** F12 / F8 |
| T263 H2 / T240 F2 / clap 5 | **Decline** F20 |
| T316 / T318 / T321–T324 | **Not stolen** |
| T325 F8 PreferRecency (`#230`) | **Not stolen** |
| T307 Blocked / T308 floor retune | **Not stolen** / **Decline** |
| last-PR Cursor `#236` / `#235` | **N/A empty** — no T326 |
| last-PR `#230` F8 recency | **T325** already Pending |
| DOCS TX | `dcb67912-8fb7-4bbd-a354-68ba41857744` |

### T319 implement residuals (2026-08-28) — non-easy lows

| Item | Disposition |
|------|-------------|
| PATH `ai-brains` until owner `cargo install` | **Deferred** F22 — hermetic/`cargo run` SoT; PATH-behind not Complete-blocking |
| Live audit id `431f6505-…` may be forgotten later | **Deferred** — Manual pass-with-observed-data; hermetic AC5–AC8 SoT |
| Vault still 0 governed evidence rows | **Deferred** — honesty not populate (H1); H2 declined |
| No `memory show <id>` remediator | **Deferred** — next-step is recall needle; T316 is list preview not show-by-id |
| Daemon `HandlePreviewDto` / Inspect* IPC unaugmented | **Deferred** F7/F30 — CLI Value overlay only by design |
| T312 PATH dump-first / T325 F8 PreferRecency | **Not stolen** |
| FEATURE TX | `ce627277-0e01-40c8-8b96-b810f07186c4` |

### T319 fold-in (2026-08-28) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode m1 daemon `run_show_daemon` takes no ctx | **Folded** F2 / F30 / Phase 2 / §5.1 — thread `&ctx`; source Error before `expect_daemon_ok` `:671` |
| OpenCode m2 line counts 1029/463/305/303 vs 1133/496/320/322 | **Folded** §2.3; F32/F23 80-net is phase diff |
| OpenCode m3 AC15 conditional found-kind | **Folded** AC3 `kind: "Evidence"` fixture; AC15 = AC3 |
| OpenCode O1 AC1 daemon proof | **Folded** F23 |
| OpenCode O2 AC8 stderr order | **Folded** AC8 T221 F5 bare hint |
| OpenCode O3 word 740→669 | **Note** §2.1 |
| Agy m1 HEAD `fa353c7` vs `14198b5` | **Folded** snapshot `14198b5` / ahead **1** |
| Agy m2 EXISTS `Err` → false | **Already** F1 |
| Agy O1/O2/O3 F7 / F3–F4 / F6 | **Already** |
| last-PR `#234` Cursor | **Affirm** N/A empty — no T326 |
| Agy/OpenCode B / M | None filed |
| DOCS TX | `09c2659f-962a-40e5-a04f-92f2de9c4f8d` |

### T319 full plan (2026-08-28) — handle vs memory UUID namespace

| Item | Disposition |
|------|-------------|
| Audit `evidence show` / `source show` on a vault `memory_id` | **Absorb** F1–F8 / AC5–AC8 / AC11 — name namespace; no H2 |
| Audit `query expand` same UUID hole | **Absorb** F3 / AC5–AC6 — preview replaces `Handle not found.` when EXISTS |
| Evidence Unknown empty preview (T263 overlay never wired) | **Absorb** F2 |
| T263 H1 `Handle not found.` / Unknown exit **0** | **Affirm** F3 / F11 / AC9 stay-green for non-memory UUID |
| Source `NOT_FOUND` exit **4** | **Affirm** F4 — additive `details.hint` only |
| T290 granted-empty list copy | **Not stolen** F12 |
| T263 H2 pin→Approved / migrate | **Decline** F9 / F20 |
| T167 EvidenceId prefers `memory_id` | **Not stolen** — import-only |
| T316 / T317–T318 / T320–T324 | **Not stolen** |
| T325 F8 PreferRecency (`#230`) | **Not stolen** |
| T307 Blocked / T308 floors / clap 5 / T240 F2 | **Not stolen** / **Decline** |
| last-PR Cursor `#234` | **N/A empty** — no T326 |
| last-PR `#230` F8 recency | **T325** already Pending |
| DOCS TX | `844bdbed-7295-4635-a04f-968d224e41ec` |

### T317 implement residuals (2026-08-28) — non-easy lows

| Item | Disposition |
|------|-------------|
| PATH `ai-brains` until owner `cargo install` | **Deferred** F15 — source/`cargo run --features graph` + hermetic SoT; not Complete-blocking |
| Live N on `431f6505-…` moves (plan 11 / OpenCode 12 / Manual **12**) | **Deferred** — hermetic AC1/AC14 SoT; Manual pass-with-observed-data |
| Kept session PREVIEW still `## Objective` on capped RECALLS | **Deferred** — T278/T293 honesty; DoD is cardinality not caption rewrite |
| Hierarchy of a real pin stays a leaf | **Deferred** — by design; next-step is orientation only |
| Sparse E/N ~0.41 floors | **Deferred** — T308 frozen; not stolen |
| JSON still lists all RECALLS (11/12) | **Deferred** — F3 by design (dual-truth) |
| T312 PATH dump-first / T325 F8 PreferRecency | **Not stolen** |
| FEATURE TX | `39e0e1e4-577c-4b18-a4d9-59101d163020` |

### T317 fold-in (2026-08-28) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode m1 F31 lists only `:1129`; live also `:1229` / `:1383` | **Folded** F31 / Phase 2 — `(2,0)` / `(2,0)` / `(51,0)` |
| OpenCode m2 uuid workspace 1.13 vs lock 1.23.1 | **Folded** §2.4 lock **1.23.1** |
| OpenCode O1 AC9 unnamed / overlap `:616` | **Folded** AC9 in `graph_human_cli.rs`; RECALLS count ≥ 4 |
| OpenCode O2 live N 11→12 | **Note** AC11 observed-data; §2.1 |
| Agy m1 HEAD `dae7df3` vs `e17678d` | **Folded** snapshot `e17678d` / ahead **1** |
| Agy m2 footer order limit then RECALLS | **Already** §5.2; **tightened** F9 / AC17 |
| Agy O1/O2/O3 F3 / F9 / F2 | **Already** |
| last-PR `#233` Cursor | **Affirm** N/A empty — no T326 |
| Agy/OpenCode B / M | None filed |
| DOCS TX | `e1ef2696-8ee0-47e3-9136-04f41d336cdc` |

### T317 full plan (2026-08-28) — human RECALLS cap 3 + hierarchy leaf nightly next

| Item | Disposition |
|------|-------------|
| Audit `graph neighbors` RECALLS spam (19; live **11** on `431f6505-…`) | **Absorb** F1 / F9 / AC1 / AC5 / AC11 / AC14 |
| Audit hierarchy `synthesized_from` empty | **Absorb** F2 / AC7 / AC12 — `next: ai-brains nightly --status`; AC9 still forbids graph update/rebuild |
| T293 prefer-authority / no-drops | **Affirm** F5 — cap **after** prefer; JSON still full |
| T293 F8 no `--label` / F11 hierarchy freeze | **Affirm** F8; **supersede F11 for leaf copy only** |
| T246 JSON keys / T262 RECALLS survive | **Affirm** F3 / F7 / AC8 / AC15 |
| T308 R1 live E/N ~0.41 | **Decline** floor change F7 |
| T278 F18 2-hop / projector delete RECALLS | **Decline** F4 / F7 |
| T316 / T318–T324 / T325 F8 recency | **Not stolen** |
| T313 `#233` / T312 / T314 / T315 | **Not stolen** |
| T307 Blocked / T308 floors / H2 / clap 5 | **Not stolen** / **Decline** |
| last-PR Cursor `#233` | **N/A empty** — no T326 |
| last-PR `#230` F8 recency | **T325** already Pending |
| DOCS TX | `0db2a64d-6ae6-4c25-b2fc-3a6db62d0dfa` |


### T313 implement residuals (2026-08-28) — non-easy lows

| Item | Disposition |
|------|-------------|
| PATH `ai-brains` until owner `cargo install` | **Deferred** F16 — source bin / hermetic prove DoD; not Complete-blocking |
| Broad rescue token `graph` still first-seen | **Deferred** F4 — honesty of heading, not scoring / length-sort |
| T211 `Note:` vs F7 `Note:` same prefix | **Deferred** F9 — heading differentiates; do not rename F7 |
| ndjson still vault-only (no combined envelope) | **Deferred** F7 — contract track if ever needed |
| Ledgerful still phrase-wraps spaces | **Deferred** F11 — other repo; T313 does not wait |
| 10 ledger rows on rescue (no `--limit` on argv) | **Deferred** F11 — T211 vault 5 vs ledger 10 freeze |
| T312 PATH dump-first / T325 F8 PreferRecency | **Not stolen** |
| FEATURE TX | `a58ee509-ed84-420b-9fd0-c4112782289d` |

### T313 fold-in (2026-08-28) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode m1 / Agy m2 heading `is_empty()` vs F25 whitespace | **Folded** F1 / F3 / F25 / AC3 — `!tok.trim().is_empty()` + `Some("   ")` |
| OpenCode m2 WORKFLOWS.md `:316` | **Folded** F14 / AC10 / §12 |
| OpenCode m3 AC13 `git diff -- C:\dev\Ledgerful` exit 128 | **Folded** AC13 in-repo `crates/` name-only |
| OpenCode m4 existing ndjson hermetics drop non-JSON | **Folded** AC14 new Phase 1 green-on-arrival |
| OpenCode O1 clap `Query` line | **Folded** §2.3 `:3590` / `:3629–3647` |
| OpenCode O2 `join` vs three `println!` | **Folded** §5.2 SoT |
| OpenCode O3 pin-count 4544→4545 | **Note** volatile |
| Agy m1 HEAD `cd7bfde` vs `2bec83e` | **Folded** snapshot `2bec83e` / ahead **1** |
| Agy O1/O2/O3 F10 / F2 / F7 | **Already** |
| last-PR `#232` Cursor | **Affirm** N/A empty — no T326 |
| Agy/OpenCode B / M | None filed |
| DOCS TX | `5fa5626e-ce2f-42df-97f4-744053ba09a5` |

### T313 full plan (2026-08-28) — rescued heading; F7 banner exact

| Item | Disposition |
|------|-------------|
| Audit `sync query` phrase→fuzzy opacity / “can’t tell which results came from where” | **Resolved** T313 implement — heading names rescued token |
| T271 F7 banner already prints on PATH | **Resolved** T313 heading `(rescued token: '<tok>')`; F2 banner exact |
| T271 F6 first-seen + cap 3 | **Affirm** F4 — no scoring / no skip-`graph` |
| T273 `--` / T271 F5 no FTS-quote | **Affirm** F5 / F6 |
| T231 always-pretty / ndjson vault-only | **Affirm** F7 — no combined JSON (no key exists) |
| T211 ledger-first `Note:` | **Affirm** F9 — do not rename F7 `Note:` |
| T271 residual Ledgerful token-OR / `--limit` / merge tables | **Decline** F11 |
| T312 rank / T314 clap / T315 summary / T316–T324 | **Not stolen** |
| T325 F8 PreferRecency (`#230`) | **Not stolen** |
| T307 Blocked / T308 floors / H2 / clap 5 | **Not stolen** / **Decline** |
| last-PR Cursor `#232` | **N/A empty** — no T326 |
| last-PR `#230` F8 recency | **T325** already Pending |
| DOCS TX | `bdf8fddd-84f9-4d9d-9b7d-64887dd834e2` |

### T314 implement residuals (2026-08-28) — non-easy lows

| Item | Disposition |
|------|-------------|
| PATH `ai-brains` until owner `cargo install` | **Deferred** F18 — source bin / hermetic prove DoD; not Complete-blocking |
| `query progressive --dry-run "query text"` may bool-steal positional | **Deferred** F34 by design — examples keep query first |
| Expand `--format auto` human only when explicit (default stays json) | **Deferred** F8 — Family C |
| `fail_cp` stays JSON even under `--format human` | **Deferred** F10 — CP errors not format-threaded this track |
| AC4 unit name `query_expand__format_human__parses` folded into `…format_json__parses` | **Deferred** low naming drift — behavior covered; split optional |
| T325 F8 PreferRecency / T319 handle UUID / clap 5 / T321 / T324 | **Not stolen** |
| FEATURE TX | `26f296f5-fd76-4d04-afba-6d26e54a1bc5` |

### T314 fold-in (2026-08-28) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode m1 AC7 named test absent / stay-green skip | **Folded** F6 / AC7 — new Phase 1 green-on-arrival unit |
| OpenCode O1 found-preview newlines | **Folded** F9 / AC16 |
| OpenCode O2 AC14 missing `--project-id` | **Folded** AC14 |
| OpenCode O3 PROTOCOL-COMPAT no expand row | **Folded** F25 / AC13 — **add** row |
| OpenCode O4 help-shape AC | **Decline** as DoD (optional) |
| OpenCode F1 clap 4.6.1 probe | **Folded** §2.4 |
| Agy m1 HEAD `ae6615d` | **Folded** snapshot `2a1eb35` / ahead **1** |
| Agy m2 F32 / O1/O2/O3 | **Already** F32 / F2 / F11 / F9 |
| last-PR `#231` Cursor | **Affirm** N/A empty — no T326 |
| Agy/OpenCode B / M | None filed |
| DOCS TX | `0d3c2e80-a309-41c0-b49b-08627ec2d373` |

### T314 full plan (2026-08-28) — clap `--format` / `--dry-run` unify

| Item | Disposition |
|------|-------------|
| Audit 5 clap errors (expand `--format`; progressive `--dry-run` value; scan-roots `--dry-run` unknown) | **Absorb** F1/F7/F11 / AC1–AC6 |
| Briefing project/personal same `ArgAction::Set` trap | **Absorb** F5 / AC3 |
| T290 F10 progressive JSON-only | **Affirm** F6 — no `--format`; **AC7 is a new guard** (fold-in) |
| T291 `--dry-run false` persist SOOT | **Affirm** F3 — optional-value, not `--commit` |
| T268 scan-roots dry-run-only | **Affirm** F11 no-op alias; do not write |
| T266 Family A auto rewrite | **Decline** — expand stays Family C default json |
| T319 / T321 / T324 | **Not stolen** |
| clap 5 / T240 F2 / T263 H2 / rotate-datakey `require_backup` | **Decline** F13/F14 |
| last-PR Cursor `#231` | **N/A empty** — no T326 |
| last-PR `#230` F8 recency | **T325** already Pending |
| DOCS TX | `23da7568-f134-4dde-8a9a-3842eb213cb7` |

## T312–T324 placeholders (2026-08-27) — post-T311 live CLI dogfood (0.1.3)

Minted from PATH **0.1.3** non-destructive dogfood + **entire** this file’s open residuals. Full F-list on `/plan-track TNN`. **Do not implement Placeholders.** last-PR Cursor **#230** (T312) minted **T325**. Series README `conductor/tracks/README-T312-T324-CLI-DOGFOOD.md`. Series DOCS TX `a6d3c404-1d64-4cba-a743-d75ac16c74cd`.

### T315 full plan (2026-08-28) — empty-decisions next-step + word-count label

| Item | Disposition |
|------|-------------|
| ~~Audit preflight 0/0/0 + opaque `Total Word Count`~~ | **Landed** T315 Completed — SOOT + `Budget window words:` |
| T286 live Index `## Objective` (R1-1) | **Decline steal** F11 — residual stands (not easy; Index SQL) |
| ~~T220 F30 human label vs JSON `word_count`~~ | **Landed** F7 `Budget window words:` |
| ~~T241 optional JSON `next_step`~~ | **Landed** reuse key; F5 grants win |
| T288 / T290 granted-empty overlay | **Decline steal** — needle `LIST_RECALL_QUERY` only (F3) |
| T263 H2 / T240 F2 / clap 5 | **Decline** F4 / F24 / F20 |
| T313–T324 / T307 / T308 floors | **Not stolen** / **Decline** |
| last-PR Cursor `#230` F8 Prefer-OR skips recency | **Mint T325** (does not fit T315 / T313–T324) |
| last-PR `#229` empty | **Superseded** by `#230` |
| DOCS TX | `ca5b1614-6849-416d-ad27-1d44a23198d7` |

### T315 implement residuals (2026-08-28) — non-easy lows

| Item | Disposition |
|------|-------------|
| T286 Index still `## Objective` on live vault (R1-1) | **Deferred** — needs Index SQL / budget fit track; T315 next-step honesty only |
| PATH `ai-brains` until owner `cargo install` | **Deferred** F18 — source bin proves AC11; not Complete-blocking |
| In-context decisions remain 0 after T315 | **By design** — product is `next:` remediator, not window stuffing |
| T325 F8 PreferRecency on OR-fill | **Not stolen** — placeholder Pending |
| Optional `after_help` T315 sentence | **Skipped** (plan optional); not deferred as debt |
| FEATURE TX | `a38a0cba-b820-4f36-8924-c13bff46b50a` |

### T315 fold-in (2026-08-28) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode m1 AC7 “next_step omitted” unreachable (project-scoped T241 bootstrap) | **Folded** F5/F30/AC7 — assert **not the T315 SOOT** |
| OpenCode O1 AC5/AC6 project-scoped fixture | **Folded** F38 scope-none empty vault |
| OpenCode O2 F23 existing import | **Folded** — one **new** `LIST_RECALL_QUERY` import |
| OpenCode O3 `lexical.rs` lines | **Folded** F21 `:197–214` / `:215–252` |
| OpenCode O5 AC4–AC6 names | **Folded** AC table |
| Both m/O HEAD `44520d8` | **Folded** snapshot `2b6919c` / ahead **1** |
| Agy m2 insert helper prefix | **Folded** F8 dual prefix |
| Agy O1/O2/O3 F7/F3/F5 | **Already** |
| last-PR `#230` Cursor | **Affirm** T325 — no T326 |
| Agy/OpenCode B / M | None filed |
| DOCS TX | `c90c1c71-aa57-40b4-8ee6-7b068837b4bc` |

### T325 mint (2026-08-28) — `#230` Cursor leftover (placeholder)

| Item | Disposition |
|------|-------------|
| `#230` Bugbot medium: F8 OR-fill no PreferRecency (`lexical.rs:231–250` vs AND `:197–213`) | **T325** Pending placeholder |
| T315 summary 0/0/0 | **Not stolen** (this plan) |
| T312 F8/F42 grammar | **Affirm freeze** until `/plan-track T325` |

### T312 implement (2026-08-27) — recall rank v3 (Completed)

| Item | Disposition |
|------|-------------|
| ~~Audit recall rank dump-first / F5 F6 F8~~ | **Done** — FEATURE TX `7f7e99bb`; hermetic AC1–AC17 + CLI AC10/12/13 |
| Codex P1 “structured synth not boosted” | **Soft residual R1** — spec §11 F6-by-design; no `KIND_SYNTH` this track |
| PATH until `cargo install` | **Soft residual R2** — F21; hermetic/`cargo run` SoT |
| Live `graph backend` may still dump-first if no OR-matching pin | **Soft residual R3** — honest; AC5 hermetic SoT |
| Pretty `score=` still raw BM25 | **Soft residual R4** — F38 decline |
| More ATX chrome tokens as vault grows | **Soft residual R5** — closed set; extend only with evidence |
| Semantic dumps above floor | **Soft residual R6** — T218 freeze; inherit F5/F6 only |
| exit_contract graph feature-off flake under parallel nextest | **Unrelated** — passes alone; not T312 regression |
| last-PR `#229` Cursor | **N/A empty** — no T325 |

### T312 full plan (2026-08-27) — authority-OR fill + verbose-Other penalty

| Item | Disposition |
|------|-------------|
| ~~Audit recall rank still dump-first (T285 shipped)~~ | **Done** T312 implement |
| ~~Live `graph backend` AND-retain empty~~ | **Done** F8 |
| ~~Prose dump #1 not chrome~~ | **Done** F6 |
| ~~`# Preview` substring false-hit~~ | **Done** F5 |
| ~~T285 “more chrome prefixes as vault grows”~~ | **Done** F5; R5 soft for future tokens |
| T217 OR helpers | **Partial** reuse; **F9** gate unchanged |
| T218 floors / `candidate_depth` / KIND bump | **Decline** F4 |
| T315 / T313 / T317 / T316 | **Not stolen** |
| T307 Blocked / T308 floors / H2 / clap 5 | **Not stolen** / **Decline** |
| last-PR `#229` Cursor | **N/A empty** — no T325 |
| DOCS TX | `8b1b418b-acbb-4398-b867-7ea297d10e41` |

### T312 fold-in (2026-08-27) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode M1 AC5 UUID token split greens T217 | **Folded** F42 / AC5 query `"t312or backend"` |
| OpenCode M2 AC2 `LEADING_QUERY_BONUS` masks F6 | **Folded** AC2 pin without query tokens |
| OpenCode M3 AC4/AC12–14 / Manual same redness | **Folded** AC4 stay-green; CLI F42 |
| OpenCode m1 F8 vs T217 R2 | **Folded** F9 note |
| OpenCode m2 `DUMP_OTHER_*` module | **Folded** F39 in `session_chrome.rs` |
| Both m1/m3 HEAD `27731be` | **Folded** snapshot `413aa33` / ahead **2** |
| OpenCode O1 F6 hygiene | **Folded** §2.2 / §5.2 |
| OpenCode O2 AC3 crumb-first | **Folded** AC3 index 0 |
| OpenCode O3 `recall.rs:571` | **Folded** §2.3 |
| Agy m2 raw_query thread | **Already** F8 |
| Agy O1/O2/O3 F7 / recall.rs / F5 | **Already** |
| last-PR `#229` Cursor | **Affirm N/A** — no T325 |
| Agy/OpenCode B | None filed |
| DOCS TX | `2e553fb4-57c6-459e-b5b7-ea774cd74021` |

| Item | Disposition |
|------|-------------|
| ~~Audit recall rank still dump-first (T285 shipped)~~ | **T312 Completed** (2026-08-27) |
| `sync query` ledger phrase→fuzzy opacity | **T313 Completed** `#233` |
| `--format` / `--dry-run` clap friction | **T314 Completed** `#232` |
| ~~preflight 0/0/0 + word count~~ | **T315 Completed** 2026-08-28 (SOOT + Budget window words) |
| `memory list` preview + forget nudge | **T316** Planned (Pending until go) |
| `graph neighbors` RECALLS spam | **T317 Completed** `#234` |
| `backup list` residual noise | **T318** Planned (Pending until go) |
| evidence/source show vault UUID | **T319** Completed `#235` |
| unified `status` opportunity | **T320** Completed `#237` `c3abe19` |
| `safety sync` write surprise | **T321 Planned** (Pending until go) |
| T311 R2 `--as-of` | **T322** Pending |
| T311 R5 conclusion in-force | **T323** Pending |
| T311 R7 PowerShell empty TERM | **T324** Pending |
| T312 F8 Prefer-OR skips recency (`#230`) | **T325** Pending |
| T311 R1 daemon `ListInForce` | **Decline** — no consumer; mint later if a DTO caller appears |
| T311 R3 sibling Approved | **Decline** — T311 F7 earliest-root by design |
| T311 R4 `approved_at` | **Partial T322** — column only if plan proves `updated_at` insufficient |
| T311 R6 PATH install | **Done** — owner elevated install 2026-08-27 |
| T263 H2 / governed populate from pins | **Decline** standing |
| `sync pull/push` / replicate / device | **Decline** — T92 / T298 honesty |
| T307 dual tower-http | **Not stolen** (Blocked) |
| T308 density floors | **Decline** standing |
| T310 R1 self-replace / F15 `--version` | **Decline** |
| `recovery_kit_event` doctor warn | **Decline** this series (doctor Q=9) |
| last-PR `#229` Cursor | **N/A empty** — superseded by `#230` |
| last-PR `#230` Cursor F8 recency | **T325** Pending |

### T311 implement (2026-08-27) — decision in-force (Completed) — residual promotion

R2 / R5 / R7 **promoted** to T322 / T323 / T324 (Pending placeholders). R1 / R3 / R4 / R6 as in the T312–T324 table above.

## T285–T300 placeholders (2026-08-22) — post-T283 live CLI quality (0.1.2)

Minted from PATH **0.1.2** non-destructive dogfood. Full F-list on `/plan-track TNN`. **Do not implement Placeholders.** last-PR Cursor **#228** empty. **T285–T306 Completed.** **T307 Blocked (F3 2026-08-26).** **T308 Completed (2026-08-26).** **T309 Completed (2026-08-26).** **T310 Completed (2026-08-27).** **T311 Completed (2026-08-27).**

### T311 implement (2026-08-27) — decision in-force (Completed)

| Item | Disposition |
|------|-------------|
| Archived T95 in-force | **Done** — CP `in_force.rs`, not retrieval; successor `state=in_force` |
| AC1–AC11 | **Done** — 12 tests PASS; `dev-check` **3545** passed, 1 skipped |
| Codex | **PASS** — 0 findings (`review.codex.md`, gpt-5.6-sol) |
| T307 / T308 floors / H2 / daemon wire | **Not stolen** |
| Residual R1 — daemon `ListInForce` | **Soft** F13 — **declined** T312–T324 mint (no daemon consumer) |
| ~~Residual R2 — `--as-of`~~ | **Promoted T322** Pending |
| Residual R3 — sibling Approved same term | **Soft** F7 earliest-root — **declined** remint |
| Residual R4 — `approved_at` column | **Soft** JSON `updated_at` — **partial T322** |
| ~~Residual R5 — conclusion in-force~~ | **Promoted T323** Pending |
| Residual R6 — PATH until elevated install | **Done** 2026-08-27 owner elevated install |
| ~~Residual R7 — PowerShell `""` drops empty TERM~~ | **Promoted T324** Pending |
| FEATURE TX | `e88743aa-e92c-407a-8093-6c6e4e6d9b53` |

### T311 fold-in (2026-08-27) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode m F3 `OutputFormat::parse` swallows unknown | **Folded** F3 `value_parser` + AC8 `--format nope` |
| OpenCode m F12 `format_authorized_empty_next` | **Folded** F12 human; JSON F4 unchanged |
| Agy m2 `decision_valid_at` `pub(crate)` | **Folded** F9; no copy |
| Agy m1 HEAD `b7ca150` | **Folded** snapshot |
| Agy O1 / O2 / O3 | **Already** F4 / F8 / F2+F10 |
| OpenCode supersede O | **Already** F5/F8; §2.3 note; write path not tightened |
| OpenCode density 0.409 | **Decline** floor change |
| OpenCode summary new projection / FTS `decision list` | **Decline** — T150 `Option<String>`; FTS non-goal |
| last-PR `#228` Cursor | **Affirm N/A** — T312–T324 minted from 2026-08-27 audit + T311 residuals, not `#228` |
| T307 / T308 floors | **Not stolen** |
| DOCS TX | `e5f9e657-83e8-4402-9fdf-1f7089c151d7` |

### T311 plan-write (2026-08-27) — decision in-force (Pending)

| Item | Disposition |
|------|-------------|
| Archived T95 `track-t95-in-force` unique commits | **Absorb** — tag `archive/track-t95-in-force` @ `7812b61`; rewrite in CP (not retrieval) |
| last-PR `#228` Cursor | **N/A empty** — no T312 |
| T307 / T308 floors / H2 / recovery kit / clap 5 / `--version` | **Not stolen** |
| DOCS TX | `67c2081c-5040-464e-9214-4022556e7f25` |

### T310 owner elevated PATH install (2026-08-27)

| Item | Disposition |
|------|-------------|
| Non-elevated `cargo install` Access denied (T306 R1) | **Affirmed** — owner used elevated shell |
| PATH `ai-brains.exe` | **Done** — **26,842,112** B; mtime **2026-08-27 05:52:13**; `0.1.3`; `graph_feature=available` |
| PATH `ai-brainsd.exe` | **Done** — **22,377,984** B; mtime **2026-08-27 05:51:37** (was T310 OR-path 22,173,184 B / 12:04:58 AM) |
| `cipher_page` | **Done** — `cipher_version=4.14.0 community` |
| T308 PATH remediator still rebuild | **Done** — doctor JSON omits `graph_density.remediation` |
| Residual R1 — `daemon update` self-replace os error 5 | **Soft** — elevated install is the live PATH replace; cargo#3486 stands |
| Residual R2 — `ai-brainsd --version` Missing | **F15** — unchanged |

### T310 implement (2026-08-27) — `update` graph-on + PATH daemon 4.14 (Completed)

| Item | Disposition |
|------|-------------|
| T306 F9 `run_update` graph-off | **Done** — `UPDATE_CLI_CARGO_ARGS` reconstructs `GRAPH_REINSTALL_SOOT` |
| T306 F8 PATH `ai-brainsd` 4.10 | **Done** — mtime **2026-08-27 12:04:58 AM**; size 22,173,184 B |
| F10.1 SOOT CLI | **Done** — PATH `ai-brains.exe` mtime **2026-08-27 12:00:49 AM** |
| PATH `ai-brains daemon update` | **Tried** — os error 5 self-replace (running CLI). Used F10 **OR** path: `cargo install --path crates/ai-brainsd --locked` + `daemon start` |
| AC3 doctor | **Done** — `graph_feature` available; `cipher_page` `cipher_version=4.14.0 community` |
| SCM | **Affirmed** Stopped (F12) |
| T307 / T308 floors | **Not stolen** |
| Residual R1 — `ai-brains daemon update` cannot replace running `ai-brains.exe` | **Soft** — F14 / cargo#3486; OR path is the live sequence |
| Residual R2 — `ai-brainsd --version` Missing | **F15** |
| FEATURE TX | `65008805-2230-485d-84d3-580659b519b8` |

### T310 fold-in (2026-08-26) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 HEAD `e577c8c` / 0/0 | **Folded** snapshot `87919dd` / ahead **1** |
| Agy m2 F10 chicken-egg | **Already** F9 / F10 |
| Agy O1 argv in `daemon.rs` | **Folded** F1 `UPDATE_CLI_CARGO_ARGS` / `UPDATE_DAEMON_CARGO_ARGS` |
| Agy O2 reconstruct unit | **Already** AC1 |
| OpenCode m1 `#227` `mergedAt` 22:22:01Z | **Folded** — list 22:02:39Z is not merge |
| OpenCode m2 `run_update` `:1100` / `run_start` `:20` | **Folded** §2 |
| OpenCode O1 scan hotspots empty | **Folded** Phase 0 re-check; F1 unchanged |
| OpenCode O2 AC4 unit | **Folded** lock `run_update_daemon_args__no_graph_feature` |
| last-PR `#227` Cursor | **Affirm N/A** — no T311 |
| T307 / T308 floors | **Not stolen** |
| DOCS TX | `20060ded-80be-4a78-b10b-a7dd69e4f817` |

### T310 full plan (2026-08-26) — `update` graph-on + PATH daemon 4.14

| Item | Disposition |
|------|-------------|
| T306 F9 `run_update` omits `--features graph` | **Absorb** F1 / F9 / AC1 |
| T306 F8 PATH `ai-brainsd` mtime 2026-08-22 | **Absorb** F4 / F10 / F11 / AC2 (or AC9) |
| T309 R3 T310 placeholder | **Absorb** — this plan |
| Chicken-egg PATH `update` before new CLI | **F10** — SOOT CLI first, then update |
| SCM `AI-Brains-Daemon` Stopped | **F12** — do not `sc start`; ImagePath is PATH exe |
| Daemon `cipher_page` / doctor 16th | **Decline** F7 / F11 |
| T307 dual tower-http | **Not stolen** (Blocked) |
| T308 floors / PATH-behind remediator | **Decline as DoD** |
| T197 silent zero | **Decline** |
| clap 5 / Cargo `default = []` | **Decline** |
| last-PR Cursor `#227` | **N/A empty** — no T311 |
| DOCS TX | `4e15b2eb-cc78-40e0-aaf2-0dd362814c7e` |

### T309 implement (2026-08-26) — rusqlite `table_exists` (Completed)

| Item | Disposition |
|------|-------------|
| T213 L4 / T305 R2 `table_exists` | **Done** — `has_core_tables` + `has_graph_tables` call `Connection::table_exists` |
| AC5 `has_core_tables__*` units | **Done** — empty + both-tables hermetic PASS |
| F6 docstring sqlite_master | **Done** — `has_graph_tables` doc rewritten |
| Key-probe `SELECT count(*) FROM sqlite_master` | **Affirmed** — `backup.rs:252` / `:488` unchanged |
| Residual R1 — test-local `fn table_exists` helpers still sqlite_master | **By design** — non-goal |
| Residual R2 — PATH binary until `cargo install` | **Soft** — source SoT |
| Residual R3 — T310 placeholder | **Not stolen** |
| T307 dual tower-http | **Not stolen** (Blocked) |
| Pin | rusqlite **0.40.2** unchanged |
| CHORE TX | `473e1069-374e-4a2d-96ba-38d64b417cd7` |

### T309 fold-in (2026-08-26) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 `has_core_tables__*` units | **Folded** AC5 (regression net, not a red) |
| Agy m2 docstring sqlite_master | **Folded** F6 / AC1 |
| Agy O1 / O2 unwrap_or + count probes | **Already** F3/F4 / AC3 |
| OpenCode m1 line `:281` → `:288` | **Folded** spec §2 |
| OpenCode m2 last-PR `#222` → `#226` | **Folded** §2 / §9; comments `[]`; **no T311** |
| OpenCode m3 no behavioral red | **Folded** §7 / plan |
| OpenCode m4 ConnectionRef docs 404 | **Folded** — link `Connection::table_exists` |
| OpenCode O1 views → false | **Folded** F4 + SQLite `sqlite3_table_column_metadata` |
| T310 / T307 | **Not stolen** |
| DOCS TX | `04a90ce4-f45e-43ca-875a-f2d8324ff2a7` |

### T308 implement (2026-08-26) — Sparse remediator None (Completed)

| Item | Disposition |
|------|-------------|
| T306 R4 / Sparse graph-on remediator rebuild | **Done** — `remediation: None`; JSON omits key; note keeps lag nuance |
| empty_lag / orphan / projection_lag rebuild | **Affirmed** stay-green |
| Graph-off Sparse reinstall SOOT | **Affirmed** |
| Floors `MIN_EDGE_NODE_RATIO=0.50` | **Unchanged** F1 |
| Residual R1 — live E/N still ~0.41 | **Not easy / by design** — floors frozen; projector more-edges is different track |
| Residual R2 — never-rebuilt Sparse has no rebuild remediator | **Not easy / by design** F2 — empty_lag/orphan still rebuild; freshness arm declined |
| ~~Residual R3 — PATH binary still shows rebuild until `cargo install`~~ | **Done** 2026-08-27 owner elevated PATH — doctor JSON omits `graph_density.remediation` |
| Residual R4 — `recovery_kit_event` doctor warn | **Not this track** (T306 R5) |
| Residual R5 — GHA `pull_request` CI did not auto-start on `#225` (close/reopen/empty commit no-op); used `workflow_dispatch` run `32988264560` (all green) | **Not easy** — ops/Actions quirk; branch unprotected so dispatch gate was authoritative |
| Floor retune / projector rewrite / T309 `table_exists` / T310 | **Not stolen** |
| T307 dual tower-http | **Not stolen** (Blocked) |
| FEATURE TX | `d62a3884-5af8-44fc-9434-3b8c31a656af` |

### T308 fold-in (2026-08-26) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Both m1 stale HEAD `037262e` / 0/0 | **Folded** snapshot `0d0fdab` / ahead **1** |
| OpenCode O1 PROTOCOL-COMPAT `:96` already optional | **Folded** — drop from stale-doc row; not AC8 |
| Agy O3 same PROTOCOL-COMPAT | **Folded** with O1 |
| Agy m2 OPERATIONS/CAPABILITIES/CHANGELOG | **Already** AC8 |
| Agy O1 doctor.rs forward / skip_serializing_if | **Already** F7 / F6 |
| Agy O2 smoke F17 | **Already** F5 / AC5 |
| OpenCode O2 loop-stop Osmani | **Already** §2.4 |
| Emitter `graph.rs:381–383` omit-on-None | **Folded** F2 / §12 — no production graph.rs edit |
| last-PR `#224` Cursor | **Affirm N/A** — no T311 |
| Agy/OpenCode B / M | None filed |
| DOCS TX | `91f8fbcd-655e-4fbd-bd64-635e9fa271bf` |

### T308 full plan (2026-08-26) — Sparse remediator None; floors frozen

| Item | Disposition |
|------|-------------|
| T306 R4 / T300 still sparse; doctor SOOT `graph rebuild` | **Absorb** F2 / AC1 / AC4 / AC9 — graph-on Sparse `remediation = None` |
| Live PATH 2026-08-26 E/N **0.410** (63040/25844); coverage ~0.80 | **Evidence** — warn is E/N not projection_lag; note already has lag nuance |
| Floor retune `MIN_EDGE_NODE_RATIO=0.50` | **Decline** F1 |
| empty_lag / orphan / projection_lag rebuild | **Affirm** F3 — T232 F4 remainder |
| Graph-off Sparse reinstall SOOT | **Affirm** F4 |
| Grow `doctor.rs` / 16th check | **Decline** F7 / F13 |
| `has_graph_tables` `table_exists` | **Decline steal → T309** |
| T84 `run_update` / PATH daemon 4.10 | **Decline steal → T310** |
| T307 dual tower-http | **Not stolen** — Blocked |
| Event↔graph freshness | **Decline** — T213 F31 |
| last-PR Cursor `#224` | **N/A empty** — no T311 |
| clap 5 | **Still declined** |
| DOCS TX | `96f0ce16-3a64-43cc-92ac-b9a4d89c46ae` |

### T307 F3 halt closeout (2026-08-26) — go re-verify; Blocked

| Item | Disposition |
|------|-------------|
| Phase 0 `cargo info reqwest` latest | **0.13.4** (version only — F22) |
| Declared pin (docs.rs `Cargo.toml.orig` + master) | `tower-http = { version = "0.6.8", … follow-redirect }` — still **0.6.x** |
| Lock dual | **0.6.11** (reqwest) + **0.7.0** (api-server) — unchanged |
| `cargo tree -i tower-http@0.6.11 --locked` | Still lists reqwest → models/desktop (AC2 F3 path) |
| reqwest#3062 | Still **open** (`merged: false`; created 2026-06-29; last update **2026-07-13**) |
| Product bump / `[patch]` / git-dep | **Not done** — F3 / F4 / F11 |
| Conductor | **Blocked** (not Completed) — F13 / AC1 |
| CHANGELOG / crates / lock | **No product diff** |
| T304 R2 dual | **Still open** — parked until crates.io reqwest allows tower-http **0.7** |
| Residual R1 — dual remains until crates.io reqwest | **Not easy** — upstream; watch `#3062` only |
| Residual R2 — `#3062` open; decompression hang class | **Not easy** — do not git-dep (`unknown-git = "deny"`) |
| Residual R3 — tower-http **0.7.1** unpublished (#712/#722 git-only) | **Not easy** — F12; accept 0.7.0 when unblocked |
| Residual R4 — `deny.toml` `multiple-versions = "warn"` (thiserror/http/hyper too) | **Decline flip this track** — F21 |
| Residual R5 — dual hyper 0.14 / http 0.2 / desktop 0.1.2 | **Decline** — not tower-http dual |
| T308 / T309 / T310 | **Not stolen** |
| last-PR Cursor `#223` | **N/A empty** — no T311 |
| DOCS TX | `a4f3ba1d-d478-4768-a2b5-1eb6bebf254f` |

### T307 fold-in (2026-08-26) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Both m1 stale HEAD `34379bf` / 0/0 | **Folded** snapshot `a084610` / ahead **1** |
| Agy m1 `#223` time 11:38:11Z | **Decline** — `createdAt` / `gh pr list`; `mergedAt` **12:34:00Z** |
| OpenCode m2 agy timestamp misread | **Folded** — spec was already correct |
| OpenCode m3 `cargo info` ≠ 0.6.8 pin | **Folded** F22 / AC1 / AC9 |
| Agy m2 F3 halt via `cargo info` | **Partial** — halt Already F3; pin SoT is Cargo.toml |
| Both O1 AC2 / O2 json-only F12 | **Already** |
| last-PR `#223` Cursor | **Affirm N/A** — no T311 |
| Agy/OpenCode B / M | None filed |

### T307 full plan (2026-08-26) — dual 0.6.11; reqwest still 0.6.8

| Item | Disposition |
|------|-------------|
| T304 R2 dual tower-http 0.6.11 via reqwest 0.13.4 | **Absorb** F1–F3 / AC1–AC2 — unify **up** only |
| crates.io reqwest **0.13.4** / master still `tower-http 0.6.8` | **F3 Stop-Before** on go if unchanged; conductor **Blocked** not Completed |
| reqwest#3062 open (2026-06-29; last 2026-07-13); tower-http #712/#722 git-only | **Decline git-dep** F11; `unknown-git = "deny"` |
| `[patch.crates-io]` / fork / `tower-reqwest` | **Decline** F4 |
| T304 R4 csrf | **Decline** F5 |
| T308 / T309 / T310 | **Not stolen** |
| Dual hyper 0.14 / desktop 0.1.2 | **Decline** — not this dual |
| last-PR `#223` Cursor | **N/A empty** — no T311 |
| clap 5 / floor retune | **Still declined** |

### T306 implement closeout (2026-08-26)

| Item | Disposition |
|------|-------------|
| PATH F1 `cargo install --path crates/ai-brains-cli --locked --features graph` | **Done** AC1 / F1 (elevated retry after Access denied) |
| PATH `doctor --json` `cipher_page` **`cipher_version=4.14.0 community`** | **Done** AC2 / F2 |
| PATH `graph_feature=available`; vault_open read-only; no key leak | **Done** AC3–AC5 |
| `git diff -- crates/ Cargo.toml Cargo.lock` empty | **Done** AC6 / F3 |
| T305 R3 PATH pre-0.40.2 / 4.10 | **Done** — absorbed; PATH now 4.14 |
| First F1 Access denied (hung PATH `ai-brains preflight --summary` PID) | **Residual** R1 — ops; elevated retry cleared; do not daemon-stop |
| ~~PATH `ai-brainsd` still 4.10-era (mtime 2026-08-22); mixed CLI/daemon~~ | **Done T310** — PATH mtime **2026-08-27 12:04:58 AM** |
| ~~T84 `run_update` omits `--features graph`~~ | **Done T310** — CLI argv reconstructs `GRAPH_REINSTALL_SOOT` |
| `graph_density` sparse E/N≈0.409; remediator rebuild | **Done** R4 — **T308** graph-on Sparse remediator `None` |
| `recovery_kit_event` doctor warn | **Residual** R5 — not this track |
| INSTALL.md header still 0.1.2 | **Residual** R6 — docs drift; not DoD |
| Harness reinstall after cargo install | **Residual** R7 — F24 soft |
| T307 / T308 / T309 / T310 | **Not stolen** |
| mint row “T306 Planned” | **Superseded** — Completed |

### T306 fold-in (2026-08-26) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 `--json` over `--summary` | **Already** F2 / AC2 |
| Agy m2 `--features graph` | **Already** F1 / F7 / AC3 |
| Agy m3 Perl pre-check | **Already** F12 / AC9 |
| Agy O1 filtered JSON | **Already** F6 / AC5 / §7 |
| OpenCode m1 stale HEAD `cb5aa49` | **Folded** snapshot `30894bf` / ahead 2 |
| OpenCode O1 F25 mtime empty-proof | **Folded** F25 / AC1 supporting; AC2 SoT |
| OpenCode O2 mixed CLI/daemon | **Already** F8 / F22 |
| OpenCode `#222` `mergedAt` 03:51:25Z | **Folded** §2.1 |
| last-PR `#222` Cursor | **Affirm N/A** — no T311 |
| Agy/OpenCode B / M | None filed |

### T306 full plan (2026-08-26) — PATH proven 4.10; mint T310

| Item | Disposition |
|------|-------------|
| T305 R3 PATH `cipher_version=4.10.0 community` | **Done** T306 — PATH now `cipher_version=4.14.0 community` |
| T84 `run_update` omits `--features graph`; PATH `ai-brainsd` mtime 2026-08-22 | **T310** placeholder (T306 F3/F4 no src / no daemon stop) |
| INSTALL.md header still 0.1.2 | **Decline** — docs drift; not R3 |
| last-PR `#222` Cursor | **Affirm N/A empty** — T310 is live src, not Cursor |
| T307 / T308 / T309 | **Not stolen** |

### T306–T309 mint (2026-08-26) — leftover placeholders (owner-requested)

| Item | Disposition |
|------|-------------|
| T305 R3 PATH pre-0.40.2 | **Done** T306 — PATH `4.14.0 community` |
| T304 R2 dual tower-http 0.6.11 via reqwest 0.13.4 | **T307** **Blocked** (F3 2026-08-26) — reqwest still `tower-http 0.6.8` |
| T300 still sparse E/N 0.409; doctor SOOT rebuild | **T308** ✅ Completed — Sparse remediator None; floors frozen |
| T213 L4 / T305 R2 `table_exists` | **T309** Planned |
| T305 R1 / T304 R1 Dependabot close hygiene | **Decline** — standing; not a track |
| T305 R4 / T304 R3 lock extra variance | **Decline** — do not hand-edit |
| clap 5 / T305 R5 | **Still declined** |
| T304 R4 csrf | **Still declined** |
| T278 floor retune | **Still declined** — T308 is remediator copy, not 0.50 |
| last-PR `#222` Cursor | **N/A empty** — no Cursor leftover; **T310** minted from T306 live baseline (daemon + T84) |
| T302 R2 dual thiserror 1.x | **Decline** — Tauri/json-patch; not these leftovers |

### T305 implement closeout (2026-08-25)

| Item | Disposition |
|------|-------------|
| Workspace rusqlite exact `0.39.0` → `0.40.2` (same 4 features) | **Done** AC1 / F1 |
| Lock rusqlite **0.40.2**; libsqlite3-sys **0.38.2**; hashlink **0.12.1** via `--precise` | **Done** AC1 / F12 / F13 |
| Observed `PRAGMA cipher_version` **`4.14.0 community`** (2026-08-25 EDT) | **Done** AC2 / F2 |
| Encrypt/open/wrong-key + sqlcipher_export rotate + backup KATs | **Done** AC3 / AC4 / F9 |
| `cipher_version` query errors propagated (no `unwrap_or_default`) | **Done** Codex P2-02 |
| Full gate (fmt/clippy/nextest 3529 pass / 1 skip / deny / audit + ledgerful full) | **Done** AC5 |
| CHANGELOG + COMPATIBILITY F8 | **Done** AC7 |
| Live new-binary doctor vault_open + cipher_page ok; no key leak | **Done** AC8 |
| Do not merge Dependabot remote `#61` | **Done** F8 |
| Close `#61` as superseded after squash; do not delete remote | **Residual** R1 — standing Dependabot hygiene |
| T213 L4 `Connection::table_exists` not adopted (F5 optional) | **Residual** R2 — not easy product churn; local helpers unrelated |
| PATH-installed `ai-brains` remains pre-0.40.2 until operator install | **Done** T306 — PATH `cipher_version=4.14.0 community` |
| `#61` windows-sys/socket2 extras absent on live HEAD | **Residual** R4 — F13 variance; do not hand-edit |
| clap 5 still declined | **Residual** R5 — standing decline |
| mint row “T305 Planned” | **Superseded** — Completed |

### T304 implement closeout (2026-08-25)

| Item | Disposition |
|------|-------------|
| Workspace tower-http `0.6.6` → `0.7` (limit/cors/trace) | **Done** AC1 / F1 |
| Lock api-server → tower-http **0.7.0**; reqwest keeps **0.6.11** | **Done** AC1 — dual required |
| Bare `cargo update -p tower-http --precise 0.7.0` fails (reqwest `^0.6.8`) | **Residual** R2 — not easy; needs reqwest that accepts 0.7 |
| T161 CORS deny + layers `:66`/`:68` unchanged; no CorsLayer/csrf/fs | **Done** AC3 / AC4 / F2 / F3 |
| rusqlite / clap / thiserror / tokio 1.53.1 unchanged | **Done** F4 |
| `git diff -- crates/` empty | **Done** constructors unchanged |
| Targeted api-server nextest 39 (CORS + body-limit) | **Done** AC2 |
| CHANGELOG Unreleased Changed row | **Done** AC6 |
| Full gate (fmt/clippy/nextest/deny/audit + ledgerful full) | **Done** AC5 (after Phase 5) |
| Do not merge Dependabot remote `#58` | **Done** F6 |
| Close `#58` as superseded after squash; do not delete remote | **Residual** R1 — standing Dependabot hygiene |
| `#58` F9 windows-sys/socket2/windows-core extras absent on live HEAD | **Residual** R3 — expected variance after T303; do not hand-edit |
| Opt-in `csrf` feature in 0.7 not enabled | **Residual** R4 — product non-goal |
| mint row “T304 Planned” | **Superseded** — Completed |

### T303 implement closeout (2026-08-25)

| Item | Disposition |
|------|-------------|
| Workspace tokio floor `1.52` → `1.53` full | **Done** AC1 / F1 |
| Lock tokio 1.52.3 → 1.53.1 via `--precise` | **Done** AC1 / F8 |
| F9 windows-sys edge re-resolutions toward 0.61.2 | **Done** expected extras |
| rusqlite / tower-http / clap / thiserror unchanged | **Done** AC4 |
| `git diff -- crates/` empty; no live `daemon stop` | **Done** F3 / F7 |
| Targeted ai-brainsd 87 + CLI daemon_status 9 | **Done** AC3 |
| CHANGELOG Unreleased Changed row | **Done** AC5 |
| Full gate (fmt/clippy/nextest/deny/audit + ledgerful full) | **Done** AC2 (after Phase 5) |
| Do not merge Dependabot remote `#59` | **Done** F6 |
| Close `#59` as superseded after squash; do not delete remote | **Residual** R1 — standing Dependabot hygiene |
| Multi-version windows-sys 0.45/0.52/0.59/0.60.2/0.61.2 remains | **Residual** R2 — not easy; ecosystem unify |
| Live lock edges ≠ exact Dependabot `#59` flips (more 0.61.2) | **Residual** R3 — F9 variance; do not hand-edit |
| `#8095` mpsc drop-waker — monitor only if future hang | **Residual** R4 — AC3 green; no src change |
| mint row “T303 Planned” | **Superseded** — Completed |

### T302 implement closeout (2026-08-25)

| Item | Disposition |
|------|-------------|
| thiserror/thiserror-impl 2.0.18→2.0.20 via `thiserror@2.0.18 --precise` | **Done** AC1 / F8 |
| chrono 0.4.44→0.4.45 via `--precise` | **Done** AC2 / F8 |
| Workspace carets still 2.0 / 0.4; `git diff -- crates/` empty | **Done** AC3 / AC5 |
| thiserror 1.0.69 unchanged; rusqlite/tokio/tower-http/clap unchanged | **Done** AC7 |
| F9: thiserror-impl→syn 3.0.3; iana-time-zone windows-core 0.62.2→0.61.2 | **Done** expected extras |
| CHANGELOG Unreleased Changed row | **Done** AC6 |
| Full gate (fmt/clippy/nextest/deny/audit + ledgerful full) | **Done** AC4 |
| Do not merge Dependabot remotes `#60`/`#62` | **Done** F5 |
| Close `#60`/`#62` as superseded after squash; do not delete remotes | **Residual** R1 — standing Dependabot hygiene |
| Dual thiserror 1.0.69 + 2.x (bare `-p thiserror` ambiguous) | **Residual** R2 — Tauri/json-patch stack; out of scope |
| Dual windows-core 0.61.2 + 0.62.2 after F9 edge | **Residual** R3 — expected; unify needs Tauri/windows work |
| mint row “T302 Planned” | **Superseded** — Completed |

### T301 implement closeout (2026-08-25)

| Item | Disposition |
|------|-------------|
| checkout v7.0.1 all 4 sites (3× ci + release) | **Done** AC1 |
| upload-artifact v7.0.1 / download-artifact v8.0.1 / attest v4.2.2 / gh-release v3.0.2 peeled | **Done** AC2 / F10 / F11 |
| No floating `@vN`; no `pull_request_target` / `workflow_run` | **Done** AC3 / AC4 |
| CHANGELOG + release.yml header SHA table 2026-08-25 | **Done** AC6 / F9 |
| Zero crate / Cargo.lock edits | **Done** AC7 |
| Do not merge Dependabot remotes `#68–#72` | **Done** F3 |
| Release.yml tag-only — no PR job exercises attest/publish | **Residual** R1 — YAML + input-compat review only; soft attest F5 |
| Node 20 runner deprecation timeline | **Residual** R2 — out of scope; Node 24 is action `runs.using` |
| dtolnay/rust-toolchain + Swatinem/rust-cache unpinned-from-this-batch | **Residual** R3 — F2 / non-goal |
| mint row “T301 Planned” | **Superseded** — Completed |

### T301 fold-in (2026-08-25) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode M1 peel annotated-tag SHA (action-gh-release) | **Folded** F10 / Phase 0 — pin commit `3d0d9888…` not tag `fe965f7a…` |
| OpenCode m1 last-PR `#217` vs `#216` | **Folded** F8 |
| OpenCode O1 / Agy m1 attest v4.2.2 | **Folded** F11 latest v4.x patch at execute |
| OpenCode O2 no SHA-pin script | **Folded** §7 `rg` checklist |
| OpenCode O3 Node-24 date | **Folded** drop unverified date |
| Agy m2 “tag-object SHA also works” | **Decline**; F10 commit SHA only |
| Agy m3 release.yml header table | **Already** F9 / AC6 |
| Agy O1 three ci.yml checkout jobs | **Folded** AC1 |
| Agy O2 floating-tag `rg` | **Already** AC3 |
| last-PR `#216`/`#217` Cursor | **Affirm N/A** — no T306 |
| No B | Nothing to decline of B |

### T302 fold-in (2026-08-25) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m2+m3 precise `thiserror@2.0.18 --precise 2.0.20` | **Folded** F8 — `cargo pkgid thiserror` is ambiguous with 1.0.69 |
| OpenCode m1 stale changelog text | **Folded** spec §2 — thiserror syn 3 + clippy #454; chrono 0.4.45 tz-only |
| OpenCode m2 last-PR `#216` → `#218` | **Folded** §2 / §9 |
| OpenCode O1 `#62` windows-core 0.62.2→0.61.2 | **Folded** F9 / AC7 — both versions stay |
| OpenCode O2 syn 3 already in lock | **Folded** F9; PR body at execute |
| Agy m1 lockfile-only Cargo.toml | **Already** F2 / AC3 / AC5 |
| Agy O1 clippy `--all-targets` | **Already** AC4 |
| Agy chrono 0.4.45 DateTime Copy / `days_since` | **Decline** — not v0.4.45 (tz #1787/#1789) |
| last-PR `#218` / `#217` Cursor | **Affirm N/A** — no T306 |
| No B / M | Nothing to decline of B/M |

### T303 fold-in (2026-08-25) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode m2 caret `"1.52"` already allows 1.53.1; `#59` lock-only | **Folded** F1 restated — toml bump is **floor**, not caret-unblock |
| Agy m1 “must widen caret to resolve 1.53” | **Partial** — keep `1.53` floor; **decline** false Cargo claim |
| Agy m3 `--precise 1.53.1` | **Folded** F8 |
| OpenCode O-1 `#59` windows-sys edge flips | **Folded** F9 / AC4 |
| OpenCode m1 last-PR `#216` → `#219` | **Folded** §2 / §9 |
| Agy m2 target 1.53.1 not 1.53.0 | **Already** F1 |
| Agy O1 targeted daemon/CLI nextest | **Already** AC3 |
| Agy `#8252` timer race as 1.53.1 product fix | **Partial** — changelog **unstable**; F2 |
| last-PR `#219` Cursor | **Affirm N/A** — no T306 |
| No B / M | Nothing to decline of B/M |

### T304 fold-in (2026-08-25) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 workspace `0.7` required (`^0.6.6` cannot reach 0.7) | **Folded** F1 affirmed — unlike T303, this **is** a caret-unblock |
| Agy m3 `--precise 0.7.0` | **Folded** F8 |
| OpenCode O1 `#58` windows-sys/socket2/windows-core extras | **Folded** F9 — live graph after T303 may differ; do not hand-edit |
| OpenCode m1 last-PR `#216` → `#220` | **Folded** §2 / §9 |
| Agy m2 keep limit/cors/trace; no csrf/fs | **Already** F2 |
| Agy O1 targeted `-p ai-brains-api-server` | **Already** AC2 |
| OpenCode “limit/cors/trace no breaking API” | **Partial** — constructors unchanged; gRPC classify unused |
| last-PR `#220` Cursor | **Affirm N/A** — no T306 |
| No B / M | Nothing to decline of B/M |

### T305 fold-in (2026-08-26) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m3 `--precise 0.40.2` | **Folded** F12 |
| OpenCode extras libsqlite3-sys 0.38.2 + hashlink 0.12.x | **Folded** F13 — live graph after T303/T304 may differ from `#61` |
| OpenCode m1 last-PR `#216` → `#221` | **Folded** §2 / §9 |
| OpenCode local `fn table_exists` ≠ rusqlite API | **Folded** F5 |
| Agy m1 workspace exact `0.40.2` | **Already** F1 / AC1 |
| Agy m2 pre-write COMPATIBILITY `4.14.0 community` | **Partial** — F2 records **observed**; SQLCipher test shape is expectation only |
| Agy O1/O2 targeted store nextest + live doctor | **Already** AC3/AC4/AC8 |
| Agy “4.14 opens 4.10 transparently” | **Partial** — Zetetic same-major format; **F9 is proof** |
| last-PR `#221` Cursor | **Affirm N/A** — no T306 |
| No B / M | Nothing to decline of B/M |

### T301–T305 mint (2026-08-25) — Dependabot tracks (owner-requested)

| Item | Disposition |
|------|-------------|
| GHA `#68–#72` SHA-pin majors | **T301** Completed |
| thiserror `#60` + chrono `#62` patches | **T302** Completed |
| tokio `#59` 1.53.1 | **T303** Completed |
| tower-http `#58` 0.7 | **T304** Completed |
| rusqlite `#61` 0.40.2 (prior series decline) | **T305** Planned — **reopened** by owner |
| last-PR `#221` Cursor | **N/A empty** — no T306 |
| clap 5 | **Still declined** (not in this Dependabot batch) |
| Merge Dependabot remotes as-is | **Declined** — recreate on `track/TNN-*` |

### T300 implement closeout (2026-08-25)

| Item | Disposition |
|------|-------------|
| Rebuild remediator UX: `--dry-run` + density stdout + daemon Safety fail-closed | **Done** F4–F8 / AC1–AC5 / AC14 dry-run |
| Floors frozen; never force `live`; T232 remediator exact; no `--confirm` | **Done** F2 / F3 / F8 / AC8 / AC16 |
| Shared `graph_health_report`; `rebuild.rs` / `graph_density.rs` / `doctor.rs` unchanged | **Done** F27 / AC13 |
| Live mutate on operator vault | **Done** 2026-08-25 owner: daemon stop → `graph rebuild` (~91s, 57919 events) → E/N **0.149→0.407** still `sparse` honest; doctor `graph_density` agrees; daemon restarted PID 17404 |
| PATH until `cargo install --features graph` | **Done** 2026-08-25 owner: `cargo install --path crates/ai-brains-cli --locked --features graph` → PATH **0.1.3** graph-on |
| `read_all_events` full Vec RAM | **Residual** F9 / F25 — engine freeze |
| Mid-rebuild daemon start / crash TOCTOU | **Residual** F25 — re-run rebuild; probe≠atomic DELETE |
| JSON dry-run omits `dry_run` key | **Residual** F10 — by design (human-only extras) |
| Hermetic mutate early-return when host daemon Running | **Residual** C3 — unit `daemon_up=false` is mutate SoT; not easy without daemon stop |
| clap 4.6 workspace / rusqlite 0.40 Dependabot | **T305** Planned (rusqlite); clap 5 still declined |
| leftover `--write` / T240 F2 / T263 H2 / clap 5 / floor retune | **Residual** F24 — declined |
| No T301 (last-PR #215 empty) | **Superseded** — T301–T305 Dependabot series; last-PR `#216` empty |

### T300 fold-in (2026-08-25) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 extract `graph_health_report` | **Already** F27 |
| Agy m2 async dispatch `.await` | **Already** F32 |
| Agy m3 inject matrix three daemon/dry-run cases | **Folded** AC10 (case 3 daemon-down mutate) |
| Agy O1 COUNT fail-open omit `N` | **Already** F6 |
| Agy O2 clap `human\|json` reject `auto` | **Already** F5 / AC9 |
| OpenCode m1 F10 inline debate | **Folded** F10 — decision only |
| OpenCode O1 mid-rebuild TOCTOU residual | **Folded** F25 / §11 |
| OpenCode O2 crate path `graph_density.rs` | **Folded** §2.3 / F2 / F16 / §12 |
| OpenCode O3 `graph.rs` 1214 vs 1130 | **Folded** Isolation — 1214 physical |
| last-PR #215 Cursor | **Affirm N/A** — no T301 |
| No B/M | Nothing to decline of B/M |

### T300 planning absorption (2026-08-25) — owner-confirm rebuild; daemon fail-closed; floors frozen

| Item | Disposition |
|------|-------------|
| Audit graph sparse E/N ~0.14; doctor `graph_density` warn | **Absorb** F1–F8 / AC1–AC5 / AC14 |
| Placeholder Manual `graph update` + owner-confirm `graph rebuild` + doctor agree | **Absorb** AC14 / F1 / F3 |
| Placeholder floors frozen; never force `live` | **Absorb** F2 / F3 |
| Placeholder skip = T262 hermetic + written skip | **Absorb** F1 / F11 / AC6 |
| T278 F8 no live rebuild as DoD | **Lift to owner-confirm** F1 (T295 class) |
| T278 F7 / T213 floors 0.50 | **Affirm freeze** F2 / AC8 |
| T232 remediator exact `ai-brains graph rebuild` | **Affirm** F8 — **no `--confirm`** |
| T262 pin = node without rebuild | **Affirm** F11 / AC6 |
| T188 daemon Safety for mutate | **Absorb pattern** F7 / AC3 |
| T295 live `--no-prune` analog | **Absorb class** F1 |
| T293 neighbors ranking | **Decline steal** — Completed `#209` |
| T299 closeout T300 steal | **Absorb** (this track) |
| Silent rebuild stdout / no `--dry-run` / daemon Running race | **Absorb** F4 / F6 / F7 |
| Floor retune / Cargo default-on / projector more-edges / streaming `read_all_events` | **Decline** F9 / F24 |
| leftover `--write` / T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F14 / F24 |
| last-PR Cursor **#215** | **N/A empty** — **no T301** F18 |
| Identity leftover `7d97a456` vs `fcb8a40f` | **Not this track** |
| `recovery_kit_event` doctor warn | **Not this track** |

### T299 implement closeout (2026-08-25)

| Item | Disposition |
|------|-------------|
| Empty forgotten `Pinned: N` + `next: memory list` (shared backend) | **Done** F1–F6 / AC1–AC6 / AC14 |
| JSON nine keys frozen; no `next_step` | **Done** F10 / AC5 (exact key-set assert) |
| `forget.rs` production unchanged; helper in `memory.rs` | **Done** F6 / AC15 |
| Docs CAPABILITIES/OPS/WORKFLOWS/CHANGELOG/CLI-EXIT-CODES/after_help | **Done** F19 / AC12 |
| PATH until `cargo install` | **Residual** F17 — not easy without owner install |
| Live Forgotten: 0 | **Residual** F13/F25 — honest AC14 empty SoT |
| JSON `next_step` / `--summary` on forget / tag histogram / `--offset` | **Residual** F10/F9/F24 — declined |
| `count_pinned_memories` vs session-join residual | **Residual** F25 — this track uses inventory COUNT |
| clap 4.6 workspace / rusqlite 0.40 Dependabot | **Residual** F14 — not stolen |
| T300 graph sparse | **Not stolen** F24 |

### T299 fold-in (2026-08-25) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 tag param on `emit_list_human` | **Already** F26 / F31 / §5.2 |
| Agy m2 rstest four remediator cases | **Already** AC10 |
| Agy m3 hermetic stdout parity | **Folded** F6 / AC2 `assert_eq!` |
| Agy O1 fail-open `.ok()` | **Already** F2 |
| Agy O2 both after_help | **Already** F22 / AC12 |
| OpenCode m1 JSON absence is stay-green not red | **Folded** §7 / AC16 |
| OpenCode m2 AC14 names `cargo run` | **Folded** AC14 |
| OpenCode O1 CAPABILITIES Empty row `:275` | **Folded** F19 |
| OpenCode O2 CLI-EXIT-CODES required add | **Folded** F19 |
| OpenCode O3 two COUNTs | **Already** F32 |
| last-PR #214 Cursor | **Affirm N/A** — no T301 |
| No B/M | Nothing to decline of B/M |

### T299 planning absorption (2026-08-25) — empty forgotten Pinned N + next memory list; JSON frozen

| Item | Disposition |
|------|-------------|
| Audit `forget --list-forgotten` U=6 empty | **Absorb** F1–F6 / AC1–AC6 / AC14 |
| Placeholder Manual `forget --list-forgotten --limit 5` + `memory list --summary` | **Absorb** AC14 / F13 |
| Placeholder keep `No forgotten memories.` + `Pinned: N` + `next: ai-brains memory list` | **Absorb** F1 / F3 / F4 |
| Placeholder JSON additive `next_step` if keys allow | **Rewrite** F20 / F10 — human-only; nine keys frozen |
| T216 F14 empty const / exit 0 | **Affirm** F4 / F23 |
| T216 F36 skip next on empty | **Partial lift** F27 — stdout `next:` on forgotten-empty; stderr restore still nonempty-only |
| T216 F10 / T287 F10 JSON keys | **Affirm freeze** F10 / AC5 |
| T216 F28 no `--summary` on forget / F6 limit 50 | **Affirm** F9 / F11 |
| T216 closeout tag histogram / `--offset` / auto-forget / CE wipe | **Decline** F24 / F13 |
| T287 F7 forgotten recency; empty next parked here | **Affirm** F7; **absorb** empty next |
| T287/T290/T291/T292/T293/T294/T298 “Decline → T299” | **Absorb** (this track) |
| T298 closeout T299 steal | **Absorb** (this track); **T300** still not stolen |
| T274–T284 declined forget empty as E=8 | **Reopened** as this track |
| T300 graph sparse live rebuild | **Decline** F24 |
| leftover `--write` / T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F14 / F24 |
| last-PR Cursor **#214** | **N/A empty** — **no T301** F18 |
| Identity leftover `7d97a456` vs `fcb8a40f` | **Not this track** |

### T298 implement closeout (2026-08-25)

| Item | Disposition |
|------|-------------|
| Empty `device status` four-line: T198 + this-machine + short honesty + `next:` last | **Done** F1/F4/F5 / AC1 / AC14 |
| Enrolled `device status` hyphen fingerprint + honesty + `next:` last | **Done** F7 / AC2 |
| Human `replicate status` 19-char `this machine:` after `enrolled_count` | **Done** F8 / AC6 / AC9 |
| JSON six keys frozen; no `this_machine`; `--quiet` unchanged | **Done** F9/F10 / AC7/AC8 |
| `emit_device_roster` returns `Vec`; list/fingerprint frozen | **Done** F26/F6 / AC3/AC4 |
| Fail-open malformed fp → `enrolled; fingerprint unavailable` | **Done** F2 / AC11 |
| No live bootstrap; no `hostname` / `serial_test` crates | **Done** F13/F14/F27 |
| Docs CAPABILITIES/OPERATIONS/INSTALL/PROTOCOL-COMPAT/CHANGELOG | **Done** F19 / AC12 (Codex P2 INSTALL dual form fixed) |
| PATH until `cargo install` | **Residual** F17 — not easy without owner install |
| Live vault stays 0 enrolled | **Residual** F13/F25 — honest AC14 empty SoT |
| `device list --format json` / combined dashboard / doctor 16th | **Residual** F16/F25 — declined |
| Singular error-copy unify (`load_local_*`) | **Residual** T251 F12 — not this hole |
| clap 4.6 workspace pin / rusqlite 0.40 Dependabot | **Residual** F14 — not stolen |
| T299 forget-list / T300 graph sparse | **Not stolen** F24 |

### T297 implement closeout (2026-08-24)

| Item | Disposition |
|------|-------------|
| Stopped+Open prints `backend TCP Open ≠ daemon`; `next:` still last | **Done** F1–F5 / AC1–AC6 / AC8 |
| Running+Open omits contrast (manual AC10) | **Done** F3 |
| Status `after_help` TCP connect + unknown `--format` clap exit 2 | **Done** F20 / AC7 |
| CAPABILITIES / OPERATIONS / CHANGELOG T281 vs T297 | **Done** F19 / AC11 |
| Live force-restore drills soft-skip when daemon Running | **Done** (recovery_drills + smoke) — T188 Safety vs T297 F11 |
| PATH until `cargo install` | **Residual** F13 — not easy without owner install |
| Live daemon Running hides Stopped+Open on this machine | **Residual** F11 — units+AC8 SoT; do not stop daemon |
| Doctor Safety 3×1000 ms vs status Status 1×300 ms | **Residual** F27 — probe-policy, not this hole |
| T249 F12 `--format json` / uptime / `sc query` | **Residual** F8 / F17 — declined JSON surface |
| Force-restore hermetics vacuous when live daemon Running | **Residual** — CI Stopped proves; local soft-skip until owner stop or IPC isolation |
| T298 Completed; T299–T300 | **Not stolen** (T298 later Completed) |

| Item | Track |
|------|-------|
| recall/search/semantic/sync-vault still chrome Q=4 | **T285 Completed** |
| preflight Index `## Objective`; summary decisions 0 vs 3647 pins | **T286 Completed** |
| `memory list` just-now ingest | **T287 Completed** |
| briefing granted-empty vs pins (no H2) | **T288 Completed** |
| leftover dest-missing; context skip vault upsert | **T294 Completed** |
| 0 usable encrypted backup | **T295 Completed** |

### T295 implement closeout (2026-08-24)

| Item | Disposition |
|------|-------------|
| Live `--no-prune` create + list Readable + verify ≥1 OK + doctor `backup_recent` ok | **Done** F2a / AC8 (N 22→23; `vault-2026-08-24T10-01-54.db.bak`) |
| Create `after_help` + AC5 F37 | **Done** |
| CAPABILITIES / OPERATIONS / CHANGELOG / RECOVERY-DRILLS | **Done** AC7 |
| T277 engine / doctor remediator / keep-10 / residuals | **Affirm freeze** — untouched |
| Doctor remediator still omits `--no-prune` | **Residual** F7 (docs+after_help carry it) |
| PATH until `cargo install` (T285–T294 not on PATH) | **Residual** F16 |
| Live 22 residual `.bak` still KeyMismatch/plain/Incomplete | **Expected** F5; verify exit 1 |
| T296–T300 | **Not stolen** |

### T295 planning absorption (2026-08-24) — live `--no-prune` current-key file; T277 engine frozen

| Item | Disposition |
|------|-------------|
| Audit 0 OK/22 FAIL; doctor `backup_recent` no usable encrypted backup | **Absorb** F2–F8 / AC5–AC8 |
| Placeholder Manual `backup create --no-prune` + list + verify + doctor | **Absorb** AC8 |
| T277 closeout live 22 residual until owner create | **Absorb** F2 / F3 — hermetic skip is **not** Complete here |
| T225 residual “operator still runs live `backup create`” | **Absorb** F2 / AC8 |
| CAPABILITIES green path omits `--no-prune`; example `--output-dir` vs doctor sibling dir | **Absorb** F6 / F8 / AC5 / AC7 |
| T277 F2 fail-closed engine / mixed hermetic | **Affirm freeze** F1 / AC1–AC4 |
| T277 F8 doctor remediator `ai-brains backup create` | **Affirm** F7 — do not grow `doctor.rs` |
| T277 F20 prune dry-run `remaining_count` | **Decline** F19 |
| T244 F17/F18 / T187 `cipher_integrity_check` / restore-on-create | **Decline** F19 / F9 |
| Default keep-10 change | **Decline** F4 |
| Rekey / transcode T244 `.bak` | **Decline** F5 |
| T294 leftover `--write` / T296–T300 | **Decline →** those tracks |
| T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F24 |
| last-PR Cursor **#210** | **N/A empty** — **no T301** F25 |
| Identity leftover `7d97a456` vs `fcb8a40f` | **Not this track** — T258 / leftover data |

### T295 fold-in (2026-08-24) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 AC5 distinct substrings | **Folded** F37 / AC5 |
| Agy m2 verify exit 1 mixed + no nudge | **Already** F14 / AC3 / AC8 |
| Agy O1 OPERATIONS `--output-dir` vs list/doctor | **Already** F8; **tightened** AC7 |
| Agy O2 `--dry-run --no-prune` in after_help | **Already** §5.1; **folded** AC5 example (F37) |
| OpenCode m1 plan HEAD `56d905a` vs `cd9701a` | **Snapshot** — preflight refreshed |
| OpenCode m2 F12 `cli_help_ia.rs` vs `help_ia.rs` | **Folded** F12 — both `src/help_ia.rs` and `tests/cli_help_ia.rs` |
| OpenCode m3 T277 F20 remaining_count consistent | **Already** F19 |
| OpenCode O1 AC5 `--dry-run --no-prune` example | **Folded** F37 / AC5 |
| OpenCode O2 combined streams + no-vault help | **Already** §5.5; **tightened** F35 / F37 |
| OpenCode O3 AC8 exact 22+1 | **Folded** F38 — Phase 0 N, after N+1 |
| last-PR #210 Cursor | **Affirm N/A** — no T301 |
| No B/M | Nothing to decline of B/M |

### T296 implement closeout (2026-08-24)

| Item | Disposition |
|------|-------------|
| Human Ready+267014 → `Router: Ready` + `last run: terminated` | **Done** F1/F2 / AC1 / AC9 |
| Running+267009 status-only; blank terminated/running; hex + whitespace | **Done** F3/F33/F34 / AC2/AC3 |
| JSON `last_result` / hint frozen; `explain_last_task_result` untouched | **Affirm** F5/F6 / AC5/AC7 |
| after_help 267014 success; CLI-EXIT-CODES both SCHED_S | **Done** F7/F19 / AC6/AC10 |
| CAPABILITIES / OPERATIONS / CHANGELOG | **Done** AC10 |
| Full gate `dev-check.ps1` | **Done** AC13 |
| T297–T300 / leftover `--write` / 750 raise / doctor 16th | **Not stolen** |
| PATH until `cargo install` | **Residual** F13 |

### T296 planning absorption (2026-08-24) — human Router omits 267014 HRESULT; JSON frozen

| Item | Disposition |
|------|-------------|
| Audit `Router: Ready last result: 267014` + `SCHED_S_TASK_TERMINATED` vs Nightly 0 | **Absorb** F1–F7 / AC1–AC3 / AC6 / AC9 |
| Placeholder Manual `nightly --status --quick` | **Absorb** AC9 |
| T269 “do not restyle Router” | **Supersede human only** F1–F3; JSON + heading + Nightly Last Result **affirm** |
| T255 AC6/AC15 human numeric | **Absorb / rewrite** AC2 / AC3 |
| T255 JSON keys / `last_result_hint` SCHED_S | **Affirm freeze** F5 |
| `explain_last_task_result` strings | **Affirm freeze** F6 |
| T281 750 raise / HTTP vs TCP | **Affirm freeze** F8 |
| T255 doctor 16th / persist / `.cmd` / `--no-vault` | **Decline** F10 |
| T297 daemon Stopped vs LLM Open | **Decline steal** F11 |
| T298–T300 / T294 leftover `--write` / T295 engine | **Decline** F17 |
| T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F12 / F17 |
| last-PR Cursor **#211** | **N/A empty** — **no T301** F18 |
| Identity leftover `7d97a456` vs `fcb8a40f` | **Not this track** F27 |

### T296 fold-in (2026-08-24) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 hex parse `0x41306` | **Folded** F33 / AC3 |
| Agy m2 blank/whitespace Status no invented `Ready` | **Already** F2 / F26; **tightened** F34 |
| Agy O1 CLI-EXIT-CODES both SCHED_S success | **Already** F19; **tightened** AC10 |
| Agy O2 help unit 267014 | **Already** F7 / AC6 |
| OpenCode m1 plan HEAD `8b95181` vs `c7d6e3e` | **Snapshot** — preflight refreshed |
| OpenCode m2 word 367→428 | **Snapshot** |
| OpenCode m3 daemon Stopped is T297 | **Already** F11 |
| OpenCode O1 hermetic HRESULT omission | **Already** AC1 + AC8 |
| OpenCode O2 keep `Ready` | **Already** F1 / AC1 |
| OpenCode “no help change” | **Decline** F7 / AC6 |
| OpenCode `nightly.rs` production edit | **Decline** F9 |
| last-PR #211 Cursor | **Affirm N/A** — no T301 |
| No B/M | Nothing to decline of B/M |

### T297 planning absorption (2026-08-24) — Stopped + backend TCP Open contrast; do not start daemon

| Item | Disposition |
|------|-------------|
| Audit `daemon status` Stopped vs llama.cpp `:8081` Open | **Absorb** F1–F6 / AC1–AC6 / AC10 |
| Placeholder Manual `daemon status` — do not start daemon | **Absorb** AC10 / F11 |
| Placeholder string `llama.cpp HTTP Open ≠ daemon` | **Rewrite** F1/F31 — live TCP (not HTTP); `--no-project-context` is Ollama `:11434` |
| T281 closeout F27 Daemon Stopped + port Open | **Absorb** |
| T296 F11 / OpenCode m3 daemon Stopped is T297 | **Absorb** |
| T249 F4 last-line `next:` / no JSON | **Affirm** F5 / F8 / F24 |
| T249 F5/F11 no live start-stop / no sc query | **Affirm** F11 |
| T249 F12 daemon json / uptime / sc query | **Decline** F8 |
| T199 TCP 5×100 ms / keyless / exit 0 | **Affirm** F6 / F7 |
| T255 F18 / T281 F2 raise 750 | **Decline** F10 |
| T281 F10 unify HTTP | **Decline** F9 |
| Doctor `daemon_reachable` vs status Stopped | **Decline** F27 |
| T298–T300 / leftover `--write` / T296 Router | **Decline** F17 |
| T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F12 / F17 |
| last-PR Cursor **#212** | **N/A empty** — **no T301** F14 |
| Identity leftover `7d97a456` vs `fcb8a40f` | **Not this track** |

### T297 fold-in (2026-08-24) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode M1 AC8 vacuous / no `run_status` wiring proof | **Folded** F28 / AC8 keep-bound `TcpListener` + last-line `next:` |
| Agy m1 8-permutation bools | **Folded** F35 / AC1 rstest |
| Agy m2 U+2260 | **Already** F18 / AC5 |
| Agy m3 Running/Stopped skip | **Partial** — Running skip stays; Stopped+Open required via M1 |
| Agy O1 in-process `--help` | **Folded** AC7 `try_parse_from` |
| Agy O2 T281 vs T297 docs | **Folded** F19 |
| OpenCode m1 name-match capture | **Folded** F36 / §5.2 |
| OpenCode m2 conductor Placeholder | **Already** Planned on `18ff6f7` |
| OpenCode m3 F30 both-Open red | **Folded** AC6 rstest |
| OpenCode m4 `TCP connect` after_help | **Already** F20; **tightened** exact sentence |
| OpenCode O1 hermetic last `next:` | **Folded** AC8 |
| OpenCode O2 T85 `:8080` | **Folded** §5.5 |
| OpenCode “fold-in cannot edit conductor.md” | **Decline** — skill allows it |
| last-PR #212 Cursor | **Affirm N/A** — no T301 |
| No B | Nothing to decline of B |

### T298 planning absorption (2026-08-25) — this-machine + short honesty; no bootstrap

| Item | Disposition |
|------|-------------|
| Audit `device status` / `replicate status` U=5 empty | **Absorb** F1–F8 / AC1–AC9 / AC14 |
| Placeholder Manual `device status` + `replicate status` — no bootstrap | **Absorb** AC14 / F13 |
| Placeholder hostname or fingerprint + `local-only; not PQ; not remote wipe` + existing `next:` | **Absorb** F1 / F4 / F5 |
| Placeholder replicate `this machine: fingerprint-or-none` | **Rewrite** F20 — empty is `{hostname} (not enrolled)`, not token `none` |
| T251 F2 last-line `next:` / no `--format` | **Affirm** F5 / F11 |
| T251 F14 status does not reprint honesty paragraph | **Partial lift** F4 — one short line |
| T251 F6 / PROTOCOL-COMPAT JSON keys | **Affirm** F9 / AC7 |
| T251 F12 list JSON / combined dashboard / doctor 16th | **Decline** F16 |
| T198 F7 plural empty copy | **Affirm** F12 |
| T297 closeout T298 steal | **Absorb** (this track) |
| T299 forget-list / T300 graph sparse | **Decline** F24 |
| leftover `--write` / T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F14 / F24 |
| last-PR Cursor **#213** | **N/A empty** — **no T301** F18 |
| Identity leftover `7d97a456` vs `fcb8a40f` | **Not this track** |

### T298 fold-in (2026-08-25) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 `emit_device_roster` returns `Vec` | **Folded** F26 — required |
| Agy m2 `os_hostname` trim CR/whitespace | **Already** F3; **tightened** AC10 |
| Agy m3 AC11 active-without-local + malformed | **Folded** AC11 four cases |
| Agy O1 19-char replicate prefix | **Folded** F8 |
| Agy O2 docs dual empty/enrolled | **Already** F19; **tightened** |
| OpenCode m1 `serial_test` not a dep | **Folded** F27 / AC10 — no crate |
| OpenCode m2 AC6/AC9 env inject | **Folded** same child env as AC1 |
| OpenCode m3 fail-open `(not enrolled)` on enrolled vault | **Folded** F2 / AC11 case 4 `{hostname} (enrolled; fingerprint unavailable)` |
| OpenCode O1 Phase 0 re-locate doc anchors | **Folded** plan Phase 0 |
| OpenCode O2 enrolled last-line CLI | **Already** AC2; **tightened** `last_nonempty_line` |
| OpenCode O3 hostname crates.io date | **Snapshot** — 2025-11-28 publish; decline stands |
| last-PR #213 Cursor | **Affirm N/A** — no T301 |
| No B/M | Nothing to decline of B/M |

### T294 closeout residuals (2026-08-24)

| Residual | Notes |
|----------|-------|
| PATH until `cargo install` | F18 soft |
| Live leftover 5 roots still on `7d97a456` until owner `--write --yes` | F11 |
| gimp / homebrew-tap still no `.env` until first-init | F28 |
| Minted dest label `(no alias) — {8hex}` | F38 |
| Quote-strip `.env` values | T282 F32 |
| T295–T300 | Not stolen |

### T294 planning absorption (2026-08-24) — already-initialized upserts `.env` dest; no `.env` rewrite

| Item | Disposition |
|------|-------------|
| Audit leftover 5 roots dest-missing; `context` already-initialized skips vault ensure | **Absorb** F1–F4 / AC3–AC4 / AC10 |
| Placeholder Manual `context` + `project list` contains env id + print-only rebind dest exists | **Absorb** AC3 / AC4 / AC10 |
| T259 F9 dest must exist / runbook `context` first (currently a lie) | **Absorb as honesty** F8 / F19 — rebind still does not mint |
| T259 F5 memories stay | **Affirm** F10 |
| T240 F2 no silent `.env` | **Affirm** F2 |
| T258 adopt-path / cwd `mismatch: false` | **Decline steal** F9 — Completed; opposite direction |
| T276 F9 live leftover `--write --yes` | **Affirm Stop-Before** F11 |
| T282 `--show` leftover shell | **Decline steal** F6 — Completed |
| T293 neighbors dump sessions | **Decline** — Completed `#209` |
| T295–T300 peers | **Decline →** those placeholders |
| T240 F2 reopen / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F24 |
| last-PR Cursor **#209** | **N/A empty** — **no T301** F25 |
| Identity leftover `7d97a456` vs `fcb8a40f` | **Not this track** — T258 / leftover data T276 |

### T294 fold-in (2026-08-24) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 session parse trim + `_EXTRA` skip | **Already** F3; **folded** AC1 `_EXTRA` |
| Agy m2 malformed session UUID exit 1 | **Already** F14 / AC7 |
| Agy O1 WORKFLOWS leftover ensure-without-rewrite | **Already** F17 / AC9; **tightened** leftover-block sentence |
| Agy O2 second `context` zero duplicate events | **Folded** AC15 |
| OpenCode m1 plan HEAD `2325adc` vs `6fe734c` | **Snapshot** — preflight refreshed |
| OpenCode m2 lossy `:113` must not upsert | **Already** F3 |
| OpenCode m3 `--show` return `:88` | **Already** F6 / AC8 |
| OpenCode O1 AC3 KEY/comment/blank bytes | **Folded** AC3 fixture |
| OpenCode O2 AC4 local seed / `fixture_rebind` private | **Folded** F39 |
| OpenCode O3 `Vault:` only when both IDs parse | **Folded** AC3 count==1 + AC6 absence |
| last-PR #209 Cursor | **Affirm N/A** — no T301 |
| No B/M | Nothing to decline of B/M |

### T288 planning absorption (2026-08-23) — granted-empty vault-pin stanza; no H2

| Item | Disposition |
|------|-------------|
| Audit `briefing project` granted-empty `_None_` vs 3k pins | **Absorb** F2–F5 / AC1–AC4 / AC12 |
| Dual model: briefing = Approved; pins via recall | **Absorb** F1 keep split + F2 labeled stanza |
| T263 F24 soft vault pin COUNT | **Absorb / promote** F4 inventory `count_pinned_memories` |
| T263 F3 / T227 F3 never scrape pins into authority | **Affirm** F1 |
| T263 F29 next-step ≤140 | **Affirm** F6 / AC6 |
| T275 denied grant-wall | **Affirm** F7 / AC5 |
| T287 `list_authority_memories` / `preview_line` | **Reuse** F5 / F15 |
| T287 R1-1 live GLOB 0 | **Absorb** F4/F32 — COUNT not GLOB |
| Placeholder JSON if T180 else human-only | **Absorb JSON overlay** F3 — T180 additive; non-TTY default |
| Personal deny `_None_` | **Decline → T289** |
| Lists/progressive pin count | **Decline → T290** |
| graph neighbors dump sessions | **Decline → T293** |
| leftover dest-missing / context skip upsert | **Decline → T294** |
| T287 list mix | **Completed** — not stolen |
| T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F24 |
| last-PR Cursor #203 | **N/A** empty — **no T301** |
| Identity mismatch leftover `7d97a456` | **Not this track** — T258 / T294 |

### T288 fold-in (2026-08-23) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 fail-open `Repository:` / `ProjectId` parse | **Already** F13/F14; **tightened** F14 + **AC17** |
| Agy m2 Hotspot exclude `Decision \|\| Constraint` | **Already** F5 / AC16 |
| Agy O1 PROTOCOL-COMPAT CLI extras vs daemon DTO | **Already** F25 / AC10 |
| Agy O2 rstest denied / nonempty / 0-pin / with-pin | **Already** AC14 + AC4 + AC1 |
| OpenCode m1 3822 session-join vs ~3821 `count_pinned_memories` | **Folded** F4 / AC12 / spec §5.6 |
| OpenCode m2 pin env `PROJECT_ID`+`SESSION_ID` | **Folded** AC1 / F20 |
| OpenCode m3 NeverInject/Sealed previews | **Folded** F36 display-only; no `is_injectable_privacy` |
| OpenCode O1 `VaultPinStanza` in `renderer.rs` | **Folded** F11 required re-export |
| OpenCode O2 fetch limit 8→32 | **Folded** F5 `limit = 32` |
| last-PR #203 Cursor | **Affirm N/A** — no T301 |
| No B/M | Nothing to decline of B/M |

### T288 closeout residuals (2026-08-23)

| Item | Disposition |
|------|-------------|
| Live `cargo run -- briefing project --format human` on `3581317d` prints `Pinned: 3889` and `_No leading-line DECISION/CONSTRAINT samples in this scope._` (R1-1 / CX1 P3) | **Residual** — pass-1 GLOB 0 (F32); inventory COUNT is Manual SoT; hermetic AC1/AC2 for samples; F17 PATH until `cargo install` |
| Daemon/HTTP packet unaugmented | **Residual** — F29 |
| Personal `_None_` / lists/progressive pin count | **T289 Completed** / **T290** — not stolen |
| Governed preflight no stanza | **Residual** — F27 / T170 D21 |
| First `dev-check` fail-fast `backup_restore__daemon_down_force__succeeds` | **Environmental** — daemon was Running; temporary `daemon stop`; gate re-run **3399** passed. Daemon left **Stopped**. |

| personal briefing deny `_None_` | **T289 Completed** |

### T289 planning absorption (2026-08-23) — denied Personal omits `_None_`; no bootstrap

| Item | Disposition |
|------|-------------|
| Audit `briefing personal` deny + `_None_` prefs U=4 | **Absorb** F1–F4 / AC1–AC2 / AC10 |
| Placeholder Manual `--format human` | **Absorb** AC2 / AC10 |
| T275 F32 Personal `_None_` optional | **Absorb / promote** F1 |
| T275 F35 no project-wall leak | **Affirm** F3 / AC6 |
| T263 F4 recall next | **Affirm freeze** F4 |
| T227 empty_continuity allowed-empty | **Affirm** F6 / AC5 |
| T288 closeout Personal `_None_` | **Absorb** (this track) |
| T288 vault-pin stanza | **Decline** F8 — project-only Completed |
| Lists/progressive pin count | **Decline → T290** |
| T227 #18 synthetic continuity | **Decline** F24 |
| Auto Personal grant | **Decline** F7 |
| T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F19 |
| last-PR Cursor #204 | **N/A** empty — **no T301** |
| Identity leftover `7d97a456` | **Not this track** — T258 / T294 |

### T289 fold-in (2026-08-23) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 helper private `fn` | **Folded** F11 / §5.3 |
| Agy m2 AC4 exact string / one-line / ≤140 | **Already** F2 / AC4 |
| Agy O1 allowed-empty Preferences `_None_` | **Folded** AC5 |
| Agy O2 Personal after_help | **Already** F20 / AC8 |
| OpenCode m1 CP denied test path | **Folded** §2.3 `tests/personal_briefing.rs:154` |
| OpenCode m2 T288 overlay not on `run_personal` | **Folded** F5 / AC3 |
| OpenCode m3 const `_None_` / bootstrap guards | **Folded** AC4 |
| OpenCode O1 reuse `empty_personal` | **Already** AC1; named `:383` |
| OpenCode O2 CAPABILITIES extend not add | **Folded** F20 / AC8 |
| last-PR #204 Cursor | **Affirm N/A** — no T301 |
| No B/M | Nothing to decline of B/M |
| governed lists/progressive empty U=6 | **T290** |
| `query trace` bare `null` | **T291** |
| `policy check` JSON-only | **T292** |
| graph neighbors dump sessions | **T293** |
| leftover dest-missing; context skip vault upsert | **T294** |
| 0 usable encrypted backup | **T295** |
| nightly Router 267014 / TASK_TERMINATED | **T296 Completed** |
| daemon Stopped vs llama Open | **T297** |
| device/replicate U=5 | **T298 Planned** |
| forget-list empty U=6 | **T299** |
| graph sparse live rebuild | **T300** |
| T240 F2 / T263 H2 / 750 ms / clap 5 / density floors | **Declined** — see README-T285-T300 |

### T289 closeout residuals (2026-08-23)

| Item | Disposition |
|------|-------------|
| PATH `ai-brains` still T281-era until `cargo install` | **Residual** — F13; source/hermetic SoT; T282 leftover `--show` + T283 cwd-first + T285–T289 not on PATH |
| Allowed-empty Personal `_None_` | **Residual** — F6 freeze |
| T290 lists/progressive pin count | **Not stolen** |

### T290 planning absorption (2026-08-23) — granted-empty lists/progressive copy-paste recall + Pinned: N; no H2

| Item | Disposition |
|------|-------------|
| Audit evidence/source/review list + `query progressive` granted-empty U=6 | **Absorb** F1–F7 / AC1–AC6 / AC12 |
| Placeholder Manual four commands | **Absorb** AC12 |
| T263 F8 parenthetical “vault pins are not governed evidence” | **Absorb / promote** F3 / F7 copy-paste + `(Pinned: N)` |
| T263 F9 leave T243 ellipsis | **Partial reopen** — granted-empty `next_step` string growth; **affirm** F8 deny stderr const |
| T243 F5 progressive `next_step` overlay | **Affirm gate**; **grow** contents (operator query) |
| T214 `count_pinned_memories` | **Reuse** F4 — fail-open; 0-pin honesty AC2 |
| T288 `vault_pin_*` keys / briefing stanza | **Decline** F3 / F23 — Completed project-only |
| T289 Personal deny `_None_` | **Decline** — Completed `#205` |
| `query trace` bare `null` | **Decline → T291** |
| `policy check` JSON-only | **Decline → T292** |
| graph neighbors dump sessions | **Decline → T293** |
| leftover dest-missing / context skip upsert | **Decline → T294** |
| forget-list empty next | **Decline → T299** |
| T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F24 |
| last-PR Cursor #205 | **N/A** empty — **no T301** |
| Identity leftover `7d97a456` | **Not this track** — T258 / T294 |

### T290 fold-in (2026-08-23) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 review human `(none)` + next line | **Folded** AC3 / F2 |
| Agy m2 formatter single-line | **Folded** F6 / F7 / AC1 / AC4 |
| Agy O1 CLI-EXIT-CODES + OPERATIONS exit 0 | **Already** F25 / AC10; **tightened** AC10 |
| Agy O2 sanitize rstest tab / newline / quotes / 80 | **Already** AC4; **folded** tab + formatter `!contains('\n')` |
| OpenCode m1 `QueryStore` import in four callers | **Folded** F12 / §5.2 |
| OpenCode m2 AC6 not `progressive_cmd` `"x"` | **Folded** AC6 |
| OpenCode O1 exact `next_step` `assert_eq!` | **Folded** AC1 unit exact |
| OpenCode O2 progressive no `--format` | **Already** F10 |
| last-PR #205 Cursor | **Affirm N/A** — no T301 |
| No B/M | Nothing to decline of B/M |

### T290 closeout residuals (2026-08-23)

| Item | Disposition |
|------|-------------|
| PATH `ai-brains` still T281-era until `cargo install` | **Residual** — F17; source/hermetic SoT; T282 leftover `--show` + T283 cwd-first + T285–T290 not on PATH |
| Daemon list overlay has copy-paste query but no `(Pinned: N)` | **Residual** — F14 |
| Personal/Workspace list COUNT skipped | **Residual** — F4 |
| T291 `query trace` `null` / T292 policy-check human / T293–T300 | **Not stolen** |
| Codex CX1 P1 PowerShell `$`/backtick interpolators | **Fixed** — sanitize drops `$` and backtick; rstest + formatter unit. **#206 Bugbot Low** (collapse around dropped interpolators / final trim) → **T291 F16**. |

### T291 planning absorption (2026-08-23) — missing envelope + human next; no invented traces

| Item | Disposition |
|------|-------------|
| Audit `query trace` bare `null` U=3 | **Absorb** F1–F8 / AC1–AC4 / AC11 |
| Placeholder Manual `missing-id` + `--format human` | **Absorb** AC11 |
| Placeholder JSON stay-null **or** wrap | **Absorb missing-only envelope** F1/F7 — not `{trace:null}` found wrap |
| Placeholder `query progressive --trace` | **Decline** — flag does not exist (F8 names `--dry-run false`) |
| T263 F6 / F26 scalar `null` freeze | **Lift** F1 with PROTOCOL-COMPAT §5 row |
| T202 F31 no project-id | **Affirm** F5 / AC4 |
| T152 progressive `dry_run` default true | **Affirm** F9 — next names `--dry-run false` |
| T290 lists/progressive next | **Decline** — Completed `#206` |
| T292 `policy check` human | **Decline → T292** |
| T293 neighbors dump sessions | **Decline → T293** |
| T294 leftover dest-missing | **Decline → T294** |
| T298 device/replicate empty | **Decline → T298** |
| T299 forget-list empty | **Decline → T299** |
| T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F24 |
| last-PR Cursor **#206** Bugbot Low `sanitize_recall_query` interpolator collapse | **Absorb** F16 / AC8 — no T301 |
| Identity leftover `7d97a456` | **Not this track** — T258 / T294 |

### T291 fold-in (2026-08-23) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 `--format JSON` must not use `OutputFormat::parse` | **Folded** F3 / AC7 / §5.5 |
| Agy m2 sanitizer space boundaries / no double-space | **Already** F16; **tightened** |
| Agy O1 parent + Trace after_help | **Already** F14; **tightened** AC10 `:1589` / `:1892` / `:1924` |
| Agy O2 e2e persist then trace | **Already** AC5; **tightened** bootstrap + `--dry-run false` |
| OpenCode M-1 hotspot #3 → #2 | **Folded** snapshot |
| OpenCode M-2 / M-3 preflight words / doctor warn count | **Snapshot only** (volatile); Phase 0; not DoD |
| OpenCode o-1 clap `value_parser` | **Folded** (same as Agy m1) |
| OpenCode o-2 bootstrap covers `get_query_trace` | **Folded** AC5 / §5.6 |
| OpenCode o-3 `api_version` `"1"` | **Already** F7 |
| OpenCode o-4 `auto` TTY | **Already** F3 |
| OpenCode o-5 `cli_help_ia.rs` lock | **Decline** — AC10 hermetic `--help` |
| OpenCode o-6 OPERATIONS two null sentences | **Folded** AC10 |
| last-PR #206 Cursor | **Affirm F16** — no T301 |
| No B | Nothing to decline of B |

### T291 closeout residuals (2026-08-23)

| Item | Disposition |
|------|-------------|
| PATH `ai-brains` still T281-era until `cargo install` | **Residual** — F17; source/hermetic SoT; T282 leftover `--show` + T283 cwd-first + T285–T291 not on PATH |
| Found `--format human` still QueryTraceDto JSON | **Residual** — F10 by design |
| Default progressive still does not persist | **Residual** — F9 by design |
| T292 `policy check` human / T293–T300 | **Not stolen** |
| Codex CX1 P1 process (full gate / publish pending at review time) | **verified_fixed** after closeout + Phase 6 |

### T292 planning absorption (2026-08-23) — Family A auto TTY human / pipe JSON; deny two-line SHORT; no parse flip

| Item | Disposition |
|------|-------------|
| Audit `policy check` JSON-only U=7 | **Absorb** F1–F7 / AC1–AC4 / AC10 |
| Placeholder Manual `--format human` + `--format json` | **Absorb** AC10 |
| Placeholder `--format auto` TTY human / pipe JSON | **Absorb** F1 / F21 / AC6 / AC7 |
| Placeholder deny `denied: <cap> — next` | **Absorb** F7 two lines (SHORT exact) |
| T266 F1 Family D for policy | **Partial lift** — check only → A; show/bootstrap **affirm D** F26 |
| T266 F11 / T227 F34 `OutputFormat::parse` | **Affirm freeze** F9 |
| T241 F6 catalog / F14 SHORT | **Affirm** F8 / F7 |
| T226 soft-resolve | **Affirm** F8 / AC5 |
| T160 R1-01 one JSON deny document | **Affirm** F6 / AC5 |
| T210 F13 no auto-grant | **Affirm** F8 / F19 |
| T291 query-trace envelope | **Decline** — Completed `#207` |
| T293 neighbors dump sessions | **Decline → T293** |
| T294 leftover dest-missing | **Decline → T294** |
| T298 device/replicate empty | **Decline → T298** |
| T299 forget-list empty | **Decline → T299** |
| T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F19 |
| last-PR Cursor **#207** | **N/A empty** — **no T301** F20 |
| Identity leftover `7d97a456` | **Not this track** — T258 / T294 |

### T292 fold-in (2026-08-23) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 helpers `pub(crate)` | **Already** F25; **tightened** not `pub` / not `mod.rs` re-export |
| Agy m2 skip `fail_api` | **Already** F7; **tightened** AC3 stderr empty + §5.7 |
| Agy O1 CAPABILITIES Family A row | **Already** F12 / AC9 |
| Agy O2 clap `JSON`/`Pretty` InvalidValue | **Already** AC6 JSON / F3; **folded** AC6 Pretty |
| OpenCode m1 AC2 principal env | **Folded** F27 / AC2 / §5.6 — `hermetic_bin` denylist; not T210 helpers |
| OpenCode m2 `CheckOptions.format: String` | **Already** one clap constructor; **folded** F28 |
| OpenCode O1 OPERATIONS script `--format json` | **Folded** F12 / AC9 exact sentence |
| OpenCode O2 `stdout().is_terminal()` | **Already** F1; **tightened** |
| OpenCode O3 T241 F6b catalog after_help | **Folded** F8 / F29 / AC9 |
| OpenCode word/pin snapshot | **Snapshot only** — not DoD |
| F3/F26 AC-id slips | **Folded** InvalidValue = AC6; peers = AC8 |
| last-PR #207 Cursor | **Affirm N/A** — no T301 |
| No B/M | Nothing to decline of B/M |

### T292 closeout residuals (2026-08-23)

| Item | Disposition |
|------|-------------|
| PATH until `cargo install` (F13) | **Residual** — source/hermetic SoT |
| Propose*/Approve*/Erase/Export human deny next still SHORT (F24) | **Residual** — bootstrap does not issue those caps |
| `policy show` / `policy bootstrap` TTY still JSON (F26) | **Residual** — Family D peers |
| Human deny stderr empty (no `POLICY_DENIED:` CODE) | **Residual** — F7 by design |
| T293–T300 | **Not stolen** |

### T293 planning absorption (2026-08-23) — human-only prefer-authority 1-hop; JSON F9 freeze; no 2-hop

| Item | Disposition |
|------|-------------|
| Audit `graph neighbors` dump sessions U=7 | **Absorb** F1–F6 / AC1–AC4 / AC12 |
| Placeholder Manual `--format human --limit 8` | **Absorb** AC12 |
| Placeholder JSON freeze vs human-only | **Absorb** F2 / AC4 (T246 F9 JSON; pretty permute) |
| Placeholder PREVIEW still `{n} memories · first line` | **Absorb** F6 / AC5 T278 freeze |
| T278 F18 2-hop pretty rows | **Affirm decline** F3 |
| T246 F5 keys / F9 JSON sort | **Affirm freeze** F2 |
| T262 pin = node | **Affirm** F10 / AC7 |
| T287 human prefer-fill pattern | **Reuse** F1; do **not** reuse `prefer_fill_authority` F30 |
| T285 chrome-seed skip (recall `--graph-boost`) | **Decline** F20 |
| T292 policy-check human | **Decline** — Completed `#208` |
| leftover dest-missing / context skip upsert | **Decline → T294** |
| T295–T299 peers | **Decline →** those placeholders |
| T300 live rebuild / floors | **Decline → T300** F7 |
| T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F21 |
| last-PR Cursor **#208** | **N/A empty** — **no T301** F19 |
| Identity leftover `7d97a456` | **Not this track** — T258 / T294 |

### T293 fold-in (2026-08-23) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 `" · "` split | **Already** F4/AC13; **tightened** `split_once` + AC13 dots case |
| Agy m2 `sort_by_key` stable | **Already** F1 original index; **folded** no `sort_unstable_by_key` |
| Agy O1 PROTOCOL-COMPAT array-order | **Already** F14/AC11; **tightened** `:95` |
| Agy O2 four-tier rstest | **Folded** AC2 case 6 |
| OpenCode m1 no `memory_projection` insert helper | **Folded** F31 / §5.4 — new helper; T278 DROP COLUMN is fail-open only |
| OpenCode m2 `" · "` split | **Same as Agy m1** |
| OpenCode O1 PROTOCOL-COMPAT `:103` neighbors | **Decline citation** — live `:103` is `project scan-roots`; `:95` is the array-order row |
| OpenCode O2 four-tier rstest | **Same as Agy O2** |
| OpenCode O3 exact AC3 first-row UUID | **Already** AC4 dump JSON; **folded** AC3 first pretty row is not dump id |
| OpenCode HEAD/word snapshot | **Snapshot only** — not DoD |
| last-PR #208 Cursor | **Affirm N/A** — no T301 |
| No B/M | Nothing to decline of B/M |

### T293 closeout residuals (2026-08-23)

| Item | Disposition |
|------|-------------|
| PATH until `cargo install --features graph` | **Residual** — F15; source/hermetic SoT |
| Live chrome-only 1-hop (`b189ad20`) still dump-first (`## Objective`) | **Residual** — F25 honest; Manual AC12 recorded; hermetic AC3 SoT |
| Session PREVIEW first-line still chrome when pin buried later | **Residual** — F6; do not rewrite caption |
| Sparse E/N ~0.12 / live rebuild | **Decline → T300** |
| T294–T299 peers | **Not stolen** |
| Codex P1 process (gates/closeout at review time) | **Closed by** full gate + publish |

### T287 closeout residuals (2026-08-23)

| Item | Disposition |
|------|-------------|
| Live `cargo run -- memory list --limit 5` on `3581317d` still `## Objective` (R1-1) | **Residual** — pass-1 GLOB matched 0 rows in-scope (F32 recency-fill); hermetic AC1 SoT; F17 PATH until `cargo install` |
| Duplicate GLOB vs `index_pass1_glob_sql` | **Residual** — F27 no shared helper |
| USER/SYSTEM TAGS GLOB | **Residual** — F29; default assistant |
| F18 canary pin not in this project's count | **Residual** — leftover-shell `PROJECT_ID` vs `.env` (`pin.rs` `std::env::var`); T282 leftover / not this track |
| T288 / T293 / T299 | **Not stolen** |

### T287 planning absorption (2026-08-23) — human prefer-fill; JSON/store recency frozen

| Item | Disposition |
|------|-------------|
| Audit `memory list --limit 5` just-now ingest | **Absorb** F1–F6 / AC1–AC3 / AC15 |
| Placeholder Manual DoD `--limit 5` + `--summary` | **Absorb** AC1/AC7/AC15 |
| T274 F13 / T285 F14 / T286 F15 / T286 AC15 list ORDER | **Lift human pinned** F1; **affirm store+JSON** F2/F3/AC4 |
| T216 default limit 50 (placeholder said 5) | **Affirm freeze** F11; DoD uses `--limit 5` |
| Preview `TAGS:` envelope | **Absorb** F6 / AC3 `first_contentful_line` |
| JSON freeze vs human-only permute | **Absorb** F2 JSON recency (T283 analog); F9 no `--authority` |
| T216 `--status` / `--summary` / JSON keys / exit 2 | **Affirm freeze** F7/F8/F10/F13 |
| T216 `--offset` / tag histogram | **Decline** T216 F24 |
| `forget --match` two-pass | **Decline** F14 — T274 F18 |
| USER/SYSTEM TAGS GLOB | **Decline** F29 — T285 F7 |
| briefing granted-empty vs pins | **Decline → T288** |
| graph neighbors dump sessions | **Decline → T293** |
| forget-list empty next | **Decline → T299** |
| leftover dest-missing / context skip upsert | **Decline → T294** |
| T240 F2 / T263 H2 / T211 F25 / clap 5 / rusqlite 0.40 | **Decline** F16 |
| last-PR Cursor #202 | **N/A** empty — **no T301** |
| Identity mismatch leftover `7d97a456` | **Not this track** — T258 / T294 |

### T287 fold-in (2026-08-23) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 empty-contentful `preview_line` fallback | **Already** F6; **tightened** AC3 TAGS-only |
| Agy m2 exact GLOB + single `AND (` | **Already** F4 / F27 / AC5 |
| Agy O1 `prefer_fill_authority` multi-case | **Folded** AC16 rstest |
| Agy O2 after_help dual-truth | **Already** F30 / AC17 |
| OpenCode m1 `preview_line` forget/graph callers | **Folded** F6 / F24 inherit-only (`forget.rs:19/24`, `graph.rs:248`) |
| OpenCode m2 pinned/word-count 237 vs 612 | **Folded** volatile snapshot |
| OpenCode m3 `run_inventory` `:136` vs `:137` | **Folded** live `:137` |
| last-PR #202 Cursor | **Affirm N/A** — no T301 |
| No B/M | Nothing to decline of B/M |

### T286 closeout residuals (2026-08-23)

| Item | Disposition |
|------|-------------|
| Live `cargo run --pretty -m 1500` Index item 1 still `## Objective` on `3581317d` (R1-1 / Codex P3-01) | **Residual** — hermetic AC1/AC5 SoT; drain breaks if first addable row exceeds `max_words`; F21 PATH until `cargo install`; F22 no live canary |
| Session section still recency chrome (F12) | **Residual** — Index is the decision list |
| Index items 2+ may still be `## Objective` (F14) | **Residual** — prefer-fill not hard-exclude |
| `USER:` / `SYSTEM:` TAGS GLOB miss pass-1 (OpenCode L1) | **Residual** — T285 F7; default assistant |
| Duplicate OR-join with lexical Prefer (F27) | **Residual** — no shared helper this track |
| `in_context_decisions` still a substring not vault COUNT (F10) | **Affirm freeze** |
| T287 / T288 / T293 | **Not stolen** |

### T286 planning absorption (2026-08-23) — Index TAGS-or-GLOB + envelope titles; no Session steal

| Item | Disposition |
|------|-------------|
| Audit Index `## Objective`; summary decisions 0 vs 3647/3716 pins | **Absorb** F1–F4 / AC1–AC6 / AC16 |
| Placeholder Manual DoD `--pretty` + `--summary` | **Absorb** AC5/AC6/AC16; Manual `-m 1500` |
| T285 F13 / closeout Index/summary still Objective | **Absorb** (this track) |
| T274 AC6/AC7 untagged Index + window `DECISION:` | **Absorb as regression** F5 / AC3 |
| T274 two-pass GLOB misses TAGS envelope | **Absorb** F2 `index_pass1_glob_sql` |
| Index title `content.lines().next()` is `TAGS:` | **Absorb** F4 `first_contentful_line` |
| Placeholder (b) show Pinned N next to 0 | **Already T214** — insufficient; get pin into window (F10) |
| Placeholder (a) vault-authority count key | **Decline** F10 — T220 no new keys |
| Session + Index both Objective | **Partial** — Index DoD; Session recency **decline F12** |
| T214 dual counts / 9-arg formatters | **Affirm freeze** F10 |
| T220 JSON keys / T265 `sections[]` | **Affirm freeze** F10 / F11 |
| T272 Safety skip-set / T264 caps | **Affirm freeze** F6 / F9 |
| T279 Safety SQL | **Decline** F7 — Completed |
| T250 Index not line-capped | **Affirm** F8 |
| T264 Index fetch-80 leftover-heavy | **Decline** — T264 soft; not leftover drop |
| `memory list` just-now ingest | **Decline → T287** |
| briefing granted-empty vs pins | **Decline → T288** |
| graph neighbors dump sessions | **Decline → T293** |
| leftover dest-missing / context skip upsert | **Decline → T294** |
| T240 F2 / T263 H2 / T211 F25 / clap 5 / rusqlite 0.40 | **Decline** F16 / F20 |
| last-PR Cursor #201 | **N/A** empty — **no T301** |
| Identity mismatch leftover `7d97a456` | **Not this track** — T258 / T294 |

### T286 fold-in (2026-08-23) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 empty envelope → `Untitled Memory` | **Already** F4 / AC10; **tightened** replace `:538` |
| Agy m2 `debug_assert!(is_safe_sql_ident)` | **Folded** F2 / AC4 |
| Agy O1 tagged summary JSON file | **Already** AC6; named `preflight_summary_json.rs` |
| Agy O2 CAPABILITIES Index envelope | **Already** F29 |
| OpenCode L1 USER/SYSTEM TAGS GLOB | **Decline as DoD** — T285 F7; residual §11 |
| OpenCode L2 duplicate OR-join | **Already** F27 |
| OpenCode pinned COUNT 3647 vs 3716 | **Folded** volatile snapshot |
| last-PR #201 Cursor | **Affirm N/A** — no T301 |
| No B/M | Nothing to decline of B/M |

### T285 planning absorption (2026-08-22) — envelope + detector + chrome-seed skip; no Index steal

| Item | Disposition |
|------|-------------|
| Audit recall/search/semantic/sync-vault still chrome Q=4 | **Absorb** F1–F12 / AC1–AC6 / AC12–AC14 |
| Placeholder Manual DoD `--tag` canary | **Absorb** F2 envelope so TAGS pins classify |
| T274 closeout “live dumps until install” | **Absorb / reopen** — PATH **0.1.2** still Q=4 |
| T274 closeout detector not rstest | **Absorb** F26 / AC2 new prefixes |
| T274 I1 `ASSISTANT:` + CLI `TAGS:` line | **Absorb** F2 |
| Live `# AI-Brains Session Onboarding Complete` / `# Review of Track` | **Absorb** F5 |
| Default `graph_hop_depth=1` chrome seeds dumps | **Absorb** F10 (T260 analog) |
| T274 AC4 needle-in-both-bodies | **Absorb** AC4 **asymmetric** needle |
| T274 closeout GLOB lowercase | **Partial F9** — envelope/retain SoT |
| T274 closeout AC16 semantic helper | **Decline as DoD** — AC14 fallback; no HTTP |
| Preflight Index `## Objective` / summary 0 vs 3648 | **Decline → T286** |
| `memory list` just-now ingest | **Decline → T287** |
| `graph neighbors` dump sessions | **Decline → T293** |
| leftover dest-missing / context skip upsert | **Decline → T294** |
| T276 F39 `--global` skip leftover MATCH | **Decline F23** |
| T240 F2 / T263 H2 / T211 F25 / T218 floors / clap 5 / rusqlite 0.40 | **Decline** |
| last-PR Cursor #200 | **N/A** empty — **no T301** |
| Identity mismatch leftover `7d97a456` | **Not this track** — T258 / T294 |

### T285 fold-in (2026-08-22) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode M1 AC4 dumps without needle not red | **Folded** AC4/AC5 body-MATCH + chrome first line |
| OpenCode M2 hop-1 untestable graph-off | **Folded** F36 unit + F37/AC17 CLI `test(graph)` |
| OpenCode M3 snapshot tuple has no content | **Folded** F10 read `blended` content |
| Agy m1 recency-retry `NOT IN` binds | **Folded** F34 / AC15 |
| Agy m2 role+TAGS no panic / whitespace | **Folded** F2 / AC1 |
| OpenCode m1 live `:180` pre-retain gate | **Already** F7; tightened |
| OpenCode m2 F8 vs substring_fallback | **Folded** F8 / §5.2 |
| OpenCode m3 envelope order | **Already** F2 |
| Agy O1 CAPABILITIES graph sentence | **Already** F31 |
| Agy O2 dumps without needle | **Decline** — conflicts with M1 |
| OpenCode O1 retrieval graph CI line | **Decline as DoD** — CLI graph job enough |
| OpenCode O2 `sync.rs:529` → `:532` | **Folded** §2.3 |
| OpenCode O3 ROLE_PREFIXES via core | **Decline** — duplicate three tokens in ranking.rs |
| last-PR #200 Cursor | **Affirm N/A** — no T301 |

### T285 closeout (2026-08-22)

| Item | Disposition |
|------|-------------|
| PATH until `cargo install` | **Residual** F21 — source has rank v2; PATH 0.1.2 until owner asks install |
| Live canary pin landed in `[test-alias]` without forcing cwd PROJECT_ID | **Residual** — ranking SoT is hermetic AC12/AC13; `--global` GUID hit #1 |
| Pretty `[session]` badge | **Soft** spec §11 |
| More chrome prefixes as vault grows | **Soft** closed list |
| Index/summary still Objective | **T286** |
| `memory list` just-now | **T287** |
| graph neighbors dump sessions | **T293** |

## T274–T284 placeholders (2026-08-21) — post-T270 live CLI quality

Minted from PATH dogfood + last-PR Cursor #188 (2 Bugbot Mediums, verified on `14d42af`). Full F-list on `/plan-track TNN`. **Do not implement Placeholders.** T284 is **Completed** 2026-08-22 (`#193`). **T278 Completed** 2026-08-22 (`#194`). **T279 Completed** 2026-08-22 (`#195`). **T280 Completed** 2026-08-22 (`#196`). **T281 Completed** 2026-08-22 (`#197`). **T282 Completed** 2026-08-22 (`#198`). **T283 Completed** 2026-08-22.

| Item | Track |
|------|-------|
| recall/search/semantic/preflight/memory-list session dumps over pins | **T274 Completed** |
| briefing/progressive/lists POLICY_DENIED (0 of 3 grants) | **T275 Completed** |
| leftover `7d97a456` ~18k / `--global` junk | **T276 Completed** (prefer-fill + labels; T264 F11 no drop; live 11 roots still F9) |
| 22/22 backup FAIL; no usable encrypted file | **T277 Completed** (F2 fail-closed create + mixed hermetic; live `--no-prune` skipped — owner did not confirm) |
| graph sparse + neighbors blank preview | **T278 Completed** 2026-08-22 |
| preflight Safety = review-track Objective | **T279 Completed** 2026-08-22 |
| deny/`policy show` `--scope …` vs doctor omit | **T280 Completed** |
| nightly Completion timeout vs daemon Open (750 ms not raised) | **T281 Completed** `#197` |
| `context --show` misses leftover shell | **T282 Completed** 2026-08-22 `#198` |
| `project list` leftover-first | **T283 Completed** 2026-08-22 |
| #188 Work hides CE when held dominates; apply samples prefer overlay ids | **T284 Completed** 2026-08-22 |
| device/replicate/query-trace/forget empty; T266 JSON; T240 F2; T255 750ms; T263 H2 | **Declined** — see README-T274-T284 |

### T274 planning absorption (2026-08-21) — leading-line + two-pass; no Safety steal

| Item | Disposition |
|------|-------------|
| Audit recall/search/semantic/preflight Index/summary dumps over pins | **Absorb** F1–F12 / AC1–AC7 / AC14 |
| `memory list` just-now ingest | **Partial** F13 — T216 recency stays |
| `sync query` vault dumps | **Absorb** F14 / AC15 via `recall_full` |
| T211 F4 anywhere-in-body `decision:` | **Lift** F2 — buried JSON/skill text → Other |
| T260 demote-only fails when depth is all chrome | **Absorb** F7 two-pass (not hard-exclude transcripts) |
| Preflight Safety = `## Objective` | **Decline → T279** |
| briefing/progressive POLICY_DENIED | **Decline → T275** |
| leftover `7d97a456` | **Decline → T276** |
| last-PR Cursor #188 Work / apply samples | **Decline → T284** (2 Mediums; still true on `14d42af`) |
| T240 F2 / T263 H2 / T211 F25 / T218 floors / clap 5 / rusqlite 0.40 | **Decline** |
| Identity mismatch quiet (T242 analog) | **Not this track** — T258 adopt-path; leftover data T276 |

### T274 fold-in (2026-08-21) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 parameterized pass-2 `NOT IN` | **Folded** F35 / AC17 |
| Agy m2 chrome-only first-line dedupe | **Already** F10; AC5 tightened |
| Agy O1 `AUTHORITY_GLOB_SQL` const | **Folded as helper** F36 (column arg) |
| Agy O2 skip pass 2 when pass1 full | **Already** F7 / §5.2 |
| OpenCode m1 HEAD `9a99117` vs `deabae7` | **Folded** §2.1 |
| OpenCode o1 summary 1/1/1 vs 0/0/0 | **Folded** volatile; fold-in 0/0/0 @ 3324 |
| OpenCode o2 hotspot 3.999 vs 3.990 | **Folded** snapshot |
| OpenCode o3 grep `classify_pin_kind` | **Folded** Phase 0 |
| last-PR #188 Cursor | **Affirm T284** — no T285 |
| No B/M | Nothing to decline of B/M |

### T275 planning absorption (2026-08-21) — grant-wall + CLI bootstrap lock; no auto-grant

| Item | Disposition |
|------|-------------|
| Audit briefing/progressive/lists POLICY_DENIED (0 of 3) | **Absorb** F1–F6 / AC1–AC5 |
| Denied human `_None_` looks like empty vault | **Absorb** F1/F2 / AC1 |
| CLI `policy bootstrap` → briefing/evidence untested (T210 gap) | **Absorb** F5/F6 / AC4/AC5 (System principal; T221 F31) |
| T241 F21 skill one-liner | **Absorb** F23 docs |
| T241 F20 `preflight --install-grants` | **Decline** F9 — mutation stays `policy bootstrap` |
| Auto-grant on `init` / first preflight | **Decline** F8 — T210 F13 |
| T280 deny hint `--scope …` vs doctor omit | **Decline** F11 → **T280** |
| T263 H2 pin→Approved | **Decline** F12 |
| leftover `7d97a456` | **Decline** → **T276** |
| last-PR Cursor #189 | **N/A** — comments/reviews empty |
| last-PR #188 Work / apply samples | **Affirm T284** — no T285 |
| T240 F2 / T255 750 ms / clap 5 / rusqlite 0.40 | **Decline** |
| Identity mismatch quiet | **Not this track** — T258 adopt-path; leftover data T276 |
| Live operator bootstrap | **F10** — `--dry-run` only unless owner confirms at go |

### T275 fold-in (2026-08-21) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 GRANT_WALL ≤140 | **Already** F2 / AC2 — frozen **88** chars |
| Agy m2 personal deny no project bootstrap | **Folded** F35 / AC16 |
| Agy O1 AC4 principal-trap comment | **Folded** F36 |
| Agy O2 const adjacency | **Folded** F37 |
| OpenCode m1 HEAD `c576b58` vs `8cb1ce0` | **Folded** §2.1 — product crates identical |
| OpenCode m2 Bootstrap `:2211` | **Folded** §2.3 |
| OpenCode m3 `empty_denied` `:218` | **Folded** §2.3 |
| OpenCode m4 JSON no grant-wall prose | **Already** F3 — `denied: true`; do not extend `denial_hint` |
| OpenCode m5 preflight budget analog | **Folded** F29 — renderer order only; no `preflight.rs` growth |
| OpenCode m6 F16 “Domain in CLI” | **Folded** F16 — CP `renderer.rs` |
| last-PR #189 Cursor | **Affirm N/A** — no T285 |
| No B/M | Nothing to decline of B/M |

### T275 closeout residuals (2026-08-21)

| Residual | Disposition |
|----------|-------------|
| PATH `ai-brains` until `cargo install` / `Build-AIBrains.ps1` | F18 |
| Live vault still 0 of 3 grants | F10 — owner did not confirm live bootstrap; hermetic is DoD |
| T280 deny hint still `--scope …` vs doctor omit | **T280 Completed** 2026-08-22 |
| Personal denied `_None_` left | F32 optional; F35 contamination locked |

### T276 planning absorption (2026-08-21) — prefer-fill + labels; no silent exclude

| Item | Disposition |
|------|-------------|
| Audit leftover `7d97a456` ~18k / `--global` junk | **Absorb** F1–F6 / AC1–AC5 |
| T264 leftover-first recall / filter-flag residual | **Partial** — prefer-fill + pretty tags **DoD**; `--exclude-project` **decline F20** |
| T259 leftover memory reclassify | **Decline F7** — memories stay |
| Live leftover 11 `C:\dev\*` roots | **F9** Stop-Before; hermetic is DoD |
| whoami mismatch:false | **Already T258** — F10; shell leftover **T282** |
| `project list` leftover-first | **Decline → T283** |
| last-PR Cursor #190 | **N/A** — comments/reviews empty |
| last-PR #188 Work / apply samples | **Affirm T284** — no T285 |
| Identity mismatch `7d97a456` vs `fcb8a40f` | **Partial** — leftover volume this track (F2 cwd preferred); adopt-path T258; no T285 |
| T240 F2 / T274 chrome / clap 5 / rusqlite 0.40 | **Decline** |

### T276 fold-in (2026-08-21) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 tag before score, one space | **Folded** F4 / AC4 |
| Agy m2 HashSet merge dedupe | **Folded** F38 / AC1 |
| Agy O1 `format_pretty_hit_line` `project_tag` | **Already** F18 |
| Agy O2 preferred-full skip | **Folded** F39 |
| OpenCode m1 drop COALESCE / two-search only | **Partial** — fill already F1; COALESCE **kept** F15 for tags |
| OpenCode m2 both arms + bridge None | **Folded** F40 |
| OpenCode m3 AC3 pre-rerank | **Folded** F41 |
| OpenCode O1 empty-hint Try `--global` | **Decline** — live global arm already honest |
| OpenCode CP `display_label` `:383` | **Decline citation** — CLI `project.rs:383` |
| leftover UUID `7d97a51a` | **Decline typo** — live `7d97a456` |
| last-PR #190 Cursor | **Affirm N/A** — no T285 |
| No B/M | Nothing to decline of B/M |

### T276 closeout residuals (2026-08-22) — prefer-fill + labels shipped

| Residual | Disposition |
|----------|-------------|
| PATH `ai-brains` until `cargo install` / `Build-AIBrains.ps1` | F22 |
| Live leftover 11 `C:\dev\*` roots still on `7d97a456` | F9 — owner did not confirm `--write --yes` |
| `--exclude-project` clap flag | F20 decline |
| Leftover memory reclassify / `MemoryMoved` | F7 / T259 F5 |
| JSON `project_id` on `RecallResult` | F5 T180 freeze |
| Semantic-only prefer-fill e2e | F32; lexical AC2 is the hole |
| `project list` leftover-first | **T283 Completed** 2026-08-22 |
| `context --show` leftover shell | **T282 Completed** `#198` |

### T277 planning absorption (2026-08-22) — current-key create; no rekey of T244 `.bak`

| Item | Disposition |
|------|-------------|
| Audit 22/22 FAIL; doctor no usable encrypted backup | **Absorb** F1–F7 / AC1–AC7 |
| T225 residual “operator still runs live `backup create`” | **Absorb** F4 / AC7 (owner-confirm) |
| T244 AC12 Readable `vault-2026-08-12T15-50-06.db.bak` | **Absorb regression** — live list `(unreadable key)`; new snapshot is DoD; do not transcode (F5) |
| T209 L3 real wrong-key fixture | **Partial F33** — mixed other-key `.bak` + create **hard**; verify taxonomy stays soft |
| T244 F17 / T225 F17 verify `--quiet` / JSON summary / VerifyError | **Decline** F14 |
| T244 F18 archive helper | **Decline** F14 |
| T187 `cipher_integrity_check` | **Decline** F14 |
| Restore / create daemon probe | **Decline** F9/F35 — T188 restore-only; live daemon Stopped |
| Default live `--keep 10` (would prune 12) | **Decline** F4/F19 — go uses `--no-prune` |
| last-PR Cursor #191 | **N/A** — comments/reviews empty |
| last-PR #188 Work / apply samples | **Affirm T284** — no T285 |
| leftover `7d97a456` / T278–T283 | **Decline** peers |
| T240 F2 / clap 5 / rusqlite 0.40 | **Decline** |

### T277 fold-in (2026-08-22) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 drop dest before classify | **Folded** F42 |
| Agy m2 Err names class | **Folded** F2 / F43 |
| Agy O1 shared other-key utility | **Partial** F44 — file-local helper only |
| Agy O2 rustdoc usable invariant | **Folded** F2 |
| OpenCode m `drop(dst)` before `remove_file` | **Folded** F42 (same as Agy m1) |
| OpenCode O AC1 `!exists()` | **Folded** F43 / AC1 |
| OpenCode vault size drift | **Folded** F36 volatile snapshot |
| last-PR #191 Cursor | **Affirm N/A** — no T285 |
| No B/M | Nothing to decline of B/M |

### T277 closeout residuals (2026-08-22) — fail-closed create + mixed hermetic

| Residual | Disposition |
|----------|-------------|
| PATH `ai-brains` until `cargo install` / `Build-AIBrains.ps1` | F16 |
| Live 22 residual `.bak` still KeyMismatch / plain / Incomplete | F5/F9 — owner did not confirm live `backup create --no-prune` |
| Default keep-10 would prune 12 residuals | F4/F19 — product default stays; live skip used `--no-prune` in hermetic |
| Prune dry-run `remaining_count` lie | F20 |
| Class-aware prune / archive / `backups/legacy/` | F14 / T244 F18 |
| verify `--quiet` / JSON summary / VerifyError | T244 F17 |
| `cipher_integrity_check` | T187 |
| Offsite / immutable copy | local-first |
| Integrity-check fail leaves dest file | pre-T277; F2 is post-meta |
| Shared crate other-key fixture module | F44 |

### T284 planning absorption (2026-08-22) — Work dispose counts + apply samples; no live apply

| Item | Disposition |
|------|-------------|
| #188 Work empty when held dominates CE class | **Absorb** F1/F6 / AC1/AC3 — class dispose counts, not dominant `mechanism` |
| #188 `RetentionApplied.sample_ids` prefer overlay pins | **Absorb** F7 / AC2 — dispose ids first; no pad with pins when dispose > 0 |
| T270 F9 Work dispose-only | **Lift** F1 — still dispose-only; filter is per-class CE/PD counts |
| T270 F8 `Nothing to dispose.` = no CE/PD | **Affirm** AC4 — inventory-only freeze |
| T270 overlay / `none_auto` / pin hold | **Affirm** F9 — do not remove |
| Change `dominant_mechanism` / split `classes[]` | **Decline** F2/F3 — matrix stays majority (tie → `held`) |
| Optional class-bucket JSON extras | **Partial F4** — skip-if-zero; report keys unchanged; live inventory omits |
| Event `class_counts` split | **Decline** F8 |
| Live `retention apply --confirm` | **Decline** F16 — hermetic `prepare_retention_apply` is DoD |
| T248 F16 doctor retention / T270 F20 nightly restyle | **Decline** F17 |
| last-PR Cursor #192 | **N/A** — comments/reviews empty |
| last-PR #188 Work / apply samples | **Absorb** (source) — **no T285** |
| leftover `7d97a456` / T278–T283 | **Decline** peers |
| T277 live `backup create --no-prune` | **Decline** — T277 Completed hermetic; live skip residual |
| T240 F2 / clap 5 / rusqlite 0.40 / DTO required keys | **Decline** F19/F32 |

### T284 fold-in (2026-08-22) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 Work fallback `sample_ids` | **Already** F6; **folded test** AC17 |
| Agy m2 `audit_sample_ids` de-dupe | **Already** F7; locked by AC16 |
| Agy O1 exact 5-key serde omit | **Folded** F37 / AC5 |
| Agy O2 stale comment | **Already** F38 |
| OpenCode m1 stale comment `:629` | **Already** F38 |
| OpenCode m2 no `Default`; ~6 literals | **Already** F28 |
| OpenCode O1 helper unit no event-log | **Folded** F41 / AC16 |
| last-PR #192 Cursor | **Affirm N/A** — no T285 |
| No B/M | Nothing to decline of B/M |

### T284 closeout residuals (2026-08-22) — Work dispose counts + apply samples

| Residual | Disposition |
|----------|-------------|
| PATH `ai-brains` until `cargo install` / `Build-AIBrains.ps1` | F15 — operator; tests use source |
| Live vault still 0 CE / 0 projection | Honest; mixed hole is hermetic |
| Event `class_counts` still dominant | F8 |
| Two Work rows sharing samples if CE+PD mix | F29 |
| Nightly `candidates=` includes held | T270 F20 |
| Doctor retention check | T248 F16 |
| Live `retention apply --confirm` | F16 — not run |
| `class_dispose_count` is `pub` (cross-crate) | F27 amended; `audit_sample_ids` stays `pub(crate)` |

### T278 planning absorption (2026-08-22) — session PREVIEW; density honesty frozen

| Item | Disposition |
|------|-------------|
| Audit neighbors PREVIEW blank on session `RECALLS` | **Absorb** F1–F4 / AC1–AC3 — T246 F10 lift (`kind == "session"`) |
| Audit sparse E/N ~0.11 (live **0.130**) | **Partial** — honesty already T213/T232 (AC8/AC9); do **not** retune floors or live-rebuild |
| T246 F10 memory-only PREVIEW | **Lift** F1 — session added; memory unchanged |
| T246 F18 projector completeness | **Partial** — pin `RECALLS` is T262; session **caption** this track; projector edges **decline F11** |
| T213 floor flags / contracts / rusqlite `table_exists` / two-tier coverage | **Decline** F7/F12 |
| Auto rebuild / projector more edges / Cargo default-on / WCC | **Decline** F8/F10/F11 |
| 2-hop pretty / hierarchy captions / mermaid (T246 F17) | **Decline** F18/F19 |
| last-PR Cursor #193 | **N/A** — comments/reviews empty |
| last-PR #188 Work / apply samples | **Decline** — **T284 Completed** `#193` |
| Dependabot `#61` rusqlite 0.40.2 | **Decline** F12 — **no T285** |
| leftover `7d97a456` / T279–T283 | **Decline** peers |
| T240 F2 / clap 5 / rusqlite 0.40 / DTO required keys | **Decline** F12/F22 |
| Identity mismatch quiet | **Not this track** — T258 adopt-path; leftover data T276; shell leftover T282 |

### T278 fold-in (2026-08-22) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 fail-open no `?` on session-arm I/O | **Already** F4; **folded** F33 / AC5 |
| Agy m2 skip empty first memory preview | **Already** F3; **folded test** AC14 (F34 pure helper) |
| Agy O1 UTF-8 80-cap | **Already** F2; **folded** AC1 CJK case |
| Agy O2 pure units `(0,"")` / `(1,"preview")` | **Already** F14 / AC1; **tightened** AC1 |
| OpenCode m1 density `:14–16` vs crate-root `:10–16` | **Folded** §2.3 |
| OpenCode m2 / O1 AC14 I/O stub hedge | **Folded** F34 `pick_first_nonempty` required-pure |
| OpenCode m3 HEAD/pinned snapshot drift | **Folded** §2.1 volatile |
| OpenCode O2 empty-first hermetic AC3 | **Decline as DoD** — AC14 is the skip-loop lock |
| last-PR #193 Cursor | **Affirm N/A** — no T285 |
| No B/M | Nothing to decline of B/M |

### T278 closeout residuals (2026-08-22) — session PREVIEW captions; density honesty frozen

| Residual | Disposition |
|----------|-------------|
| PATH `ai-brains` until `cargo install --features graph` / `Build-AIBrains.ps1` | F15 — operator; tests use `cargo run --features graph` |
| Live vault still sparse E/N ~0.13 | Honest; rebuild Stop-Before (F8) |
| `SessionSummaryCreated` nodes without edges | F11 / T213 projector class |
| N+1 `get_session_memories` on huge sessions | §5; COUNT+LIMIT 1 later |
| Other-kind captions (`decision` title, `project` name) | F1 v1 session+memory only |
| Hermetic AC3 empty-first memory | OpenCode O2 declined as DoD; AC14 covers skip-loop |
| Hierarchy pretty still id-only | F19 |
| `pretty_no_memory_node` still `graph update` | F32 / T262 leftover wrong-kind |
| T279–T283 peers | F22 |
| AC5 fail-open for `get_session_memories` SQL (vs `memory_preview`) | Low — same match arm; executable lock is DROP COLUMN content |

### T279 planning absorption (2026-08-22) — leading-line GLOB + live hotspots; no live pin

| Item | Disposition |
|------|-------------|
| Audit Safety = review-track Objective / ≠ `safety sync --dry-run` paths | **Absorb** F1–F3 / AC3–AC4 / AC10 |
| T274 F23 Safety SQL leftover | **Absorb** F1 — LIKE-anywhere → leading-line GLOB |
| T274 closeout AC6 buried CONSTRAINT would Safety-steal | **Absorb** AC3 — dump must not appear in Safety |
| T250 F12 HOTSPOT float reformat | **Partial** F15 — live `score={:.2}` only |
| T272 skip-set / T264 caps | **Affirm freeze** F5/F6 |
| T272 F18 session HOTSPOT skip | **Decline** F32 |
| `query_ledgerful` Intelligence empty | **Decline** F11 |
| T280 deny hint `--scope` | **Decline → T280** |
| T281 nightly 750 ms / T282 `--show` / T283 list | **Decline** peers |
| leftover `7d97a456` 11 roots | **Decline** — T276 Completed; live rebind owner-confirm |
| last-PR Cursor #194 | **N/A** — comments/reviews empty |
| last-PR #188 Work / apply samples | **Decline** — **T284 Completed** `#193` |
| Dependabot `#61` rusqlite 0.40.2 | **Decline** F12 — **no T285** |
| T240 F2 / clap 5 / rusqlite 0.40 / DTO required keys | **Decline** F12/F17 |
| Live `safety sync` pin | **Decline** F21 |
| Identity mismatch quiet | **Not this track** — T258 adopt-path; leftover data T276; shell leftover T282 |

### T279 fold-in (2026-08-22) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 first-line `[` JSON finder | **Already** F35/AC9; **folded** F36 / AC9 (`safety.rs` `:116–118`) |
| Agy m2 always `trim_to_word_budget_no_sentinel` | **Folded** F37 |
| Agy O1 `SAFETY_EMPTY` no `HOTSPOT:` | **Already** F3 / AC14 |
| Agy O2 pure parse/format units | **Already** AC2 / AC9 |
| OpenCode m1 AC6 test names `summary_smoke` + `summary_compact` | **Folded** AC6 |
| OpenCode m2 `displayScore` 3.944 vs 3.934949 | **Folded** §2.1 volatile; F2 raw `score={:.2}` |
| OpenCode O1 `--global` empty wording | **Decline** — one `SAFETY_EMPTY` |
| last-PR #194 Cursor | **Affirm N/A** — no T285 |
| No B/M | Nothing to decline of B/M |

### T279 closeout residuals (2026-08-22) — Safety GLOB + live hotspots + honest empty

| Residual | Disposition |
|----------|-------------|
| PATH `ai-brains` until `cargo install` | F20 — tests/manual use `cargo run` |
| Intelligence + Safety path dup if bridge later fills | F11 |
| CLI `safety.rs` vs retrieval JSON parse drift | F29 |
| Unbounded `ledgerful hotspots` wait | F35 |
| Session `HOTSPOT:` skip | F32 / T272 F18 |
| Safety SQL still no `is_injectable_privacy` | CX1 P0-1 **out of scope** — pre-existing; Index already filters |
| `--global` empty remediator is cwd dry-run | OpenCode O1 declined |
| Live leftover 11 roots | T276 F9 |
| Live 0 of 3 grants | T275 F10 |
| T280–T283 peers | F17 |
| `agy-review.md` trailing whitespace lines 3–6 | CX2 P3 — plan-review artifact; do not edit `*-review.md` |

### T280 planning absorption (2026-08-22) — deny HINT omit-`--scope`; show SHORT already omit

| Item | Disposition |
|------|-------------|
| Audit deny/`policy show` `--scope …` vs doctor omit | **Absorb** F1–F4 / AC1–AC7 / AC10 — show JSON `next_step` already SHORT (affirm); HINT + briefing markdown next are DoD |
| T275 F11 / closeout HINT still `--scope …` | **Absorb** F1 / F2 |
| T241 F14 markdown T227 leftover | **Absorb** F2 — `BRIEFING_DENIED_NEXT_STEP` = SHORT |
| T243 AC12 wording freeze | **Lift** F1 / F27 — new freeze string |
| T210 AC7 substring-only | **Tighten** AC5 |
| T226 O1 shared resolve wrapper | **Decline** F5 |
| clap after_help dual examples | **Decline** F6 — already both forms |
| Runtime two-arm HINT | **Decline** F4 — no-context is fail_usage (T210 AC8) |
| last-PR Cursor #195 | **N/A** — comments/reviews empty |
| last-PR #188 Work / apply samples | **Decline** — **T284 Completed** `#193` |
| Dependabot `#61` rusqlite 0.40.2 | **Decline** F12 — **no T285** |
| T281 nightly 750 ms / T282 `--show` / T283 list | **Decline** peers |
| leftover `7d97a456` 11 roots | **Decline** — T276 Completed; live rebind owner-confirm |
| T240 F2 / clap 5 / rusqlite 0.40 / DTO required keys | **Decline** F12/F17 |
| Live operator bootstrap | **F10** — hermetic is DoD |
| Identity mismatch quiet | **Not this track** — T258 adopt-path; leftover data T276; shell leftover T282 |

### T280 fold-in (2026-08-22) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 dual-site `assert_eq!` / CP function-local const | **Already** AC1–AC3; **folded** F33 hoist + exact equality |
| Agy m2 markdown next vs grant-wall vs Decisions | **Already** AC4 next-before-Decisions; **folded** AC4 full order |
| Agy O1 `CLI-EXIT-CODES.md` `:94` | **Already** F19 / AC11 |
| Agy O2 hermetic `!contains("--scope …")` | **Already** AC5 |
| OpenCode O1 T210 AC8 `:546` vs `:548` | **Folded** §2.3 fn `:548` (comment `:546`) |
| OpenCode F1 length 172 vs ~183 | **Folded** AC1 / §8 / §11 |
| last-PR #195 Cursor | **Affirm N/A** — no T285 |
| No B/M | Nothing to decline of B/M |

### T280 closeout residuals (2026-08-22) — deny HINT omit-`--scope`; markdown next = SHORT

| Residual | Disposition |
|----------|-------------|
| PATH `ai-brains` until `cargo install` | F13 — tests/manual use `cargo run` |
| Three-copy HINT (no shared crate) | F24 |
| T226 O1 shared resolve wrapper | F5 |
| clap after_help still shows `--scope Repository:<uuid>` first | F6 / F26 — valid CI form |
| HINT **172** chars (no 140 cap) | F1 |
| PATH briefing `_None_` | T275 F18 PATH-behind |
| Live 0 of 3 grants | T275 F10 |
| Live leftover 11 roots | T276 F9 |
| T281–T283 peers | F17 |
| Series README Planned at CX1 | P3-3 **verified_fixed** at closeout |

### T281 planning absorption (2026-08-22) — timeout next-line HTTP vs TCP; no 750 raise

| Item | Disposition |
|------|-------------|
| Audit nightly Completion timeout vs daemon Open (750 ms not raised) | **Absorb** F1–F5 / AC1–AC2 / AC7 / AC10 — after_help already T269; status-block next line is DoD |
| T269 closeout two-truths on `--status` | **Absorb** F1 — gate on raw `== "timeout"`; do **not** print when `ok` |
| T269 F21 JSON budget field | **Decline** F3 |
| T255 F18 raise 750 / doctor 16th / persist / wrapper | **Decline** F2 / F11 |
| Unify daemon TCP with HTTP `/health` | **Decline** F10 |
| TCP-probe from nightly to print Open | **Decline** F1 / F27 |
| Embedding-only timeout contrast | **Decline as DoD** F26 |
| last-PR Cursor #196 | **N/A** — comments/reviews empty |
| last-PR #188 Work / apply samples | **Decline** — **T284 Completed** `#193` |
| Dependabot `#61` rusqlite 0.40.2 | **Decline** F12 — **no T285** |
| T282 `--show` / T283 list | **Decline** peers |
| leftover `7d97a456` 11 roots | **Decline** — T276 Completed; live rebind owner-confirm |
| T240 F2 / clap 5 / rusqlite 0.40 / DTO required keys | **Decline** F12/F17 |
| Identity mismatch quiet | **Not this track** — T258 adopt-path; leftover data T276; shell leftover T282 |
| Live schtasks mutate / force llama load | **F16** — hermetic + pass-with-observed-data is DoD |

### T281 fold-in (2026-08-22) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode M-1 AC2 omit `"timeout (750ms)"` | **Folded** F32 / AC2 extra `#[case]` |
| Agy m1 U+2260 vs ASCII `!=` | **Already** F25; **folded** AC1 `assert_ne!` |
| Agy m2 raw `completion_label` | **Already** §5.2; **folded** F32 |
| OpenCode m-1 `scripts/dev-check.ps1` | **Folded** plan Phase 4 |
| OpenCode m-2 AC10/AC8 vs AC7 comments | **Partial** — additive AC7; do not renumber T255/T269 |
| Agy O1 docs / O2 rstest | **Already** F19 / AC2 |
| OpenCode O-3 skill `--status` missing | **Folded** F19 no-op |
| OpenCode O-5 HEAD ahead of origin/main | **Folded** Phase 0 fetch/reconcile |
| OpenCode invented F1 / nightly-is-TCP / strip T269 suffix | **Decline** §13 |
| last-PR #196 Cursor | **Affirm N/A** — no T285 |
| No B from either harness | Nothing to decline of B |

### T281 closeout residuals (2026-08-22)

| Residual | Disposition |
|----------|-------------|
| PATH `ai-brains` until `cargo install` | F13 |
| Embedding-only timeout has no F1 | F26 |
| JSON has no budget / contrast field | F3 |
| ledgerful doctor `:8081` chat ≠ nightly `/health` | F11 — not doctor 16th |
| Daemon Stopped + port Open | F27 — F1 does not say Open |
| Live leftover 11 roots / 0 of 3 grants | T276 F9 / T275 F10 |
| CX1 P2-1 call-site helper | **verified_fixed** — `completion_status_human_lines(raw)` |

### T282 planning absorption (2026-08-22) — leftover `--show` line + KEY redact; no T240 F2

| Item | Disposition |
|------|-------------|
| Audit `context --show` misses leftover shell vs `.env` (whoami has it) | **Absorb** F1–F4 / AC1–AC5 / AC10 — stdout leftover after `Repository:` iff captured shell ≠ file `PROJECT_ID` |
| Placeholder no `AI_BRAINS_KEY` / `x'` on `--show` | **Absorb** F3 / AC3 / AC6 — file KEY/VAULT_KEY → `(redacted)`; clap help stays T256 |
| T276 F10/F11 / closeout shell leftover | **Absorb** (this track) |
| T242 session-quiet hides override warn | **Partial** — motivation; do **not** restyle T242 (F6) |
| T206 L3 / CHANGELOG `context --show` mismatch warn | **Decline** F10 — T240 stderr; cwd `mismatch: false` |
| T256 `--help` hide_env_values | **Decline** F7 — help-only; file dump is this track |
| `--format json` / vault-free `--show` / SESSION leftover | **Decline** F4 / F11 / F27 |
| last-PR Cursor #197 | **N/A** — comments/reviews empty |
| last-PR #188 Work / apply samples | **Decline** — **T284 Completed** `#193` |
| Dependabot `#61` rusqlite 0.40.2 | **Decline** F12 — **no T285** |
| T283 list cwd-first | **Decline** peer placeholder |
| leftover `7d97a456` 11 roots | **Decline** — T276 Completed; live rebind owner-confirm |
| T240 F2 / clap 5 / rusqlite 0.40 / DTO required keys | **Decline** F2/F12/F17 |
| Identity mismatch quiet | **Not this track** — T258 adopt-path; leftover data T276; shell leftover **this track** |
| Live `.env` write / adopt-path `--write-env` | **F16** — hermetic no-write is DoD |

### T282 fold-in (2026-08-22) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 file PROJECT_ID strip+trim | **Already** F1/F25; **folded** F33 / AC2 padded value |
| Agy m2 exact `KEY=` prefix | **Already** F3; **folded** F34 / AC3 `KEYRING=` passthrough |
| Agy O1 `assert_no_secret_leakage` | **Already** AC6 |
| Agy O2 helper units | **Already** AC1–AC3 |
| OpenCode m-1 `.claude` skill already `--show` | **Folded** F19 / AC11 — no new section; one sentence on `:50`/`:57`/`:88` |
| OpenCode m-2 drop `VAULT_KEY` redact | **Decline** — live daemon/elevation; **folded** F36 rustdoc |
| OpenCode m-3 AC4 leftover once | **Folded** AC4 `count() == 1` |
| OpenCode O-1 `isolate_empty_home` | **Folded** AC4 must |
| OpenCode O-2 capture comment | **Folded** F35 |
| OpenCode O-3 SESSION/model passthrough | **Already** F3 / §5.4 |
| last-PR #197 Cursor | **Affirm N/A** — no T285 |
| No B/M | Nothing to decline of B/M (m-2 drop declined as false) |

### T282 closeout residuals (2026-08-22) — leftover `--show` line + KEY redact

| Residual | Disposition |
|----------|-------------|
| PATH until `cargo install` | F13 |
| Vault-free `--show` | F11 — decline as DoD |
| SESSION leftover line | F27 |
| No-`.env` leftover naming (without lie-suffix) | F26 |
| Quote-strip file PROJECT_ID | F32 |
| T283 list cwd-first | **T283 Completed** 2026-08-22 |
| Live leftover 11 `C:\dev\*` roots | T276 F9 — owner-confirm rebind |
| Live 0 of 3 grants | T275 F10 |
| T242 first-run stderr still possible | F6 — leftover is stdout |
| CX1 P1-1 process (gate/closeout) | **verified_fixed** — `dev-check` 3333 + `verify --scope full`; closeout this commit |

### T283 planning absorption (2026-08-22) — human cwd-first; JSON size-desc frozen

| Item | Disposition |
|------|-------------|
| Audit `project list` leftover-first (7/6) | **Absorb** F1–F8 / AC1–AC6 / AC10 — human table promotes cwd path-owner; remaining rows stay memory-desc |
| Placeholder JSON freeze vs human-only sort | **Absorb** F2 JSON freeze (T212 F13); F1 human-only permute |
| Placeholder “or `*` active” | **Partial** — star stays T212 env marker; **F10 decline star-as-sort** (leftover env would keep leftover-first) |
| T276 F10 / closeout `project list` leftover-first | **Absorb** (this track) |
| T282 closeout T283 peer | **Absorb** (this track) |
| T267 footer leftover-as-AI-Brains | **Decline** F3 — Completed; pass original store vec to footer |
| T212 labels / store `ORDER BY` / JSON keys | **Decline** F2 / F11 / F30 |
| T230 never-blank | **Decline** — labels unchanged |
| last-PR Cursor #198 | **N/A** — comments/reviews empty |
| last-PR #188 Work / apply samples | **Decline** — **T284 Completed** `#193` |
| Dependabot `#61` rusqlite 0.40.2 | **Decline** F12 — **no T285** |
| leftover `7d97a456` 11 roots | **Decline** — T276 Completed; live rebind owner-confirm |
| T240 F2 / clap 5 / rusqlite 0.40 / DTO required keys | **Decline** F4/F12/F17 |
| JSON reorder / `--sort` / star-as-sort | **Decline** F2 / F5 / F10 |
| Identity mismatch quiet | **Not this track** — T258 adopt-path; leftover data T276; list sort **this track** |
| Live `.env` write / adopt-path `--write-env` / leftover `set-alias AI-Brains` | **F16** — hermetic no-write is DoD |

### T283 fold-in (2026-08-22) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 fail-open `resolve_path_alias_for_location` | **Decline** — footer `:112` still `?`; **already** F26 for `current_dir`/git |
| Agy m2 `with_capacity` + no dup/drop | **Folded** F37 / AC1 |
| Agy O1 OPERATIONS T76 refresh | **Already** F19 |
| Agy O2 first/middle/last units | **Folded** AC1 middle; already-first + AC2 rest |
| OpenCode m-1 promoted id once + len | **Folded** AC1 |
| OpenCode m-2 `.claude` `:89` exists | **Already** F19 |
| OpenCode m-3 AC10 max-memory not leftover UUID | **Folded** AC10 |
| OpenCode m-4 AC5 re-env after denylist | **Folded** F38 / AC5 |
| OpenCode m-5 after_help “JSON order unchanged” | **Folded** F35 |
| OpenCode O-1 `lines().nth(1)` | **Folded** AC3 / AC5 / AC10 |
| OpenCode O-2 resolve comment | **Folded** F39 |
| OpenCode O-3 keep F26 store `?` | **Already** F26 |
| last-PR #198 Cursor | **Affirm N/A** — no T285 |
| No B/M | Nothing to decline of B/M (Agy m1 store-resolve declined as false-complete) |

### T283 closeout residuals (2026-08-22) — human cwd-first; JSON size-desc frozen

| Residual | Disposition |
|----------|-------------|
| PATH until `cargo install` | F13 |
| JSON `cwd_first` marker | F32 — decline as DoD |
| Star-only fallback when no path-owner | F10 |
| `--sort` flag | F5 |
| Duplicate git/path probe vs footer | F9 — T267 signature freeze |
| Live leftover 11 `C:\dev\*` roots | T276 F9 — owner-confirm rebind |
| Live 0 of 3 grants | T275 F10 |
| CX1 P1-1 process (gate/closeout) | **verified_fixed** — `dev-check` 3341 + `verify --scope full`; closeout this commit |

### T274 closeout residuals (2026-08-21)

| Residual | Disposition |
|----------|-------------|
| Detector tests not `rstest #[case]` (F27 style) | **Defer** low — cases covered |
| AC6 dump buried `decision:` not `CONSTRAINT:` | **Defer** low — CONSTRAINT would Safety-steal (T279) |
| SQL GLOB misses lowercase `decision:` | **F8 documented** — detector/prefer-fill SoT |
| AC16 unit is helper, not `semantic_search_with_embedding` | **Defer** low — spec: no HTTP |
| PATH `ai-brains` until `cargo install` | F21 |
| Live vault still dumps until install | F21 |
| Restore drill needs daemon Stopped | Unrelated T188; not T274 |

### T270 closeout residuals (2026-08-21)

Specified softs — not product blockers:

| Residual | Disposition |
|----------|-------------|
| Nightly one-liner `candidates=` includes held | F20 — not restyled |
| `active` / unknown statuses lumped in `other` skip | Intentional v1 |
| `list_pinned_memory_ids` still loads all ids for R11 | Pre-existing; not this overlay |
| JSON still omits zero buckets for the other eight classes | T248 F5 |
| PATH `ai-brains` until `cargo install` | F16 — installed 2026-08-21 |
| Leftover project 18k pins still owned by `7d97a456` | **T276** |
| Doctor retention check | T248 F16 |
| rusqlite `table_exists` 0.40 | T213 L4 |
| last-PR Cursor #188 Work table / apply samples | **T284 Completed** 2026-08-22 |

## Post-P12 backlog promotion (2026-08-01)

Placeholder tracks registered in `conductor/conductor.md` (status **Pending**). Residual detail below remains until each track closes:

| Residual | Promoted track |
|----------|----------------|
| ~~§59 #8 wrong-key / K-06 needs page encrypt; R-F8 / R-K06; Deviations §1~~ | **Closed by T187** (2026-08-02) |
| ~~§59 #1 recovery export; #6 restore daemon hard-fail~~ (R-DOC-CLI partial: export shipped; doctor remains) | **Closed by T188** (2026-08-02); **#2 doctor** → **T192** |
| ~~**#34.2** DataKey rotation~~ | **Closed by T189** (2026-08-02) PR #67 `9e9465e` |
| ~~**#12** path TOCTOU / openat / cap-std~~ | **Closed-with-residuals by T190** (2026-08-02). Residual elevation → **T193** |
| ~~T142 #1–2 ChangeGuard renames + source_tag; T186 L13 hermetic long-tail~~ | **Closed by T191** (2026-08-02) |
| ~~**#2** doctor CLI / R-DOC-CLI~~ | **Closed by T192** (2026-08-02) PR #75 `80837da` |
| ~~T190 ambient CLI / write / token path residuals~~ | **Closed-with-residuals by T193** (2026-08-02) PR #77 `2183127` — P0 write SOOT elevated; soft-canon / parent mkdir / ambient CLI long-tail remain honesty residuals |
| ~~Argon2 params in kit JSON (F37)~~ | **Closed by T194** (2026-08-02) PR #76 `2c06464` |
| ~~R-PIPE-IU / R-UDS-TMP / R-HTTP-SYS / R-MULTI~~ | **Closed-with-residuals by T195** (2026-08-02) PR #78 `bd375a8` — opt-in pipe ACL, XDG UDS, service HTTP refuse, ADR-0022 fence; residuals remain honesty (IU default multi-interactive, `/tmp` fallback, service HTTP when opted in) |
| ~~systemd / launchd units; CONTRIBUTING hygiene~~ | **Closed by T196** (2026-08-02) PR #79 `3f16648` — reference units + CONTRIBUTING; not product multi-OS installers |
| R-CI-BRANCH (repo admin) | **Not a code track** — admin action only |
| MSI / notarization / App Store packaging | Remains packaging residual (not T196) |
| ~~Common Changelog conversion~~ | **Declined by T196** (2026-08-02) Keep a Changelog retained; documented in CONTRIBUTING + CHANGELOG note |
| ~~CLI vault-open SQLCipher spam + key bootstrap~~ | **Closed by T197** (2026-08-03) PR #80 `72dfa62` — no silent zero on CLI 7 sites; hmac spam filtered; doctor skip vs fail; init generate+print |
| ~~CLI empty states / silent fails / graph exit 0~~ | **Closed by T198** (2026-08-03) PR #81 `5cc0418` — empty success non-blank; dogfood fail_api; graph exit 0→2 FEATURE_UNAVAILABLE |
| ~~`daemon status` requires vault key~~ | **Closed by T199** (2026-08-03) PR #82 `721d41f` — early-route + shared probe; no key required |
| ~~Graph default install / feature honesty~~ | **Closed by T200** (2026-08-03) PR #83 `84f4a23` — docs-only A + INSTALL/Release honesty + F14 CI graph-on filter; residual Cozo INFO stdout pre-existing |
| ~~CLI exit-code + error envelope contract~~ | **Closed by T201** (2026-08-03) PR #84 `a9e3b85` — clap-required scope exit 2; details.hint; dual envelope docs; exit_contract suite |
| ~~Recall/briefing/query progressive clarity~~ | **Closed by T202** (2026-08-04) PR #85 `89ea3ec` — embedding.status; empty_denied seed; TTY briefing md; progressive exit 2; soft-resolve remains **T203** |
| ~~Governed source/evidence discovery lists + soft-resolve~~ | **Closed by T203** (2026-08-04) PR #86 `2748d12` — source/evidence list; review soft-resolve exit 2; show F7; Active+LIMIT+1; core FTS sanitizer |
| ~~CLI help grouping IA~~ | **Closed by T204** (2026-08-04) PR #87 `c3a7d66` — after_long_help groups; F31 order; F33 dangerous; F9 project-id; CAPABILITIES format table |
| T196 P3 SIGTERM child delivery test | Soft residual (F36); not blocking |
| Non-destructive skill/CLI audit follow-ups (2026-08-04) | **T205-T216** - ~~T205–T216 closed~~; ~~T216 forget-list + inventory skim closed~~ (PR #99 `1980d83`) |
| ~~source/evidence/review/briefing POLICY_DENIED bootstrap (audit 3–4)~~ | **Closed by T210** (2026-08-05) PR #93 `d52df25` — `policy bootstrap` discovery Read* LocalOnly; active_grants + get_principal; dual-site hint AC7/AC11; hermetic suite |
| T210 residuals (skill / soft-resolve success / full admin) | Soft: F23 skill one-liner; AC8 success soft-resolve hermetic (fail path locked); full grant admin/revoke/daemon IssueGrant (F24–F26) |
| ~~sync query ranking + stale DECISIONs (audit quality 5)~~ | **Closed by T211** (2026-08-05) PR #94 `16990b1` — `rerank_hits` pin+recency; plan demotion + badge; ledger-first; `--limit` 5; BM25 base=-rank |
| T211 residuals (F25 blend / double shell / T215) | Soft: full vault↔ledger RRF blend (F25); optional single ledger shell call; ~~semantic/RRF → T215~~ **closed by T215** (RRF vault FTS+semantic + ScoreKind polarity; F25 ledger blend remains soft here) |
| ~~project list UUID-only / set-alias UX (audit quality 7)~~ | **Closed by T212** (2026-08-05) PR #95 `09e34ba` — label-first; last_activity; path subquery; stderr set-alias footer; `--format json`; char-safe truncate; no auto-alias |
| T212 residuals (AC10 path seed / F24 verbose) | Soft: hermetic path_alias seed (AC10); `--verbose` raw registered name (F24); detect --json remains T206 soft |
| ~~semantic recall topic drift + bridge polarity (audit 6/5)~~ | **Closed by T215** (2026-08-05) PR #96 `b5cdc98` — RRF hybrid; floor 0.55; ScoreKind M1; F14 pipeline; F11 honesty; AC1–17 |
| T215 residuals (e2e / soft F24–F29 / ANN) | ~~hermetic e2e / F25 fusion / score display → **T218 closed** PR #116 `fc4d370`~~; Soft remain: F24 always-on ok pretty; F29 skill one-liner; weighted RRF; ANN (F27); adaptive threshold declined |
| ~~Graph-on Cozo INFO pollutes recall/sync (T200 residual)~~ | **Closed by T208** (2026-08-04) PR #91 `9985ab4` — F2 demote; F8 `ai_brains_graph=warn`; F29 RUST_LOG denylist; AC1 env_remove |
| T118 smoke `RUST_LOG=""` tests ERROR-only not product default (M4) | Soft residual from T208 fold-in — optional later hygiene, not T208 DoD |
| T208 soft residuals (F10) | Soft: lazy GraphAwareEventStore construct (not DoD) |
| ~~Backup list WARN flood post-encrypt (audit 4/3)~~ | **Closed by T209** (2026-08-04) PR #92 `02a0d7d` — header-first classify; F31 size≥512; ListMode; tokens; default quiet + eprintln summary |
| T209 residuals (L3/L4) | Soft: real wrong-key SQLCipher fixture for AC9; dedicated PreT109 unit (not DoD) |
| ~~project detect test-alias / .env hijack honesty~~ | **Closed by T206** (2026-08-04) PR #89 `d727fc5` — remote-first slug; exact-first; ambiguous exit 1; env mismatch warn |
| ~~Global dotenv KEY skipped when VAULT_PATH set~~ | **Closed by T205** (2026-08-04) PR #88 `6a7fd15` — always-merge global dotenv; F11 empty-home hermetic |
| T206 soft residuals (F8/F10/F24) | Soft: no `--json` source; no `context --show` mismatch; resolve exact-first reuse |
| ~~Recall empty pretty blank + scope opacity (audit FTS 3/3)~~ | **Closed by T207** (2026-08-04) PR #90 `95b516a` — F3 always-on empty pretty hint; F4 empty Scope + F32 `get_project_by_id`; F5 omit generated Session; F6 no name dupe |
| T207 residuals (AC10 / soft L2) | Soft: AC10 non-empty pretty Scope (M3); soft L2 combined count+name query if not shipped |

Suggested order: ~~**T196**~~ ... ~~**T216**~~ **closed** (T205–T216 audit series complete). **Next series T217–T232** (post-audit CLI quality): ~~**T217**~~ **closed** PR #110 `1e22e77`; ~~**T220**~~ **closed** PR #112 `6f4f67b`; ~~**T221**~~ **closed** PR #114 `b3c4b0f`; ~~**T218**~~ **closed** PR #116 `fc4d370`; ~~**T219**~~ **closed** PR #118 `496ddd7`; ~~**T224**~~ **closed** PR #120 `a18fae6`; ~~**T222**~~ **closed** PR #122 `c1ac594`; ~~**T232**~~ **closed** PR #124 `33b28d0`; ~~**T223**~~ **closed** PR #126 `7ff8f7f`; ~~**T225**~~ **closed** PR #128 `927b8db`; ~~**T226**~~ **closed** PR #130 `5919f26`; ~~**T227**~~ **closed** PR #132 `40c7cd1`; remaining placeholders (non-empty Scope, nightly+router ops, global labels, unified search). See [README-T217-T232-CLI-QUALITY.md](tracks/README-T217-T232-CLI-QUALITY.md). Packaging residual: MSI / notarization / App Store + R-CI-BRANCH. Residual honesty: daemon `AI_BRAINS_VAULT_KEY` silent zero **closed 2026-08-16** — VAULT_KEY then KEY, refuse zero; `daemon.env` values double-quoted + vault path `/` so dotenvy does not eat `x'…'` or `\a` in `\ai-brains`.

### T213 closeout residuals (2026-08-05) — density doctor shipped

| Residual | Disposition |
|----------|-------------|
| ~~graph update effect 6 / false live~~ | **Closed by T213** — pure assessor + status `live`\|`sparse`\|`empty` + doctor `graph_density` |
| Event↔graph timestamp freshness (F31 / audit2 freshness half) | Soft residual — not DoD |
| CLI flags for density thresholds (F17) | Soft — env overrides only (`AI_BRAINS_GRAPH_MIN_*`) |
| Promote `GraphHealthOutput` to `ai-brains-contracts` (F24) | Soft — keep full field names if promoted later |
| Skill one-liner for density / rebuild | Soft — **T232 soft absorb** (skill + OPERATIONS) |
| rusqlite 0.40+ `table_exists` for F5 probe (L4) | Soft residual (no bump in T213) |
| Two-tier memory coverage 0.50 soft + 0.10 severe (L6) | Soft declined v1 (0.10 severe floor only) |
| Auto rebuild / projector more edges / graph default-on / WCC | **Not** T213 — separate product decisions |

### T214 closeout residuals (2026-08-05) — preflight global rollup shipped

| Residual | Disposition |
|----------|-------------|
| ~~preflight `--global --summary` false Project uuid (audit 6/6)~~ | **Closed by T214** — F2 Scope + F3 dispatch + dual counts |
| ~~Active Sessions always 0~~ | **Closed by T214** — F5 `count_active_sessions` QueryStore |
| ~~Marker counts as vault totals~~ | **Closed by T214** — F4 **In context** labels |
| ~~Non-summary pretty Scope header on full preflight body~~ | **Closed by T219** PR #118 `496ddd7` (F6 hard) |
| ~~`preflight --summary --format json` machine object~~ | **Closed by T220** PR #112 `6f4f67b` (pretty envelope; scope none; install-hooks stderr) |
| Ledgerful under `--global` | Soft residual F9 — product decision |
| Governed multi-project packet | **Not** T214 — F10 |
| `PreflightContextResponse` extra keys | **Not** T214 — F11 / T180 freeze |
| is-terminal → `std::io::IsTerminal` | **Closed 2026-08-16** |
| Extract `commands/scope_display.rs` | Soft residual F13 v1 used pub(crate) |
| Refactor retrieval `active_sessions` off `format!` SQL | Soft residual (pre-existing; not T214 DoD) |
| ~~T216 forget-list~~ | **Closed by T216** (2026-08-05) PR #99 `1980d83` |

### T216 closeout residuals (2026-08-05) — forget-list + inventory skim shipped

| Residual | Disposition |
|----------|-------------|
| ~~forget list effect 5 (unbounded list-forgotten; no inventory skim)~~ | **Closed by T216** — `memory list` + bounded `forget --list-forgotten` |
| ~~Counts by project~~ | **Closed** F11 `--summary` (+ global by-project; F46 tag cells) |
| Counts by tag histogram | Soft F24 — `--tag` filter shipped; Top-N histogram not DoD |
| Tag schema / pin rewrite | **Not** T216 |
| Auto-forget / CE wipe / governed discovery / HTTP list | **Not** T216 |
| `--offset` / cursor pagination | Soft F24 |
| Shared relative-time helper extract | Soft F24 |
| Tag matcher CLI/store dual (R1-06) | Soft residual — keep in sync if either changes |
| ~~AI1 M1–M7 / L1–L6/L8 / F46~~ | **Closed** in T216 ship |

### T220 closeout residuals (2026-08-09) — summary JSON honesty shipped

| Residual | Disposition |
|----------|-------------|
| ~~summary `--format json` human banner flag lie~~ | **Closed by T220** PR #112 `6f4f67b` |
| Soft skill one-liner for summary JSON | Soft residual F20/F22 |
| Optional `harnesses[]` in summary JSON | Soft residual F22 |
| Optional `scope_line` string | Soft residual F22 |
| clap ValueEnum ignore_case unify | Soft residual F22. **Closed 2026-08-16:** is-terminal → std |
| ~~T219 pretty body wall~~ | **Closed by T219** PR #118 `496ddd7` |

### Post-audit CLI quality placeholders (2026-08-05) — T217–T232

| Residual / finding | Disposition |
|--------------------|-------------|
| FTS natural-phrase empty (quality 4) | **T217** ✅ closed PR #110 `1e22e77` |
| ~~Semantic drift / scores (quality 4)~~ | **Closed by T218** PR #116 `fc4d370` — dual floor 0.55/0.60 no-FTS gate; score_kind; pretty rank+sim; hermetic fuse SOOT; soft residual F18/F19/F20/F21/AC15/httpmock-full-recall |
| ~~Preflight pretty wall (quality 5)~~ | **Closed by T219** PR #118 `496ddd7` — newline budget + Scope + role strip + section caps; soft residual `--compact` / retrieval JSON strip; ~~T228~~ **closed PR #134** |
| ~~preflight summary `--format json` lie (quality 3)~~ | **Closed by T220** PR #112 `6f4f67b` |
| ~~Governed usefulness 4–5; progressive deny exit 0~~ | **Closed by T221** PR #114 `b3c4b0f` — progressive/expand deny exit 3 + `denial_hint` + human emit_error hint; soft residual F12/F32/F18/F36 |
| ~~Graph-off PATH usefulness 3~~ | **Closed by T222** PR #122 `c1ac594` — scripts graph-on + doctor `graph_feature`; A2=no; soft residual T232 density remediations |
| ~~`.env` override double-warn spam~~ | **Closed by T223** PR #126 `7ff8f7f` — one collapsed Warning line; session-only debug; `AI_BRAINS_QUIET_ENV_WARN` shell/project only; soft residual F18 clap/truthy-core/global-reorder |
| ~~ASSISTANT: in search paths~~ | **Closed by T224** PR #120 `a18fae6` — pretty + forget previews strip; JSON/events raw; soft residual truncate triplication / JSON preview field |
| ~~Backup verify noise + legacy fleet~~ | **Closed by T225** PR #128 `927b8db` — quiet summary + first 5 FAIL — + create nudge; doctor usable/stale; soft residual F17 |
| ~~policy show/check required scope~~ | **Closed by T226** PR #130 `5919f26` — soft-resolve show\|check + F23 canonicalize; soft residual O1 shared wrapper / bootstrap success soft hermetic |
| ~~Briefing human→JSON; empty personal~~ | **Closed by T227** PR #132 `40c7cd1` — aliases→md + unknown exit 2; empty honesty; AC6 substance; no pin inject; soft residual F34 OutputFormat surface-wide / #18 / typed constraints |
| ~~Non-empty pretty Scope (T207 soft)~~ | **Closed by T228** PR #134 `e51d5e4` — always-on pretty Scope empty+non-empty + sync vault; residual F32/F34 → closed by **T231** |
| ~~Nightly schedule + router :8081/:8083~~ | **Closed by T229** PR #140 `1ec9142` — status endpoints+probe+Last Result; F5 UTF-8 truncate; F13 nil project; OPERATIONS dual schedule; soft residual F8–F12/F14; multi-root **closed by T233** |
| ~~Global summary blank labels~~ | **Closed by T230** PR #136 `b3f1a61` — never-blank `display_label` empty/ws name → `(no alias)`; orphan store+unit+live; soft residual F34 whitespace alias / F11 footer / CLI orphan E2E hermetic |
| ~~Dual recall vs sync query mental model~~ | **Closed by T231** PR #138 `0f3d83f` — A+C decision table + F32/F21 resolve/ndjson honesty + F37 gated empty next-step; soft residual: search noun / recall text→pretty arm / invalid-env clap converge |
| ~~Doctor graph rebuild vs graph-off~~ | **Closed by T232** PR #124 `33b28d0` — capability remediations (on→rebuild / off→`GRAPH_REINSTALL_SOOT`); empty-lag hybrid retired |
| ~~Nightly Ledgerful bridge cwd=System32; multi-repo roots~~ | **Closed by T233** PR #142 `38cdcc2` — Option B register-path + Phase2 multi-root; 0163 symbols; soft residual list-paths/unregister-path/from-scan |
| ~~T229 multi-root bridge half~~ | **Closed by T233** PR #142 `38cdcc2`; T229 keeps router env/health/schedule |
| ~~Ledgerful scoped symbol inventory (agent DX)~~ | **Closed by coordinated 0163** (2026-08-09) Ledgerful PR #159 `3fe44367` — `ledgerful symbols` scoped JSON; T233 consumes (frozen flags in T233 plan) |

### Harness seamless ingest series (2026-08-08) — T234–T239

| Residual / finding | Disposition |
|--------------------|-------------|
| ~~Capture Privacy SOOT missing as shared module~~ | **Closed by T234** — `message_only` F1–F47 / AC1–AC16 |
| ~~antigravity extract_turns AGY-only partial~~ | **Closed by T234** F7/F12 + ProjectChat `filter_turn` |
| ~~agy-hook role-only (no tool/thinking strip)~~ | **Closed by T234** F13/AC14 + F15 sole-tool JSON |
| ~~Live AGY thinking+tool_calls / VIEW_FILE content~~ | **Closed by T234** F5–F7/AC16 |
| ~~Live Grok reasoning/tool_result/backend_tool_call + array user content~~ | **Closed by T234** fixture SOOT F8/F10/F37 (wire T237) |
| ~~UTF-8 strip panic risk (AI1 §4)~~ | **Closed by T234** F43/AC15 |
| ~~Keep contracts thinking field (AI1 §5)~~ | **Closed by T234** F17/F46 — never populate |
| ~~OpenCode export filter~~ | **Closed by T238** PR #106 `3378a02` — nested normalize + synthetic drop + wire |
| Capture refuse `thinking: Some` | Soft **F24** residual (adapters always `None`) |
| ~~Detect + preflight hook install UX~~ | **Closed by T235** PR #101 `b1a0ecc` — detect/wiring/`harness *`/AGY F34+writer/preflight/doctor; others backend_pending → T236–T238 |
| ~~AGY2 seamless + history→project binding~~ | **Closed by T236** PR #102 `d53e4be` — wrapper stdout / step parse / history bind / turn-id / `--force` / re-summarize / AC6; soft residuals below |
| ~~Grok Build hooks + chat_history batch~~ | **Closed by T237** PR #104 `459fc55` — empty Stop stdout; F11 user_query keep; grok-hook/import/install; subagent skip; dry-run; AC6 anti-hijack |
| ~~OpenCode plugin + export batch~~ | **Closed by T238** PR #106 `3378a02` — session.idle plugin; watermark import; never SQLite |
| ~~Nightly multi-harness import~~ | **Closed by T239** PR #108 `a271a99` — multi-source + status + per-source skip; SYSTEM keeps `--skip-import` (D12) |
| ~~Display ASSISTANT: strip~~ | **Closed by T224** PR #120 `a18fae6` (orthogonal display) |
| Remove contracts `thinking` field | **Not** T234 (later) |
| T236 re-list test `thread::sleep` | Soft residual (Codex deferred P3) — non-blocking timing order |
| T236 BrainLog harness id `…0001` vs live agy `…0002` | Soft residual / T239 analytics |
| T236 batch query Err→None fail-open | Soft residual pre-existing |
| fullyIdle hard continue policy | Soft residual (F7) |
| Byte-offset watermark / import `--json` | Soft residual (F34 / soft) |

Suggested harness order: **T234–T239 series complete**. Soft residual: Claude/Codex install_ready → **T253**. See [README-T234-T239-HARNESS-INGEST.md](tracks/README-T234-T239-HARNESS-INGEST.md).

### Post-install CLI effectiveness (2026-08-11 audit) — T240–T255

| Residual / finding | Disposition |
|--------------------|-------------|
| Default project identity (env test-alias vs detect vs path) | ~~**T240**~~ ✅ **Completed** 2026-08-12 PR #144 `29b9b59` — whoami + path-first detect + mismatch warn. Soft residual: F13 detect `--json`; F14 `project use` **path-owner slice → T258 Completed** (`adopt-path`); general `use <uuid>` remains soft |
| Policy grants empty → governed dead-end | ~~**T241**~~ ✅ **Completed** 2026-08-12 PR #151 `930d0ed` — doctor/preflight/show/check/briefing discoverability. Soft residual only: F20 install-grants, F21 skill one-liner, F22 soft-resolve hermetic, L1 after_help dual-site, L2 dual short-SOOT |
| Env override warn spam (T223 residual) | ~~**T242**~~ ✅ **Completed** 2026-08-12 PR #147 `9f3148b` — session fingerprint markers (cross-process). Soft residual only: F16 clap quiet, F17 elevation QUIET/FORCE, F18 truthy→core, F19 global quiet pre-read |
| Search dual model + progressive first-run | ~~**T243**~~ ✅ **Completed** 2026-08-12 PR #153 `7a19d40` — `search`→recall alias; `text`≡pretty; progressive `next_step`/deny recall honesty. Soft residual only: F23 non-empty recall footer, F24 daemon/HTTP `next_step` |
| Backup fleet 0 usable / legacy plain | ~~**T244**~~ ✅ **Completed** 2026-08-12 PR #149 `948d2ae` — Incomplete + core-table usable SOOT; list residual `not recoverable`; CLI usable-first; verify both cores; live create green path. Soft residual only: F17 verify quiet/JSON summary/structured error; F18 archive helper |
| Harness wiring=missing | ~~**T245**~~ ✅ **Completed** 2026-08-12 PR #155 `f05e2f6` — `all-ready`; AGY IDE + CLI plugin bundle (not top-level CLI hooks.json); PATH bake; S12 idle+status; doctor ready-vs-pending. Soft residual: doctor message helper-only. `pending_track` T239+ **closed by T253**. |
| Graph neighbors JSON-only | ~~**T246**~~ ✅ **Completed** 2026-08-13 PR #159 `06cdcde` — TTY pretty; frozen JSON keys; crate `*_with_depth`; F9 sort in PROTOCOL-COMPAT. Soft residual only: F17 tree/mermaid/TTY-auto update/batch `node_kinds`; F18 projector completeness; F19 T213 F31 freshness |
| Nightly status latency + Last Result 101 | ~~**T247**~~ ✅ **Completed** 2026-08-13 PR #157 `43191ff` — `--quick`; parallel 750ms; LIST/V honesty; live missing `.cmd` named. Soft residual F11–F16 → **T255 Completed** |
| Retention plan human | ~~**T248**~~ ✅ **Completed** 2026-08-14 PR #161 `c633781` — TTY `auto` human; JSON keys frozen; `memory_legacy` → `skip`; apply default JSON. Soft residual only: F16–F18 |
| Scope/daemon/doctor presentation | ~~**T249**~~ ✅ **Completed** 2026-08-14 PR #163 `5fd264a` — TTY `auto` human; JSON keys frozen; Stopped `next:`; real `--summary`. Soft residual only: F12 daemon json/uptime/sc query / is-terminal std / shared resolver / color; F13 T241 leftovers / T226 O1 / ~~T255~~ / T250 |
| Preflight pretty density (T219 residual) | ~~**T250**~~ ✅ **Completed** 2026-08-14 PR #165 `bf23f0e` — Session/Recent line-cap 140; `--compact`; JSON/summary ignore; chrome strip. Soft residual only: F12 is-terminal std / clap pin / JSON role strip / pager / governed section caps / `--max-line` / skill / HOTSPOT float / auto-compact |
| device status missing | ~~**T251**~~ ✅ **Completed** 2026-08-14 PR #167 `038098e` — first-class `status` = roster + always `next:`; plural T198 only; CLI-EXIT-CODES footnote. Soft residual only: F12 |
| ingest dry-run empty stdin | ~~**T252**~~ ✅ **Completed** 2026-08-15 — empty/TTY/`trim` stdin → `fail_usage` exit **2** + example JSON; mid-payload stays `COMMAND_FAILED` exit **1**. Soft residual only: F12 vault-free dry-run / `--schema` / `IsTerminal` migrate / shared stdin helper / `events[0]` panic / ~~T254–T255~~ |
| Claude/Codex install_ready (T239+) | ~~**T253**~~ ✅ **Completed** 2026-08-15 — writers + UPS/Stop message-only; `install_ready` true; `all-ready` five; no nightly. Soft residual only: F34 nightly sources / Codex SessionEnd / Unix wrappers / unified PS1 |
| T233 soft list-paths/unregister/from-scan | ~~**T254**~~ ✅ **Completed** 2026-08-15 — list-paths + unregister Removed + scan-roots dry-run; refuse-steal; decline T233-F44. Soft residual only: F12 |
| T229 soft F8–F12/F14 | ~~**T255**~~ ✅ **Completed** 2026-08-16 — JSON `nightly --status --format json` (default human; pipes stay human) + read-only Router Last Result. Declined: doctor 16th check, persist probe, 50ms embed sleep, product `.cmd` / schedule-Router, `--quick --no-vault`. Soft residual only: F12. |

Series README: [README-T240-T255-CLI-EFFECTIVENESS.md](tracks/README-T240-T255-CLI-EFFECTIVENESS.md). **T240–T255 Completed.** Series closer.

### T255 closeout residuals (2026-08-16)

Specified softs (F12) plus declined bag — not product blockers:

| Residual | Disposition |
|----------|-------------|
| `--quick --no-vault` | Soft F15 / T247 O12 — `--quick` still opens the vault |
| Persist probe in `sync_state` | Declined F12 — status is a query |
| Doctor 16th model-port check | Declined F11 — frozen 15-check; status is the matrix |
| 50ms embed sleep | Declined F13 — reopen only with nightly-run timings |
| Product `.cmd` / schedule Router | Declined F14 / F30 — operator confirm |
| Shared `resolve_*_format` | **Closed 2026-08-16** — `format_resolve::resolve_human_json_format`; graph stays local (`pretty`) |
| `std::io::IsTerminal` migrate | **Closed 2026-08-16** — `is-terminal` crate removed from CLI |
| PATH reinstall | Operator / F29 — PATH `ai-brains` still pre-T247 |
| Live reschedule of missing `.cmd` | F30 — not automatic Close |
| T253 Claude/Codex nightly | Not absorbed F16 |

### Conductor planning skills (2026-08-16)

Project copies of Helping Hands `plan` / `review-track` / `foldin`, adapted for this repo:

| Skill | Path |
|-------|------|
| **plan-track** | `.agents/skills/plan-track/SKILL.md` |
| **review-track** | `.agents/skills/review-track/SKILL.md` |
| **fold-in** | `.agents/skills/fold-in/SKILL.md` |
| **implement-track** | `.agents/skills/implement-track/SKILL.md` — from `hands/.agents/skills/implement`. Same-repo (no `hands\` split); TDD; `deferred.md`; `/implement-track` **always** publishes: push branch → PR → **watch GHA `CI` until every job is green** → squash-merge → prune. Never `git push origin main` / force-push. Stop if spec is still a Placeholder. |

Adaptations (not a copy): same-repo product+conductor (no `hands\` split); `trackTNN-<kebab>`; `deferred.md` not ISSUES.md; plan reviews are `*-review.md` so they do not collide with post-implement `review.md` / `review.codex.md`; F0 plan-only until go; no full-gate plan review. Standing orders (2026-08-16 tighten): live `src/` baseline; ledgerful + ai-brains required as appropriate; knowledge stale — current pins/docs + online best-practice/implementation research (N/A must be written). Plan pass must scan **entire** `deferred.md` and the **last merged PR** (+ open PR on HEAD) for Cursor/Bugbot comments; absorb, point at an existing Pending placeholder, or **mint a new placeholder** if the leftover fits nowhere.

### Post-T255 live CLI audit (2026-08-16) — T256–T271

Non-destructive dogfood after T255 closeout. Placeholder series registered in `conductor/conductor.md` (status **Pending**). Map: [README-T256-T271-CLI-AUDIT.md](tracks/README-T256-T271-CLI-AUDIT.md). Registration ledger `1d9511b5-798b-4d6c-b0c9-ebb4b07d0b69`.

| Finding | Track |
|---------|-------|
| ~~`--help` prints live `AI_BRAINS_KEY` (quality 3)~~ | **T256 Completed 2026-08-16.** Soft residual: PATH `ai-brains` stays leaky until operator `cargo install` (F18). |
| ~~Identity warn on every command / JSON interleave (`scope` 6/5, json 7/6)~~ | **T257 Completed 2026-08-17.** JSON-effective silent + scope token + remediator skip. T240 list hermetic stands. Soft: PATH `cargo install` (F13); T223 env-override still separate (F17). |
| ~~Daily Scope `test-alias` `441837f6` vs path owner `3581317d`~~ | **T258 Completed 2026-08-16.** `project adopt-path` print-only; `--write-env --yes` one-key rewrite. T240 F2 stands. Soft: general `project use <uuid>` (F14 remainder); no-owner JSON is `COMMAND_FAILED` not frozen object; AC10 event-count inspection-only; live operator rebind + PATH `cargo install` out of band. T259 leftover / T267 list footer unchanged. |
| ~~Leftover `7d97a456` 18,028 memories / many `C:\dev\*` roots~~ | **T259 Completed 2026-08-17.** Inventory `--project`/`--shared-only` + `rebind-path` print-only / `--write --yes` one-tx. Memories stay (F5). Footer **T267**. `--global` **T260/T264**. Soft: leftover memory reclassify by path; PATH `cargo install`; `project.rs` `resolve_project_ref` duplicate. |
| ~~Recall symbol stubs beat decisions (5/4, global 3/3, real semantic 4/3)~~ | **T260 Completed 2026-08-17.** Default exclude via `symbol_content` + **GLOB** ⊆ detector (F19). `--symbols` mix. Dedupe after `rerank_hits`. Leftover-**project** exclusion stays **T264**. |
| ~~`recall ""` 5.7 s~~ | **T261 Completed 2026-08-17.** 0-contentful → T207 empty; no LIKE/bridge/embed/graph. FEATURE TX `4a317118-21c5-4667-9f8d-ae10157f20e2`. |
| ~~Graph sparse; 4h pin no node; neighbors 4/5; hierarchy 3/4~~ | **T262 Completed 2026-08-17.** Pin `turn_id` = memory + graph node + `RECALLS`. Missing-node pretty: rebuild iff vault has memory/session. Historical no-field pins stay F1b. |
| ~~Governed 3 grants / 0 authority (briefing, progressive, evidence, source, review)~~ | **T263 Completed 2026-08-18.** H1: empty_authority + Personal deny name `recall`; expand Unknown preview; authorized-empty lists `next_step`. **H2 declined**. Soft: PATH `cargo install`; daily 0-of-3 grants stays T241. |
| ~~`preflight --global` blends other repos (5/4, summary 7/6)~~ | **T264 Completed 2026-08-18.** Label + per-project caps + summary span. Recall leftover drop declined (F11). Soft: Index fetch 80; span vs word-budget; pretty formatter still in `preflight.rs`. |
| ~~`preflight --format json` `{text, word_count}` blob (7/6)~~ | **T265 Completed 2026-08-19.** Additive `sections[]`; T180 required keys stay. Soft: PATH `cargo install`; pretty walker duplication (F12). |
| ~~Format maze; list-paths 7/5; retention default 6/5~~ | **T266 Completed 2026-08-18.** Inventory tokens + four-family table; auto default stays. Nightly pipes / graph-update JSON unchanged. |
| ~~harness/whoami self-next; list footer leftover-as-AI-Brains (8/6)~~ | **T267 Completed 2026-08-18.** harness ok → `none`/omit; list footer F3/F3b; git probe best-effort. Whoami remediations **affirm T258**. Soft: PATH `cargo install`; leftover roots. |
| `scan-roots` cwd-only (4/5) | ✅ **T268 Completed 2026-08-19** — `--root` XOR positional; empty suggested when registered; human parent hint. Soft: PATH reinstall; leftover `7d97a456` (T259); JSON `next_step` declined. |
| ~~Nightly human mixes Router 267009; completion probe timeout~~ | ✅ **T269 Completed 2026-08-20** — human `Nightly:` heading + `probe=timeout (750ms)`; JSON frozen; 750 ms not raised |
| ~~`retention plan` 0 candidates on 35,300 memories (6/5)~~ | ✅ **T270 Completed 2026-08-21** — COUNT overlay pinned→held / other→skip; `Nothing to dispose.` = no CE/projection. Soft: PATH `cargo install`; nightly `candidates=` includes held. |
| ~~`sync query` ledger pane false-empty (5/5)~~ | **T271 Completed 2026-08-19.** Stop FTS-quoting `ledgerful ledger search`; first-seen token rescue; named misses. Soft: PATH `cargo install`; Ledgerful token-OR; picker vs sequential probe. |

### T257 closeout residuals (2026-08-17)

Specified softs — not product blockers:

| Residual | Disposition |
|----------|-------------|
| PATH `ai-brains` still noisy until reinstall | Soft F13 — operator `cargo install` |
| T223 env-override can still trail JSON | Decline F17 — separate SOOT |
| Compact JSON uses `note_machine_stdout` not pretty | Intentional — T265/T266 |
| Human warn prints **after** the table | F6; T240 asserts presence not order |

### T262 closeout residuals (2026-08-17)

Specified softs — not product blockers:

| Residual | Disposition |
|----------|-------------|
| `DecisionRecorded` still `_ => {}` in projector | Soft F24 — pin is ingest |
| T213 F31 last-event vs last-graph timestamp | Decline — honesty is F1 + exists |
| Historical backfill `MemoryPinned` for 36k pins | Decline F35 — invents events |
| Neighbor UUID prefix | Decline F17 |
| Wrong-kind `pretty_no_memory_node` / `pretty_no_session_node` still `graph update` | Soft — outside F1a–c / AC9 |
| PATH `ai-brains` still hasher-turn until reinstall | Soft F22 — operator `cargo install --features graph` |
| T263 governed / T264 leftover `--global` | **T263 Completed 2026-08-18**. **T264 Completed 2026-08-18**. |

### T263 planning absorption (2026-08-18) — H1 only

| Item | Disposition |
|------|-------------|
| Audit 3 grants / 0 authority | **Absorb** H1 (empty_authority → `recall`; lists `next_step`; expand preview) |
| H2 pin→DecisionProposed | **Decline** — T167 pins→Evidence; briefing needs Approved; T170 stop-before live migrate |
| Trace wrap `{trace:null}` | **Decline** — T152 F31 / P-CLI scalar `null` frozen; document only |
| Daily Scope 0 of 3 grants | **Decline → T241** (already warns). Do not live-bootstrap as T263 DoD |
| T227 F3 pin inject | **Affirm** — authority arrays stay empty |
| Personal unused vs bootstrap | **Absorb** — deny next names `recall`, not Personal bootstrap |
| T264 / T266 / T267 | **Decline** — stay Pending placeholders |
| last-PR Cursor #177 | **N/A** — empty |
| Agy m2 empty-authority length | **Folded** T263 F29 / AC14 (≤140 / one line) |
| OpenCode Personal deny = `personal.rs:121` | **Folded** T263 F4 / F23 / AC3 |
| Expand `Denied` empty preview | **Decline** — Unknown is DoD; Denied stays exit 3 |

### T263 closeout residuals (2026-08-18)

Specified softs — not product blockers:

| Residual | Disposition |
|----------|-------------|
| Daily Scope 0 of 3 grants | Decline F14 — T241; do not live-bootstrap |
| H2 pin→Approved | Decline F11 — T167 pins→Evidence; no lossless Approved |
| Daemon/HTTP list `next_step` | Soft F25 |
| Wrap trace `null` | Decline F26 — P-CLI scalar frozen |
| Vault pin COUNT overlay | Soft F24 |
| `#18` personal continuity | Decline F27 |
| PATH `ai-brains` until reinstall | Soft F21 — operator `cargo install` |
| T264 leftover `--global` / T266 / T267 | **T264 Completed 2026-08-18** (preflight only). T266 / T267 stay Pending. |

### T264 planning absorption (2026-08-18) — label + cap; no recall drop

| Item | Disposition |
|------|-------------|
| Audit blender + summary mix | **Absorb** F1–F8 / AC5–AC8 / AC14 |
| T214 body label residual | **Absorb** `[8hex]` in retrieval + pretty `display_label` |
| T214 `active_sessions` `format!` | **Partial** F10 — `sessions.rs` only |
| T219 F13 selection freeze | **Partial** — project-scoped stands; global caps this track |
| Leftover-project `--global` recall drop | **Decline** F11 — `--global` means all projects; T259 F5 memories stay |
| T265 `sections[]` | **Decline** F12 |
| T214 F9 ledgerful-on-global | **Decline** F14 |
| T266 / T267 / T268+ | **Decline** F27 |
| T240 F2 / T255 | **Decline** F28 |
| last-PR Cursor #178 | **N/A** — empty |

### T264 fold-in (2026-08-18) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode M1 AC5 every-keyword-line | **Folded** F30 / AC5 item-first-line + two-line pin. Per-line retag declined |
| OpenCode m1 / Agy m2 F24 wrap | **Folded** `truncate_chars(32)` + `]` sanitize in `preflight_pretty.rs` |
| OpenCode m2 whole-line upgrade | **Folded** F4 / AC4 leading-only |
| OpenCode m3 AC14 age-out | **Folded** pass-with-observed-data |
| Agy m1 HEAD `d8be361` vs `bc10f3e` | **Note** §2.1 |
| Agy O1 `params![]` | **Already** F10 |

### T264 closeout residuals (2026-08-18)

Specified softs — not product blockers:

| Residual | Disposition |
|----------|-------------|
| Index fetch window 80 leftover-heavy | Soft R1b-P3-1 — tags still honest |
| Span may count Recent later trimmed by word budget | Soft R1b-P3-2 — F7 post-cap pre-pretty |
| Pretty formatter still in hotspot `preflight.rs` | Soft R1b-P3-3 — peel logic is sibling |
| AC5 does not independently assert Index/Recent tags | Soft R1b-P3-4 |
| Pretty 8-hex collision keeps raw tag | Intentional CX2 — no arbitrary alias |
| Recall leftover-first under `--global` | Decline F11 — not a silent exclude |
| PATH `ai-brains` until reinstall | Soft F21 — operator `cargo install` |
| T266 / T267 | **T266 Completed 2026-08-18**. **T267 Completed 2026-08-18**. |

### T266 planning absorption (2026-08-18) — taxonomy + tokens; no default flip

| Item | Disposition |
|------|-------------|
| Audit maze; list-paths JSON wall; retention pipe JSON | **Absorb** F1–F7 — wall is `auto` + non-TTY; remediator is `--format human` + `pretty` token |
| Shared `resolve_human_json_format` | **Absorb** F4 — delete three `use_json_output` forks |
| T227 F34 OutputFormat surface-wide | **Decline** F11 |
| T246 F17 / F6 graph update TTY-auto | **Decline** F8 — T74 |
| T255 F2 nightly pipes | **Affirm** F2 |
| T265 envelope / T267 footer / T268 scan / T270 classify | **Decline** F12–F13 |
| T240 F2 / T255 bag | **Decline** F14 |
| last-PR Cursor #179 safety_ids over-exclude Index | **Mint T272** — still true at `preflight.rs:329` + `:467`; fits no T265–T271 placeholder |

### T266 fold-in (2026-08-18) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode m1 AC4 human half unnamed | **Folded** AC4 `list_paths__format_human__table_not_json` |
| OpenCode m2 `--format` arg help stale | **Folded** F7 / AC11 / Phase 3 five docstrings |
| OpenCode m3 AC3 filter-dependent empty copy | **Folded** AC3 no `--project`/`--shared-only` |
| Agy m1 HEAD `4088106` vs `8c3b7e1` | **Note** §2.1 |
| Agy m2 clap JSON/Pretty on remaining 3 | **Folded** AC7 |
| Agy O1 `is_json_output` | **Folded** F27 |
| OpenCode O1 non-empty pretty hermetic | **Folded** AC14 |
| OpenCode O2 CAPABILITIES missing-row list | **Folded** AC11 + Phase 3 |

### T266 closeout residuals (2026-08-18)

Specified softs — not product blockers:

| Residual | Disposition |
|----------|-------------|
| T246 F17 TTY-auto `graph update` | Soft F8 — T74 default JSON |
| T227 F34 `parse_or_fail` | Decline F11 — governed contract |
| Harness status no `value_parser` | Soft F25 — Family B |
| TTY/`auto` hermetics still force `human`/`json` | Soft T254 F12 |
| PATH until reinstall | Soft F21 — operator `cargo install` |
| T267 footer / T268 scan / T270 classify | **T267 Completed 2026-08-18**. T268 / T270 stay Pending. |

### T267 closeout residuals (2026-08-18)

Specified softs — not product blockers:

| Residual | Disposition |
|----------|-------------|
| Harness status no `value_parser` | Soft T266 F25 — Family B |
| PATH until reinstall | Soft F18 — operator `cargo install` |
| Live leftover still owns many `C:\dev\*` roots | T259 operator rebind — out of band |
| Daily 0 of 3 grants | T241 |
| T268 scan-roots parent / `--root` | Peer — still Pending |
| T265 / T269 / T270 / T271 / T272 | Decline F11 — T265 **Planned 2026-08-19**; others still placeholders |
| Codex `/hooks` next on install | Keep F7 — not self-next of status |

### T267 planning absorption (2026-08-18) — remediator honesty; no leftover UUID

| Item | Disposition |
|------|-------------|
| Audit harness self-next; list leftover-as-AI-Brains | **Absorb** F1 / F3 / F3b / F6 / AC2–AC4 / AC6–AC7 |
| T259 footer algorithm | **Absorb** F3 / F3b / F9 — no hardcoded leftover UUID |
| T258 whoami remediations | **Affirm** F2 / AC5 — already adopt-path |
| T212 footer chrome | **Partial** F22 — stderr example stays; pick changes |
| T235 F40 install next = status | **Affirm** F7 / AC10 |
| Shared “don’t next yourself” helper | **Decline** F4 |
| Doctor / T240 “run whoami” | **Decline** F21 |
| T265 / T268 / T269 / T270 / T271 / T272 | **Decline** F11 |
| last-PR Cursor #180 | **N/A** — empty |
| T240 F2 / T255 bag | **Decline** F12 |

### T265 closeout residuals (2026-08-19)

Specified softs — not product blockers:

| Residual | Disposition |
|----------|-------------|
| Pretty walker ≠ JSON splitter (duplicated header table) | Soft F12 — unify later if they drift |
| Index without blank lines = one item | Soft F6 v1 |
| F2b truncated `---` header stays in previous section items | Honesty — not a fabricated `index` |
| PATH `ai-brains` still 2-key until reinstall | Soft F20 — operator `cargo install` |
| T272 `safety_ids` over-exclude | Peer placeholder |
| json-v2 / typed arrays / summary envelope / clap `value_parser` | Declined F10 / F9 / F14 |

### T265 planning absorption (2026-08-19) — additive `sections[]`; no json-v2

| Item | Disposition |
|------|-------------|
| Audit `{text, word_count}` blob (7/6) | **Absorb** F1–F8 / AC1–AC4 — compact required keys stay; always-present `sections[]` |
| T214 residual extra keys on `PreflightContextResponse` | **Absorb** — `sections` only; no `api_version`/`schema_version` on this DTO (F25) |
| T220 / T264 / T266 “do not grow T180 2-key” | **Absorb as lift** — T180-C comment is “without a track”; T265 is that track. Summary path stays T220 (F9) |
| T257 compact + `note_machine_stdout` | **Affirm** F15 |
| json-v2 / `--structured` | **Decline** F10 — agents already pass `--format json`; serde/dogfood ignore extras |
| Typed `constraints[]` / `decisions[]` | **Decline** F10 — governed `briefing` / T170 D21 |
| Retrieval assembly / T272 `safety_ids` | **Decline** F11 — still true at `preflight.rs:329` + `:467` |
| T268 / T269 / T270 / T271 | **Decline** F24 |
| T240 F2 / T255 bag | **Decline** F23 |
| last-PR Cursor #181 | **N/A** — comments/reviews empty |
| clap `value_parser` on preflight `--format` | **Decline** F14 — T220 F13 case-sensitive parser would regress `--format JSON` |

### T265 fold-in (2026-08-19) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode F6 session `\n`-join → one item | **Folded** F6 / AC3. Turn-split via `is_session_turn_start` declined |
| OpenCode F5 `contains` vs `starts_with` | **Folded** F5 copies pretty; Ledgerful `contains("Ledgerful Intelligence")` |
| OpenCode AC15 live Ledgerful variants | **Folded** AC15 plain + Fallback strings |
| OpenCode preamble before first `---` | **Folded** F6 / AC5 `leading_preamble__discarded` |
| OpenCode AC11 no fabricate `empty_repo` | **Folded** AC11 / F5 |
| OpenCode `preflight_contextual_risk` | **Folded** plan Phase 3 stays green |
| Agy/OpenCode HEAD `2a00ce3` vs `7192070` | **Note** §2.1 |
| Agy m2 `pub(crate) mod` | **Partial** F12 sibling `pub mod` |
| Agy O1 contracts `SECTION_ID_*` | **Partial** CLI sibling consts only |

### T271 planning absorption (2026-08-19) — FTS-quote lift + token rescue

| Item | Disposition |
|------|-------------|
| Audit ledger pane false-empty (5/5) | **Absorb** F1–F7 / AC1–AC9 / AC13 |
| Stub F1 never-ran vs ran-empty | **Absorb** F1 / F8 / AC6 / AC8 |
| Stub F2 System32 | **Absorb** F2 / AC7 (guard; not the live repro) |
| T90 sanitize on ledger argv | **Absorb as lift** — vault MATCH keeps T90; probe must not quote |
| T91 strip ANSI | **Affirm** F5 / AC2 |
| T95 project isolation | **Decline** — vault-only; ledger is cwd/`gix` |
| T211 F12 empty → vault-only | **Partial** F9 — reorder + `--json` stay; miss/rescue display changes |
| T211 F25 blend / double shell | **Decline** F11 |
| T217 vault OR rescue | **Decline** — CLI sequential token rescue only |
| T268 / T269 / T270 / T272 | **Decline** F12 |
| T240 F2 / T255 bag | **Decline** F12 |
| last-PR Cursor #182 | **N/A** — comments/reviews empty |

### T271 fold-in (2026-08-19) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode m F18 empty-query unit | **Folded** AC17 + AC1 empty-forward |
| OpenCode m F19 classifier units | **Folded** AC18 / AC19 + F19 mapping |
| OpenCode O F19 140-char cap | **Folded** F19 first line then 140; no `project.rs` import |
| OpenCode O `ai_brains_core` FTS import | **Folded** F6 / §2.3 / §5.4 |
| OpenCode O capture count 5→9 | **Folded** §2.1 volatile ≥1; AC15 already ≥1 |
| Agy/OpenCode HEAD `e48eaa7` vs `33f72cf` | **Note** §2.1 |
| Agy m2 `pub mod` | **Already** F10 |
| Agy O1 two-phase probe | **Already** F9 / F17 |

### T271 closeout residuals (2026-08-19)

Specified softs — not product blockers:

| Residual | Disposition |
|----------|-------------|
| PATH `cargo install` | F16 — operator |
| Ledgerful token-OR / stop phrase-wrapping | F23 — other repo |
| AC5 picker `cfg(test)` vs sequential probe | R1b P3-3 / CX2 P3-1 — extra procs if collect-then-pick |
| T211 human re-run JSON fallback | R1b P3-8 — F17 still re-runs human on hits |
| T211 F25 blend / double shell | declined F11 |
| Rescue scoring / merge all token tables | spec §11 |
| T268 / T269 / T270 / T272 | declined F12. **T268 Planned 2026-08-19.** **T273 minted** from #183 Cursor (dash-query). |

### T268 planning absorption (2026-08-19) — `--root` + empty suggested; no default flip

| Item | Disposition |
|------|-------------|
| Audit cwd-only scan (4/5) + re-register suggestion | **Absorb** F1–F3 / AC1–AC7 |
| T254 positional `[PATH]` already exists | **Affirm** — `--root` is XOR named alias, not a new scan engine |
| T254 F21 default = cwd | **Affirm** F15 — do not default to `C:\dev` |
| T254 F20–F23 bounds | **Affirm** F5 |
| T254 F12 closeout (TTY auto hermetic, etc.) | **Decline** — not parent/`suggested` |
| Leftover `7d97a456` sibling roots | **Decline** F12 — T259 `rebind-path` |
| T266 format / T269 / T270 / T272 | **Decline** F6 / F18 |
| last-PR Cursor #183 dash-query Medium | **Mint T273** — still true at `sync_query_ledger.rs:157`; fits no T268–T272 placeholder |
| T240 F2 / T255 bag | **Decline** F27 |
| clap 5 / pin bumps / camino / DTO | **Decline** F9 / F10 |
| T268 fold-in 2026-08-19 (agy+opencode) | **Folded** F21/F22/F28/F29/F2-empty + AC16–AC17. **Declined** silent `--root`+PATH ignore (XOR); JSON `parent_hint`; remint T273. No B/M. |
| T268 implement residuals (2026-08-19) | PATH `cargo install` (F16); leftover `7d97a456` sibling owners (T259 `rebind-path`); JSON `next_step` declined F10; default=parent declined F15; WORKFLOWS.md `--root` mention out of F11 required docs; T273 dash-query **Planned 2026-08-19**. |

### T273 planning absorption (2026-08-19) — POSIX `--` on ledger argv; no T90

| Item | Disposition |
|------|-------------|
| #183 Bugbot Medium dash QUERY as Ledgerful flags | **Absorb** F1–F4 / AC1–AC5 / AC9 — always-on `--` before QUERY |
| Rescue never starts after clap fail | **Absorb** F4 — T271 F6 stands; argv is the remediator |
| T90 on ledger argv | **Affirm decline** F3 |
| Our Query `allow_hyphen_values` / steal vault `--limit` | **Decline** F5 / AC8 — clap 4.6.6 known flags win; operator uses `sync query -- --limit` |
| last-PR Cursor #184 Linux Path units | **Decline** F8 — already `#[cfg(windows)]` at `project_paths.rs:639+`; T268 review P1 fixed; **no T274** |
| recall `bridge_search_args` (`ledgerful search`) | **Decline as DoD** F7 — soft residual, same `--` insert |
| T269 / T270 / T272 | **Decline** F9 |
| T211 F25 / T217 MATCH OR / Ledgerful token-OR | **Decline** F10 |
| T240 F2 / T255 bag / clap 5 / DTO | **Decline** F9 / F10 |
| Historical deferred (CE wipe, connector cursor, `anyhow` allowlist, MSI, archive changeguard) | **Decline** — not query argv |

### T273 fold-in (2026-08-19) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode **B-1** AC10 `-- --limit --no-bridge` | **Folded** F21 / AC10 — `sync query --no-bridge -- --limit` (live exit 2 vs exit 0) |
| Agy m1 helper needle `"--"` | **Folded** AC4 / F19 |
| Agy m2 `after_help` contrast | **Folded** F6 / AC12 |
| OpenCode O-1 AC8 `ErrorKind` | **Folded** F22 — execute: clap 4.6.1 empty `--limit` is `InvalidValue` (not T247 `MissingRequiredArgument`) |
| OpenCode O-2 quiet optional | **Folded** F23 / AC14 required |
| Agy O1 `pub(crate)` / O2 recall residual | **Already** F11 / F7 |
| OpenCode m-1 | **Already** closed by B-1 |

### T269 fold-in (2026-08-20) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode m line counts 2124/593 vs 1964/554 | **Folded** — both: total `.Count` vs non-blank `-Line` |
| OpenCode m/O HEAD `6825343` vs `5bfc088` | **Folded** — product tree identical |
| OpenCode O docs.rs truncated | **Already** F10 live `main.rs` after_help |
| Agy m1 `== "timeout"` pass-through | **Folded** F27 / AC2 extra cases |
| Agy m2 non-Windows heading | **Folded** F1 outside `cfg(windows)` / AC8 all-OS hermetic |
| Agy O1 heading const | **Already** F1 / AC1 |
| Agy O2 after_help TCP vs `/health` | **Folded** AC6 required needles |
| No B/M | Nothing to decline |

### T269 implement residuals (2026-08-20)

| Residual | Disposition |
|----------|-------------|
| PATH `ai-brains` until `cargo install` | F16 — operator; tests/manual used `cargo run` |
| JSON `probe: "timeout"` has no budget field | F21 — scripts read CAPABILITIES; not a schema bump |
| Operator llama.cpp without #20817 still queues `/health` | Not product DoD; honesty is the remediator |
| `--quick --no-vault` | T255 F15 |
| T270 / T272 | Peers |
| T273 F7 recall `bridge_search_args` | Other crate |
| Local T188 restore tests vs live daemon IPC | Environmental; first `dev-check` fail-fast was `backup_restore__daemon_down_force__succeeds`. Gate re-run after temporary `daemon stop`/`start` (not a product change). |

### T269 planning absorption (2026-08-20) — Nightly heading + timeout budget; no 750 ms raise

| Item | Disposition |
|------|-------------|
| Audit human mixes Last Result 0 with Router 267009 | **Absorb** F1 / AC1 / AC8 / AC10 — `Nightly: AI-Brains-Nightly` before schedule block |
| Completion `probe=timeout` vs daemon Open | **Absorb as label** F3 / F4 / AC2 / AC6 — human `timeout (750ms)`; do **not** raise 750 ms (llama.cpp #20684 `/health` queue; T255 F18) |
| `--quick` skipped | **Absorb** F2 |
| JSON already split | **Affirm** F5 — no `probe_budget_ms` |
| T255 Router line / JSON keys / 750 ms `join!` | **Affirm** F4 / F5 / F6 — do not restyle Router |
| T255 doctor 16th / persist probe / embed sleep / `.cmd` / `--no-vault` | **Decline** F19 |
| Unify daemon TCP with HTTP `/health` | **Decline** F18 — T199 |
| T270 / T272 / T273 F7 `bridge_search_args` | **Decline** F20 |
| last-PR Cursor #185 | **N/A** — comments/reviews empty |
| last-PR #184 Linux Path units | **Decline** — already `#[cfg(windows)]`; **no T274** |
| T240 F2 / clap 5 / DTO / `cargo install` / live schtasks mutate | **Decline** F14 / F16 / F17 |
| Historical CE wipe, MSI, `anyhow` allowlist, archive `changeguard` | **Decline** — not status chrome |

### T273 implement residuals (2026-08-20)

| Residual | Disposition |
|----------|-------------|
| PATH `ai-brains` until `cargo install` | F16 — operator; tests/manual used `cargo run` |
| `bridge_search_args` (`ledgerful search` code) dash-query | F7 — retrieval crate; same `--` insert; not this DoD |
| Ledgerful QUERY `allow_hyphen_values` / token-OR | Other repo; `--` is our remediator (T271 F23) |
| T269 / T270 / T272 | Peers — not stolen |
| F22 ErrorKind pin | Execute correction documented; unit locks live `InvalidValue` + `--limit <LIMIT>` |

### T272 planning absorption (2026-08-20) — skip emitted Safety ids; no T264 retune

| Item | Disposition |
|------|-------------|
| #179 Bugbot Medium `safety_ids` filled from LIMIT 40 before `take_round_robin` 8 | **Absorb** F1 / AC2 — still true at `preflight.rs:329` + `:467` (HEAD `9008074`) |
| Placeholder F1–F4 | **Absorb** post-cap skip / project-scoped shown / no leftover recall drop / hermetic capped-out in Index |
| Post-`dedup_hotspots` over-exclude (insert then drop) | **Absorb** F1 / AC1 — same SOOT as the cap |
| T264 Index fetch 80 leftover-heavy (R1b-P3-1) | **Decline** F17 |
| Session `HOTSPOT:` content skip | **Decline** F18 (soft) |
| T265 json-v2 / CLI splitter / T180 2-key | **Decline** F7 — text may gain Index lines; keys frozen |
| T270 / T273 F7 `bridge_search_args` | **Decline** F16 — peers |
| last-PR Cursor #186 | **N/A** — comments/reviews empty. **No T274** |
| T240 F2 / T255 bag / clap 5 / rusqlite 0.40 / DTO / `cargo install` | **Decline** |
| Historical CE wipe, MSI, `anyhow` allowlist, archive `changeguard` | **Decline** — not skip-set |

### T272 fold-in (2026-08-20) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode m HEAD `9008074` vs `9fcfcd8` | **Folded** §2.1 — product tree identical |
| OpenCode m AC3 not Phase-1 red | **Folded** F27 — guard; AC2 is the required red |
| OpenCode m AC1 failure claim | **Already** F3 / AC1 extras-through-dedup |
| OpenCode O A-one not in Safety / optional helper | **Already** AC2 / F9 |
| Agy m1 AC2 word budget | **Folded** F26 / AC2 `-m 1500` + `Memory Index` header |
| Agy m2 AC1 extras keep freshest id | **Folded** AC1 concrete `T` |
| Agy O1 `index_section` | **Already** AC2 / plan |
| Agy O2 rebuild comment | **Folded** F28 |
| No B/M | Nothing to decline |

### T270 fold-in (2026-08-20) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| Agy m1 forgotten-only sample SQL | **Folded** F5 / AC1 second case / AC4 |
| Agy m2 `classes` sort after upsert | **Folded** F30 / AC17 |
| Agy O1 notes const | **Folded** F31 |
| Agy O2 SQL `LIMIT 5` | **Already** F5 / AC16 |
| OpenCode m HEAD `70d61cd` vs `fdd4924` | **Folded** §2.1 — product tree identical |
| OpenCode m nightly line drift | **Folded** `:511–535` |
| OpenCode O pin-count / hotspot / search lines | **Folded** snapshots |
| OpenCode O4 `ISSUES.md` | **Already** F24 |
| OpenCode deferred “F8 → `candidates==0`” | **Declined** — F8 is dispose-work |
| OpenCode SOOT = `collect_candidates` `:771` | **Partial** — call site yes; merge after `build_report` (F6) |
| last-PR #187 Cursor | **N/A** — still empty. (T274 later minted 2026-08-21 as pin-vs-ingest, not from #187) |
| No B/M | Nothing to decline of B/M |

### T270 planning absorption (2026-08-20) — inventory overlay; no migrate

| Item | Disposition |
|------|-------------|
| Audit 6/5 zero candidates on ~35k (live **38,208** pinned) | **Absorb** F1–F11 / AC1–AC13 — COUNT + ≤5 samples; pinned→held; other→skip |
| Placeholder F1 honesty sentence only | **Partial** — warning const **plus** overlay (sentence-only insufficient) |
| Placeholder F2 optional overlay | **Absorb** — chosen over migrate governed |
| Placeholder F3 `none_auto` / F4 apply confirm | **Affirm** |
| T166 §5.1.5 stream-A memory_legacy | **Absorb as inventory** (not per-row age wipe / not `soft_forget`) |
| T167 `classify_legacy` / `migrate governed` remediator | **Decline** F18 |
| T248 empty-check `candidates==0` / Work-all-classes | **Lift** F8/F9 — `Nothing to dispose.` = no CE/projection; Work = dispose mechanisms only |
| T248 F16 doctor / F17 engine leftovers / nightly restyle | **Decline** |
| Identity mismatch `7d97a456` vs `fcb8a40f` (agent observation) | **Decline** F19 at T270 plan — T240/T257/T258; leftover data residual **T276** (2026-08-21) |
| last-PR Cursor #187 | **N/A** — comments/reviews empty |
| T273 F7 / leftover rebind / T240 F2 / clap 5 / rusqlite 0.40 | **Decline** |
| Historical CE wipe, MSI, `anyhow` allowlist, archive `changeguard` | **Decline** |

### T272 closeout residuals (2026-08-20)

Specified softs — not product blockers:

| Residual | Disposition |
|----------|-------------|
| Session `HOTSPOT:` content skip | F18 — independent of `safety_ids`; may still hide a capped-out hotspot from Sessions |
| T264 Index fetch 80 leftover-heavy (R1b-P3-1) | F17 — not this HashSet |
| Live `-m` windows without an Index header | Word budget; hermetic AC2 (`-m 1500` + `Memory Index`) is DoD |
| PATH `ai-brains` until `cargo install` | F14 — operator; tests/manual used `cargo run` / hermetic |
| ~~T270 retention classify~~ / T273 F7 `bridge_search_args` | **T270 Completed 2026-08-21.** T273 F7 remains a peer residual |
| rusqlite `table_exists` 0.40 | T213 L4 — not this track |

### T267 fold-in (2026-08-18) — `agy-review.md` + `opencode-review.md`

| Item | Disposition |
|------|-------------|
| OpenCode m3 AC10 existing hermetic missing | **Folded** AC10 `harness_install__success__next_is_status` |
| Agy m2 preflight Ok unit | **Folded** F8 required + AC15 |
| Leftover live 11 paths (OpenCode said 1) | **Folded** AC7 multi-path+orphan / AC16 leftover-only |
| OpenCode m1 line count 1511/1368 | **Folded** AC12 total lines |
| OpenCode m2 pin count | **Folded** §2.1 volatile |
| Agy O1 / OpenCode O1 collect_git_identity | **Folded** F10 |
| OpenCode O3 after_help | **Partial** F14 optional |
| OpenCode O2 ledgerful hygiene | **Decline** — not T267 |
| last-PR Cursor #180 | **N/A** — still empty |

### T261 closeout residuals (2026-08-17)

Specified softs — not product blockers:

| Residual | Disposition |
|----------|-------------|
| Skip CLI graph-vault open on contentless | Soft §11 — SQLCipher open dominates |
| Skip T207 `<10` memory COUNT on contentless | Soft F9 — keep small-vault sentence |
| PATH `ai-brains` still LIKE/MATCH-all until reinstall | Soft F12 — operator `cargo install` |
| Leftover-project `--global` / preflight blender | **T264 Completed 2026-08-18** (preflight label+cap; recall drop declined) |
| Graph sparse / 4h pin no node | **T262 Completed 2026-08-17** |

### T260 closeout residuals (2026-08-17)

Specified softs — not product blockers:

| Residual | Disposition |
|----------|-------------|
| Leftover-project `--global` / preflight blender | **T264 Completed 2026-08-18** (preflight label+cap; recall drop declined) |
| `source_tag` on `memory_projection` | Soft — store/replay track |
| AC8 hermetic does not force embed `ok` + zero post-threshold | Soft — default exclude already makes F11+stub impossible |
| PATH `ai-brains` still ranks stubs until reinstall | Soft F18 — operator `cargo install` |
| `--symbols-only` / `sync query --symbols` | Declined F16 |

### T259 closeout residuals (2026-08-17)

Specified softs — not product blockers:

| Residual | Disposition |
|----------|-------------|
| Reclassify leftover memories onto dest by path/provenance | Soft — F5; later importer if ever |
| `--global` leftover-first / blender | Symbol monopoly **T260 Completed**. Preflight blender **T264 Completed 2026-08-18** (label+cap; recall drop declined). |
| `project list` footer leftover-as-AI-Brains | **T267 Completed** F3/F3b |
| `project.rs` private `resolve_project_ref` duplicate | Soft — F12 hotspot freeze |
| No-owner / dest-missing JSON is generic `COMMAND_FAILED` | Soft — F8/AC7/AC8; T257 owns JSON interleave |
| PATH `ai-brains` lacks rebind until reinstall | Soft F23 — operator `cargo install` |
| Live leftover roots still on `7d97a456` | Operator out of band (`context` then `rebind-path --write --yes` per path) |

### T256 closeout residuals (2026-08-16)

Specified softs (F18) — not product blockers:

| Residual | Disposition |
|----------|-------------|
| PATH `ai-brains` still leaky until reinstall | Soft F18 — operator `cargo install`; tests use hermetic/source bin |
| `AI_BRAINS_VAULT_PATH=` on help | Intentional F5 |
| `init` one-shot print | Declined F6 |
| Daemon `AI_BRAINS_VAULT_KEY` sidecar | Declined F7 |

### T254 closeout residuals (2026-08-15)

Specified softs (F12) plus deferred review lows — not product blockers:

| Residual | Disposition |
|----------|-------------|
| AC3 `--format auto` TTY/pipe not hermetic | Soft F12 — same `IsTerminal` as whoami; suite forces json/human |
| AC13 helper-vs-loop wiring | Soft F12 — unit drives helper; loop calls helper on Ok/Err |
| F16 no pin/symbol assertion after unregister | Soft F12 — unregister only appends Removed |
| Scan/dry-run alias-count vs event-log length | Soft F12 — product claim is no aliases written |
| Concurrent multi-operator F21 atomicity | Declined F5 — refuse-steal is the safety net |
| T233-F44 `ledgerful endpoints` ingest | Declined F4 |

### T253 closeout residuals (2026-08-15)

Specified softs (F34) plus deferred review lows — not product blockers:

| Residual | Disposition |
|----------|-------------|
| Nightly Claude/Codex sources + skip flags | Soft F34 — T239 D16 stays |
| Codex SessionEnd ingest (3s) | Soft F34 |
| Unix `.sh` wrappers / unified PS1 template | Soft F34 |
| Doctor helper `backend pending (T253)` on synthetic `install_ready=false` | Deferred P3 — no production id is pending |
| Uninstall `serde_json::to_string(...).unwrap_or_default()` | Deferred P3 — not `unwrap!` |
| Historical research banners mention `codex_hooks` as the stale claim | Deferred P3 |

### T252 closeout residuals (2026-08-15)

Specified softs (F12), not review deferrals:

| Residual | Disposition |
|----------|-------------|
| Vault-free `ingest --dry-run` / `run_sync_path_free` | Soft — F9 vault still required |
| `ingest --schema` | Soft — T83 siblings; not DoD |
| `std::io::IsTerminal` migrate | **Closed 2026-08-16** |
| clap 4.6 workspace pin | Soft — lock 4.6.1 stays |
| T86 `read_json_from_stdin` swallow | Soft — preflight `--stdin`, not ingest |
| Shared `read_stdin_trimmed` SOOT | Soft — isolation |
| `outcome.events[0]` if `events` empty | Soft — pre-existing live path |
| T253 | Closed — Claude/Codex install_ready |
| T254 | Closed — list-paths / unregister / scan-roots |
| T255 | Closed — JSON status + read-only Router |

### T251 soft residuals (2026-08-14)

Specified softs, not review deferrals:

| Residual | Disposition |
|----------|-------------|
| F12 `device list --format json` (T176 leftover) / bootstrap→outbox / doctor enrollment check / combined list+replicate dashboard / `visible_alias = "stat"` / default `device` → status / is-terminal → std / clap 4.6 workspace pin / unify singular error copy in `load_local_signing_key` / `load_local_device` | Soft — not DoD |

### T250 soft residuals (2026-08-14)

Specified softs, not review deferrals:

| Residual | Disposition |
|----------|-------------|
| F12 clap 4.6 workspace pin / retrieval JSON role strip / pager / governed section caps / `--max-line` / T241 `--install-grants` / skill one-liner / HOTSPOT float-score reformat / auto-compact from terminal height | Soft — not DoD. **Closed 2026-08-16:** is-terminal → std |

### T249 soft residuals (2026-08-14)

Specified softs, not review deferrals:

| Residual | Disposition |
|----------|-------------|
| F12 Daemon uptime / `sc query` / `--format json` / `--quick` / compact doctor JSON DTO / T204 Start-here rewrite that removes json / color / pager / `comfy-table` | Soft — not DoD. **Closed 2026-08-16:** T214 is-terminal → std; shared `resolve_*_format` (graph stays local) |
| F13 T241 F20–F22 leftovers / T226 O1 shared resolve wrapper / ~~T255 nightly/router~~ / ~~T250 preflight density~~ | Peer tracks (T250 + T255 closed) |

### T238 soft residuals (2026-08-09)

| Residual | Disposition |
|----------|-------------|
| S1 min-interval debounce beyond in-flight | Soft — not DoD |
| S2 compaction_continue explicit key polish | Soft (synthetic already dropped) |
| S3 live `message.updated` incremental | Soft |
| S4 pure-export live if SDK drifts | Soft (F12 fallback present) |
| S5 npm `@ai-brains/opencode-plugin` | Soft |
| S6 project-local plugin opt-in | Soft (C7 global default) |
| S7 import `--json` report | Soft |
| S8 Claude/Codex install_ready | ~~**T253**~~ ✅ Closed |
| ~~S9 multi-harness nightly~~ | **Closed by T239** PR #108 `a271a99` |
| S10 compacting pre-archive hook | Soft / non-goal |
| S11 opt-in child ingest | Soft (default skip hard) |
| ~~S12 dual-subscribe `session.status` idle~~ | **Closed by T245** — idle **or** status+`"idle"` (not deprecation) |
| msg-id true event-store delta | Soft (index+watermark Grok-class honesty) |

### T237 planning absorption (2026-08-08)

| Residual | Disposition |
|----------|-------------|
| T234 wire `filter_grok_history_*` | **Absorb** T237 F4/F11 / AC1–AC2 |
| T235 Grok backend_pending / install_ready false | **Absorb** F21–F25 / AC9–AC11 |
| Live synthetic_reason / system-reminder as type:user | **Absorb** F11 / AC2 |
| T236 wrapper stdout / unbound / path meta / turn-id lessons | **Absorb** F6/F8/F14/F16 (Grok empty stdout ≠ AGY allow JSON) |
| updates.jsonl as resume authority | **Not content SOOT** — F18 / AC14 (chat_history only) |
| UserPromptSubmit prompt field unclear in docs | Soft **S1** (not DoD) |
| Subagent sessions | Default **skip hard F12/AC18**; opt-in soft **S2** |
| ~~OpenCode / multi-harness nightly / SYSTEM skip-import~~ | OpenCode **closed T238**; multi-nightly + SYSTEM **closed T239** PR #108 |
| Claude/Codex install_ready | Soft **S6** (not T237 body; fix pending labels F33) |

### T237 AI review fold-in (2026-08-08)

| Item | Disposition |
|------|-------------|
| AI2 **M1** empty Stop stdout (not AGY `decision:allow`) | **F6 hard** / **AC12** |
| AI2 **M2** user_info/git_status without synthetic_reason; user_query-only keep | **F11 hard** / **AC2** matrix |
| AI2 **M3** subagent walk pollution | **F12 hard** / **AC18** |
| AI2 **M4** percent encode/decode helper | **F7 hard** / **AC19** |
| AI2 **M5** turn-id + source_ts honesty | **F35** / CAPABILITIES; fingerprint **S8** |
| AI2 **M6** no `$` in command | **F34** / **AC19** |
| AI2 M7 timeout | **F23** timeout 120 |
| AI2 M8 Claude/Cursor vendor merge | **F27** caveat |
| AI2 M9 Phase 1 Red = live chrome | plan reorder |
| AI1 path scan / foreign hooks / multipart / locks | F7/F22/F4/F15 affirmed |

### T238 planning absorption (2026-08-08)

| Residual | Disposition |
|----------|-------------|
| T234 wire `filter_opencode_*` + live export schema | **Absorb** T238 F1–F7 / AC1–AC2 / AC19 (nested `{info,parts}`; part type `tool`) |
| T235 OpenCode backend_pending / install_ready false | **Absorb** F27–F32 / AC9–AC11 |
| OpenCode plugin + export batch (deferred) | **Absorb** F8–F26 |
| Dual-path lessons (unbound / path meta / turn-id / force) | **Absorb** F13–F14, F20–F22 |
| Multi-MB export cost | **Absorb** F18 watermark + F12/F19 timeout 120 / AC16 |
| Never raw SQLite | **Hard** F24 / AC14 / D18 |
| Stale Opencode-Hooks-Research config shape | **F37** supersede — not implement Stage 2 as written |
| session.created inject / compacting archive | Soft / non-goal (S10) |
| ~~multi-harness nightly / SYSTEM skip-import~~ | **Closed by T239** PR #108 (D12 keep SYSTEM skip) |
| Claude/Codex install_ready | Soft **S8**; pending labels **T239+** (F32) — **not** T239 body |

### T239 closeout residuals (2026-08-09)

| Item | Notes |
|------|-------|
| Claude/Codex install_ready | Soft **T239+** / S8 / S-CLAUDE |
| S-SYS opt-in SYSTEM import | Soft — not DoD |
| S-DOC doctor/preflight last-import line | Soft |
| S-HOME SYSTEM empty-home counter | Soft (wrapper default prevents) |
| S-CAP list_capped false-positive tighten | Soft |
| S-JSON / S-FORCE / S-BRAINLOG / S-BUDGET | Soft |

### T239 AI review fold-in (2026-08-09)

| Item | Disposition |
|------|-------------|
| AI1 **M1** hermetic MultiImportOptions + malformed-fixture AC2 | **Absorbed hard** D20 / F18/F21 / AC1–AC2 |
| AI1 **M2** one corrupt file aborts source | **Absorbed min** D22 path-in-error; full per-session soft-skip **S-SESSION** |
| AI1 **M3** per-source StoreSink + partial import counters | **Absorbed hard** D5/D21 / AC13 |
| AI1 **M4** OpenCode health counters in report/status | **Absorbed hard** D6/F9/AC12 |
| AI1 **M5** corrupt last_multi_import status | **Absorbed hard** D23/AC11 |
| AI1 **M6** at = to_rfc3339 | **Absorbed** D8 |
| AI1 **M7** non-JSON stderr under SYSTEM json log | **Absorbed** D24 document |
| AI1 **M8** Antigravity-only touch-list | **Absorbed** F10 |
| AI1 **M9** SYSTEM empty-home ok/0 | Soft **S-HOME** |
| AI1 **M10** list_capped false-positive | Soft **S-CAP** |
| AI1 affirms D12/adapters/sync_state/status/deps | Affirmed |
| AI2 architecture + AC table + typed report + Claude honesty | Affirmed / F24 / F13 |

### T238 AI review fold-in (2026-08-09)

| Item | Disposition |
|------|-------------|
| AI1 affirm (timeout, foreign plugins, T239+ labels, implement map) | Affirmed — already F12/F19/F28/F32 |
| AI2 **M1** child/subagent `session.idle` | **F10 hard** / **AC21** |
| AI2 **M2** synthetic/ignored/editor_context text | **F2/F3 hard** / **AC22** Phase 1 Red |
| AI2 **M3** session.idle deprecated | **F34** / S12; batch backstop |
| AI2 **M4** list cap-100 + projectId | **F17 hard** / **AC23** |
| AI2 **M5** live SDK messages + in-flight | **F12 hard** (S4 promoted); **F15** |
| AI2 **M6** full part-type union | **F3** / **AC1** |
| AI2 **M7** compaction skip key | **S2** rewrite (synthetic + metadata) |
| AI2 **M8** OPENCODE_CONFIG_DIR | **F40** / F34 |
| AI2 **M9** prefer worktree | **F20 hard** |

### T236 planning absorption (2026-08-08)

| Residual | Disposition |
|----------|-------------|
| Live AGY2 transcript step-shaped; agy-hook `{role,content}` only | **Absorbed** T236 F1–F2 / AC1–AC3 (P0) |
| Batch `project_hash: None` → cwd/default hijack | **Absorbed** F9–F12 / AC5–AC7 |
| history.jsonl unused | **Absorbed** F9–F11 / AC16–AC17 |
| Docs “no hooks” | **Absorbed** F20 |
| Quiescence no `--force` | **Absorbed** F18 / AC9–AC10 |
| Re-summarize after new turns | **Absorbed** F17 / AC13 (or T239 waiver) |
| T235 install / F34 map | Keep; regression AC14; **wrapper rewrite F8** |
| fullyIdle hard policy | Soft residual (F7) |
| conversations.db primary | Soft / not DoD |
| Grok/OpenCode/nightly multi | T237 Planning / T238 / T239 |

### T236 AI review fold-in (2026-08-08)

| Item | Disposition |
|------|-------------|
| AI1 affirm + serde/fail-open + re-summarize OR | Affirmed / F1 / F17 |
| AI2 M1 wrapper stdout | **Elevated** F8 / AC18 |
| AI2 M2 turn-id diverge | **Elevated** F2 / AC19 |
| AI2 M3 hook normalize | **Elevated** F3 / AC17 hook |
| AI2 M4 env hijack on live | **Elevated** F3(4) / AC20 / F33 |
| AI2 M5 transcript_full | **Elevated** F29 / AC21 |
| AI2 M6 source_meta path | **Elevated** F30 / AC22 |
| AI2 L1–L5 | F32 / F31 / F12 / F9 / F16 |
| AI2 L6 scheduled skip-import | T236 D16 honesty → **T239 D12:** SYSTEM **keeps** skip-import (user-schedule completeness); no silent re-enable |
| AI2 L7 watermark | Soft F34 |
| AI2 “plan.md missing” | Stale — plan.md present |

### T216 planning residuals absorption (2026-08-05) — historical

| Residual | Disposition |
|----------|-------------|
| forget list effect 5 (unbounded list-forgotten; no inventory skim) | **Absorbed then closed by T216** F1–F48 / AC1–AC20 |
| Counts by project | **Absorbed** F11 `--summary` (+ global by-project) |
| Counts by tag | Partial: `--tag` filter F12 hard (two-stage M2); histogram soft F13/F24 (content `TAGS:` only; no migration) |
| Tag schema / pin rewrite | **Not** T216 |
| Auto-forget / CE wipe / governed discovery / HTTP list | **Not** T216 |
| AI1 M1 exit-2 plumbing | **Absorbed** F3 `fail_usage` / `GovernedCliError` |
| AI1 M2 tag LIKE false-match | **Absorbed** F12/F41/F43 |
| AI1 M3–M7 / L1–L6/L8 | **Absorbed** F4/F6/F8/F9/F11/F15/F17/F22/F26/F36/F38 |
| AI2 core affirm | **Affirmed** F45 |

---

## From T142 — Ledgerful state-dir + product-name migration (2026-06-29)

### ~~1. Functional symbol rename: `ChangeGuardHotspot` and friends~~ — **Closed by T191** (2026-08-02)
- ~~Type/fn renames across safety, capture verification_gate, brain intervention, retrieval preflight/recall, symbol_bridge, nightly.~~
- **Resolved:** hard renames to `Ledgerful*` / `query_ledgerful*` / `ingest_*_from_ledgerful` / `refresh_ledgerful_index` / `query_symbols_from_ledgerful` (no deprecated type alias).

### ~~2. `source_tag: "changeguard:symbol"` dedup identity~~ — **Closed by T191** (2026-08-02)
- ~~Flip write tag alone would re-ingest duplicates.~~
- **Resolved:** dual-read (`SOURCE_TAG_SYMBOL_LEGACY` \| `SOURCE_TAG_SYMBOL`) + new writes `ledgerful:symbol`; T167 preserve path unchanged.

### ~~3. `CHANGEGUARD_TX_ID` in Docs/OPERATIONS.md env table~~ — Resolved (T142 closeout 2026-07-24)
- ~~`Docs/OPERATIONS.md` still listed only `CHANGEGUARD_TX_ID`.~~
- **Resolved:** env table documents `LEDGERFUL_TX_ID` (preferred) and `CHANGEGUARD_TX_ID` (deprecated alias).

### 4. `conductor/archive/**` and completed track specs
- Historical record; intentionally NOT rewritten in T142 per user preference.
- If full-purge of "changeguard" from the repo is ever desired later, a separate track can sweep the archive and complete track specs. Low priority.

### 5. Pre-existing `cargo audit` allowlist entry RUSTSEC-2026-0190
- `anyhow` unsoundness in `Error::downcast_mut()`. Currently in `deny.toml` allowlist (pre-existing).
- Monitor for upstream fix; remove allowlist entry once `anyhow` publishes a patched release.

### ~~6. `scripts/dev-check.ps1` PowerShell parse error~~ — Resolved (T146 + T147)
- ~~Reported by Track 1 worker as pre-existing; not investigated (out of T142 scope).~~
- ~~The script does not run at all due to a parse error.~~
- **Resolved:** T146 em-dash fix + T147 baseline re-verify. `powershell.exe -NoProfile -File scripts\dev-check.ps1 -CheckOnly` exits 0 under Windows PowerShell 5.1 (2026-07-24); tool pins bumped to nextest 0.9.140 / deny 0.20.2 / audit 0.22.2.

---

## From nightly investigation (2026-07-01)

### ~~7. Nightly scheduled task fails as SYSTEM~~ — Fixed by T143 (+ T145 live ACL)
- ~~T132 registered bare `ai-brains.exe nightly` as SYSTEM without env vars / flags.~~
- **Fixed by T143** (`c7585d3`, `634249e`): CLI generates wrapper with baked `AI_BRAINS_*` env, `--no-project-context --skip-import`, and `--dry-run` preview.
- **Hardened by T145:** wrapper lives under `%ProgramData%\AI-Brains\` with SYSTEM+Administrators ACL; live schedule verified 2026-07-21 (task Run As SYSTEM, Last Result 0).

### ~~8. Privilege escalation: SYSTEM executes user-writable binaries~~ — Addressed by T145
- ~~**Issue:** `--run-as-system` schedules a SYSTEM task that executes a wrapper script + binary, both in user-writable locations (vault parent dir, `C:\Users\RyanB\.cargo\bin\`). Any user-level process can replace either file and gain SYSTEM execution.~~
- ~~**Pre-existing:** T132 had the same risk (bare exe invocation as SYSTEM). T143 moved the wrapper to the vault parent (not `%TEMP%`) and added `cd /d`, but the underlying risk remains.~~
- ~~**Codex review:** Flagged as critical on two consecutive reviews. Reviewer won't clear without ACL hardening.~~
- **Addressed by T145** (`conductor/tracks/trackT145-system-task-acl-hardening/`): wrappers + `daemon.env` relocated to `%ProgramData%\AI-Brains\` with `icacls` `SYSTEM:F` + `Administrators:F` only; reparse/symlink refuse; ACL verified before `schtasks` register (fail closed). **Residual (accepted):** cargo-bin binary path remains user-writable — documented in OPERATIONS.md / review.md; packaging copy-to-ProgramData out of scope.

---

## From T147 — Governed Memory Baseline + Edition 2024 + Shadow (2026-07-24)

Squash-merged PR #17. Full gate green (fmt / clippy / nextest 426 / deny / audit / ledgerful verify). Claude cross-model **PASS**; Codex primary blocked by account usage limit until ~2026-07-28.

### 9. Optional Codex re-audit of T147 (process residual)
- Codex `exec` rate-limited during T147 closeout; Claude used as skill fallback (`review.claude.md` + `review.claude.round2.md` **PASS**).
- Optional: re-run Codex read-only track audit when quota resets and archive as `review.codex.md` for symmetry with T145. Not blocking.

### 10. Turn-derived `memory_id` non-determinism (fixture golden omission)
- Turn projector assigns `MemoryId::new()` per turn projection; golden export omits `memory_id` so R1 snapshots stay deterministic (T147-F4 accepted residual).
- Follow-up only if a later track needs stable turn→memory IDs (e.g. derive from event_id). Out of T147 scope.

### 11. `TempEnv` public API surface
- `ai_brains_core::temp_env::TempEnv` is always-public so dependent crates' integration tests can use it (T147-F7 accepted residual).
- Optional later: feature-gate via `test-util` if public surface becomes a concern. No correctness impact.

### 12. Shadow dry-run still opens source (no migrate)
- Dry-run / create opens the source vault read-only for event count and copy; does **not** call `migrate()` on source (T147-F5 fixed).
- May still create/touch WAL companions beside source under SQLite open. Acceptable for P0; full soft-canonicalize / handle TOCTOU remains P6.
- **T153 (2026-07-26) partial:** connector contract documents reparse/symlink refuse + locator normalize as implementor invariants; **no** soft-canonicalize implementation in T153 (T154+ path-bearing connectors).
- **T154 (2026-07-27) implementable slice done:** Markdown/Obsidian connector (`builtin.obsidian`) owns vault **containment resolve + reparse/symlink refuse** on list/observe/preview. `is_reparse_or_symlink` / `refuse_if_reparse` promoted into `ai-brains-path`; CLI is a thin wrapper. Residual check-then-open TOCTOU without `openat`/cap-std remains documented; shadow vault WAL soft-canonicalize is **not** claimed closed by T154 alone.
- **T168 (2026-07-29) planned:** `migrate governed` dry-run same honesty — open source RO for classify/report; never `migrate()` source; soft reparse re-check residual documented (does not claim TOCTOU closed).

---

## From T153/T154 connector port review (2026-07-27)

### 23. Connector `list()` has no cursor / page token (port-level)

- T153 shipped `Connector::list(&self, ctx) -> Result<Vec<SourceHandle>, ConnectorError>` with no limit parameter and no continuation token (sync port; no `Stream`).
- **T154 (done as v1 limitation):** `MarkdownObsidianConnector` hard-caps materialization with `max_files` (default 10_000) and exposes connector-local `last_list_truncated()`; T153 trait left unchanged.
- **T155 (2026-07-28) done (caps):** Git `max_handles` default **16**, Ledgerful `max_records` default **256**, both with `last_list_truncated` side-channels; contract/integration tests cover the 0-cap path. **Does not** implement port-level cursor / page token.
- **Residual:** true progressive list (cursor/token on still-sync API) when a consumer forces it (T156+ / briefing refresh). Do not silently grow unbounded `Vec`s.
- **T156 (2026-07-28) planned:** Hermes/Honcho connectors use small **max_handles** (default 256) + truncation flag; **no** port cursor.
- Related: design consistency with T152 progressive retrieval / budgets — document only until a consumer forces the change.

### 25. Circularity guard for external memory (T156)

- Master plan: items written by the control plane and read back from Hermes/Honcho must not become independent supporting evidence; preserve origin event/source lineage.
- **T156 planned:** pure `CircularityClass` + `may_count_as_independent_support` (only `Independent`); observe payloads embed `ExternalItemMeta`; fixture RED tests. Full wiring into every `propose_conclusion` path may remain partial if helper is proven and call sites documented.
- **Rule 3 locked (review 2026-07-28):** no origin marker + no `OutboundIndex` match → **`Unknown`**, never `Independent`. Paraphrase/summary from external memory defeats marker/byte matching; “no evidence of circularity” is not independence.
- **OutboundIndex:** test-seeded (and future export accounting); **empty in production v1** because track is read-only with no outbound recorder — do not claim two-layer production defense.
- **Independent:** only via explicit trusted assert/fixture path in v1, not classifier default.
- **Missing privacy on external items:** default **`Privacy::Sealed`** (most restrictive).
- **License:** Honcho OSS is **AGPL-3.0** — adapters must not depend on AGPL SDKs; fixture/export-first; optional our-DTO HTTP only.

---

## From T148 — Governed Domain, Events, Contracts (2026-07-24)

### ~~13. Known event-type tag registry triplication (INT-M2)~~ — Fixed by T150
- ~~`KnownPayload`, `is_known_payload_type()`, and `EventKind` each list known tags independently.~~
- **Fixed by T150:** `impl From<&Payload> for EventKind` + `EventBuilder` derives `event_type` from payload at `build` (mismatched pairs unrepresentable). Residual: `KnownPayload` / `is_known_payload_type` still list tags for serde (deserialize path); kind/payload construction coupling is closed.

### ~~14. `ConclusionMarkedStale` optional-only fields (INT-M3)~~ — Fixed by T149
- ~~Both `changed_source_version_id` and `unavailable_reason` may be `None` at type level.~~
- **Fixed by T149:** `ConclusionMarkedStalePayload::try_new` / `validate`; `EventBuilder::build` and store `append_event(s)` reject both-None. Additive optional `source_id` for source-specific unavailable revalidation.

---

## From T149 — Source / Evidence / Fingerprints / Invalidation (2026-07-25)

Squash-merged PR #20 (`4c2aec7` on `main`). Engineering DoD met; nextest workspace **557** at closeout. Codex rounds 1–2 code findings fixed; round 3 process-only.

### 15. `source_alias_projection` has no write path (T149-F6 / Codex-R2-P3-1)
- Migration `0020` creates table; replay truncates it; no event/projection INSERT.
- **Follow-up:** Source-alias UX track (or P4/P6 connector registry) — add `SourceAliasAdded`-style fact + projection when alias UX lands.
- Low; schema reserved intentionally.
- **T154 (2026-07-27) out of scope:** rename in Markdown/Obsidian connector = **new** vault-relative identity (not alias). Do not implement `SourceAliasAdded` in T154.

### 16. Verification-gate evidence ensure-source (T149-F10)
- Capture emits `EvidenceRecorded` for well-known `verification_gate_source_id()` without one-time `SourceRegistered` (F9 removed re-register spam).
- FK on `evidence_projection(source_id)` not enforced (`foreign_keys` off); graph synthesizes source nodes.
- **T150 F′ deferred (reaffirm):** ensure-once `SourceRegistered` not landed — store projection already inserts orphan source row on `EvidenceRecorded`; full ensure-source event remains low polish for a later track.
- Low integrity polish; not blocking P2/P3 acceptance.

### ~~17. Optional: single source of truth for event-type tags (#13 still open)~~ — Fixed by T150
- ~~Reaffirm #13; T149 added more kinds/payloads — drift risk slightly higher.~~
- **Fixed by T150** with #13 structural `From<&Payload> for EventKind` + builder derivation.

---

## From T150 — Epistemic Lifecycle + Review + Conflict (2026-07-25)

### 18. Session-summary synthesis not under `AI_BRAINS_GOVERNED_SYNTHESIS` (Codex P3-1 / F6)
- Hierarchical `MemorySynthesizer` flips to `ConclusionProposed` when flag on.
- Session-summary path in `ai-brains-brain/src/lib.rs` still always emits `MemorySynthesized` (graph edge provenance) and does not call `cloud_route_allowed`.
- Spec minimum “at least one path” met; full dual-path productization deferred.
- **T157 (2026-07-28) residual:** hierarchical + registry + EmbeddingService gated; session-summary dual-path remains optional residual.

### ~~26. Model provider registry privacy gate weaker than policy~~ — **Fixed by T157**
- `ProviderRegistry::select_provider` uses shared `cloud_route_allowed` / `privacy_is_local_strict` (LocalOnly|NeverInject|Sealed).
- `AI_BRAINS_ALLOW_CLOUD_EXTRACTION` default false; local-first among viable providers; structured reason codes.

### ~~27. Provider `is_local()` hardcoded; endpoint not classified~~ — **Fixed by T157**
- `classify_endpoint` (loopback-only local; LAN/remote = CloudApi); Ollama/LlamaCpp store classification at construct.
- Local-first selection; `deployment` derived from `endpoint_class` via `with_endpoint_class` / `from_provider`.

### 28. ModelProvenance dual-source fields (T157 Codex P3)
- Public `deployment` + `endpoint_class` can still disagree if hand-set or via malicious JSON; production paths use helpers.
- Optional follow-up: deserialize normalize or builder-only API (serde compatibility constraint).

### 19. Replay truncate omits legacy conflict/recipe/hierarchy tables (F8)
- Pre-existing: `rebuild_projections` does not DELETE `conflict_projection` / `recipe_projection` / hierarchy edges.
- T150 added epistemic truncate order only. Full FK-safe truncate of all projections = recovery track.

---

## From T151 — Scopes, Principals, Grants, Policy (2026-07-25)

### 20. Unresolved scope nil ProjectId sentinel (R1-F10 / Codex P3)
- `resolve_scope` returns `ScopeRef::Repository(ProjectId::nil())` with Low confidence when unresolved.
- Callers should use `ResolvedScope::is_authoritative()`; prefer `Option<ScopeRef>` API in a follow-up.
- **T158 (2026-07-28) protocol partial done:** contracts + daemon `ScopeResolvedResponse` is a **full wire mirror** of resolver utility — `authoritative`, `confidence`, `evidence[]`, `warnings[]`, `alternatives[]` (not a bare bool) so CLI/desktop can disambiguate.
- ~~**T159 handler map**~~ **T159 done:** `ai-brainsd` `services::map_resolved_scope` fills full wire response (authoritative false on Low/Ambiguous; evidence/warnings/alternatives preserved).
- ~~**T160 CLI scope surface**~~ **T160 done (2026-07-28):** CLI `scope resolve` maps full wire (`authoritative`/warnings/alternatives); local + daemon paths; low confidence forces `authoritative: false`. Scope-bearing list/inspect use CP scope filter (`list_open_review_items_for_scope`) — no vault-wide leak on local path.
- **Residual:** core `Option<ScopeRef>` cleanup remains follow-up (nil ProjectId sentinel).

### 28. Daemon protocol lacks governed op variants (T158)

- ~~**Live:** `DaemonRequest` is only Ping/Ingest/Sync/Shutdown~~ **T158 done (protocol):** additive request/response variants + contracts DTOs + legacy wire goldens.
- ~~**T159 handlers**~~ **T159 done:** real control-plane-backed handlers for all T158 governed ops; mutations single-writer; queries off-queue; spool only with `command_id`. Zero residual `UNSUPPORTED_OPERATION` for those ops.

### 29. Daemon live request dispatch duplicated (main vs windows_service) (T158 review)

- ~~**Live:** Near-verbatim `match DaemonRequest` loops~~ **T158 done:** `ai-brainsd/src/dispatch.rs` `handle_daemon_request` + `write_dispatch_result`; `main.rs` and `windows_service.rs` both call it. Spool replay in `lib.rs` stays separate (fire-and-forget; new variants delete/skip without panic).
- ~~**T159 constraint**~~ **T159 done:** single shared dispatch + `GovernedServices`; both hosts pass writer + services.

### 30. Daemon governed-handler residuals (T159 planning 2026-07-28)

- **Governed spool without `command_id`:** live-only (no durable spool) — clients that need crash durability must send `command_id`. Documented in OPERATIONS (T159).
- **Briefing daemon dry_run default:** daemon v1 always dry_run; non-dry briefing writes stay on **local CP** path (CLI already supports `dry_run=false`). T160 does **not** require daemon briefing writes unless free.
- ~~**Durable erasure ticket**~~ **T159 done:** `ErasureTicketAccepted` event before `accepted`; still **no** CE wipe / `ContentKeyId`. Residual: CE key destroy + projection purge — **design frozen in T162 / ADR-0016 Accepted** (per-unit DEK under DataKey; AES-256-GCM); implement **T163–T165**.
- ~~**Idempotent propose**~~ **T159 done:** control-plane detect-already-done when pre-assigned aggregate id is set; daemon derives ids from `command_id` via uuid v5.
- ~~**ResolveReviewItem scope on wire**~~ **T159 done:** additive optional `scope` + `command_id` on contracts.
- **Principal identity residual:** pipe ACL + optional principal_id / env; no multi-user federation.
- ~~**T160 CLI command_id / erasure / principal**~~ **T160 done (2026-07-28):** CLI auto-generates `command_id` on mutations; shared `id_from_command` + NS_* in control-plane; erasure always daemon-required with CE-wipe warning; principal_id wire on daemon path. Residual: CE wipe — **T162 ADR-0016 Accepted**; implement T165.
- ~~**T161 loopback bearer**~~ **T161 done (2026-07-28):** opaque **loopback bearer** authenticates local vault owner (not OAuth/IdP); principal_id still body/env for policy. **Multi-user federation remains residual** after T161.

### 31. T159 deferred P3 — spool retain e2e inject (Codex R4 2026-07-28)

- **Live:** Unit contract proves retriable `EventAppend`/`Query`/`Clock` map to `Err` so writer keeps spool; happy-path AC6 spool replay asserts event count = 1.
- **Gap:** No end-to-end harness that injects a failed `append_events` on the writer path and asserts the governed spool file remains on disk.
- **Why deferred:** Requires fault-injection/store mock at writer boundary; non-blocking (production composition correct); coverage optional hardening.
- **Owner follow-up:** daemon reliability track preferred; **T160 out of scope** unless free (do not block CLI surface).

### 32. CLI governed surface residuals (T160 2026-07-28)

~~Promoted from T160 expansion~~ **T160 CLI half done (2026-07-28)** — landed on `feature/T160-cli-governed-surface`:

- ~~**DaemonClient gap**~~ **done:** `DaemonClient::request` + timeout / ambiguous outcome flags.
- ~~**Path split**~~ **done:** queries/dry-run prefer local; mutations prefer daemon; local only pre-send-down or `--local`; no silent post-send fallback (`classify_daemon_mutation_error`).
- ~~**command_id derivation**~~ **done:** `id_from_command` + NS_* in `ai-brains-control-plane`; CLI local propose uses shared helper.
- ~~**erasure request**~~ **done:** always daemon-required (`--local` rejected); exit 5 on true daemon-down; CE wipe never claimed.
- ~~**Exit codes**~~ **done:** 0/1/2/3=POLICY_DENIED/4=NOT_FOUND/5=DAEMON_UNAVAILABLE/6=INVALID_PAYLOAD.
- ~~**source inspect SQL in CLI**~~ **done (R1-03):** `GovernedQueryStore::get_source` + `source_row_to_dto` in CP; CLI/daemon thin adapters.
- ~~**local review list vault-wide**~~ **done (R1-02):** shared `list_open_review_items_for_scope` / `review_item_matches_scope`.
- **Residuals after T160:**
  - **R1-05 test hygiene:** ambient `ledgerful-bridge` may answer Ping → erasure assert_cmd accepts exit 5 or 1; prefer pipe-name isolation when available.
  - **R1-06 coverage:** CP unit tests cover `id_from_command` determinism; hermetic local propose assert_cmd (grant + evidence seed → conclusion_id match) still optional.
  - ~~**T161 parity**~~ **T161 done (2026-07-28):** HTTP responses use raw `DaemonResponse` JSON (`type`/`payload` tags) matching IPC wire; mock + unit parity tests in `ai-brains-api-server`. Full live vault CLI/IPC/HTTP three-way golden optional residual.
  - **policy issue/revoke admin UX:** out of T160 (read-only show/check only).
  - **T152-R1-08** empty personal continuity: document only; no synthetic fill in CLI.
  - ~~**CE wipe / ContentKeyId:**~~ **T162 design (2026-07-28):** ADR-0016 **Accepted** freezes hierarchy/AEAD/legacy impossibility; schema T163, service T164, wipe command T165. Still **no** production CE.

### 33. Loopback HTTP residuals (T161 2026-07-28)

~~Promoted from T161 expansion~~ **T161 implementer complete (code + security suite):**

- ~~**Process model**~~ **done:** HTTP runs **in-process on `ai-brainsd`**; no separate multi-writer HTTP process in v1.
- ~~**Dispatch**~~ **done:** `HttpDispatch` port in `ai-brains-api-server`; `DaemonHttpDispatch` wraps `handle_daemon_request`. Pipe + windows_service + HTTP = three callers.
- ~~**Token SDDL**~~ **done:** `USER_TOKEN_FILE_SDDL = D:P(A;;FA;;;OW)`; apply-then-verify; not SY+BA.
- ~~**Stack**~~ **done:** axum 0.8.9 + tower-http 0.6.x (MIT); deny/audit gated.
- ~~**Auth / bind / CORS / body limit / constant-time**~~ **done** (security tests green).
- ~~**Default off**~~ **done:** `AI_BRAINS_HTTP=1` / `--http`.
- **Residuals after T161:**
  - **Non-goals residual:** OAuth/OIDC, mTLS, public bind hardening, OpenAPI UI.
  - **Multi-user federation / IdP:** still residual (#30).
  - **Host header rebinding check:** optional defense-in-depth not implemented (bearer + CORS deny + loopback bind are primary).
  - **Three-way live vault golden (CLI/IPC/HTTP):** unit/mock parity done; optional e2e residual.
  - **Windows LocalSystem service HTTP token (R1-02 residual):** Under the service host, bearer token is under the SYSTEM profile with owner-only ACL — not readable by interactive desktop clients. Documented in OPERATIONS; service logs strong warning when HTTP enabled; hard-fails on start error. Full multi-session shared token path out of scope.
  - **Parent `.ai-brains` dir owner-only ACL (R1-10):** plan said “if free”; not applied.
  - **Incomplete per-route dispatch tests (R1-11):** missing some decisions/evidence/sources route tests; architecture still thin adapters.
  - **Post-spawn HTTP serve death:** bind success returns Ok; runtime serve errors log only (no auto-restart).

### 21. Adapter `principal_binding` still None (R1-F11 / Codex P3)
- Full harness adapters declare governed reads/writes for Agent intent; `principal_binding` remains `None` until connector registry maps adapters to PrincipalIds.
- **T153 (2026-07-26) partial:** **source-connector** in-process registry binds `PrincipalId` on `ConnectorManifest` for Connector-kind policy. Harness `AdapterCapability.principal_binding` remains `None` unless a later track maps adapters explicitly (not required for T153 DoD).

### 22. Git discovery softens command errors to empty (Codex P3)
- **T155 (2026-07-28) done partial:** Git connector uses `collect_metadata_strict` / `collect_metadata_strict_with_timeout` (`SoftFailPolicy::Strict`): Timeout / Io / non-not-a-repo `CommandFailed` propagate as `Err(ConnectorError)` with `last_unavailable_reason` set (`timeout:` / `io:` / `command_failed:` prefixes). Genuine not-a-repository remains soft empty + side-channel (`not_a_repository`) with contract tests. Scope resolver and other legacy callers still use soft `collect_metadata` (`SoftFailPolicy::Soft`).
- **Residual:** Soft helpers (status/branch/commit under Soft policy) still degrade mid-collect failures to defaults for resolver/capture paths; only the connector path is strict. Progressive hardening of soft callers is optional follow-up.
- **T156 (2026-07-28) planned:** Hermes/Honcho connectors apply the same anti-silent-empty discipline (Err-first hard fail; soft empty + contract-tested `last_unavailable_reason`) — not git-specific, but same failure-mode class.

### 24. Git CLI interactive hang + process-tree kill (from T155 review 2026-07-28)

- **T155 shipped:** env guards on **every** git spawn (`GIT_TERMINAL_PROMPT=0`, no-op `GIT_ASKPASS`, `GCM_INTERACTIVE=never`, `SSH_ASKPASS_REQUIRE=never`) as primary defense; sync timeout + direct `Child::kill` as backstop.
- **Windows ASKPASS packaging:** prefer `scripts/git-askpass-noop.cmd` under crate manifest or next to `current_exe()/scripts/`; else fall back to `%SystemRoot%\System32\cmd.exe` (fail-closed; may non-zero exit — hang prevention still holds via env + timeout). Packaged installs should ship the `.cmd`.
- **Residual after T155:** `Child::kill` does not kill git descendants (`ssh.exe`, credential helpers, gpg). Full Windows Job Object whole-tree kill remains a **follow-up** (more important if daemon periodic refresh lands).
---

## From T152 — Briefings + Progressive Retrieval (2026-07-26)

Codex R4 **PASS WITH DEFERRED P3**. Internal R3+ CLEAN WITH DEFERRED LOWS.

---

## From T175 — Sync threat model + ADR-0018 (2026-07-30) — **Completed** (PR #46 / main@2a2eb60)

### 50. T175 design freezes → ADR-0018 **Accepted** (Codex R3 PASS)

**Completed** design-only track. Normative: `Docs/DECISIONS/ADR-0018-encrypted-event-replication-protocol.md` + `conductor/tracks/trackT175-sync-threat-model-adr/threat-model.md` (§7 matrix). **Implement sync only in T176+** under Accepted freezes.

- **Product:** encrypted **event envelope** replication + local projectors; **no** SQLCipher file sync; **no** default CRDT; **no** LWW; **single-owner / single-vault** (multi-user needs new ADR).
- **L1–L16** freezes include: dual-key enrollment fingerprint (Ed25519+X25519); enrolled-signer enroll/revoke; **DeviceId permanently retired** after revoke; complete outer `signed_bytes` (event_id, content_key_id, sorted wrap list); **control cleartext** signed payloads (`N=0`, not DEK-encrypted); data body `nonce(12)‖ct‖tag(16)`; **per-recipient** X25519+HKDF+AES-GCM DEK wrap (epoch KEK **not** v1 primary); topological apply; signed ACK round-trip (attestation residual — not wipe proof); gap (L13) + size-bucket padding (L14); PQ non-claim (L16).
- **Deps named only (T176):** ed25519/x25519-dalek **3.x**, **curve25519-dalek 5.x** transitive, hkdf **0.13**; **HPKE considered-deferred**; OpenMLS deferred; **zero crates in T175**.
- **Naming:** keep `ai-brains sync` + `ai-brains safety sync`; multi-device CLI = **`device`** + **`replicate`**; crate `ai-brains-sync`.
- **Migration:** T176 **`0027+`** (0026 = CE).
- **#34 split (not wholesale strike):** (1) ACK **design absorbed** (L7) — **implement T176–T178**; (2) DataKey rotation **direction** in ADR-0018 — **implementation residual remains open** (P11 hygiene); (3) historical.
- **Review:** Internal R2 CLEAN; Codex R1 FAIL→fix; R2 FAIL→fix; **R3 PASS** before Accept.
- **T176 expansion (2026-07-30):** track **Proposed / Expanded** — live crates.io pins (ed25519/x25519-dalek **3.0.0**, curve25519-dalek **5.0.0**, hkdf **0.13.0**, hpke **0.14.0** deferred); schema `0027_replication_state` + locks **R1–R30** in `trackT176-sync-crate-schema/spec.md`.
- **T176 AI fold-in (2026-07-30):** dual-layer DataKey+DPAPI (R6); revoke wrap DELETE (R23); hyphen fingerprint (R24); panic-free keygen (R25, no x25519 `getrandom`); HKDF `Some(&[])` for clarity (SHA-256 equivalent to `None` — AI2); `local` status + first-device self-sign + enroll ceremony; private-blob layout AAD 0x03; upsert wraps; drop content_hash; single erasure code 0x0012.
- **T176 implement (2026-07-31):** **Completed** on PR (see conductor). `ai-brains-sync` + migration `0027_replication_state` + CLI `device`/`replicate`. Membership via `DeviceEnrolled`/`DeviceRevoked` event log + `ReplicationProjection` (atomic bootstrap with private-key wrap). Codex **R3 PASS**.
- **#34.1 partial absorb:** schema + types + `erasure_ack_projection` + control encode/verify unit tests **in T176**; multi-device CE orchestration **T177 Complete** (C10–C11); **security proof / forged-ACK suite → T178** (Proposed / Expanded 2026-07-31).
- **T176 deferred lows:** package enrollment `schema_version` allowlist on enroll (ID-13) → absorbed T177; full WRAP golden KAT matrix → **T178** (expanded); relay push/pull → T177 Complete.

---

## From T176 — Sync crate + schema (2026-07-31) — residuals after Complete

### 51. T176 deferred P3 / follow-ups

- **ID-13 (low):** ~~package schema allowlist~~ **absorbed T177 F10/C13** (engine + C13 test).
- **Non-Windows private key export:** `--write-private-key` is Windows DPAPI-only; passphrase wrap for other platforms is optional follow-up.
- **Signer-must-be-enrolled:** ~~projector-path residual~~ **absorbed T177 F9/C8** (L8 pre-verify on relay apply).
- **#34.2 DataKey rotation:** still open (not closed by T176).

### 52. T177 Fake Relay Convergence — **Completed** (Codex R5 PASS)

- **Shipped:** AIBR wire codec; RelayPort Memory/File/Adversarial; ReplicateEngine (L8 pre-verify, multi-gap drain, F21 CE+ACK, durable outbox 0028); TwinVaults C1–C13/C15; CLI push/pull explicit `--fake-relay` only; store→sync prod edge.
- Fake-relay-first: no production network; AdversarialRelay handoff to T178.
- Convergence oracle: applied **event_id** sets + membership (+ CE/ACK); never SQL file equality; never LWW.
- Engine: L8 before verify; gap fail-closed + range re-fetch; ACK tick N=3; CLI only with explicit fake-relay config.
- Absorbs #34.1 ACK **over relay** (C10–C11), #51 signer+schema gates; leaves #34.2 open.
- Normative: `conductor/tracks/trackT177-fake-relay-convergence/{spec,plan}.md`.
- **AI1–AI2 fold-in (2026-07-31):** F1–F22 — wire codec A0 (`AIBR` hand-roll); `RelayPort` all `&self` + interior mutability; multi-gap drain (F19); CE tombstone → `destroy_content_key_wrap` (F21); revoked pre-verify; no `replicate sync`; C5 delay-not-delete; C15 seq-collision; TwinVaults + `assert_converged`; `AdversarialRelay<R>` for T178; deny+audit after store→sync prod edge.

---

## From T162 — Content-envelope crypto spike (2026-07-28)

### 34. P8 implementation residuals after T162 design freeze

~~Design open (algorithm / hierarchy / legacy CE impossibility)~~ **T162 complete + ADR-0016 Accepted 2026-07-28** (`Docs/DECISIONS/ADR-0016-content-envelope-cryptography.md`):

- Per content-unit DEK under vault `DataKey`; AES-256-GCM; random 96-bit nonce; AAD ≥ schema version + `content_key_id`.
- CE = destroy DEK wrap + purge FTS/embeddings/projections; ticket (`ErasureTicketAccepted`) and soft forget are **not** CE.
- Legacy plaintext in append-only log **cannot** claim CE.
- NIST SP 800-88r2 honesty: no physical media / offline-copy / pre-erase-backup claims.
- Prefer **zero new deps** (aes-gcm 0.10.3 workspace); no event envelope v2 for v1.
- **Review fold-in (2026-07-28):** DataKey wrap-nonce budget (vault-lifetime KEK, same AES-GCM random nonce as seals) documented as accepted residual + future rotation gap; NIST Purge non-claim self-documents **no FIPS-validated module** (RustCrypto).

**Still open (implementation tracks):**

- ~~**T163:** schema + projections; migration **`0026_content_envelopes_erasure`**~~ **done** (side stores + erasure/tombstone projections; rebuild retains key/blob stores; S13 CHECK + S14 no demote).
- ~~**T164:** `content_envelope` + crypto `content_key_store` APIs; KAT + tamper + zeroize~~ **done 2026-07-29:** seal/open + wrap/unwrap under DataKey; AAD AIBC; ContentDek zeroize; store integration destroy→cannot open; Codex R3 PASS.
- ~~**T165:** governed CE command state machine~~ **done 2026-07-29:** CE wipe path (contracts/daemon/HTTP/CLI); destroy DEK wrap + purge memory/evidence FTS; rebuild re-purge; E1–E16; Codex R3 PASS.
- ~~**T166:** class-based retention preferring CE for envelope classes~~ **Promoted 2026-07-29:** track **expanded** (`trackT166-class-based-retention/`) — class matrix, dry-run plan, apply+confirm, T165 wipe reuse; **Completed 2026-07-29** T166 PR (class retention plan/apply).
- ~~**Human accept ADR-0016**~~ **Accepted 2026-07-28** — T163–T165 Complete (product CE for envelope-backed under ADR §12).
- **Pre-erase backup residual:** physical fact remains; honesty in T165 wipe + T166 plan/apply warnings for CE candidates.
  - **T181 (2026-08-01 Expanded):** productize residual as **drill proof** (T181-E-01 pre-wipe backup still opens after live wipe) + `Docs/RECOVERY-DRILLS.md` honesty — does **not** eliminate offline copies.
- **DataKey rotation / wrap-nonce accounting:** future gap if volume or multi-device demands it (P8+ / P11) — not required for v1 CE.
  - **T175 (2026-07-30 Completed / ADR-0018 Accepted):** **direction** frozen; multi-device wrap keys are per-recipient ephemeral (O(1) seals) — better than vault-lifetime DataKey budget. **Implementation residual remains open** (P11 hygiene track) — do **not** treat as fully closed by T175.
- **P11 multi-device key tombstone / erasure ACK:** out of T162–T165.
  - **T175 design absorbed** (L7 signed ACK round-trip + attestation residual); **implement T176–T178** (ADR-0018 Accepted 2026-07-30).
  - **T176 (2026-07-31 Complete):** schema + types + `erasure_ack_projection` + control encode/verify + local projection APIs **done**. Multi-device CE orchestration / relay proof remains **T177–T178**.
  - **T177 Complete (2026-07-31):** C10–C11 tombstone + ErasureAck + timeout under fake relay.
  - **T178 expanded (2026-07-31):** security claim matrix + forged ACK / WRAP KAT — implement on go-ahead.

### 38. T166 design freezes absorbed into track (2026-07-29 expansion) — **Completed with T166**

Folded into T166 spec/plan (implement on go-ahead):

- Classes: `raw_turn`, `evidence`, `decision_approved`, `secret`, `review_trace`, `query_trace`, `memory_legacy`, `orphaned_envelope`, `unclassified`.
- **R1** dry-run default; apply needs confirm. **R2** CE only via T165 wipe. **R3** projection delete ≠ CE.
- **R6** no age auto-wipe of active approved decisions. **R7** nightly CE opt-in false.
- Reports: counts/sample ids only — no plaintext bodies. Prefer no migration (0027 only if forced).
- Zero new deps; Phase 8 rollup closes with T166 dry-run evidence.
- **Review fold-in (2026-07-29):** **R11** pinned holds; **R12** `RetentionApplied` on apply; **R13** stream A/B de-dupe (no double-count; future `subject_kind=turn`); **R14** terminal `updated_at` clocks; **R15** hierarchy parent resynthesis mark (auto, not review queue); **R16** orphan wraps at **7d** (not 24h).

### 39. T167 design freezes absorbed into track (2026-07-29 expansion + review fold-in) — **Closed 2026-08-16**

Importer shipped with T168 (`legacy_import.rs` + `migrate governed`). Track closeout 2026-08-16: no residual standalone importer. Historical L-locks:

- **L1** classify/dry-run default (full plan + plan_hash even on dry-run); apply needs confirm. **L2** no live-vault default (T168 owns CLI).
- **L3** uuid v5 domain ids; plan-determinism (not applied envelope event_id) is the contract. **L4** under-promote.
- **L5** forgotten exclude + two-pass cascade reasons (`forgotten_source` / `missing_source`). **L6** legacy ≠ CE.
- **L8/L18** raw `build_event` only — **no** `observe_source`; **no** non-existent `RecordEvidence` capability.
- **L9** always `DecisionProposed` + raw `ReviewItemOpened` (`NS_LEGACY_REVIEW`); no auto-Approved.
- **L12** envelope privacy only (no multi-input hedge). **L17** preserve `source_tag` (**#2**).
- **L19** optional `ImportOpts.default_scope` (else `missing_scope`) — not silent Personal.
- **L20** `LegacyImportApplied` on apply (plan_hash + counts; no bodies).
- **§5.4** EvidenceId prefers `memory_id`; DecisionId from event_id (not MemoryId cast).
- **§5.7** add `has_evidence` port (do not probe via `evidence_privacy`).
- **§6.1** canonical plan_hash (sorted ActionView without bodies).
- Zero new deps; CLI migrate = T168.
- Related **not** absorbed: #1 renames, #15 source_alias, #19 truncate, #18 live synthesizer rewrite.

### 40. Workspace dependency version hygiene (not T167)

AI2 inventory 2026-07-29: workspace pins lag crates.io for several crates. **Do not** bump inside T167 (L7 + Unrelated-Failures).

- Safe-later minor/patch (separate INFRA chore): `uuid` 1.13→1.24, `serde`/`serde_json`/`time`/`thiserror`/`tokio`/`regex`/`rusqlite` as verified.
- Breaking — dedicated track when needed: `sqlx` 0.8→0.9, `base64` 0.22→0.23, `tower-http` 0.6→0.7.
- Skip: `aes-gcm` 0.11 (CE residual), `argon2` 0.6-rc.

No `conductor/ISSUES.md` in tree; this deferred entry is the tracking note.

### 41. T168 design freezes absorbed into track (2026-07-29 expansion + review fold-in)

Folded into T168 spec/plan — implement on go-ahead (**T167 merged**):

- **M1–M15** core: dry-run default + confirm; T167 reuse; T147 safety; no plaintext; live dest/source gates; report path refuse; zero new deps; CE honesty; `--default-scope` → T167 **L19** (live); T160 exit codes (6 = INVALID_PAYLOAD; PATH_REFUSED → 1).
- **M16** content-based `migrate_source_fingerprint` (not shadow mtime). **M17** copy-events only when dest empty; re-apply import-only.
- **M18** mandatory `migrate-manifest.json` on confirm; re-apply requires fingerprint match (AI3). **M19** `--source-key`/`--destination-key` raw-key; DPAPI out of scope.
- **M20** envelopes only (no projection copy). **M21** 5k batch copy + stderr progress. **M22** event order = store `occurred_at, event_id`.
- **`--force-overwrite`** for explicit clean recreate (not silent wipe).
- Declined: INSERT OR IGNORE as default; AI2 claim that L19/L20 are phantom (they landed with T167); dropping exit 6.
- Absorbs **#12** honesty. Out of scope: #40, T169/T170, soft-canonicalize.

### 37. T165 design freezes absorbed into track (2026-07-29 expansion)

Folded into T165 spec/plan (implement on go-ahead):

- Dual path: `erasure wipe` (CE) ≠ `erasure request` (ticket) ≠ `forget` (soft).
- E2: never `ContentErased` without successful `destroy_content_key_wrap` + verify.
- Dry-run default true; execute requires `--confirm`; daemon-required.
- Purge FTS/embeddings/subject plaintext; ciphertext blob may remain.
- NIST SP 800-88r2 honesty: no Purge/Destroy claim; pre-erase backup residual in warnings.
- Zero new deps; resume rules for crash mid-wipe.
- **Review fold-in (2026-07-29):** **E13** multi-blob; **E14** verification = store wrap_absent (not independent AEAD open_fails post-destroy); **E15** dependents only via registered SourceId (T149 ports are source-keyed); **E16** post-commit `wal_checkpoint(TRUNCATE)` dual-tier (BUSY → warn; not VACUUM/Purge).

### 35. T163 design freezes absorbed into track (2026-07-28 expansion)

Folded into T163 spec/plan (not separate work items after T163 ships):

- Migration **0026** (not master-plan 0025 name).
- Side store vs event projection split (wrap/blob retained on rebuild; erasure/tombstone replayed).
- Wire shape: 12-byte nonce column + ciphertext\|\|tag BLOB; schema version 1; no plaintext DEK/content columns.
- Ticket (`ErasureTicketAccepted`) and soft forget do not write CE tables.
- Zero new crates; no `aes-gcm` 0.11 upgrade in schema track.

### 36. T164 design freezes absorbed into track (2026-07-28 expansion)

Folded into T164 spec/plan (implement on go-ahead):

- Crypto modules `content_envelope.rs` + `content_key_store.rs` (≠ SQL table); no rusqlite in crypto.
- AES-256-GCM via workspace **aes-gcm 0.10.x**; `Payload` AAD; 12-byte random nonces; 32-byte random ContentDek.
- Binary AAD: `AIBC` + kind + version + UUID ids (seal binds blob_id; wrap binds content_key_id).
- KAT via test-only fixed nonce; production never accepts caller nonces.
- ZeroizeOnDrop ContentDek; open prefers `Zeroizing<Vec<u8>>`; Debug redaction.
- Zero new deps; no aes-gcm 0.11 / hkdf / GCM-SIV / XChaCha in v1.
- **Review fold-in:** public `SealAad.blob_id` is **mandatory `Uuid`** (not `Option`); zero-byte bind test-only. Destroyed-wrap CE proof is store integration only; crypto unit tests empty/short wrap buffer, not SQL NULL columns.

---

## From T152 — Briefings + Progressive Retrieval (2026-07-26) (cont.)

### T152-R1-07 env-only governed_briefing flag
- `AI_BRAINS_GOVERNED_BRIEFING` env + API option only for this cycle; config-file `governed_briefing = true` not wired.
- Documented in OPERATIONS.md / preflight comments. Follow-up when config surface is unified.

### T152-R1-08 empty personal continuity + constraint scrape
- Personal continuity summary always empty (#18 session synthesis out of scope).
- Project constraints substring-scraped from conclusion statements (`CONSTRAINT:`/`INVARIANT:`) rather than typed constraint projection.
- **T227 closed (2026-08-11):** empty honesty + next-step half shipped (no synthetic fill). **#18** session-synthesis continuity fill and typed constraint projection remain residual.

### T152-R2-02 source_versions test strength
- Production populates source_versions from evidence rows; many tests use synthetic evidence UUIDs without projection rows → empty lists (correct). Strengthen with seeded evidence row assert.

### T152-R2-03 store rebuild column asserts
- Store rebuild test counts rows; control-plane rebuild fidelity covers ranking/scope/principal. Optional column-level store asserts.

### T152 optional lows
- Progressive privacy envelope has production combine path but no dedicated NeverInject→QueryTraceRecorded integration test (project/personal covered).
- Cache valid-time refilter does not re-run budget or conflict capability filter (grant VV miss mitigates grants).

---

## From T156 — Hermes / Honcho Read Adapters (2026-07-28)

### T156 anti-#22 discipline applied
- Hermes/Honcho connectors: disabled → soft empty + `connector disabled`; hard unavailable → soft empty + reason, observe Err; invalid JSONL path load → Err (not silent empty). Contract-tested.

### #23 list cursor residual (reaffirm)
- Hermes/Honcho use `max_handles` default 256 + `last_list_truncated` (same pattern as T154/T155). Port-level progressive list cursor remains out of scope.

### OutboundIndex production empty (honest residual)
- Rule 2 (fingerprint/event match against outbound index) is test-seeded only. Production v1 has no outbound export recorder; fail-closed Unknown is the live unlabeled-content guard. Future track may seed OutboundIndex if write-back/export accounting lands.

### Circularity Independent only via assert path
- `classify_circularity` never returns Independent. Positive provenance / provider attestation for Independent is future work (out of T156).

### Live HTTP Honcho/Hermes
- Optional future; fixture/export first. No AGPL Honcho SDK; deny posture unchanged.

### Control-plane support-graph wiring
- `may_count_as_independent_support` / `filter_independent_support` live in `ai-brains-sources`. Call sites on `propose_conclusion` / support graphs deferred (document only; avoid sources↔control-plane cycle).

### 42. T168 deferred P3 (2026-07-29 Codex PASS WITH DEFERRED P3)

- **P3-01** No integration test for ≥1000-event copy progress stderr (M21 impl present; 5k batch covered by unit structure).
- **P3-01b** Golden totals from fixtures/governed-memory/legacy-v1-events.ndjson not wired; migrate tests use init+pin fixtures.
- Not DoD blockers; residual test completeness only. Track still Complete on engineering DoD + Codex PASS WITH DEFERRED P3.
- **T169 note:** optional later seed reuse of legacy NDJSON — not required for v1 10-pack synthetic scenarios.

### 43. T169 design freezes absorbed into track (2026-07-29 expansion + AI1–AI3 fold-in)

Folded into T169 spec/plan (`trackT169-governed-evaluation-corpus/`) — implement on go-ahead:

- **E1–E26** locks: hermetic per-scenario temp vaults; trust-first hard gates (`stale_as_current=0` via **E9a** warning×current cross-ref, `unauthorized_scope_leakage=0`, `cross_project_leakage=0`); **E23** anti zero-recall `min_valid_claims_count`; no LLM-as-judge/network; zero new Rust crates; no AGPL; optional Python stdlib only; scenario schema v1 + typed seed params.
- **report_hash:** exclude `created_at`, all `latency_ms`, any `generated_at`; path-normalize path-like fields; sort scenarios by id.
- **Exit codes:** `0` pass; **`1` EXIT_INTERNAL** (harness/path broke); `6` INVALID_PAYLOAD; **`7` HARD_GATE_FAILED** (trust gates failed — T170/T185 branch); `--strict-soft` → 7. Soft-only default still exit 0.
- **10 scenarios:** 1–9 CP Rust seeds; **5** = personal deny + Project-Alpha/Beta isolation; **8** = in-process `wipe_content_envelope` (no daemon); **9** = path alias → same scope_key; **10** = **sources-crate tests** (CP does not depend on sources).
- **v1 seeds:** Rust programs only (**E24**); **no** required T168 redacted-shadow vault seeds (T170 later).
- **human_review_seed:** ≤20 claim ids sorted by `(scenario_id, claim_id)`; all warning ids sorted.
- **Research stance:** outcome-based system asserts (not transcript LLM-judge); LoCoMo/etc. not CI hard gates; DeepEval/Ragas/Promptfoo not product deps.
- **Out of scope:** #40 dep bumps; soft-canonicalize; #18 session-summary dual-path; CP→sources dep.
- **T170 owns:** redacted-shadow dogfood (E24 follow-on), human 20-claim review, flag rollback drill, stop-before live enablement.

### 44. T170 design freezes absorbed into track (2026-07-30 expansion + AI1–AI3 fold-in)

Folded into T170 spec/plan (`trackT170-shadow-dogfood-gate/`) — implement on go-ahead:

- **D1–D26** locks: live never mutated; stop-before live without approval; Stages A→B→C→D; T169 exit **0** (7 product / 1 tool); redact shadow default; human ≥20 claims + all **risk** warnings as **`(kind, subject_id)` refs** (no warning id on DTO); flag rollback primary; no auto-enable; no AGPL/SaaS.
- **D21/§8:** rollback observability via `preflight --format json` `(governed)` probe + **`briefing project --format json`** for authority — **never** `preflight --summary` for governed (legacy marker scrape → false zeros).
- **D26 (critical):** compare uses global **`--vault-path`** to shadow/migrated — **never** set `AI_BRAINS_VAULT_PATH` to shadow (would break `resolve_live_vault_path` live refuse).
- **D24:** live vault file SHA-256 pre/post must match. **D23:** User-env emergency clear documented; scripts never set User scope. **D25:** Stage D min observation (1 session or ≥3 governed invocations).
- **D15:** Stage B = T169 seed; Stage C = stratified sample from governed packet (Decision/Conclusion, up to 5 each then fill to 20).
- **Compare sources:** governed = `briefing project --format json`; legacy = `preflight --format json` flag off + marker counts (not typed claim_count).
- **Stage C source:** prefer **operator test vault**; active user vault allowed with documentation.
- **Absorbs:** T152-R1-07; #12 honesty; T169 exit 7 + human_review_seed; E24 shadow dogfood.
- **Out of scope:** #40; config-file flag; soft-canonicalize; reverse-migrate; Stage D automation; optional polish to make `--summary` governed-aware.

### 45. T171 design freezes absorbed into track (2026-07-30 expansion + AI1–AI3 fold-in)

Folded into T171 spec/plan (`trackT171-desktop-tauri-scaffold/`) — **shipped / Completed** on main (scaffold live under `apps/desktop`).

- **S1–S24:** adapter-only; Tauri v2; stack **Vite 8 + TypeScript 7 + React 19 + npm/package-lock + engines node≥22**; workspace after Windows smoke; no AGPL/GPL.
- **S7 CSP (critical):** non-null; must include `connect-src ipc: http://ipc.localhost` (+ asset/customprotocol defaults) or invoke breaks; SC4 asserts value not key-only.
- **S8:** strip template capabilities; **`AppManifest::commands`** allowlist in `build.rs`.
- **S6:** cargo deny + **tauri-apps org provenance** for `tauri*` crates (crates.io typosquat/TrapDoor-class risk).
- **S23:** npm license via **license-checker-rseidelsohn** or evergreen fork — **not** abandoned `license-checker`.
- **S21:** WebView2 missing → clear dialog (rare on Win10 1803+/Win11).
- **S22:** optional `get_daemon_connection_info` over invoke only.
- **S24:** gitignore node_modules/dist/target/gen.
- **Smoke:** static `ping`; optional Rust probe **`/health` or `/v1/health`** — **not** `/v1/ping` (route absent).
- **Promoted to T172 (2026-07-30 expansion):** single-instance plugin (soft); full DTO surface (hand-sync default; specta optional soft); product screens.
- **Promoted to T173 (2026-07-30):** Isolation Pattern; deep shell/fs → scoped opener; CSP tighten; full a11y.
- **Absorbs:** T161 SYSTEM token honesty; capture independence; no analytics.
- **Out of scope (T171):** Electron; weakening T161 CORS; adding `/v1/ping` in T171.

### 46. T172 design freezes absorbed into track (2026-07-30 expansion + review fold-in)

Folded into T172 spec/plan (`trackT172-desktop-minimum-screens/`) — implement on go-ahead:

- **M1–M24:** adapter-only; **invoke → Rust reqwest → T161 `/v1`** (no webview fetch); user-session token only in Rust; HashRouter (**M14a** confirm v8 import path); E1 empty/denied/offline; AppManifest expansion; erasure ticket≠wipe honesty.
- **M23 (review Medium):** TanStack Query v5 default 3× ~7s retry **forbidden** for `offline`/`denied`. Structured Rust `kind`; QueryClient `retry: false` or transient-only. SC2a/SC2b prompt offline/denied.
- **M24 (review Medium):** T171 prod CSP has no Vite HMR/`unsafe-inline` style headroom. Dev-only CSP relaxation allowed; **never** ship in `tauri build`. SC16.
- **Stack add-ons (research 2026-07-30):** `react-router` **8.x** (MIT), `@tanstack/react-query` **5.x** (MIT), `lucide-react` **1.x** (ISC); optional soft `@xyflow/react` **12.x** (MIT); workspace `reqwest` 0.13 (plaintext loopback; rustls default N/A).
- **Live map verified:** T161 routes as listed; connectors/retention/grants-list/`/v1/ping` absent; RetentionPlanReport contract without HTTP.
- **Priority:** Home + Review first; propose forms + xyflow + specta + single-instance (~2.4.x tauri-apps) = soft.
- **Absorbs from #45:** single-instance (soft); hand DTO types (specta soft); product screens.
- **Absorbs residuals:** #20 scope `authoritative` honesty; T165 dual-path erasure warnings; T161 CORS/SYSTEM token reaffirm.
- **Promoted to T173 (2026-07-30 expansion):** Isolation Pattern; safe open; confirm+impact polish; keyboard review; further prod CSP harden; single-instance soft (if still missing post-T172).
- **Promoted to T174 (2026-07-30 expansion):** Playwright / visual states / offline beta gate — see #49.
- **Out of scope (T172):** new T161 routes; in-process GovernedServices default; Electron; CORS weaken; prod CSP weaken; #40 unless forced.

### 47. T173 design freezes — **Completed** (merged 2026-07-30, PR#44 / main@022f990)

Absorbed and shipped in T173 (Codex R2 PASS WITH DEFERRED P3). See §48 for residual P3 only.

### 48. T173 deferred P3 residuals (2026-07-30 Codex R2)

- **Live WebView2 Isolation + full keyboard GUI smoke** → **Absorbed by T174 L3/L4 automation + residual L5** (see §50). Structural + automated Escape/WIPE/source/stale evidence shipped; live daemon WebView2 still operator-once.
- **Isolation hook cannot deny IPC** — hygiene/audit pass-through only (C13 honesty). Tests must not claim denylist. Remains documentation residual after T174.
- **Path capability `"**"` breadth** — intentional for vault locators; Layer-1 still refuses empty/`..`/device forms; Layer-2 mirrors default.json. Not a T174 scope change.
- Not T173 DoD blockers. Engineering DoD + dual-layer opener + typed WIPE + Isolation mandated shipped.

### 49. T174 design freezes — **Completed** (merged 2026-07-30, PR#45 / main@7f3dd91)

Folded into and shipped by T174 (`trackT174-desktop-tests/`) — Codex R2 **PASS WITH DEFERRED P3**:

- **D1–D27 / DT1–DT20:** L1 Rust → L2 Vitest+RTL+mockIPC → L3 Playwright renderer → L4 ARIA primary + pixel secondary → L5 live WebView2 human residual.
- **Tool freeze (installed):** vitest **4.1.0** MIT; @playwright/test **1.62.0** Apache-2.0; RTL **16.3.0**; jest-dom 7 / user-event 14; jsdom **30.0.0**; soft axe not added.
- **AI1 B1–B16 folded:** license:check **production-only**; crypto + dialog polyfills; `context.addInitScript`; visual pins; Node ≥22; build+preview webServer; HashRouter `gotoRoute`; clearMocks+restoreAllMocks; source locator; user-event; vite `test:` block; httpmock 0.7; gitattributes binary snaps.
- **Absorbs #46.** #48 live residual → §50. Out of scope at ship: multi-OS visual/WDIO (→ **§56 / T179**); hard axe gate; Electron; prod CSP weaken; AGPL tools; httpmock 0.8.
- ~~**Multi-OS visual / WDIO matrix residual**~~ → **Promoted to T179 expansion (2026-07-31)** — see §56 / `trackT179-compatibility-matrix/` (T2 desktop note; not hard multi-OS e2e gate).

### 50. T174 deferred P3 residuals (2026-07-30 Codex R2)

- **Live WebView2 Isolation + real daemon smoke** — operator once before release packaging (`evidence/SMOKE.md`). Not PR merge blocker; L1–L4 offline gates green.
- **Full keyboard-only GUI tab traversal in live WebView** — L3 Escape/Enter/WIPE covered; live residual.
- **Soft D8 progressive AppManifest command-shape matrix** — soft; key httpmock coverage present; full matrix residual.
- **Pixel wipe PNG cross-host drift** — advisory; ARIA primary.
- Product honesty fix shipped in T174: non-https URI schemes classify as display-only `text` (never Reveal/Open).

### 44. T169 deferred P3 (2026-07-30 Codex PASS WITH DEFERRED P3)

- **F-009 / P3-01** Seeds use `SystemClock` (wall time) rather than a fixed `Clock` port. Event timestamps are not included in `report_hash` (stable uuid-v5 claim ids + strip latency/created_at). Residual only if validity-window edge flakiness appears.
- **P3-02** (closed in docs commit) `must_be_absent_present_count` metric documented in GOVERNED-MEMORY-MVP.
- Not DoD blockers. Track Complete on engineering DoD + Codex PASS WITH DEFERRED P3.
- Residual domain honesty (not T169 blockers): T156 OutboundIndex empty in prod; scen 8 authority absence uses post-wipe `reject_conclusion` + verified wipe status (CE does not auto-drop non-source claims).



### 53. T177 residuals after Complete

- **#34.2 DataKey rotation:** still open (not closed by T177).
- ~~**T178:** full threat-model §7 claim matrix; WRAP KAT; adversarial meta-swap / forged ACK suite (AdversarialRelay exported).~~ → **Promoted to T178 expansion (2026-07-31)** — see `trackT178-sync-security-tests/{spec,plan}.md` (Proposed / Expanded; implement on go-ahead).
- **CLI bootstrap→outbox:** first-device bootstrap does not auto-enqueue DeviceEnrolled to replication_outbox; convergence uses engine seal / OOB enroll. Optional follow-up: enqueue signed controls from device CLI.
- **C14:** optional FileFake twin smoke not required; unit file_relay tests present.

### 54. T178 Sync Security Tests — **Completed** (2026-07-31)

Shipped: F1–F28 suite; F23 `tests/common/twin_vaults`; F19 expanded snapshot; F20 static+seeded WRAP KATs (`pub(crate)` seed helper); F21 capture Cargo.toml gate; F22 replay; F24 dual forged-ACK; F25 body flip; F26 OPERATIONS multi-device residuals; F27 honesty scanner; multi-device revoke **omit** wraps; revoke-past AEAD open proof. Ledger TX `87b2f538`. Internal R2 CLEAN WITH DEFERRED LOWS; Codex R1 FAIL→fix; R2 engineering verified; final Codex R3 after closeout.

- **Absorbed:** #53 T178 handoff; T176 WRAP golden residual; #34.1 security proof.
- **Still open:** **#34.2 DataKey rotation** (not closed).

### 55. T178 residuals after Complete

- **#34.2 DataKey rotation:** still open.
- **IR1-L1 / R2-L1:** L3 ceremony fingerprint “reject” is structural package-hash binding; raw `insert_device_identity` accepts caller-supplied fp (production OOB recomputes).
- **IR1-L1:** R-ack-attestation behavioral pin thin (doc scanner primary).
- **CR2-P3:** F21 parses capture `Cargo.toml` only (not full transitive cargo-metadata graph); `cargo tree -p ai-brains-capture` confirmed no sync edge at ship.
- **L10 / L15 / HPKE / PIN / CAVP / pre-erase backups:** explicit product/formal defers (unchanged).

### 56. T179 Compatibility Matrix — **Completed** (2026-08-01)

P12.1 shipped on `track/T179-compatibility-matrix` / PR #51. GHA run **30683807812** all gates green.

- **Landed:** `Docs/COMPATIBILITY.md`; CFG inventory; `.github/workflows/ci.yml`; `scripts/dev-check.sh`; Unix hygiene; Phase F CI fixes (T80 hermetic pin, WSL map-before-soft-resolve, macOS path canonical compare).
- **Absorbs:** T174 multi-OS residual as T2 desktop; PRD secondary Ubuntu/WSL; SQLCipher honesty.
- **Residuals (not DoD blockers):** F26 release SHA-pin → **T185**; rust-toolchain multi-target expand → Low; arm64 T3; Unix CLI→HTTP-only not DoD; hermetic helper suite → **T186**; #34.2 still open (unrelated).
- ~~**T180 protocol compat**~~ → **Promoted to T180 expansion (2026-07-31)** — see §57.
- **Out of scope (unchanged):** App Store/notarization/MSI; SQLCipher flip as DoD; Electron; AGPL CI.

### 57. T180 Protocol Compat Tests — **Completed** (2026-08-01)

P12.2 shipped: `Docs/PROTOCOL-COMPAT.md`; elevate T158; additive helper; honesty/CLI/HTTP/EVENT suites. Internal R2 PASS; Codex R1 **PASS** (0 findings).

- **Residuals (open, not blockers):** F36 runtime api_version enforcement; F35 single API_VERSION SOOT; F24 binary N−1 post-release; F34 optional jsonschema; serde_json minor pin → **T185** / T183 handoff notes.
- **Out of scope (unchanged):** Infinite history; third-party clients; multi-OS; #34.2; OpenAPI DoD; Upcast migrations as DoD.

### 58. T186 Hermetic CLI / Multi-OS Test Hygiene — **Completed** (2026-08-01) — see also §64

P12 residual after T179 multi-OS GHA. Shipped: shared hermetic helper; ambient denylist; priority+soft suite migration; soft-canonicalize KAT expansion; GHA `--profile ci` (no-fail-fast); wall-clock docs; R-CI-PIN PR `ci.yml` SHA pins.

- **Out of scope (unchanged):** platform tiers; T180; T181 productization; #34.2; R-CI-BRANCH (admin); full long-tail rewrite.
- ~~**Long-tail residual (L13):** 25 `cargo_bin` sites / 5 files inventoried — not DoD blockers.~~ — **Closed by T191**

### 59. T181 Backup Recovery Drills — **Completed** (2026-08-01)

P12.3 implemented. Normative: `conductor/tracks/trackT181-backup-recovery-drills/{spec,plan,review}.md` (**F1–F48**, AC1–AC11). Playbook: `Docs/RECOVERY-DRILLS.md`.

**Shipped:** automated R/K/E/F drills; `assert_no_secret_leakage` (hex/base64/url-safe/raw/Debug byte forms); CE pre-erase residual productized as E-01 (physical residual remains); kit library-only honesty; dual-mode wrong-key residual when `rusqlite` is plain `bundled` (not SQLCipher). Internal R2 mediums fixed; Codex R1 P2 `fs::copy` fixed; full nextest 1704 green; deny/audit green.

**Residuals remaining (not fixed by T181):**

1. ~~No `recovery export` CLI~~ — **Closed by T188** (2026-08-02): `ai-brains recovery export`
2. ~~No `doctor` CLI~~ — **Closed by T192** (2026-08-02) PR #75 `80837da`
3. ~~Argon2 KDF params not in kit JSON (F37)~~ — **Closed by T194** (2026-08-02)
4. ~~#34.2 DataKey rotation~~ — **Closed by T189**
5. F-REC-03/04 projection/graph rebuild drills — soft
6. ~~Restore hard-fail while daemon running~~ — **Closed by T188** (robust probe + hard-fail)
7. Optional intermediate-hex zeroize tighten in `from_data_key` — soft crypto
8. ~~**Wrong-key / K-06 fail-closed requires SQLCipher page encryption**~~ — **Closed by T187** (2026-08-02): live `bundled-sqlcipher-vendored-openssl`; strict F-02/K-06; Deviations §1 resolved; R-F8/R-K06 claims flipped
9. Low: rstest preference for F-matrix; store Online Backup mirror vs BackupService; duplicate dry-run smoke/recovery_drills

**Absorbed (productized, not eliminated):** §34 / T162–T166 / T178 pre-erase backup residual → E-01 drill + docs honesty.

### 65. T187 SQLCipher Page Encryption — **Completed** (2026-08-02)

Post-P12 residual: live SQLCipher page encryption.

**Shipped:** workspace `bundled-sqlcipher-vendored-openssl`; plain-header sniff + `LegacyPlaintextVault`; `vault encrypt` via `sqlcipher_export`; zero-key refuse + `AI_BRAINS_ALLOW_ZERO_KEY`; `SqlCipherKey::validate`/`is_zero`; keyed `run_backup` source; strict recovery drills; Perl prereq docs/CI; claims/docs flip.

**Residuals (not DoD blockers):**
- `cipher_integrity_check` on backup verify (soft / out of scope)
- Zero-key escape hatch honesty remains (**R-ZERO-KEY** residual language)
- Windows MSVC Perl PATH hygiene on developer machines (documented)
- #34.2 DataKey rotation still **T189**

---

## From T182 — Connector Sandbox Decision (2026-08-01)

### 60. T182 Connector Sandbox — **Completed** (2026-08-01)

P12.4 complete. Normative: [ADR-0019](../Docs/DECISIONS/ADR-0019-connector-sandbox-execution-model.md) **Accepted**; companion threat-model + track specs under `conductor/tracks/trackT182-connector-sandbox-decision/`. Internal R2 PASS; Codex R3 **PASS WITH DEFERRED P3** (easy P3 fixed); full gate 1708 passed.

**Locks frozen (ADR-0019 L1–L10):** v1 = `TrustedBuiltin` only; two-layer serde+registry defense; no native DLL load; no AGPL host; future third-party = subprocess (OS Job Objects / Landlock / sandbox profiles) first, then WASI with **two-crate** `wasmtime`+`wasmtime-wasi` pin, FilePerms re-verify, Extism lag honesty, tokio/sync tension; zero prod Wasmtime/Extism/cap-std deps.

**Soft R1-06 shipped:** layer-1 serde unknown sandbox (`Subprocess` / `UntrustedExternal` → `ManifestError::Json`); layer-2 `#[cfg(test)] SandboxMode::TestUntrustedPlaceholder` → `RegistryError::SandboxNotAllowed`.

**Residuals remaining (not fixed by T182):**

1. ~~**#12** path TOCTOU / openat / cap-std residual~~ — **Closed-with-residuals by T190** (2026-08-02)

2. **CloudOk** constructible-unused / registry does not enforce trust label — future feature-flag non-LocalOnly
3. List cursor **#23** — out of scope (consumer-driven)
4. Plugin host (subprocess / WASI) — future track under L7/L8 gates
5. Harness `AdapterCapability.principal_binding` residual — out of scope
6. Optional pin via `ai-brains pin` for ADR-0019 — soft

**Absorbed (productized / locked, not eliminated):** T153 R1-06 (soft tests); T154 cap-std as builtin hardening candidate (not plugin sandbox); vision §7.2 subprocess-first as L7.

---

## From T183 — Release Documentation Pack (2026-08-01)

### 61. T183 Release Documentation — ✅ **Completed** (shipped 2026-08-01)

P12.5 complete. Normative: `Docs/README.md`, `Docs/INSTALL.md`, `Docs/SECURITY-LIMITS.md`, root `SECURITY.md`, `CHANGELOG.md`, elevated F8 rewords, track `evidence/*`. Review: internal R2 + Codex R1 content clean; final Codex R2 as publish gate.

**Absorbed / productized (documentation):**

| Source | Outcome |
|--------|---------|
| T179 HANDOFF + F8 | INSTALL locks + elevated SQLCipher honesty |
| T180 protocol honesty | INSTALL upgrade notes + PROTOCOL-COMPAT links |
| T181 doctor / recovery export | Documented **absence** (DTO ≠ CLI); RECOVERY-DRILLS |
| T182 ADR-0019 | Cited in SECURITY-LIMITS non-claims |
| Implementation-Plan §8 phantoms | Drift banner |
| status.md staleness | Demoted historical |
| OPERATIONS “17 subcommands” | Banner replaced |
| Missing Docs index / CHANGELOG / SECURITY | Created |

**Residuals (open, not T183 blockers — hand to T185 / future):**

1. Formal claims gate re-grep elevated docs + CLAIMS-CROSSCHECK consumption — **T185**
2. Version-banner CI sync — **T185**
3. MSI / notarization / App Store packaging — **T185**
4. Historical SQLCipher wording outside AC7 elevated set (`AGENTS.md`, PRD body, archives) — soft T185 re-grep
5. Implement `doctor` / `recovery export` product CLIs — future (honestly documented as absent)
6. #34.2 DataKey rotation; systemd/launchd production units; CONTRIBUTING.md; Common Changelog; T186 suite — unchanged out of scope

**Evidence handoff for T185:** `conductor/tracks/trackT183-release-documentation/evidence/CLAIMS-CROSSCHECK.md`

---

## From T184 — Independent Security Review (2026-08-01)

### 62. T184 Independent Security Review — **Completed** (2026-08-01)

P12.6 executed. Normative: `conductor/tracks/trackT184-independent-security-review/{spec,plan,charter,residuals,review}.md` + `evidence/`.

**Shipped remediations:** pipe SDDL World→SY+BA+IU (F-1 High); UDS post-bind `0o600` (F-2); SECURITY-LIMITS/OPERATIONS honesty; CI `permissions: contents: read` + Dependabot; SECURITY.md 90-day disclosure.

**Residual handoff (cite IDs in T185 claims):**

| Residual | Follow-up |
|----------|-----------|
| R-12, R-34.2, R-F8, R-K06, R-CE-PRE, R-WAL-CKPT | Product honesty (prior tracks) |
| R-ACK, R-META, R-PQ | ADR-0018 |
| R-MULTI, R-PIPE-IU, R-UDS-TMP | Multi-user Interactive residual after F-1 |
| R-HTTP-SYS, R-DOC-CLI, R-TB, R-CLOUDOK | Prior honesty |
| R-API-VER, R-BRIDGE, R-DTO-GOLDEN | Protocol honesty / T185 |
| R-CI-PIN | **Closed (T186)** — PR `ci.yml` + release.yml full SHA pins |
| R-CI-BRANCH | Repo admin — enable branch protection on `main` |
| R-CI-SAST | Optional later (clippy ≠ SAST) |
| R-SLSA | **T185** provenance axis |
| R-ZERO-KEY, R-DESKTOP-OPEN, R-AUDIT-UNMAINT | Low/Info accepted |

**Closed in T184:** R-DISCLOSURE-TL, R-CI-PERM, R-CI-DEPBOT (and corrected R-CHANGELOG-PATH to root `CHANGELOG.md`).

**Out of scope remains:** full multi-OS pentest; ASVS/SOC2 certification; doctor/export/DataKey rotation product work; SBOM packaging (T185).
---

---

---

## From T185 - Claims + SBOM Release Gate (2026-08-01)

### 63. T185 Claims + SBOM Release Gate — **Completed** (2026-08-01)

P12.7 executed. Normative: `conductor/tracks/trackT185-claims-sbom-release-gate/{spec,plan,review}.md` + `evidence/`.

**Shipped:**
- `Docs/RELEASE-CLAIMS.md` — claim/non-claim, full residual cross-walk (L3), “what we don’t ship”
- `Docs/RELEASE-CHECKLIST.md` — ordered gate + dry-run human sign-off
- Scripts: `generate-sbom.ps1/.sh`, `generate-notices.ps1/.sh`, `check-release-claims.ps1`, `check-version-banners.ps1`, `generate-checksums.ps1`, `dev-release-check.ps1`
- Committed `about.toml` + `about.md.hbs` (+ default `about.hbs`); CycloneDX **1.5** per-binary via cargo-cyclonedx **0.5.9**; cargo-about **0.9.1** `--features cli`
- Soft `.github/workflows/release.yml` — SHA-pinned actions; soft `actions/attest` (L1-oriented; no L3 claim)
- Impl-Plan §17 F8-honest vault storage; ci-tooling pins; dry-run archive under `evidence/dry-run-2026-08-01/`

**R-SLSA disposition:** release workflow may emit GitHub Artifact Attestations (Build L1-oriented fields auto-populated). **Not** SLSA Build L3 / certified. Dry-run did not publish attestations.

**Absorbed (closed as T185 process work):**
| Source | Item |
|--------|------|
| §56 T179 | F26 release-workflow SHA-pin; platform smoke rows on checklist |
| §57 T180 | Protocol honesty language in RELEASE-CLAIMS |
| §61 T183 | CLAIMS-CROSSCHECK consumption; elevated re-grep script; version-banner; soft historical re-grep |
| §62 T184 | Residual full cross-walk; R-SLSA axis honesty |
| HANDOFF-T183-T185 | F8 honesty; deny/audit exit-code gate on checklist |
| T169/T170 | Evaluation pointers in evidence index (hard gates only) |
| Impl-Plan §17 | Storage encryption → F8-honest |

**Explicit non-DoD residuals (remain open):**

1. MSI / notarization / App Store packaging
2. systemd / launchd production units
3. PR `ci.yml` full action SHA-pin — **Closed T186** (release.yml was T185)
4. Branch protection (**R-CI-BRANCH** — repo admin)
5. doctor CLI remains; ~~recovery export~~ **T188**; #34.2 **T189**; ~~SQLCipher page-encrypt~~ **T187**
6. T186 hermetic CLI suite (parallel)
7. Soft historical PRD “Storage is encrypted…” line (report-only; not elevated)
8. **NOTICE noise:** `cargo-about` may still list first-party PolyForm workspace crates despite `private.ignore` (presentation only; deny policy remains SOOT for allowed licenses)

**Out of scope remains:** public `v*` marketing release without human re-walk of checklist; SLSA L3; SOC2/ASVS certification.

**Review closeout:** Internal PASS WITH DEFERRED P3; Codex R1 FAIL→fix; R2 FAIL→easy P3; **R3 PASS WITH DEFERRED P3** (final gate).

---

## From T186 - Hermetic CLI / Multi-OS Test Hygiene (2026-08-01)

### 64. T186 Hermetic CLI / CI Hygiene — **Completed** (2026-08-01)

P12 residual implemented 2026-08-01. Normative: `conductor/tracks/trackT186-hermetic-cli-ci-hygiene/{spec,plan,review}.md` (L1–L13, AC0–AC10).

**Shipped:**
- **AC0:** `nextest.toml` → `.config/nextest.toml`; `slow-timeout = { period = "30s", terminate-after = 4 }` (120s kill); profile.ci discoverable
- **Helper:** `tests/common/mod.rs` (`hermetic_bin` / `hermetic_vault` / `hermetic_cmd`); 11-key denylist (elevation + SCOPE + PREFLIGHT)
- **AC2:** `hermetic_smoke.rs` ambient pollution proof
- **Priority+soft migration:** smoke, migrate, shadow, device, recovery, preflight, mapping, sync_query, CARGO_BIN_EXE trio
- **Path:** `resolve_best_effort__missing_child_under_existing_parent__soft_resolves` KAT
- **GHA:** `--profile ci` on Win/Linux/macOS; R-CI-PIN full SHA pins aligned with release.yml
- **Docs:** `Docs/ci-tooling.md` hermetic + nextest; COMPATIBILITY/RELEASE pin wording
- **Evidence:** `evidence/INVENTORY.md` dual-pattern inventory

**Local gates:** nextest `--workspace --profile ci` **1713 passed**; clippy/fmt/deny/audit green. Internal R1 PASS after inventory/AC2 fixes. Codex R1 FAIL (closeout honesty) → fixed deferred/conductor; final Codex after PR CI.

**Absorbed:** §56/§58 T179 hermetic suite + ambient + soft-canonicalize + no-fail-fast; §62 R-CI-PIN PR pins.

**Explicit non-DoD residuals (remain open elsewhere):**
1. ~~Long-tail 25 `cargo_bin` sites / 5 files (L13 inventoried)~~ — **Closed by T191**
2. ~~#12 TOCTOU / openat / cap-std~~ — closed-with-residuals by **T190**
3. R-CI-BRANCH (repo admin)
4. Platform tier / desktop T1
5. #34.2 DataKey rotation
6. Optional: `LEDGERFUL_TX_ID` denylist expansion (Info)

**AI1/AI2 fold-in applied at implement:** A1–A12 accepted (nextest path, terminate syntax, dual inventory, denylist, SHA align, pollution test). Rejected: Fully Compliant claims; actionlint DoD; mandatory checkout v7.

---

## From T188 — Restore Safety + Recovery Operator Surface (2026-08-02)

### 66. T188 Restore Safety + Recovery Operator Surface — **Completed** (2026-08-02)

**Shipped:** mutating `backup restore` hard-fails when robust IPC probe true (3×≥1000ms); dry-run notice while daemon up; `ai-brains recovery export` (passphrase-file / rpassword TTY, min 8, schema_version=1, reparse refuse, kit file only, RecoveryKitCreated best-effort, no migrate while daemon up); R-DOC-CLI partial (export yes, doctor no). Full gate: fmt/clippy/nextest **1749**/deny/audit.

**Closed:** §59 #1 recovery export; §59 #6 restore daemon hard-fail; T181-F-03 product hard-fail language.

**Remains open:**
- ~~**#2 doctor** CLI (R-DOC-CLI residual)~~ — **Closed by T192** (2026-08-02) PR #75 `80837da`
- Live-daemon busy-restore integration drill (unit-injected daemon-up covers safety; optional)
- Restore still opens AppContext (migrate) before probe (P3 residual; overwrite still blocked)
- Dry-run notice stdout process-capture (P3 test hardening)
- ~~Argon2 params in kit JSON (F37)~~ — **Closed by T194** (2026-08-02); ~~#34.2~~ closed T189

---

## T192 closeout (2026-08-02) — Doctor CLI shipped

**Closed:** deferred **#2** / R-DOC-CLI doctor residual; SECURITY-LIMITS / INSTALL / CAPABILITIES / RECOVERY-DRILLS doctor-absent language; claims invented-doctor rule #54; stale invented recovery-export forbid.

**Shipped:** read-only `ai-brains doctor` (`open_read_intent` only; no AppContext migrate); F17b `backup_dir_read_only`; contracts `DoctorReport` schema_version=1; exit 0 for ok|degraded; optional `--kit-path` + soft RecoveryKitCreated event; Codex R2 **PASS WITH DEFERRED P3**. PR #75 `80837da`.

**Honest residuals after ship:**
- Offline kit without `--kit-path` still operator responsibility
- Daemon probe = our IPC only (bool; cannot distinguish probe error vs down) — P3
- Spec F16 erratum: live `event_type` is unquoted after store `trim_matches('"')` (code correct; AC16)
- No hook doctor; no auto-fix; TTY-smart format optional later


## T189 closeout (2026-08-02)

- ~~#34.2 DataKey rotation~~ closed by T189 PR #67 `9e9465e`.
- **P3 residual (documented):** Windows exclusive `drop(source)` → `MoveFileEx` micro-window (OS cannot replace open DB). See ADR-0020 / R-34.2 / OPERATIONS.

## T190 residual (2026-08-02)

- **Soft-skip symlink proof** when create privilege missing (F17 / Codex R3 P3) — multi-OS CI re-proves when privilege available; product path fail-closed. **Kept** as verification residual (R-SOFT-SKIP) after T193 ship.
- ~~**T188 write / token path / ambient CLI**~~ — **Closed-with-residuals by T193** PR #77 `2183127`: P0 `write_protected_artifact`, token load/write, `recovery::write_kit_file` elevated via shared `cap_open` write SOOT. Remaining honesty residuals: soft-canon, parent `create_dir_all`, P2 ambient CLI long-tail, perfect Windows TOCTOU, R-SOFT-SKIP.

### T237 soft residuals (2026-08-08)

| Item | Notes |
|------|-------|
| UserPromptSubmit live (S1) | Not DoD |
| Opt-in subagent include (S2) | Default skip hard |
| Fingerprint turn-ids (S8) | Filter-version risk documented |
| AdapterKind::Grok registry | Optional; grok_capability() exported |
| Claude/Codex install_ready | Soft S6 / T238+ labels |


### T221 closeout residuals (2026-08-09) — progressive deny honesty shipped

| Residual | Disposition |
|----------|-------------|
| F12 doctor `policy_grants` warn | ~~**Absorbed into T241 DoD**~~ ✅ **Shipped** T241 PR #151 `930d0ed` — matrix 15 + `policy_grants` warn |
| F32 `--principal-id` progressive/expand | Soft skip — not DoD |
| F18 daemon/HTTP progressive 200+denied | Soft residual (CLI is DoD) |
| F36 trace `applied_policy` string | Soft residual — out of DoD |
| Dual-site POLICY_DENIED_HINT drift | Comments + hermetic wording; residual |


### T218 closeout residuals (2026-08-09) — semantic quality v2 shipped

| Residual | Disposition |
|----------|-------------|
| F18 first-line / DECISION-line boost | Soft — not DoD |
| AC15 response-level `fusion` object (effective rrf_k) | Soft — not DoD |
| F19 weighted RRF env | Soft residual |
| F20 ANN / HNSW productization | Soft residual (also T215 F27) |
| F21 nomic task-prefix re-embed + floor re-tune | Soft residual |
| F24 skill one-liner | Soft (T215 F29 family) |
| Optional httpmock full `recall_full` hermetic | Soft — production SOOT is `fuse_local_and_semantic`; F12 preferred injection seam |

### T219 closeout residuals (2026-08-09) — pretty readability shipped

| Item | Notes |
|------|-------|
| `--compact` flag / PrettyOpts | ~~**T250**~~ ✅ **Completed** PR #165 `bf23f0e` — `--compact` + small `PrettyCaps` |
| is-terminal → `std::io::IsTerminal` | Soft F22 |
| clap workspace pin bump | Soft F41 — no bump DoD |
| Role strip inside retrieval for JSON text | Soft F5 residual |
| ~~T224 search-path role strip~~ | **Closed by T224** PR #120 `a18fae6` |
| ~~T228 non-empty recall Scope~~ | **Closed** PR #134 `e51d5e4` |
| scope_display extract / pager | Soft F22 |
| truncate_preview triplication (ingest/pin) | Soft F14 residual from T224 — not DoD |
| Optional JSON `preview` / `--strip-roles` | Soft F6 residual from T224 |
| Promote `strip_role_prefix` to core | Soft residual (retrieval converge) |

### T225 (2026-08-11) soft residuals
- F17: verify `--quiet`; JSON `summary` field; structured `VerifyError` / 4-class rollup (O1); optional 3-class substring rollup omitted (M5)
- Operator still runs `ai-brains backup create` on live encrypted vaults → **T277 Planned** (2026-08-22)

| T233 soft residual (list-paths / unregister-path / from-scan / route metadata) | ~~**T254**~~ ✅ **Completed** 2026-08-15 — O2/F31/F15 + refuse-steal + `bridge_roots_failed`; **declined** T233-F44 + concurrent F21 |

## AI-Brains T241 (2026-08-12)

**Closed in AI-Brains** (PR #151 `930d0ed`): Policy cold-start bootstrap discoverability — doctor `policy_grants` matrix 15; show/check UX; briefing `denial_hint`; preflight grants line with project_id-wired probe. Codex CX3 **PASS**.

| Date | Repo | Track | Residual | Notes |
|------|------|-------|----------|-------|
| 2026-08-12 | ai-brains | T241 | low | F20 soft: `preflight --install-grants` opt-in | Soft residual; not DoD |
| 2026-08-12 | ai-brains | T241 | low | F21 soft: skill one-liner for bootstrap | Soft residual; not DoD |
| 2026-08-12 | ai-brains | T241 | low | F22 soft: bootstrap success soft-resolve hermetic | Soft residual; not DoD |
| 2026-08-12 | ai-brains | T241 | low | L1 after_help dual-site vs CAPABILITY_CATALOG | Sync comment; clap after_help static |
| 2026-08-12 | ai-brains | T241 | low | L2 dual short-SOOT constants CLI vs CP | Substring locked by tests |
