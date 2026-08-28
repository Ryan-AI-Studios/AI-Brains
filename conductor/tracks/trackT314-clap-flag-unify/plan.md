# T314 Plan — clap flag unify (placeholder)

**Status:** **Placeholder.** Spec [spec.md](./spec.md). Full plan on `/plan-track T314`.
**Ledger (planning):** series DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`

## Phase 0 (on `/plan-track T314`)

- [ ] Inventory every `ArgAction::Set` `dry_run` vs `SetTrue` in `main.rs`
- [ ] Re-read T290 F10 / T291 `next_step` `--dry-run false` string
- [ ] Re-verify clap **4.6.1** `SetTrue` / `default_missing_value` on docs.rs

## DoD (after full plan + go)

`query expand --format json` parses. `query progressive "q" --dry-run` parses. `project scan-roots --dry-run` parses. Progressive still has no `--format`.

## Isolation

No clap 5. No `cargo install`. Never `git push origin main`.
