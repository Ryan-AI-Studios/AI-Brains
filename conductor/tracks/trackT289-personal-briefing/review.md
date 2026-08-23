# T289 review log — Personal deny must not look like empty preferences

**Track:** T289-PersonalBriefing
**Branch:** `track/T289-personal-briefing`
**FEATURE TX:** `95ad50a2-ecc8-413d-8ae0-ef06ee07cf41`
**Reviewers:** implementer (R1) → codex-review (FEATURE)

## Scope

Denied `briefing personal --format human` omits `_None_` under `## Preferences` / `## Continuity` and uses `BRIEFING_PERSONAL_DENIED_BODY` = `_(optional continuity; not a missing vault)_` via private `personal_empty_section_placeholder`. T263 `BRIEFING_PERSONAL_DENIED_NEXT_STEP` / `DENIAL_HINT` frozen (names `recall`, not Personal bootstrap). JSON `denied: true` + empty arrays + no T288 overlay keys. Allowed-empty still `_None_` + T227 empty_continuity. Renderer-only; `personal.rs` unchanged.

**Did not:** auto Personal grant / live `policy bootstrap`; H2; T288 vault-pin on Personal; T290 lists; DTO new keys; clap 5 / rusqlite 0.40; `cargo install`; `.env` write; grow `personal.rs` / `project.rs` / CLI `preflight.rs` / `governed_common.rs`.

## DoD matrix

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | **Met** | `render_personal_markdown__denied__no_none_placeholder` PASS (red: `_None_` still printed; green: BODY under both headings, recall, no bootstrap / GRANT_WALL / HIDDEN / `BRIEFING_DENIED_NEXT_STEP`) |
| AC2 | **Met** | `briefing_personal__no_grants__human_omits_none` PASS (exit 0, header, `**Denied:**`, `recall`, no `_None_`, no `policy bootstrap`) |
| AC3 | **Met** | `briefing_personal__no_grants__soft_deny_denial_hint` PASS (`denied: true`, empty prefs, `continuity.summary` `""`, hint `recall` not bootstrap, no `vault_pin_*`) |
| AC4 | **Met** | `briefing_personal_denied_body__exact_optional_one_line` PASS (exact F2 string, one line, ≤140, `optional`, const `!_None_` / `!policy bootstrap`) |
| AC5 | **Met** | `render_personal_markdown__allowed_empty__emits_empty_continuity_next_step` PASS (`## Preferences\n_None_` and `## Continuity\n_None_` + empty_continuity notice) |
| AC6 | **Met** | `render_personal_markdown__denied__names_recall_not_personal_bootstrap` PASS (T275 AC16 GRANT_WALL/HIDDEN freeze) |
| AC7 | **Met** | `briefing_project__format_banana__exit_2_no_stdout_json` PASS (shared format classifier) |
| AC8 | **Met** | CAPABILITIES Denied packets row extended (not a new section); CHANGELOG Unreleased T289; `briefing_personal__help__names_optional_body_not_none` PASS |
| AC9 | **Met** | `cargo clippy -p ai-brains-cli -p ai-brains-control-plane --all-targets -- -D warnings` PASS; no new crate; clap lock 4.6.1; rusqlite 0.39.0 |
| AC10 | **Met** | Manual `cargo run -p ai-brains-cli --quiet -- briefing personal --format human` — BODY under both headings, `recall`, no `_None_`, exit 0. JSON `denied: true`, `preferences: []`, `continuity.summary: ""`, `denial_hint` recall, no `vault_pin_*` |
| AC11 | **Met** | `git diff` vs `personal.rs` empty (run path `briefing.rs` also empty except after_help in `main.rs`) |
| AC12 | **Met** | `personal_empty_denied__json__no_new_keys` PASS (frozen key set; `denial_hint` omitted on contracts `empty_denied`; no overlay keys) |

F23 unit: `render_personal_markdown__denied_with_pref__keeps_pref_text` PASS.

## Findings

| id | severity | description | status | evidence |
|----|----------|-------------|--------|----------|
| R1-1 | low-info | PATH `ai-brains` still T281-era (no T285–T289) until `cargo install`. Source/hermetic SoT. | deferred | F13 |

No critical / high / medium. Internal R1 **PASS**.

## Codex CX1 (gpt-5.6-luna, read-only)

Product **PASS**. No product P0–P2. P1-1 was process (full gate/closeout pending at review time).

| id | severity | disposition |
|----|----------|-------------|
| P1-1 | process | **verified_fixed** — `dev-check` 3405/1 skipped + `verify --scope full` exit 0 + closeout + Phase 6 |

## Targeted gates (pre-full)

- `cargo fmt --check` PASS
- `cargo clippy -p ai-brains-cli -p ai-brains-control-plane --all-targets -- -D warnings` PASS
- Red was assertion-fail (not compile-error-only): AC1 `_None_` still printed; AC4 stub `T289_RED_STUB` ≠ F2 string
- Control-plane personal renderer units + CLI hermetic personal/banana **PASS**

## Manual

```
cargo run -p ai-brains-cli --quiet -- briefing personal --format human
cargo run -p ai-brains-cli --quiet -- briefing personal --format json
```

Human: `# Personal Continuity Briefing`, Denied blockquote, T263 recall next, Preferences/Continuity BODY, no `_None_`. JSON: `denied: true`, empty arrays, `denial_hint` recall, no overlay keys. PATH not reinstalled (F13). Did not Personal bootstrap. Did not write `.env`.

## Full gate

- `.\scripts\dev-check.ps1` **SUCCESS** nextest **3405** passed / 1 skipped (9 slow)
- `ledgerful verify --scope full` exit 0 (`fmt` / workspace clippy / nextest / deny / audit)

Did **not** `cargo install`. Did **not** write `.env`. Did **not** Personal `policy bootstrap`. Daemon left **Stopped**.
