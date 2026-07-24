# T142 Review Log — Ledgerful State-Dir + Product-Name Migration

## Status

**Complete** on main (`35083aa`, `8fbbb14`). Conductor registry closeout 2026-07-24.

## Implementation summary

- `find_ledgerful_dir` prefers `.ledgerful/`, falls back to `.changeguard/`
- `extract_project_id_from_ledgerful`; deprecated aliases retained
- `LEDGERFUL_TX_ID` preferred; `CHANGEGUARD_TX_ID` deprecated fallback + warn
- Context consumer + skill/docs rebrand; `.gitignore` both dirs

## Verification (closeout 2026-07-24)

| Check | Result |
|-------|--------|
| `cargo nextest run -p ai-brains-path --test ledgerful_dir_discovery` | **11 passed** |
| Independent DoD audit (explore) | **PASS WITH DEFERRED P3** (intentional #1–2, #4–5) |
| Codex primary | Rate-limited until ~2026-07-28 (not obtained) |

## Deferred residuals (intentional)

| deferred.md | Topic |
|-------------|--------|
| #1 | Functional symbol rename `ChangeGuard*` → `Ledgerful*` |
| #2 | `source_tag: "changeguard:symbol"` migration |
| ~~#3~~ | OPERATIONS env table — **fixed at closeout** |
| #4 | Archive/historical track prose |
| #5 | `cargo audit` RUSTSEC-2026-0190 allowlist |

## Completion decision

Engineering ACs met and merged. Registry + deferred #3 closed. Remaining #1–2, #4–5 stay deferred by design.
