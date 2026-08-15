# Deferred Follow-Ups

Tracks deferred from T142. Append-only; strike through when promoted to a real track.

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

Suggested order: ~~**T196**~~ ... ~~**T216**~~ **closed** (T205–T216 audit series complete). **Next series T217–T232** (post-audit CLI quality): ~~**T217**~~ **closed** PR #110 `1e22e77`; ~~**T220**~~ **closed** PR #112 `6f4f67b`; ~~**T221**~~ **closed** PR #114 `b3c4b0f`; ~~**T218**~~ **closed** PR #116 `fc4d370`; ~~**T219**~~ **closed** PR #118 `496ddd7`; ~~**T224**~~ **closed** PR #120 `a18fae6`; ~~**T222**~~ **closed** PR #122 `c1ac594`; ~~**T232**~~ **closed** PR #124 `33b28d0`; ~~**T223**~~ **closed** PR #126 `7ff8f7f`; ~~**T225**~~ **closed** PR #128 `927b8db`; ~~**T226**~~ **closed** PR #130 `5919f26`; ~~**T227**~~ **closed** PR #132 `40c7cd1`; remaining placeholders (non-empty Scope, nightly+router ops, global labels, unified search). See [README-T217-T232-CLI-QUALITY.md](tracks/README-T217-T232-CLI-QUALITY.md). Packaging residual: MSI / notarization / App Store + R-CI-BRANCH. Residual honesty: daemon `AI_BRAINS_VAULT_KEY` silent zero (T199 F16 / T205 F14).

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
| is-terminal → `std::io::IsTerminal` | Soft residual F24 (L1) |
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
| is-terminal → `std::io::IsTerminal`; clap ValueEnum ignore_case unify | Soft residual F22 |
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
| Default project identity (env test-alias vs detect vs path) | ~~**T240**~~ ✅ **Completed** 2026-08-12 PR #144 `29b9b59` — whoami + path-first detect + mismatch warn. Soft residual only: F13 detect `--json`, F14 `project use` |
| Policy grants empty → governed dead-end | ~~**T241**~~ ✅ **Completed** 2026-08-12 PR #151 `930d0ed` — doctor/preflight/show/check/briefing discoverability. Soft residual only: F20 install-grants, F21 skill one-liner, F22 soft-resolve hermetic, L1 after_help dual-site, L2 dual short-SOOT |
| Env override warn spam (T223 residual) | ~~**T242**~~ ✅ **Completed** 2026-08-12 PR #147 `9f3148b` — session fingerprint markers (cross-process). Soft residual only: F16 clap quiet, F17 elevation QUIET/FORCE, F18 truthy→core, F19 global quiet pre-read |
| Search dual model + progressive first-run | ~~**T243**~~ ✅ **Completed** 2026-08-12 PR #153 `7a19d40` — `search`→recall alias; `text`≡pretty; progressive `next_step`/deny recall honesty. Soft residual only: F23 non-empty recall footer, F24 daemon/HTTP `next_step` |
| Backup fleet 0 usable / legacy plain | ~~**T244**~~ ✅ **Completed** 2026-08-12 PR #149 `948d2ae` — Incomplete + core-table usable SOOT; list residual `not recoverable`; CLI usable-first; verify both cores; live create green path. Soft residual only: F17 verify quiet/JSON summary/structured error; F18 archive helper |
| Harness wiring=missing | ~~**T245**~~ ✅ **Completed** 2026-08-12 PR #155 `f05e2f6` — `all-ready`; AGY IDE + CLI plugin bundle (not top-level CLI hooks.json); PATH bake; S12 idle+status; doctor ready-vs-pending. Soft residual only: `pending_track` still T239+; doctor message helper-only |
| Graph neighbors JSON-only | ~~**T246**~~ ✅ **Completed** 2026-08-13 PR #159 `06cdcde` — TTY pretty; frozen JSON keys; crate `*_with_depth`; F9 sort in PROTOCOL-COMPAT. Soft residual only: F17 tree/mermaid/TTY-auto update/batch `node_kinds`; F18 projector completeness; F19 T213 F31 freshness |
| Nightly status latency + Last Result 101 | ~~**T247**~~ ✅ **Completed** 2026-08-13 PR #157 `43191ff` — `--quick`; parallel 750ms; LIST/V honesty; live missing `.cmd` named. Soft residual only: F11–F16 → **T255** |
| Retention plan human | ~~**T248**~~ ✅ **Completed** 2026-08-14 PR #161 `c633781` — TTY `auto` human; JSON keys frozen; `memory_legacy` → `skip`; apply default JSON. Soft residual only: F16–F18 |
| Scope/daemon/doctor presentation | ~~**T249**~~ ✅ **Completed** 2026-08-14 PR #163 `5fd264a` — TTY `auto` human; JSON keys frozen; Stopped `next:`; real `--summary`. Soft residual only: F12 daemon json/uptime/sc query / is-terminal std / shared resolver / color; F13 T241 leftovers / T226 O1 / T255 / T250 |
| Preflight pretty density (T219 residual) | ~~**T250**~~ ✅ **Completed** 2026-08-14 PR #165 `bf23f0e` — Session/Recent line-cap 140; `--compact`; JSON/summary ignore; chrome strip. Soft residual only: F12 is-terminal std / clap pin / JSON role strip / pager / governed section caps / `--max-line` / skill / HOTSPOT float / auto-compact |
| device status missing | ~~**T251**~~ ✅ **Completed** 2026-08-14 PR #167 `038098e` — first-class `status` = roster + always `next:`; plural T198 only; CLI-EXIT-CODES footnote. Soft residual only: F12 |
| ingest dry-run empty stdin | **T252** Placeholder |
| Claude/Codex install_ready (T239+) | **T253** Placeholder |
| T233 soft list-paths/unregister/from-scan | **T254** Placeholder |
| T229 soft F8–F12/F14 | **T255** Placeholder |

