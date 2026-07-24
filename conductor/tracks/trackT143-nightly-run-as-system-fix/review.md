# T143 Review Log — Nightly `--run-as-system` SYSTEM Context Fix

## Status

**Complete** on main (`c7585d3`, `634249e`). Conductor closeout 2026-07-24. Closes `deferred.md` #7.

## Implementation summary

- `--run-as-system` generates wrapper with baked `AI_BRAINS_VAULT_PATH` + model/embedding env
- Auto `--no-project-context --skip-import --log-format json` on nightly SYSTEM schedule
- Daemon path: env bake + `--no-project-context`
- `--dry-run` prints schtasks args + wrapper content
- T145 evolution: wrapper path → `%ProgramData%\AI-Brains\` with ACL (ACs still hold)

## Verification (closeout 2026-07-24)

| Check | Result |
|-------|--------|
| Targeted nextest (run_as_system / wrapper / schedule) | **passed** (part of 31 cli filtered tests) |
| Independent DoD audit (explore) | **PASS** |
| Live SYSTEM task | Confirmed in T145 review 2026-07-21: Run As SYSTEM, Last Result **0** |
| Codex primary | Rate-limited until ~2026-07-28 (not obtained) |

## DoD

| AC | Status |
|----|--------|
| AC1 env bake | Met |
| AC2 no-project-context + skip-import | Met |
| AC3 working dir (via `cd /d` in wrapper) | Met |
| AC4 dry-run | Met |
| AC5 daemon schedule | Met |
| AC6 tests | Met |
| AC7 migration docs / replace manual bat | Met (CLI generates ProgramData wrapper; manual scripts/nightly-task.bat retired) |
| AC8 CI at merge | Met (claimed green at merge; later main gates green through T147) |

## Completion decision

Engineering complete and live-proven via T145. Mark Complete; strike deferred #7.
