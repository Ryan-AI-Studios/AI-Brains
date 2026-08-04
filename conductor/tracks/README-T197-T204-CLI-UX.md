# T197–T204 — CLI UX Draft Series (2026-08-02 audit)

Source: non-destructive global-install CLI audit after T196. Drafts only — implement on go-ahead.

## Suggested implement order

| Order | Track | Priority | One-line |
|-------|-------|----------|----------|
| 1 | [T197](trackT197-vault-open-ux-key-bootstrap/spec.md) | P0 | ✅ **Completed 2026-08-03** PR #80 `72dfa62` — silent-zero stop; spam filter; F8; bootstrap docs |
| 2 | [T198](trackT198-empty-states-exit-hygiene/spec.md) | P1 | ✅ **Completed 2026-08-03** PR #81 `5cc0418` — empty success non-blank; dogfood fail_api; graph exit 0→2 |
| 2 | [T199](trackT199-daemon-status-vault-independence/spec.md) | P1 | ✅ **Completed 2026-08-03** PR #82 `721d41f` — status no key; shared probe; soft tasklist |
| 3 | [T200](trackT200-graph-feature-install-honesty/spec.md) | P1/P3 | ✅ **Completed 2026-08-03** PR #83 `84f4a23` — docs-only A; INSTALL/Release honesty; F14 CI graph-on |
| 4 | [T201](trackT201-cli-error-exit-contract/spec.md) | P2 | ✅ **Completed 2026-08-03** PR #84 `a9e3b85` — clap-required scope exit 2; details.hint; BREAKING 6→2; exit_contract suite |
| 5 | [T202](trackT202-recall-briefing-clarity/spec.md) | P2 | ✅ **Completed 2026-08-04** PR #85 `89ea3ec` — embedding.status; empty_denied; TTY md; progressive exit 2; Codex R2 PASS |
| 5 | [T203](trackT203-governed-discovery-reads/spec.md) | P2 | source/evidence list; review scope soft-default |
| 6 | [T204](trackT204-cli-help-ia/spec.md) | P3 | Grouped `--help` |

## Audit → track map

### Opportunities for significant improvement

| Opportunity | Track |
|-------------|-------|
| Suppress SQLCipher logs; one lock line | **T197** |
| Key bootstrap docs + pre-open validate | **T197** |
| `daemon status` without vault | **T199** |
| Graph feature non-zero / default install | **T198** + **T200** |
| Empty-state copy (backup verify, project list, dogfood) | **T198** |
| Unify error envelope + exit table | **T201** |
| Recall TTY + semantic health | **T202** |
| Governed discovery lists | **T203** |
| Group help IA | **T204** |

### Scores &lt; 7 (E or C)

| Command / path | Score issue | Track |
|----------------|-------------|-------|
| live `doctor` no key | E≈3 C≈2 | **T197** |
| live `daemon status` no key | E≈4 C≈4 | **T199** |
| `backup verify` empty | E5 C3 | **T198** |
| `project list` empty | C6 | **T198** |
| `project detect` | E6 | **T198** (msg/docs) |
| `dogfood compare` miss | E4 C2 | **T198** |
| `graph *` feature-off | E2 | **T198** + **T200** |
| `device fingerprint` empty | E6 | **T198** |
| `query progressive` | E5 C6 | **T202** |
| `briefing project/personal` | C6 / E6 | **T202** |
| `review list` deny/ceremony | E5 C6 | **T201** + **T203** |
| `evidence show` / `source show` no list | E5 | **T203** |
| clap vs JSON missing-arg inconsistency | clarity | **T201** |

### Explicitly not drafted as tracks

| Item | Why |
|------|-----|
| MSI / notarization / App Store | Packaging residual (not CLI UX) |
| R-CI-BRANCH | Repo admin only |
| T196 SIGTERM delivery test P3 | Soft residual; optional micro later |
| Capture independence / crypto redesign | Out of audit scope |

## Non-goals for the series

- Product multi-user IdP  
- Weakening zero-key refuse  
- Renaming top-level commands (T204 is presentation-only)  
