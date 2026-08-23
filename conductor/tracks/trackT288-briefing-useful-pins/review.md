# T288 review log — granted-empty briefing vault-pin stanza

**Track:** T288-BriefingUsefulPins
**Branch:** `track/T288-briefing-useful-pins`
**FEATURE TX:** `cc07c4f0-1067-42ec-83bd-b2e49a2aa931`
**Reviewers:** implementer (R1) → codex-review (FEATURE)

## Scope

Granted-empty `briefing project` (`!denied` + empty `decisions[]`/`conclusions[]`) prints `## Vault pins (not Approved)` with inventory `Pinned: N` (`count_pinned_memories`, `mp.project_id = ?`) plus up to 3 leading-line `DECISION:`/`CONSTRAINT:` previews (`list_authority_memories` limit 32, retain `PinKind::Decision | Constraint`, `preview_line` 80). CLI JSON overlays optional `vault_pin_count` / `vault_pin_previews` (E1: omit when overlay off; `0`/`[]` when granted-empty zero pins). Authority arrays stay empty. `render_project_markdown` signature unchanged (preflight `None`). Fail-open `Repository:` parse (no `?` on overlay path).

**Did not:** H2 pin→Approved; DTO `ProjectBriefingPacket` fields; `personal.rs`; `governed_common.rs`; `query_store.rs`; CLI `preflight.rs`; `project.rs`; clap 5 / rusqlite 0.40; `cargo install`; live pin / `.env` / extra `policy bootstrap`.

## DoD matrix

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | **Met** | `briefing_project__granted_with_decision_pin__human_stanza_not_under_decisions` PASS (red: no heading; green: heading + nonzero `Pinned:` + needle + `_None_` + `recall`) |
| AC2 | **Met** | `briefing_project__granted_with_decision_pin__json_overlay_count_and_previews` PASS (`denied: false`, arrays `[]`, `empty_authority`, `vault_pin_count ≥ 1`, preview `DECISION:`/`needle`) |
| AC3 | **Met** | `render_project_markdown_with_vault_pins__some__inserts_after_empty_authority` PASS; `render_project_markdown` (None) omits heading |
| AC4 | **Met** | `briefing_project__granted_empty_zero_pins__pinned_zero_no_fabricated_decision` PASS (`Pinned: 0`, JSON `0`/`[]`) |
| AC5 | **Met** | `briefing_project__denied__no_vault_pin_stanza` PASS (no heading, no JSON key, no `_None_`) |
| AC6 | **Met** | `briefing_empty_authority_next_step__one_line_at_most_140_chars` PASS (const unchanged) |
| AC7 | **Met** | `briefing_project__granted_substance__decision_and_conclusion_in_md_and_json` PASS + `!contains("## Vault pins")` |
| AC8 | **Met** | `project_briefing_packet__serde__omits_vault_pin_count` PASS |
| AC9 | **Met** | `preview_line__tags_envelope_t288__decision_not_tags` PASS |
| AC10 | **Met** | CAPABILITIES dual-model T288 sentence; PROTOCOL-COMPAT CLI extras vs daemon unaugmented; CHANGELOG Unreleased; OPERATIONS one-liner; `briefing_project__help__lists_human_pretty_and_example` asserts `not Approved` + `vault_pin_count` |
| AC11 | **Met** | `cargo clippy -p ai-brains-cli -p ai-brains-control-plane --all-targets -- -D warnings` PASS; no new crate |
| AC12 | **Met** | Manual `cargo run -p ai-brains-cli --quiet -- briefing project --format human` → `Pinned: 3889`, `not Approved`, `recall`, Decisions `_None_`, exit 0. JSON `denied: false`, `vault_pin_count: 3889`, `vault_pin_previews: []`. COUNT is `count_pinned_memories` (not `memory list --summary` equality). |
| AC13 | **Met** | Did not edit `query_store.rs`. Store `list_authority_memories` / `count_pinned_memories` units stay green. |
| AC14 | **Met** | `should_overlay_vault_pins__rstest_denied_nonempty_empty` (denied skip / nonempty skip / granted-empty apply) PASS |
| AC15 | **Met** | `briefing_project__granted_chrome_only__count_without_decision_preview` PASS |
| AC16 | **Met** | `briefing_project__granted_hotspot_only__preview_omits_hotspot` PASS |
| AC17 | **Met** | `parse_repository_project_id__rstest_personal_garbage_valid` PASS (`Repository:{uuid}` Some; Personal/garbage/`Repository:`/`not-a-uuid` None) |

## Findings

| id | severity | description | status | evidence |
|----|----------|-------------|--------|----------|
| R1-1 | low-info | Live `cargo run -- briefing project --format human` on `3581317d` prints `Pinned: 3889` and `_No leading-line DECISION/CONSTRAINT samples in this scope._` (pass-1 GLOB 0, same class as T287 R1-1). Hermetic AC1/AC2 are SoT for sample lines. Did not `cargo install`. Did not rewrite `.env`. | deferred | F32 / F4 COUNT |

## Codex CX1 (gpt-5.6-luna, read-only)

Product **PASS**. Verdict at review time was not-clear for open closeout gates.

| id | severity | disposition |
|----|----------|-------------|
| P0 | process | **verified_fixed** — `dev-check` 3399/1 skipped + `verify --scope full` exit 0 + closeout + Phase 6 |
| P2 | medium | **verified_fixed** — denied JSON omits both overlay keys; AC7 nonempty JSON omits both (`2a0373b`) |
| P3 | low-info | **deferred** — live GLOB 0 samples recorded in `deferred.md` (F32) |

## Targeted gates (pre-full)

- `cargo fmt --check` PASS
- `cargo clippy -p ai-brains-cli -p ai-brains-control-plane --all-targets -- -D warnings` PASS
- T288 + T263/T227 stay-green nextest **22 passed**
- Red was assertion-fail (not compile-error-only) on AC1/AC2/AC3/AC4/AC14-case3/AC17-case1/AC15/AC16

## Manual

```
cargo run -p ai-brains-cli --quiet -- briefing project --format human
cargo run -p ai-brains-cli --quiet -- briefing project --format json
```

Human: heading + `Pinned: 3889` + honest empty samples + `_None_` + `recall`. JSON: `denied: false`, arrays empty, overlay keys present. PATH not reinstalled (F17).

## Full gate

- First `dev-check` fail-fast: `backup_restore__daemon_down_force__succeeds` (daemon Running held vault). Unrelated; temporary `ai-brains daemon stop`. Isolated restore drills **3 passed**.
- `.\scripts\dev-check.ps1` **SUCCESS** nextest **3399** passed / 1 skipped (9 slow)
- `ledgerful verify --scope full` exit 0

Did **not** `cargo install`. Did **not** write `.env`. Daemon left **Stopped**.
