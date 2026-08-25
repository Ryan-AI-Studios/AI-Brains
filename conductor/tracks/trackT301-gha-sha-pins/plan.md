# T301 Plan — GHA SHA-pin refresh

**Status:** **Pending**. Spec [spec.md](./spec.md).
**Ledger:** series DOCS `30b7ca9d`. Implement **INFRA** on go.

## Phase 0 (on go)

- [ ] Fetch; branch `track/T301-gha-sha-pins`
- [ ] Re-resolve SHAs: `gh api repos/actions/checkout/git/ref/tags/v7.0.1` (and upload-artifact / download-artifact / attest / action-gh-release current tags)
- [ ] Read live `ci.yml` + `release.yml` (do not trust plan-time SHAs)
- [ ] Do **not** merge `dependabot/github_actions/*`
- [ ] INFRA TX

## Tasks

- [ ] Replace five action SHAs + version comments in **both** workflows as applicable
- [ ] Refresh release.yml SHA table comment dates
- [ ] CHANGELOG Unreleased
- [ ] `rg` AC3 no floating `@vN` for those five
- [ ] PR → `gh run watch --exit-status` CI green → squash (never `git push origin main`)

## DoD

- [ ] SHA+comment pins at Dependabot target majors
- [ ] CI green; no crate diff
- [ ] Dependabot remotes/PRs not deleted until this squash (then GitHub may auto-close)
