# T289 — Personal briefing deny must not look like empty preferences

- **Track ID:** T289-PersonalBriefing
- **Status:** **Planned** (Pending until **go**)
- **Category:** FEATURE / UX / HONESTY
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `briefing personal` **4/7**; deny + `_None_` Preferences/Continuity. Placeholder minted with T285–T300 (`76c4db9`). T263 ✅ F4 Personal deny names `recall` (**not** Personal bootstrap). T275 ✅ F35 contamination lock (no project grant-wall). T275 **F32** Personal `_None_` was **not** DoD — **this track absorbs it**. T288 ✅ project vault-pin stanza (do **not** steal).
- **Depends on:** T227 ✅ personal format + empty_continuity only when `!denied`; T263 ✅ F4 recall next; T275 ✅ F35 no project-wall leak
- **Blocks / feeds:** Operators who run `briefing personal` when denied do not conclude they have no preferences because `_None_` sat under a grant wall. Lists/progressive pin-count remains **T290**. Project briefing remains **T288**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “personal briefing deny `_None_`”; T275 F32 optional Personal `_None_`; T288 closeout “Personal `_None_` not stolen”
- **Not absorbed (DoD):** Auto Personal grant / live Personal `policy bootstrap`; T263 H2; T288 vault-pin stanza on Personal; T290 lists/progressive; T227 #18 synthetic continuity; T240 F2; clap 5 / rusqlite 0.40; DTO new keys
- **Research date:** 2026-08-23 (plan dogfood HEAD `05d7ac0` T288 `#204`; product `src/` = T288 renderer + T263/T275 personal deny; PATH **0.1.2** 2026-08-22 19:41 **without** T285–T288 — personal `_None_` hole is in **source and PATH**)
- **AI fold-in:** 2026-08-23 `agy-review.md` + `opencode-review.md` (HEAD `72fbb92`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** **Agree:** Agy m1 private helper (F11); Agy O1 Preferences `_None_` on allowed-empty (AC5); OpenCode m1 tests path ( §2.3); OpenCode m2 T288 overlay project-only (AC3/F5); OpenCode m3 const guards on AC4; OpenCode O2 CAPABILITIES *extend* (AC8/F20). **Already:** Agy m2 exact AC4 string; Agy O2 after_help (F20/AC8); OpenCode O1 `empty_personal` fixture (AC1). **No declines of B/M.** Disposition **§13**.
- **Ledger:** planning DOCS TX `25bbc580-99a6-4969-8ea5-d0e1902d374e`. Fold-in DOCS TX `45277700-a110-4f91-911b-8f921173dfdb`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** pin production decisions to the live vault as implement. Do **not** rewrite `.env`. Do **not** live `policy bootstrap` (project or Personal). Do **not** grow hotspot `project.rs` / `personal.rs` / CLI `preflight.rs` / `governed_common.rs`. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Denied Personal human is not an empty-preferences lie.** `ai-brains briefing personal --format human` when `denied: true` must **not** print `_None_` under `## Preferences` or `## Continuity`. Same class as T275 project grant-wall: agents must not treat a policy wall as “you have no preferences.”
2. **Personal stays optional.** Next-step stays T263 `BRIEFING_PERSONAL_DENIED_NEXT_STEP` (names `recall`; **not** `policy bootstrap`). Do **not** reuse project `BRIEFING_DENIED_GRANT_WALL` / `BRIEFING_DENIED_HIDDEN` / `BRIEFING_DENIED_NEXT_STEP` (T275 F35).
3. **JSON and allowed-empty stay honest.** `--format json` `denied: true`, `preferences: []`, `continuity.summary: ""`, `denial_hint` recall — **no new keys**. Granted-empty (`!denied`) still uses `_None_` + T227 `empty_continuity` notice (not this hole).
4. **North star.** Capture independence: markdown placeholder only. No new events. No hidden CoT. No auto Personal grant. Dual-model: Personal continuity ≠ vault pins (T288).

This unblocks the optional surface: T263 made deny *honest about next command*; T275 locked *no project bootstrap leak*; the 2026-08-22 audit still scores **4/7** because `_None_` trains “empty prefs.”

---

## 2. Live baseline (re-scan 2026-08-23)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `05d7ac0` T288 squash `#204`. Tree **CLEAN**. `origin/main` = HEAD. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-22 19:41**, 25 139 712 bytes, **0.1.2**. **Does not have T285–T288.** Personal `_None_` is in **source + PATH** (T288 did not touch personal markdown). **Do not `cargo install`.** |
| `preflight --summary` | Scope `3581317d`. Pinned **3889**. In-context **0/0/0**. Word **1468**. |
| `briefing personal --format human` | Scope `Personal:a1b2a1b2-…`. `> **Denied:** Personal scope read denied without grant`. Next = T263 recall (optional; **no** bootstrap). Then **`## Preferences` `_None_`** and **`## Continuity` `_None_`**. Exit **0**. **This is the 4/7 hole.** |
| `briefing personal --format json` | `denied: true`, `preferences: []`, `continuity.summary: ""`, `denial_hint` names `recall`, no `policy bootstrap`. JSON is already honest; **human `_None_` is the lie.** |
| `briefing project --format human` | T288 stanza on source (`cargo run`); PATH-behind. **Not this track.** |
| Last GitHub PR | [#204](https://github.com/Ryan-AI-Studios/AI-Brains/pull/204) T288 (2026-08-23). `comments` / `reviews` / issue comments / inline `/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, `#59` tokio, `#60` thiserror, actions `#68–#72`). **No leftover to mint. No T301.** |
| Identity / doctor | ledgerful doctor 4 warn (legacy `.changeguard` / sig-pin / timings / :8081). **0 pending / 0 drift.** Hotspot **#1** `project.rs` — **do not touch.** `personal.rs` **#7** (2.186) — **do not grow** (`:121` already PERSONAL hint). CLI `preflight.rs` #8. Extend **`renderer.rs`** (not top-10). |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why denied `_None_` still trains “no prefs”

| Layer | Truth |
|-------|--------|
| T263 F4 shipped | Markdown next + JSON `denial_hint` name `recall`, not Personal bootstrap. Live stdout already has that next line. |
| T275 F35 shipped | Personal deny must not contain project GRANT_WALL / HIDDEN / `policy bootstrap`. Unit `render_personal_markdown__denied__names_recall_not_personal_bootstrap` **does not** assert `!_None_`. |
| T275 F32 parked `_None_` | “Not audit 8/3. **Not required** to change Personal `_None_`.” **This track is the absorb.** |
| Renderer still prints `_None_` | `render_personal_markdown` `:262–273`: empty prefs/continuity **unconditionally** `_None_`, including `packet.denied`. Project path uses `empty_section_placeholder(denied)` → HIDDEN. Personal does **not**. |
| T227 F9 | `empty_continuity` footer only when `!denied` (`:278`). Denied does not get that notice — only `_None_`. |
| JSON already honest | Empty arrays + `denied: true`. Do **not** add overlay keys (T288 pattern is project-only). |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| Markdown | `control-plane/.../briefings/renderer.rs` `render_personal_markdown` `:243` | Denied blockquote + `BRIEFING_PERSONAL_DENIED_NEXT_STEP` `:257`. Prefs/Continuity `_None_` `:262–273`. **Change here.** |
| Project analog | `empty_section_placeholder` `:234` | Denied → `BRIEFING_DENIED_HIDDEN`. **Do not call from personal** (T275 F35). |
| Personal next | `BRIEFING_PERSONAL_DENIED_NEXT_STEP` `:50–51` | Frozen T263. **Do not lengthen.** |
| Personal JSON hint | `BRIEFING_PERSONAL_DENIED_DENIAL_HINT` `:54–55` | Already recall. `personal.rs:121` sets it. **Do not edit `personal.rs`.** |
| Empty continuity | `BRIEFING_EMPTY_CONTINUITY_NOTICE` `:58` / `NEXT_STEP` `:61` | Allowed-empty only. **Freeze** (T227 AC8). |
| Packet | `PersonalContinuityBriefingPacket` `contracts/briefings.rs:296` | `denied`, `denial_hint` skip if none, `preferences[]`, `continuity`. **No new fields.** |
| `empty_denied` | `:327` | Empty prefs/continuity. Markdown overlay only. |
| CLI | `briefing.rs` `run_personal` | `render_personal_markdown(&packet)`. No overlay needed if renderer is SoT. |
| clap | `main.rs` `BriefingCommands::Personal` `:1838–1853` | `--user-id` / `--max-words` default **800** / `--dry-run` default **true** / `--format`. **No new flag.** |
| Hermetic | `briefing_format_substance.rs` `briefing_personal__no_grants__soft_deny_denial_hint` `:353` | JSON only. Additive human AC. Header test `:222` does not assert `_None_`. |
| T263 unit | `renderer.rs` `:520` | Recall + no bootstrap + F35 consts. **Extend** with `!_None_` + new body. |
| T227 AC8 | `:706` | Allowed-empty `_None_` + empty_continuity **stays green.** |
| CP denied | `crates/ai-brains-control-plane/tests/personal_briefing.rs` `personal_briefing__without_grant__denied` **`:154`** (OpenCode m1: not `src/briefings/`) | Asserts Denied + reason; **no** `_None_` assert. |
| Hotspots | `personal.rs` #7 | Isolation: renderer-only. |

### 2.4 Deps / pins (researched 2026-08-23 — snapshot, re-verify at execute)

| Item | Workspace / lock | crates.io / upstream (this pass) | Decision |
|------|------------------|----------------------------------|----------|
| clap | Cargo.toml `4.5`; lock **4.6.1** | crates.io **4.6.6**. **No clap 5.** | **No bump** |
| rusqlite | **0.39.0** | crates.io **0.40.2** (`#61`) | **No bump** |
| serde_json | lock **1.0.150** | crates.io **1.0.151** | **No bump** |
| chrono | lock **0.4.44** | crates.io **0.4.45** (`#62`) | **No bump** |
| rustc / nextest / workspace | **1.95.0** / **0.9.140** / **0.1.2** | — | Freeze |
| Zero new crates | Required | — | No extras |

### 2.5 Online / product research

| Finding | Application |
|---------|-------------|
| [clig.dev](https://clig.dev/) — “Make it easy to see the current state”; distinguish errors from empty collections | Denied is **no access**, not empty prefs. T275 project analog. Human copy change is OK; JSON stays stable. |
| clig — suggest next command | T263 next already names `recall`. Do **not** add Personal bootstrap as next. |
| T180 PROTOCOL-COMPAT | No new packet keys. Human-only body. N−1 JSON unchanged. |
| T275 grant-wall | Project uses HIDDEN + GRANT_WALL. Personal **must not** copy those strings (F35). Dedicated optional body. |
| T227 #18 | No synthetic personal summary. **Affirm.** |
| N/A | SQLCipher, schtasks, llama `/health`, FTS5, clap 5 (not released). |

**Could not verify (plan pass):** GitHub clap-rs release page fetch empty. **Fold-in:** OpenCode re-verified crates.io clap **4.6.6** / no clap 5. Snapshot — re-verify at execute.

**ledgerful / ai-brains:** `briefing personal --format human` Denied + `_None_` + recall next. `search "render_personal_markdown"` → `renderer.rs:243` + CLI + CP tests. Hotspots `personal.rs` #7. Semantic recall of T263 F4 is plan-audit chrome — live src is SoT.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until **go**. Planning is DOCS. Implement starts a **FEATURE** TX. |
| **F1 — Denied human omits `_None_`** | When `packet.denied` and prefs empty / continuity summary empty: do **not** emit `_None_` under `## Preferences` / `## Continuity`. Headings **stay**. |
| **F2 — Personal denied body const** | New `BRIEFING_PERSONAL_DENIED_BODY` = `_(optional continuity; not a missing vault)_` (exact). One line, no `\n`, `.chars().count() <= 140`, contains `optional`, does **not** contain `_None_` or `policy bootstrap`. Used for **both** empty sections when denied. |
| **F3 — T275 F35 contamination** | Denied Personal markdown must **not** contain `BRIEFING_DENIED_GRANT_WALL`, `BRIEFING_DENIED_HIDDEN`, `BRIEFING_DENIED_NEXT_STEP`, or `policy bootstrap`. Do **not** call `empty_section_placeholder`. |
| **F4 — T263 next frozen** | Do **not** edit `BRIEFING_PERSONAL_DENIED_NEXT_STEP` / `BRIEFING_PERSONAL_DENIED_DENIAL_HINT`. Still the next-step / JSON hint. |
| **F5 — JSON freeze** | `denied: true`, `preferences: []`, `continuity.summary: ""` (never null), `denial_hint` recall. **No** new keys (`vault_pin_*`, grant-wall). E1 unchanged. T288 overlay is **`run_project` only**; `run_personal` `:263–267` is `render_personal_markdown` + `to_string_pretty(&packet)` (OpenCode m2). |
| **F6 — Allowed-empty freeze** | `!denied` empty still `_None_` + `BRIEFING_EMPTY_CONTINUITY_NOTICE` / `NEXT_STEP` (T227 AC8). Placeholder “granted-empty unused line” **already shipped** as NOTICE. **Do not** restyle as DoD. |
| **F7 — No auto Personal grant** | No `policy bootstrap` Personal as required next. No live Personal grant. Hermetic deny fixture is SoT. |
| **F8 — No H2 / no T288 steal** | No pin→Approved. No vault-pin stanza on Personal. Project `_None_` / T288 overlay **untouched**. |
| **F9 — No new clap flag** | Format aliases T227. Exit **0** soft deny. Unknown format exit **2**. |
| **F10 — DTO freeze** | Do **not** add fields to `PersonalContinuityBriefingPacket`. |
| **F11 — File growth** | Production: `renderer.rs` only (const + **private `fn`** `personal_empty_section_placeholder` — same visibility as `empty_section_placeholder` `:234`; **not** `pub`; **not** re-exported from `mod.rs`/`lib.rs` — Agy m1). Two call sites. Units in `renderer.rs`. Hermetic additive in `briefing_format_substance.rs`. **Do not** edit `personal.rs`, `project.rs`, CLI `preflight.rs`, `governed_common.rs`, `briefing.rs` run path (after_help sentence OK), `query_store.rs`, `ci.yml`. |
| **F12 — last-PR Cursor** | #204 empty → **N/A**. Dependabot not this track. **No T301.** |
| **F13 — PATH** | Do not `cargo install` unless the user asks. |
| **F14 — Capture independence** | Markdown + consts only. No models, events, graph. `--dry-run` default true stays. |
| **F15 — Tests** | Naming `function_or_feature__condition__expected_result`. **AC1 required red** unit. Hermetic AC2. |
| **F16 — Cross-model** | FEATURE (operator honesty). After Phase-1 clean, `codex-review`. |
| **F17 — Debt file** | `conductor/ISSUES.md` does **not** exist. |
| **F18 — Decline peers** | T290 lists; T291 trace; T288 Completed; T294 leftover. |
| **F19 — Decline pins** | T263 H2; T240 F2; clap 5; rusqlite 0.40; T227 #18 synthetic fill. |
| **F20 — Docs** | **Extend** CAPABILITIES Denied packets row (`:322` already names recall / optional continuity — OpenCode O2); do **not** add a new section. CHANGELOG T289. `briefing personal` after_help one sentence (denied human optional-body not `_None_` — Agy O2 already F20). |
| **F21 — PowerShell** | `;` not `&&`. |
| **F22 — Existing tests stay green** | T263 personal deny recall; T275 AC16; T227 AC8/AC9b; JSON denial_hint hermetic; format human header. |
| **F23 — Nonempty denied** | If denied packet ever has prefs/continuity text, render that text (don’t blank it). `empty_denied` is empty today. |
| **F24 — `#18` continuity** | No synthetic summary. Body is a placeholder, not generated prefs. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: reuse existing `empty_personal(true)` (`renderer.rs:383` — OpenCode O1; do **not** mint a second fixture). `render_personal_markdown(&empty_personal(true))` does **not** contain `_None_`; contains `BRIEFING_PERSONAL_DENIED_BODY` under both Preferences and Continuity; contains `BRIEFING_PERSONAL_DENIED_NEXT_STEP` / `recall`; does **not** contain `policy bootstrap` / GRANT_WALL / HIDDEN / `BRIEFING_DENIED_NEXT_STEP`. **Required red.** |
| **AC2** | Hermetic: `briefing personal --format human` (no Personal grants) exit **0**; stdout has `# Personal Continuity Briefing`, `**Denied:**`, `recall`; **no** `_None_`; **no** `policy bootstrap`. |
| **AC3** | Same fixture `--format json`: `denied: true`, `preferences` empty array, `continuity.summary` `""`, `denial_hint` contains `recall` not `policy bootstrap`; **no** `vault_pin_count` (T288 overlay is project-path only — `run_personal` `:266` serializes the packet; OpenCode m2). Existing `briefing_personal__no_grants__soft_deny_denial_hint` **stays green.** |
| **AC4** | Unit: `BRIEFING_PERSONAL_DENIED_BODY` **exact** `_(optional continuity; not a missing vault)_` (Agy m2); `!contains('\n')`; `chars().count() <= 140`; contains `optional`; **also** `!contains("_None_")` and `!contains("policy bootstrap")` on the **const** (OpenCode m3 — not only the rendered markdown). |
| **AC5** | Unit T227: `render_personal_markdown(&empty_personal(false))` still contains `## Preferences\n_None_` **and** `## Continuity\n_None_` + empty_continuity notice (**stays green**; Agy O1 Preferences lock). |
| **AC6** | T275 AC16 unit **stays green** (no project-wall leak). |
| **AC7** | T227 unknown `--format` still exit **2** zero stdout. |
| **AC8** | Docs: **extend** CAPABILITIES Denied packets row `:322` (already recall/optional — OpenCode O2) + `briefing personal` after_help one sentence + CHANGELOG T289. |
| **AC9** | No new crate. No clap 5. No `unwrap`/`expect`/`panic` in production. `cargo clippy -p ai-brains-cli -p ai-brains-control-plane --all-targets -- -D warnings` clean on go. |
| **AC10** | Manual (source/hermetic bin, not PATH): `cargo run -p ai-brains-cli -- briefing personal --format human` — no `_None_`; has `recall`; exit **0**. |
| **AC11** | `personal.rs` **unchanged** (diff). |
| **AC12** | `serde_json::to_value` of `empty_denied` has **no** new keys vs today. |

---

## 5. Design notes

### 5.1 No access ≠ empty prefs

Project T275: denied Decisions use `_(hidden until discovery grants)_` because discovery grants **are** the remediator. Personal T263: continuity is **optional**; remediator is `recall`, not Personal bootstrap. Therefore a **different** body string (`optional continuity; not a missing vault`).

### 5.2 Why renderer-only

JSON is already `denied: true` + empty arrays. The lie is markdown `_None_`. `personal.rs:121` already sets the hint. Growing hotspot `personal.rs` is unnecessary.

### 5.3 Helper

```text
fn personal_empty_section_placeholder(denied: bool) -> &'static str  // private, renderer.rs only
```

Denied → `BRIEFING_PERSONAL_DENIED_BODY`; else → `_None_`. Do **not** share `empty_section_placeholder`. Do **not** `pub` or re-export (Agy m1).

---

## 6. Non-goals

- Auto Personal grant / live `policy bootstrap`
- Pin → Approved (H2)
- Vault-pin stanza on Personal (T288)
- Lists/progressive pin count (T290)
- Synthetic continuity (#18)
- DTO new keys / clap 5 / rusqlite 0.40
- Lengthening T263 Personal next-step
- Growing `personal.rs` / `project.rs` / CLI `preflight.rs`
- `cargo install` / `.env` write

---

## 7. Verification plan (TDD)

**Red first:**

1. `render_personal_markdown__denied__no_none_placeholder` (AC1)
2. `briefing_personal_denied_body__exact_optional_one_line` (AC4)
3. `briefing_personal__no_grants__human_omits_none` (AC2)

Then AC3/AC5/AC6 stay-green; docs AC8; Manual AC10.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Copy project HIDDEN → trains bootstrap | F3 / AC1 / AC6 |
| Agents still see empty sections | F2 “optional / not a missing vault” + T263 recall next |
| Allowed-empty regresses | F6 / AC5 |
| DTO churn | F5 / F10 / AC12 |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| Audit `briefing personal` deny + `_None_` prefs U=4 | **Absorb** F1–F4 / AC1–AC2 / AC10 |
| Placeholder Manual `briefing personal --format human` | **Absorb** AC2 / AC10 |
| T275 F32 Personal `_None_` optional | **Absorb / promote** F1 |
| T275 F35 no project-wall | **Affirm** F3 / AC6 |
| T263 F4 recall next | **Affirm freeze** F4 |
| T227 empty_continuity allowed-empty | **Affirm** F6 / AC5 |
| T288 closeout Personal `_None_` | **Absorb** (this track) |
| T288 vault-pin stanza | **Decline** F8 — Completed project-only |
| Lists/progressive pin count | **Decline → T290** |
| T227 #18 synthetic continuity | **Decline** F24 |
| Auto Personal grant | **Decline** F7 |
| T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F19 |
| last-PR Cursor #204 | **N/A** empty — **no T301** |
| Identity leftover `7d97a456` | **Not this track** — T258 / T294 |
| Open T291–T300 | **Not related** except named declines |
| Closed T274–T288 | **Stay closed** |

---

## 10. Implement order (on go)

1. Phase 0 re-verify HEAD / deferred / #204 still empty / live personal read-only
2. FEATURE TX
3. Red AC1/AC4
4. Green: const + helper + two call sites in `renderer.rs`
5. Red/green AC2 hermetic; AC3/AC5/AC6 stay-green
6. Docs AC8
7. Clippy + nextest + deny/audit
8. Manual AC10
9. Phase-1 review → codex-review
10. Publish: push `track/T289-*` → PR → watch GHA `CI` green → squash-merge → prune

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| PATH until `cargo install` | F13 |
| Personal unused on this machine | Honest; deny is the live path |
| Allowed-empty `_None_` | F6 freeze |
| T290 lists | Peer |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-control-plane/src/briefings/renderer.rs` | Const + helper + prefs/continuity call sites; units AC1/AC4 |
| `crates/ai-brains-cli/tests/briefing_format_substance.rs` | Additive AC2 hermetic |
| `crates/ai-brains-cli/src/main.rs` | Personal after_help one sentence |
| `Docs/CAPABILITIES.md` | Denied Personal `_None_` → optional body |
| `CHANGELOG.md` | T289 (on go) |

**Do not touch:** `personal.rs`, `project.rs`, CLI `preflight.rs`, contracts packet fields, T288 project renderer path.

---

## 13. AI fold-in

Inputs (not edited): `agy-review.md` + `opencode-review.md` at HEAD `72fbb92`. Live verify: `empty_section_placeholder` **`fn`** `:234`; `_None_` `:263`/`:273`; `run_personal` emit `:263–267`; CP denied test `tests/personal_briefing.rs:154`; `empty_personal` `:383`; CAPABILITIES `:322`. Pins **snapshot — re-verify at execute** (clap lock 4.6.1 / crates.io 4.6.6; rusqlite 0.39.0; no clap 5).

### Pins locked by fold-in

1. **F11 (Agy m1):** `personal_empty_section_placeholder` is private `fn` in `renderer.rs` — not `pub`, not re-exported.
2. **AC5 (Agy O1):** allowed-empty asserts `## Preferences\n_None_` as well as Continuity.
3. **§2.3 (OpenCode m1):** CP denied test path is `crates/ai-brains-control-plane/tests/personal_briefing.rs:154`.
4. **AC3/F5 (OpenCode m2):** T288 overlay is `run_project` only; Personal JSON is raw packet serde.
5. **AC4 (OpenCode m3):** const itself `!contains("_None_")` / `!contains("policy bootstrap")`.
6. **AC8/F20 (OpenCode O2):** CAPABILITIES *extend* Denied row `:322`, not a new section.
7. **Already:** Agy m2 exact AC4 string; Agy O2 after_help; OpenCode O1 `empty_personal` fixture named in AC1.

### Per-AI disposition

| Source | Item | Disposition |
|--------|------|-------------|
| Agy | B / M | None filed |
| Agy | **m1** helper private `fn` | **Folded** F11 / §5.3 |
| Agy | **m2** AC4 exact string / one-line / ≤140 | **Already** F2 / AC4 |
| Agy | **O1** allowed-empty Preferences `_None_` | **Folded** AC5 |
| Agy | **O2** Personal after_help | **Already** F20 / AC8 |
| OpenCode | B / M | None filed |
| OpenCode | **m1** `personal_briefing.rs:154` vs tests crate | **Folded** §2.3 live `tests/personal_briefing.rs:154` |
| OpenCode | **m2** T288 overlay not on `run_personal` | **Folded** F5 / AC3 |
| OpenCode | **m3** const-level `_None_` / bootstrap guards | **Folded** AC4 |
| OpenCode | **O1** reuse `empty_personal` | **Already** AC1; **tightened** name `:383` |
| OpenCode | **O2** CAPABILITIES extend not add | **Folded** F20 / AC8 |
| both | last-PR #204 Cursor | **Affirm N/A** — no T301 |
| both | deferred T290 / H2 / T288 | **Affirm** |

No Blockers. No Majors. No new placeholder minted. Do **not** edit `*-review.md`.