Series README: [README-T240-T255-CLI-EFFECTIVENESS.md](tracks/README-T240-T255-CLI-EFFECTIVENESS.md). **T240–T251 Completed.** Remaining tracks plan-only until go.

### T251 soft residuals (2026-08-14)

Specified softs, not review deferrals:

| Residual | Disposition |
|----------|-------------|
| F12 `device list --format json` (T176 leftover) / bootstrap→outbox / doctor enrollment check / combined list+replicate dashboard / `visible_alias = "stat"` / default `device` → status / is-terminal → std / clap 4.6 workspace pin / unify singular error copy in `load_local_signing_key` / `load_local_device` | Soft — not DoD |

### T250 soft residuals (2026-08-14)

Specified softs, not review deferrals:

| Residual | Disposition |
|----------|-------------|
| F12 is-terminal → std / clap 4.6 workspace pin / retrieval JSON role strip / pager / governed section caps / `--max-line` / T241 `--install-grants` / skill one-liner / HOTSPOT float-score reformat / auto-compact from terminal height | Soft — not DoD |

### T249 soft residuals (2026-08-14)

Specified softs, not review deferrals:

| Residual | Disposition |
|----------|-------------|
| F12 Daemon uptime / `sc query` / `--format json` / `--quick` / compact doctor JSON DTO / T214 is-terminal → std / shared `resolve_*_format` / T204 Start-here rewrite that removes json / color / pager / `comfy-table` | Soft — not DoD |
| F13 T241 F20–F22 leftovers / T226 O1 shared resolve wrapper / T255 nightly/router / ~~T250 preflight density~~ | Peer tracks (T250 closed) |

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
| S8 Claude/Codex install_ready | **T253** Placeholder |
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

### 39. T167 design freezes absorbed into track (2026-07-29 expansion + review fold-in)

Folded into T167 spec/plan (`trackT167-legacy-memory-classification-import/`) — implement on go-ahead:

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
- Operator still runs `ai-brains backup create` on live encrypted vaults

| T233 soft residual (list-paths / unregister-path / from-scan / route metadata) | Soft: O2 list-paths CLI; F31 unregister-path; F15 from-scan; F44 route method/path_pattern; F21 non-atomic CLI; bridge_roots failed-count under-sum |

## AI-Brains T241 (2026-08-12)

**Closed in AI-Brains** (PR #151 `930d0ed`): Policy cold-start bootstrap discoverability — doctor `policy_grants` matrix 15; show/check UX; briefing `denial_hint`; preflight grants line with project_id-wired probe. Codex CX3 **PASS**.

| Date | Repo | Track | Residual | Notes |
|------|------|-------|----------|-------|
| 2026-08-12 | ai-brains | T241 | low | F20 soft: `preflight --install-grants` opt-in | Soft residual; not DoD |
| 2026-08-12 | ai-brains | T241 | low | F21 soft: skill one-liner for bootstrap | Soft residual; not DoD |
| 2026-08-12 | ai-brains | T241 | low | F22 soft: bootstrap success soft-resolve hermetic | Soft residual; not DoD |
| 2026-08-12 | ai-brains | T241 | low | L1 after_help dual-site vs CAPABILITY_CATALOG | Sync comment; clap after_help static |
| 2026-08-12 | ai-brains | T241 | low | L2 dual short-SOOT constants CLI vs CP | Substring locked by tests |
