# T302 — Cargo patch: thiserror 2.0.20 + chrono 0.4.45

- **Track ID:** T302-CargoPatchThiserrorChrono
- **Status:** **Completed** (2026-08-25)
- **Category:** CHORE / DEPS
- **Owner:** Grok
- **Source:** Dependabot `#60` thiserror 2.0.18→**2.0.20**; `#62` chrono 0.4.44→**0.4.45**. Owner requested 2026-08-25.
- **Depends on:** workspace `thiserror = "2.0"` (caret already allows 2.0.20); `chrono = { version = "0.4", … }` (caret already allows 0.4.45). Lockfile is the actual pin.
- **F0:** Plan-only until go. Do **not** merge Dependabot remotes. Do **not** steal T303 tokio / T304 tower-http / T305 rusqlite / T301 GHA.
- **Ledger:** series DOCS TX `30b7ca9d-4932-4f00-97b8-82d5d25e633b`. Fold-in DOCS TX `a7caf3bc-3da5-4cf7-9b27-7aabec6372b4`. Implement starts **CHORE** TX on go.
- **AI fold-in:** 2026-08-25 `agy-review.md` + `opencode-review.md` (HEAD `4fce4e5`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** **Agree:** Agy m2+m3 precise `thiserror@2.0.18` (pkgid is ambiguous with 1.0.69); OpenCode m1 changelog text; OpenCode m2 last-PR `#218`; OpenCode O1 `#62` windows-core edge; OpenCode O2 syn 3 already in lock. **Already:** Agy m1 = F2/AC3/AC5; Agy O1 = AC4. **Decline:** Agy chrono 0.4.45 “DateTime Copy + `Weekday::days_since`” — GitHub [v0.4.45](https://github.com/chronotope/chrono/releases/tag/v0.4.45) is tz-only. Disposition **§13**.

## 1. Objective

Advance **Cargo.lock** (and workspace pin **only if required**) for two **patch** crates already in the 2.0 / 0.4 caret ranges. Full gate: fmt/clippy/nextest/deny/audit. No API rewrite.

## 2. Live baseline (2026-08-25 fold-in)

| Pin | Workspace | Lock | crates.io (fold-in) | Action |
|-----|-----------|------|---------------------|--------|
| thiserror | **2.0** (`Cargo.toml:41`) | **2.0.18** *and* transitive **1.0.69** | **2.0.20** (`#60`) | `cargo update -p thiserror@2.0.18 --precise 2.0.20` |
| chrono | **0.4** + serde (`Cargo.toml:48`) | **0.4.44** | **0.4.45** (`#62`) | `cargo update -p chrono@0.4.44 --precise 0.4.45` |
| clap / rusqlite / tokio / tower-http | 4.5 / 0.39.0 / 1.52 / 0.6 | 4.6.1 / 0.39.0 / 1.52.3 / 0.6.11 | — | **Do not bump** |

**`cargo pkgid thiserror` is ambiguous** (`thiserror@1.0.69` via json-patch/Tauri desktop; `thiserror@2.0.18` workspace). Bare `-p thiserror` fails. Use the **2.x pkgid**. Leave 1.0.69 untouched.

**Research (verified fold-in; re-verify changelog at execute):**

- thiserror **2.0.19** = [“Update to syn 3”](https://github.com/dtolnay/thiserror/releases/tag/2.0.19) (`thiserror-impl` syn 2.0.117 → syn 3.0.3). **2.0.20** = [“Suppress `redundant_field_names` clippy lint in generated code (#454)”](https://github.com/dtolnay/thiserror/releases/tag/2.0.20). rust-version **1.71**. MIT OR Apache-2.0. `#60` lock diff is thiserror 2.x version/checksum + `thiserror-impl` syn edge + workspace/tauri *2.x* edges — not 1.x.
- chrono **0.4.45** (2026-06-04) = tz-only: [reject TZ offset hour of 24 (`FixedOffset` overflow) #1787](https://github.com/chronotope/chrono/releases/tag/v0.4.45) + Android tzdata paths #1789. **Not** serde. **Not** DateTime Copy / `Weekday::days_since` (those are not this tag; 0.4.44 is docs MSRV + `track_caller`). rust-version **1.62.0**. Product use `crates/ai-brains-contracts/src/time_convert.rs:12-15` is `Utc.timestamp_opt` → `LocalResult::{Single, Ambiguous, None}` — stable through 0.4.45.
- syn **3.0.3 already in lock** (`serde_repr 0.1.21`). thiserror-impl 2.0.20 adds no new package.
- `#62` also re-resolves `iana-time-zone 0.1.65` Windows edge `windows-core 0.62.2` → **0.61.2** (crates.io req `>=0.56, <=0.62`). Both windows-core versions remain (0.61.2 tauri/tao; 0.62.2 `windows 0.62.2`). Expected — do not revert.

Workspace toolchain `rust-toolchain.toml` channel **1.95.0** (above both rust-version floors).

last-PR Cursor: **`#218`** (T301, HEAD) **and** `#217` (workspace 0.1.3) — comments/reviews **empty**. **No T306.**

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0** | Plan-only until go. |
| **F1** | Only thiserror **2.x** + chrono. No tokio/tower-http/rusqlite/clap. |
| **F2** | Prefer lockfile-only if workspace carets already match. Do not widen to 3.x / 0.5. |
| **F3** | `cargo deny check` + `cargo audit` must stay green. |
| **F4** | No product src edits unless clippy `-D warnings` forces a one-line fix (keep separate if unrelated). |
| **F5** | Do not merge `dependabot/cargo/thiserror*` / `chrono*` remotes. |
| **F6** | Never `git push origin main`. |
| **F7** | CHANGELOG Unreleased chore row. |
| **F8** | Unambiguous pkgids + `--precise` ([cargo-update](https://doc.rust-lang.org/cargo/commands/cargo-update.html)): `cargo update -p thiserror@2.0.18 --precise 2.0.20` ; `cargo update -p chrono@0.4.44 --precise 0.4.45`. Re-check latest patch at execute. **Do not** `cargo update -p thiserror` (ambiguous with 1.0.69). |
| **F9** | Expected lock extras — **not** unrelated: (a) `thiserror-impl` 2.0.20 → `syn 3.0.3` (already locked); (b) `iana-time-zone` windows-core **0.62.2 → 0.61.2** (both versions stay). Do not revert. `thiserror 1.0.69` must remain. |

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | `Cargo.lock` thiserror **2.0.20** (or current patch ≥2.0.20 same 2.0 line at execute). |
| **AC2** | `Cargo.lock` chrono **0.4.45** (or current 0.4.x ≥0.4.45). |
| **AC3** | Workspace toml still `thiserror = "2.0"` and `chrono = { version = "0.4", … }` unless lock cannot resolve — then still 2.0 / 0.4 carets. |
| **AC4** | `cargo clippy --workspace --all-targets -- -D warnings` + `cargo nextest run --workspace` + `cargo deny check` + `cargo audit` green. |
| **AC5** | `git diff -- crates/` empty (or only the documented clippy one-liner). |
| **AC6** | CHANGELOG Unreleased. |
| **AC7** | Lock allowlist: thiserror 2.x + chrono + F9 transitives only. `thiserror 1.0.69` unchanged. No rusqlite / tokio / tower-http / clap version bump. |

## 5–12

**Non-goals:** T301/T303/T304/T305; clap 5; edition bump; thiserror 1.x; Tauri stack; new crates.

**Risk:** deny license false-positive on a transitive — triage, do not broadly clean up. `--precise` still re-resolves the F9 windows-core edge (Dependabot `#62` does). Abort only if tokio/rusqlite/tower-http/clap or thiserror 1.x move.

**§9:** Absorb `#60` `#62`. Decline rusqlite/tokio/tower-http steal (T305/T303/T304). last-PR `#218` / `#217` N/A empty — **no T306**.

**Touch:** `Cargo.lock` (maybe not `Cargo.toml`); `CHANGELOG.md`; conductor.

**Isolation:** No live vault mutate; no `cargo install`; no Dependabot remote delete until squash.

---

## 13. AI fold-in

Inputs (not edited): `agy-review.md` + `opencode-review.md` (HEAD `4fce4e5`). Fold-in verify: `cargo pkgid thiserror` → ambiguous `1.0.69` / `2.0.18`; lock thiserror-impl 2.0.18 → syn 2.0.117; syn 3.0.3 already via serde_repr 0.1.21; iana-time-zone 0.1.65 → windows-core 0.62.2 today; `#62` flips that edge to 0.61.2; GitHub thiserror 2.0.19/2.0.20 and chrono v0.4.45 as §2; last merged `#218` comments/reviews empty; `#217` empty; `time_convert.rs` LocalResult unchanged.

### Pins locked by fold-in

1. **F8 (Agy m2 + m3):** `thiserror@2.0.18 --precise 2.0.20` and `chrono@0.4.44 --precise 0.4.45`. Bare `-p thiserror` is illegal here.
2. **F9 / AC7 (OpenCode O1 + O2):** expected syn 3 edge + windows-core edge; thiserror 1.0.69 stays; do not abort the gate over those two lock lines.
3. **§2 (OpenCode m1):** thiserror = syn 3 + clippy #454; chrono 0.4.45 = tz-only.
4. **§2 / §9 (OpenCode m2):** last-PR Cursor is `#218` (and `#217`); both empty; no T306.

### Per-AI disposition

| Source | Item | Disposition |
|--------|------|-------------|
| Agy | B / M | None filed |
| Agy | **m1** lockfile-only `Cargo.toml` | **Already** F2 / AC3 / AC5 |
| Agy | **m2** thiserror 1.x isolation | **Folded** F8 / AC7 — verified `thiserror@1.0.69` via json-patch/Tauri |
| Agy | **m3** `--precise` | **Folded** F8 — [cargo-update `--precise`](https://doc.rust-lang.org/cargo/commands/cargo-update.html) |
| Agy | **O1** clippy `--all-targets` | **Already** AC4 |
| Agy | chrono 0.4.45 “DateTime Copy / `days_since`” | **Decline** — not v0.4.45 (tz #1787/#1789). Re-trigger: chrono ships a different 0.4.x than 0.4.45 |
| OpenCode | B / M | None filed |
| OpenCode | **m1** stale spec research | **Folded** §2 |
| OpenCode | **m2** last-PR `#216` → `#218` | **Folded** §2 / §9 |
| OpenCode | **O1** `#62` windows-core edge | **Folded** F9 / AC7. Range from crates.io is `>=0.56, <=0.62` (not ≤0.63) |
| OpenCode | **O2** syn 3 already in graph | **Folded** F9; note in PR body at execute |
| both | last-PR Cursor empty | **Affirm** — `#218` + `#217` N/A; **no T306** |

No Blockers/Majors to decline. No new placeholder. Do **not** edit `*-review.md`. Do **not** execute until go.
