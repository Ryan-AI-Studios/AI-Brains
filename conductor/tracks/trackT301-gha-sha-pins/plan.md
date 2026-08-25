# T301 Plan — GHA SHA-pin refresh

**Status:** **Pending**. Spec [spec.md](./spec.md).
**Ledger:** series DOCS `30b7ca9d`. Fold-in DOCS `3402c6ed`. Implement **INFRA** on go.

## AI fold-in (2026-08-25)

OpenCode **M1** folded: peel annotated tags to **commit** SHA. Attest latest v4.x patch. last-PR `#217`. Disposition in spec **§13**.

## Phase 0 (on go)

- [ ] Fetch; branch `track/T301-gha-sha-pins`
- [ ] Re-resolve SHAs. For each tag: `gh api repos/<owner>/<repo>/git/ref/tags/<tag> --jq '{type: .object.type, sha: .object.sha}'`. If `type=="tag"`, peel: `gh api repos/<owner>/<repo>/git/tags/<sha> --jq .object.sha`. **Never** put a tag-object SHA in `uses:`.
- [ ] `softprops/action-gh-release` v3.0.2 is annotated (plan-day peel `3d0d9888cb7fd7b750713d6e236d1fcb99157228`) — re-verify
- [ ] Attest: latest v4.x tag (plan-day **v4.2.2**), not necessarily Dependabot v4.2.1
- [ ] Read live `ci.yml` + `release.yml` (do not trust plan-time SHAs). Count checkout call sites (expect 3 in ci + 1 in release)
- [ ] Do **not** merge `dependabot/github_actions/*`
- [ ] INFRA TX

## Tasks

- [ ] Replace five action SHAs + version comments in **both** workflows as applicable (all checkout jobs)
- [ ] Refresh release.yml SHA table comment dates (header ~`:14–22`)
- [ ] CHANGELOG Unreleased
- [ ] `rg` AC3 no floating `@vN` for those five (no new pin-checker script)
- [ ] PR → `gh run watch --exit-status` CI green → squash (never `git push origin main`)

## DoD

- [ ] SHA+comment pins at Dependabot target majors
- [ ] CI green; no crate diff
- [ ] Dependabot remotes/PRs not deleted until this squash (then GitHub may auto-close)
