# T301 — GitHub Actions SHA-pin refresh (Dependabot #68–#72)

- **Track ID:** T301-GhaShaPins
- **Status:** **Completed** (2026-08-25)
- **Category:** INFRA / SECURITY
- **Owner:** Grok
- **Source:** Open Dependabot PRs `#68` upload-artifact 4→7, `#69` download-artifact 4→8, `#70` attest 2→4, `#71` action-gh-release 2→3, `#72` checkout 4→7 (2026-08-02). Owner requested tracks 2026-08-25 after T300 live rebuild.
- **Depends on:** T184 SHA-pin / token permissions; T185 F26 release.yml every `uses:` is full SHA + version comment; T186 R-CI-PIN ci.yml same
- **F0:** Plan-only until **go**. Do **not** merge Dependabot remotes. Do **not** `git push origin main`.
- **Ledger:** series DOCS TX `30b7ca9d-4932-4f00-97b8-82d5d25e633b`. Fold-in DOCS TX `3402c6ed-1134-4a4e-b974-130153d6cc4c`. Implement starts **INFRA** TX on go.
- **AI fold-in:** 2026-08-25 `agy-review.md` + `opencode-review.md` (HEAD `510a5c1`). **Agy B 0 / M 0.** **OpenCode B 0 / M 1** (annotated-tag SHA). **Agree:** OpenCode M1 peel to commit SHA; OpenCode m1 last-PR `#217`; Agy m1 / OpenCode O1 attest latest v4.x patch; Agy O1 all checkout call sites; OpenCode O2 no SHA-pin script; OpenCode O3 drop unverified Node-24 date. **Already:** Agy m3 = F9/AC6; Agy O2 = AC3. **Decline:** Agy m2 “tag-object SHA also works in GHA” — docs require a **commit** SHA ([Secure use](https://docs.github.com/en/actions/reference/security/secure-use): pin full-length **commit** SHA). Disposition **§13**.

## 1. Objective

Refresh **SHA-pinned** GitHub Actions in `.github/workflows/ci.yml` and `release.yml` to the Dependabot target majors **without** floating tags. Keep T185 F26: `uses: owner/action@<40-hex> # vN.N.N`. Node 24 runners. checkout v7 blocks fork checkout on `pull_request_target` / `workflow_run` (we do not use those triggers today — verify). Artifact v7/v8 API must still upload/download release SBOMs.

## 2. Live baseline (2026-08-25)

| Signal | Observation |
|--------|-------------|
| HEAD | `2ed5b06` workspace 0.1.3 `#217`. Tree CLEAN at mint. |
| Pins | `actions/checkout@11d5960a… # v4` in **both** workflows (**3** jobs in `ci.yml`: gate-windows / gate-linux / gate-macos + release `build-scan`). `upload-artifact@ea165f8d… # v4`. `download-artifact@d3f86a10… # v4`. `attest@ce27ba3b… # v2.4.0`. `softprops/action-gh-release@3bb12739… # v2`. dtolnay/rust-toolchain + Swatinem/rust-cache **unchanged this track**. |
| Dependabot | Open `#68–#72`. **Do not merge those branches.** Recreate SHAs on `track/T301-*`. |
| last-PR Cursor | `#217` (workspace 0.1.3) **and** `#216` (T300) — comments/reviews **empty**. **No T306 from Cursor.** |
| Triggers | CI: `pull_request` / `push main` / `workflow_dispatch`. Release: `v*` tags + `workflow_dispatch`. **No** `pull_request_target` / `workflow_run`. |
| Annotated tags | Fold-in verify 2026-08-25: checkout/upload/download/attest tags are **commit** objects. `softprops/action-gh-release` **v3.0.2** is type **`tag`** `fe965f7af51af5f2602596916f38a38df2e33de0` → peeled commit **`3d0d9888cb7fd7b750713d6e236d1fcb99157228`**. Pin the **commit**. |

**Research (snapshot — re-verify at execute):** checkout **v7.0.1** — Node 24 runtime; v7.0.0 blocks fork PR checkout on `pull_request_target`/`workflow_run`. v6 moved persist-credentials off `.git/config`. Node 24 is the action `runs.using` for these majors (attest v4 / gh-release v3); do **not** cite an unverified “runners defaulted 2026-06-16” date. **Pin peeled commit SHAs** ([Secure use](https://docs.github.com/en/actions/reference/security/secure-use): full-length **commit** SHA). `gh api …/git/ref/tags/<tag>` returns a **tag object** for annotated tags — dereference via `gh api …/git/tags/<tag-object-sha> --jq .object.sha` (or `git ls-remote --tags` peeled). Attest: Dependabot `#70` is v4.2.1; upstream latest patch **v4.2.2** (`1e69f48a…`) — **use latest v4.x patch at execute**.

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0** | Plan-only until go. |
| **F1** | Full 40-char **commit** SHA + `# vX.Y.Z` comment on every bumped `uses:` (T185 F26). **No** `@v7` floating tags. **No** annotated-tag object SHA (OpenCode M1). |
| **F2** | Bump **all five** Dependabot GHA PRs in this track (ci.yml + release.yml). rust-toolchain / rust-cache **out** unless a SHA is forced by the others. |
| **F3** | Do not merge `dependabot/github_actions/*`. Cherry-pick file hunks only after SHA verify. |
| **F4** | Confirm workflows still have no `pull_request_target` / `workflow_run` (checkout v7 fork block N/A but document). |
| **F5** | Release attest job stays **soft** (T185 skippable). Do not require SLSA L3. |
| **F6** | Never `git push origin main`. Track branch → PR → watch `CI` green → squash. |
| **F7** | Zero crate bumps. Zero product Rust. |
| **F8** | last-PR `#217` **and** `#216` Cursor N/A. T302–T305 cargo **not stolen**. **No T306.** |
| **F9** | Docs: CHANGELOG Unreleased; `release.yml` header SHA table (lines **14–22** at plan) refreshed with new SHAs + resolution date. |
| **F10** | Phase 0 SHA resolution: if `git/ref/tags` `.object.type == "tag"`, peel with `git/tags/<sha>` to `.object.sha` (commit). `softprops/action-gh-release` v3.0.2 is the annotated case today — pin **`3d0d9888cb7fd7b750713d6e236d1fcb99157228`** (re-verify at execute). |
| **F11** | Attest: pin **latest v4.x patch at execute** (plan-day **v4.2.2** `1e69f48acb82d1966a394da916b4c1698aa569d6`), not necessarily Dependabot’s v4.2.1. |

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Every `actions/checkout` call site is v7.x 40-hex **commit** + comment: **all three** `ci.yml` jobs (gate-windows / gate-linux / gate-macos) **and** `release.yml` `build-scan`. |
| **AC2** | upload-artifact v7.x, download-artifact v8.x, attest **latest v4.x patch** (F11), action-gh-release v3.x **peeled commit** (F10) — SHA+comment in release.yml (and ci.yml if present). |
| **AC3** | `rg "uses: .*/(checkout\|upload-artifact\|download-artifact\|attest\|action-gh-release)@v[0-9]"` finds **zero** floating tags. |
| **AC4** | No `pull_request_target` / `workflow_run` in `.github/workflows`. |
| **AC5** | GHA `CI` on the track PR: every job green. |
| **AC6** | CHANGELOG row; release.yml header SHA table updated. |
| **AC7** | No `Cargo.lock` / crate edits. |

## 5–6. Design / non-goals

Resolve each tag to a **commit** SHA at execute (F10). Do not paste `git/ref/tags` `.object.sha` when `.object.type == "tag"`. Do not trust Dependabot hunks without peel-verify. Test CI on the PR; release.yml is tag-only — review YAML, do not cut a `v*` tag.

**Non-goals:** rust-cache bump; Node setup action; merging Dependabot remotes; clap 5; rusqlite (T305); tokio (T303).

## 7. Verification

Red: documented `rg` checklist in plan (AC3). **No** existing SHA-pin checker: `scripts/` has `check-release-claims.ps1` + `check-version-banners.ps1` only (OpenCode O2). Do **not** mint a new pin-checker script this track.

Manual: `gh run watch` CI green.

## 8–12. Risk / deferred / order / residuals / touch

**Risk:** artifact v4→v7/v8 input rename; attest v2→v4 permission keys. Mitigation: read each action README at execute.

**§9:** Absorb `#68–#72`. Decline clap 5 / rusqlite steal. last-PR `#217`/`#216` N/A. Annotated-tag peel = F10 (not a new track).

**Order:** Phase 0 re-resolve SHAs online → edit YAML → PR → watch CI.

**Residuals:** Node 20 deprecation timeline; dtolnay/rust-toolchain unpinned-from-Dependabot.

**Touch:** `.github/workflows/ci.yml`, `release.yml`, `CHANGELOG.md`, conductor.

**Isolation:** Do not delete Dependabot remotes. Do not mutate live schtasks. Do not `cargo install`.

---

## 13. AI fold-in

Inputs (not edited): `agy-review.md` + `opencode-review.md` (HEAD `510a5c1`). Fold-in verify: `softprops/action-gh-release` v3.0.2 `git/ref/tags` type **`tag`** `fe965f7a…` peels to commit **`3d0d9888…`**; attest v4.2.2 exists (`1e69f48a…`); last merged PR **#217**; `#216`/`#217` comments **empty**. GitHub [Secure use](https://docs.github.com/en/actions/reference/security/secure-use): pin full-length **commit** SHA.

### Pins locked by fold-in

1. **F10 / Phase 0 (OpenCode M1):** Peel annotated tags; `uses:` SHA is always a **commit** object.
2. **F8 (OpenCode m1):** last-PR Cursor is `#217` (and `#216`); both empty; no T306.
3. **F11 (Agy m1 / OpenCode O1):** attest **latest v4.x patch at execute** (plan-day v4.2.2).
4. **AC1 (Agy O1):** all checkout call sites (3× ci.yml + release.yml).
5. **§7 (OpenCode O2):** no SHA-pin script; `rg` checklist is DoD.
6. **§2 research (OpenCode O3):** drop unverified “runners defaulted 2026-06-16” date.

### Per-AI disposition

| Source | Item | Disposition |
|--------|------|-------------|
| Agy | B / M | None filed |
| Agy | **m1** attest v4.2.2 | **Folded** F11 |
| Agy | **m2** tag vs commit SHA “both work” | **Decline** “both work”; **partial** document peel — F10 pins **commit** only |
| Agy | **m3** release.yml header table | **Already** F9 / AC6; tightened “lines 14–22” |
| Agy | **O1** three ci.yml checkout jobs | **Folded** AC1 |
| Agy | **O2** `rg` floating-tag check | **Already** AC3 |
| OpenCode | **M1** annotated-tag object SHA | **Folded** F10 / Phase 0 |
| OpenCode | **m1** last-PR `#217` | **Folded** F8 / §2 |
| OpenCode | **O1** attest 4.2.1 vs 4.2.2 | **Folded** F11 |
| OpenCode | **O2** no pin-checker script | **Folded** §7 |
| OpenCode | **O3** Node-24 date | **Folded** drop date |
| both | last-PR Cursor empty | **Affirm** — no T306 |

No Blockers. OpenCode **M1** folded (not declined). No new placeholder. Do **not** edit `*-review.md`.
