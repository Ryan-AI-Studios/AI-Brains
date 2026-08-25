# T301–T305 — Dependabot upgrades (owner-requested)

**Source:** Open Dependabot PRs as of 2026-08-25 after T300 `#216` + workspace 0.1.3 `#217`. Owner asked to **mint tracks** (not merge remotes). **Do not** `git push origin main`. **Do not** merge Dependabot branches as-is (T185 SHA-pin + SQLCipher honesty).
**Status:** **T301 Completed** (INFRA TX `3571d90d…`); **T302 Completed** (CHORE TX `aec6d64e…`); **T303 Completed** (CHORE TX `46c31a21…`); T304–T305 **Planned / Pending**. Full F-list in each spec. **Do not implement T304–T305 until go.**
**Ledger:** planning DOCS TX `30b7ca9d-4932-4f00-97b8-82d5d25e633b`. **T301 fold-in:** DOCS TX `3402c6ed-1134-4a4e-b974-130153d6cc4c`. **T301 implement:** INFRA `3571d90d-b7c2-4204-8556-7a2b50c2d017`. **T302 implement:** CHORE `aec6d64e-82e4-4593-ab1c-628f3112d329`. **T303 implement:** CHORE `46c31a21-dc44-4f2d-b037-0290bb792bb4`.
**last-PR Cursor:** [#219](https://github.com/Ryan-AI-Studios/AI-Brains/pull/219) T302 — **empty**. **No T306.** Dependabot remotes `#58–#62`, `#68–#72` are this series.

## PR → track map

| PR | Bump | Risk | Track | Pri |
|----|------|------|-------|-----|
| #68 upload-artifact 4.6.2 → 7.0.1 | GHA major | SHA-pin + artifact API | **T301** | P1 |
| #69 download-artifact 4.3.0 → 8.0.1 | GHA major | SHA-pin + artifact API | **T301** | P1 |
| #70 attest 2.4.0 → 4.2.1 | GHA major | SHA-pin + id-token | **T301** | P1 |
| #71 action-gh-release 2.6.2 → 3.0.2 | GHA major | SHA-pin + release job | **T301** | P1 |
| #72 checkout 4.4.0 → 7.0.1 | GHA major | Node 24; fork `pull_request_target` block | **T301** | P1 |
| #60 thiserror 2.0.18 → 2.0.20 | cargo patch | Low | **T302** | P2 |
| #62 chrono 0.4.44 → 0.4.45 | cargo patch | Low | **T302** | P2 |
| #59 tokio 1.52.3 → 1.53.1 | cargo minor | Windows signal; daemon/CLI async | **T303** | P1 |
| #58 tower-http 0.6.11 → 0.7.0 | cargo minor | Breaking ServeDir/compression; API CORS | **T304** | P1 |
| #61 rusqlite 0.39.0 → 0.40.2 | cargo minor | SQLCipher **4.10 → 4.14**; VTab; COMPATIBILITY F8 | **T305** | P0 |

## Suggested implement order

1. **T302** (smallest cargo patch) then **T303** (tokio) then **T304** (HTTP)
2. **T301** (GHA SHA re-pin; does not block cargo)
3. **T305** last (vault/SQLCipher; re-probe `cipher_version`; Stop-Before if open/encrypt KATs fail)

Never merge `dependabot/*` remotes. Cherry-pick or recreate on `track/TNN-*` after reviewing SHA/changelog. Do not delete Dependabot remotes/PRs until the corresponding track squash-merges.

## Standing declines (not reopened except T305 rusqlite)

- clap 5 (not in this Dependabot batch)
- T240 F2 silent Scope switch
- Floating `@v7` action tags (T185 F26)
