# T301 Plan — GHA SHA-pin refresh

**Status:** **Completed**. Spec [spec.md](./spec.md).
**Ledger:** series DOCS `30b7ca9d`. Fold-in DOCS `3402c6ed`. Implement **INFRA** `3571d90d-b7c2-4204-8556-7a2b50c2d017`.

## AI fold-in (2026-08-25)

OpenCode **M1** folded: peel annotated tags to **commit** SHA. Attest latest v4.x patch. last-PR `#217`. Disposition in spec **§13**.

## Phase 0 (on go)

- [x] Fetch; branch `track/T301-gha-sha-pins`
- [x] Re-resolve SHAs. For each tag: `gh api repos/<owner>/<repo>/git/ref/tags/<tag> --jq '{type: .object.type, sha: .object.sha}'`. If `type=="tag"`, peel: `gh api repos/<owner>/<repo>/git/tags/<sha> --jq .object.sha`. **Never** put a tag-object SHA in `uses:`.
- [x] `softprops/action-gh-release` v3.0.2 is annotated — peeled commit `3d0d9888cb7fd7b750713d6e236d1fcb99157228`
- [x] Attest: latest v4.x tag **v4.2.2** `1e69f48acb82d1966a394da916b4c1698aa569d6`
- [x] Read live `ci.yml` + `release.yml`. Checkout sites: 3 in ci + 1 in release
- [x] Do **not** merge `dependabot/github_actions/*`
- [x] INFRA TX `3571d90d…`

## Tasks

- [x] Replace five action SHAs + version comments in **both** workflows as applicable (all checkout jobs)
- [x] Refresh release.yml SHA table comment dates (header ~`:14–22`)
- [x] CHANGELOG Unreleased
- [x] `rg` / Select-String AC3 no floating `@vN` for those five (no new pin-checker script)
- [x] PR → `gh run watch --exit-status` CI green → squash (never `git push origin main`) — Phase 6

## DoD

- [x] SHA+comment pins at Dependabot target majors
- [x] Local `dev-check.ps1` + `ledgerful verify --scope full` exit 0; no crate diff; GHA on PR (Phase 6)
- [x] Dependabot remotes/PRs not deleted until this squash (then GitHub may auto-close)

## Local gate evidence (2026-08-25)

```
.\scripts\dev-check.ps1  → [SUCCESS] CI Gate passed!
ledgerful verify --scope full → Verification passed
```

## Execute pins (2026-08-25)

| Action | Pin |
|--------|-----|
| checkout@v7.0.1 | `3d3c42e5aac5ba805825da76410c181273ba90b1` |
| upload-artifact@v7.0.1 | `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a` |
| download-artifact@v8.0.1 | `3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c` |
| attest@v4.2.2 | `1e69f48acb82d1966a394da916b4c1698aa569d6` |
| action-gh-release@v3.0.2 | `3d0d9888cb7fd7b750713d6e236d1fcb99157228` (peeled) |
