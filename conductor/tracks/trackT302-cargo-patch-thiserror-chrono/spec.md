# T302 — Cargo patch: thiserror 2.0.20 + chrono 0.4.45

- **Track ID:** T302-CargoPatchThiserrorChrono
- **Status:** **Planned** (Pending until **go**)
- **Category:** CHORE / DEPS
- **Owner:** Grok
- **Source:** Dependabot `#60` thiserror 2.0.18→**2.0.20**; `#62` chrono 0.4.44→**0.4.45**. Owner requested 2026-08-25.
- **Depends on:** workspace `thiserror = "2.0"` (caret already allows 2.0.20); `chrono = { version = "0.4", … }` (caret already allows 0.4.45). Lockfile is the actual pin.
- **F0:** Plan-only until go. Do **not** merge Dependabot remotes. Do **not** steal T303 tokio / T304 tower-http / T305 rusqlite / T301 GHA.
- **Ledger:** series DOCS TX `30b7ca9d-4932-4f00-97b8-82d5d25e633b`.

## 1. Objective

Advance **Cargo.lock** (and workspace pin **only if required**) for two **patch** crates already in the 2.0 / 0.4 caret ranges. Full gate: fmt/clippy/nextest/deny/audit. No API rewrite.

## 2. Live baseline (2026-08-25)

| Pin | Workspace | Lock | crates.io (plan day) | Action |
|-----|-----------|------|----------------------|--------|
| thiserror | **2.0** | **2.0.18** | **2.0.20** (`#60`) | `cargo update -p thiserror` (or equivalent) → 2.0.20 |
| chrono | **0.4** + serde | **0.4.44** | **0.4.45** (`#62`) | `cargo update -p chrono` → 0.4.45 |
| clap / rusqlite / tokio | 4.5 / 0.39.0 / 1.52 | 4.6.1 / 0.39.0 / 1.52.3 | — | **Do not bump** |

**Research:** thiserror 2.0.19/2.0.20 are patch diagnostic fixes (Dependabot `#60` body). chrono 0.4.45 is a 0.4 patch (serde/tz). Semver-compatible with workspace carets. **Snapshot — re-verify changelog at execute.**

last-PR `#216` Cursor empty.

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0** | Plan-only until go. |
| **F1** | Only thiserror + chrono. No tokio/tower-http/rusqlite/clap. |
| **F2** | Prefer lockfile-only if workspace carets already match. Do not widen to 3.x / 0.5. |
| **F3** | `cargo deny check` + `cargo audit` must stay green. |
| **F4** | No product src edits unless clippy `-D warnings` forces a one-line fix (keep separate if unrelated). |
| **F5** | Do not merge `dependabot/cargo/thiserror*` / `chrono*` remotes. |
| **F6** | Never `git push origin main`. |
| **F7** | CHANGELOG Unreleased chore row. |

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | `Cargo.lock` thiserror **2.0.20** (or current patch ≥2.0.20 same 2.0 line at execute). |
| **AC2** | `Cargo.lock` chrono **0.4.45** (or current 0.4.x ≥0.4.45). |
| **AC3** | Workspace toml still `thiserror = "2.0"` and `chrono = { version = "0.4", … }` unless lock cannot resolve — then still 2.0 / 0.4 carets. |
| **AC4** | `cargo clippy --workspace --all-targets -- -D warnings` + `cargo nextest run --workspace` + `cargo deny check` + `cargo audit` green. |
| **AC5** | `git diff -- crates/` empty (or only the documented clippy one-liner). |
| **AC6** | CHANGELOG Unreleased. |

## 5–12

**Non-goals:** T301/T303/T304/T305; clap 5; edition bump.

**Risk:** deny license false-positive on a transitive — triage, do not broadly clean up.

**§9:** Absorb `#60` `#62`. Decline rusqlite steal. last-PR `#216` N/A.

**Touch:** `Cargo.lock` (maybe not `Cargo.toml`); `CHANGELOG.md`; conductor.

**Isolation:** No live vault mutate; no `cargo install`; no Dependabot remote delete until squash.
