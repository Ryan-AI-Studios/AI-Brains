# T301 — GitHub Actions SHA-pin refresh (Dependabot #68–#72)

- **Track ID:** T301-GhaShaPins
- **Status:** **Planned** (Pending until **go**)
- **Category:** INFRA / SECURITY
- **Owner:** Grok
- **Source:** Open Dependabot PRs `#68` upload-artifact 4→7, `#69` download-artifact 4→8, `#70` attest 2→4, `#71` action-gh-release 2→3, `#72` checkout 4→7 (2026-08-02). Owner requested tracks 2026-08-25 after T300 live rebuild.
- **Depends on:** T184 SHA-pin / token permissions; T185 F26 release.yml every `uses:` is full SHA + version comment; T186 R-CI-PIN ci.yml same
- **F0:** Plan-only until **go**. Do **not** merge Dependabot remotes. Do **not** `git push origin main`.
- **Ledger:** series DOCS TX `30b7ca9d-4932-4f00-97b8-82d5d25e633b`. Implement starts **INFRA** TX on go.

## 1. Objective

Refresh **SHA-pinned** GitHub Actions in `.github/workflows/ci.yml` and `release.yml` to the Dependabot target majors **without** floating tags. Keep T185 F26: `uses: owner/action@<40-hex> # vN.N.N`. Node 24 runners. checkout v7 blocks fork checkout on `pull_request_target` / `workflow_run` (we do not use those triggers today — verify). Artifact v7/v8 API must still upload/download release SBOMs.

## 2. Live baseline (2026-08-25)

| Signal | Observation |
|--------|-------------|
| HEAD | `2ed5b06` workspace 0.1.3 `#217`. Tree CLEAN at mint. |
| Pins | `actions/checkout@11d5960a… # v4` in **both** workflows. `upload-artifact@ea165f8d… # v4`. `download-artifact@d3f86a10… # v4`. `attest@ce27ba3b… # v2.4.0`. `softprops/action-gh-release@3bb12739… # v2`. dtolnay/rust-toolchain + Swatinem/rust-cache **unchanged this track**. |
| Dependabot | Open `#68–#72`. **Do not merge those branches.** Recreate SHAs on `track/T301-*`. |
| last-PR Cursor | `#216` empty. **No T306 from Cursor.** |
| Triggers | CI: `pull_request` / `push main` / `workflow_dispatch`. Release: `v*` tags + `workflow_dispatch`. **No** `pull_request_target` / `workflow_run`. |

**Research (snapshot — re-verify at execute):** checkout **v7.0.1** (2026) — Node 24; v7.0.0 blocks fork PR checkout on `pull_request_target`/`workflow_run`. v6 moved persist-credentials off `.git/config`. GitHub runners default Node 24 (2026-06-16); old majors warn then fail. Pin via `gh api repos/actions/checkout/git/ref/tags/v7.0.1`. Same for the other four actions' **latest matching tag SHA**.

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0** | Plan-only until go. |
| **F1** | Full 40-char SHA + `# vX.Y.Z` comment on every bumped `uses:` (T185 F26). **No** `@v7` floating tags. |
| **F2** | Bump **all five** Dependabot GHA PRs in this track (ci.yml + release.yml). rust-toolchain / rust-cache **out** unless a SHA is forced by the others. |
| **F3** | Do not merge `dependabot/github_actions/*`. Cherry-pick file hunks only after SHA verify. |
| **F4** | Confirm workflows still have no `pull_request_target` / `workflow_run` (checkout v7 fork block N/A but document). |
| **F5** | Release attest job stays **soft** (T185 skippable). Do not require SLSA L3. |
| **F6** | Never `git push origin main`. Track branch → PR → watch `CI` green → squash. |
| **F7** | Zero crate bumps. Zero product Rust. |
| **F8** | last-PR `#216` Cursor N/A. T302–T305 cargo **not stolen**. |
| **F9** | Docs: CHANGELOG Unreleased; release.yml SHA table comment dates refreshed. |

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | `rg "uses: actions/checkout@"` → SHA is checkout **v7.0.1** (or current v7.x at execute) 40-hex; comment names the tag. Same file in ci.yml **and** release.yml. |
| **AC2** | upload-artifact v7.x, download-artifact v8.x, attest v4.x, action-gh-release v3.x — SHA+comment in release.yml (and ci.yml if present). |
| **AC3** | `rg "uses: .*/(checkout\|upload-artifact\|download-artifact\|attest\|action-gh-release)@v[0-9]"` finds **zero** floating tags. |
| **AC4** | No `pull_request_target` / `workflow_run` in `.github/workflows`. |
| **AC5** | GHA `CI` on the track PR: every job green. |
| **AC6** | CHANGELOG row; release.yml header SHA table updated. |
| **AC7** | No `Cargo.lock` / crate edits. |

## 5–6. Design / non-goals

Resolve each tag object SHA with `gh api` at execute (do not trust Dependabot lock if it only retargets majors). Test CI on the PR; release.yml is tag-only — review YAML, do not cut a `v*` tag.

**Non-goals:** rust-cache bump; Node setup action; merging Dependabot remotes; clap 5; rusqlite (T305); tokio (T303).

## 7. Verification

Red: a unit or `rg` test in `scripts/` **or** a documented `rg` checklist in plan (existing SHA-pin script if any — Phase 0 locate `scripts/check-*.ps1`). Prefer extending an existing pin checker over a new crate test.

Manual: `gh run watch` CI green.

## 8–12. Risk / deferred / order / residuals / touch

**Risk:** artifact v4→v7/v8 input rename; attest v2→v4 permission keys. Mitigation: read each action README at execute.

**§9:** Absorb `#68–#72`. Decline clap 5 / rusqlite steal. last-PR `#216` N/A.

**Order:** Phase 0 re-resolve SHAs online → edit YAML → PR → watch CI.

**Residuals:** Node 20 deprecation timeline; dtolnay/rust-toolchain unpinned-from-Dependabot.

**Touch:** `.github/workflows/ci.yml`, `release.yml`, `CHANGELOG.md`, conductor.

**Isolation:** Do not delete Dependabot remotes. Do not mutate live schtasks. Do not `cargo install`.
