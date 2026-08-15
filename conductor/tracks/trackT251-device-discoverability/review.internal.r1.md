# T251 Internal R1 — Completeness vs spec

**Track:** T251-DeviceDiscoverability  
**Reviewer:** completeness subagent (read-only)  
**Authority:** `spec.md` F1–F16 / AC1–AC16 + `plan.md`  
**Workspace:** `C:\dev\AI-Brains` (MAIN)  
**Verdict:** **CLEAN**

No open findings. First-class `DeviceCommands::Status` (not a List alias) prints the shared enrolled roster and **always** appends `next: ai-brains replicate status`. List / fingerprint / replicate / singular error copies / contracts / pins stay frozen. AC16 is N/A after implement.

This review did **not** re-run the workspace CI gate or live-vault dogfood. Those are operator/gate ACs; code and hermetics cover the behavioral contract.

---

## DoD matrix

| Requirement | Met/Partial/Unmet | Evidence |
|-------------|-------------------|----------|
| **F1** First-class `device status`, not `visible_alias` of List; unit variant after List | **Met** | `DeviceCommands` in `crates/ai-brains-cli/src/main.rs`: `List` then `Status` (no flags, no `visible_alias`). Dispatch `DeviceCommands::Status => run_status`. Unit `device_status__parses` matches `DeviceCommands::Status`. `visible_alias` on this crate is only T243 `search` / unrelated args. |
| **F2** Body = exact list roster + always `next:`; no inline replicate; no flags; revoked-only ≡ empty | **Met** | `run_status` = `emit_device_roster` then `println!("{DEVICE_STATUS_NEXT}")`. `DEVICE_STATUS_NEXT = "next: ai-brains replicate status"`. Empty **and** enrolled hermetics assert the pointer. Roster uses `list_enrolled_devices` (`status IN ('active', 'local')`) so revoked-only is empty (AI2 L4; no revoke hermetic). No `--format` / `--quiet` / `--fake-relay`. |
| **F3** No multi-device product fill / no top-level status / no default `device` / no 16th doctor check | **Met** | Bootstrap/enroll/revoke/package bodies unchanged in role. No `Commands::Status`. `DeviceCommands` has no default subcommand. Doctor still asserts `checks.len() == 15`. |
| **F4** Shared `emit_device_roster`; one plural T198 const; fingerprint uses const only; singular errors untouched | **Met** | `EMPTY_ENROLL_HINT` used by `emit_device_roster` (list+status) and `run_fingerprint` (const only, not emitter). Production grep of the **plural** sentence is that one const. Singular `No enrolled device on this vault. Run \`ai-brains device bootstrap\` first.` remains at `device.rs` `load_local_signing_key` (L139) and `replicate.rs` `load_local_device` (L206). |
| **F5** List / fingerprint frozen (no `next:`) | **Met** | `run_list` is emitter only. Fingerprint empty prints the const and returns. Hermetics: `device_list__empty_vault__does_not_contain_next`, `device_list__enrolled_vault__does_not_contain_next`, `device_fingerprint__empty_vault__does_not_contain_next`. Existing `device_fingerprint__no_enroll__bootstrap_message_exit_0` still asserts the full T198 line. |
| **F6** Human-only; no JSON DTO; no contracts growth | **Met** | `Status` is a unit variant. No `DeviceStatusResponse` / no new type under `crates/ai-brains-contracts/src`. AC8 hermetic: `--format json` → clap exit **2**, `unexpected argument`, no JSON stdout. |
| **F7 / AC9** Docs complete (always-append + exit-code footnote) | **Met** | CAPABILITIES OutputFormat rows for `device list` / `device status` / `replicate status` JSON note. PROTOCOL-COMPAT §5 additive human-only rows (explicitly not compact↔pretty). OPERATIONS multi-device one-liner. INSTALL §7 tip. CHANGELOG Unreleased: **always** appends `next:` (empty and enrolled). CLI-EXIT-CODES: exit-0 table row + Device status footnote (empty/enrolled **0**; extra args clap **2**). Device `after_help` adds `ai-brains device status`. |
| **F8** Capture-independent; zero new crates; no pager/comfy-table/clap 5 | **Met** | String emit + existing `list_enrolled_devices`. `ai-brains-cli` deps unchanged (no new crates). Workspace clap `4.5`; lock clap **4.6.1**. |
| **F9** Exit 0 empty and enrolled; extra args clap 2; missing key not rewritten | **Met** | AC1/AC3 hermetics exit 0. AC8 exit 2. Status shares the same vault/key `AppContext` path as list; no VAULT_KEY rewrite. |
| **F10** No pin bumps | **Met** | Workspace `clap = { version = "4.5", ... }`; `Cargo.lock` clap `4.6.1`. |
| **F11** Isolation (no T243–T250 rewrite, no `OutputFormat::parse` change, no live bootstrap/daemon, no key print) | **Met** (code) | T251 markers only in CLI device/main, discoverability test, and the F7 docs. `OutputFormat::parse` still governed default-Json (`governed_common.rs`). `replicate.rs` not given a T251 rewrite. No `AI_BRAINS_KEY` print added. Live bootstrap/daemon not present in this change set. |
| **F12** Soft residuals stay deferred | **Met** | No list JSON, no doctor enrollment check, no `visible_alias = "stat"`, no default `device`→status, no singular-copy unify, no clap 4.6 workspace pin. |
| **F13** T198 empty SOOT; tests assert full sentence | **Met** | Hermetics use the exact plural `No enrolled devices. Run \`ai-brains device bootstrap\` first.` T198 empty-states test unchanged. |
| **F14** Honesty owner stays after_help / replicate; status does not reprint PQ paragraph | **Met** | Device `after_help` honesty line retained. `run_status` emits roster + `next:` only. |
| **F15** High-finding classes absent | **Met** | Not a silent List alias; does not inline `replicate::run_status`; no DTO; T198 copy unchanged; no 16th doctor check; no `sync`/`safety sync` extension; no clap 5. |
| **F16** Plan-only until go | **N/A** | Production files exist after go (AC16). |
| **AC1** Recognized `device status`; exit 0 on empty vault | **Met** | `device_status__empty_vault__outputs_hint_and_next_replicate_status` — not unrecognized; exit 0. |
| **AC2** Empty stdout = exact T198 plural **and** last data line `next:` | **Met** | Same test: `contains(T198_EMPTY)` + `last_nonempty_line == DEVICE_STATUS_NEXT`. |
| **AC3** After bootstrap: `DEVICE_ID` or `local` **and** `next:`; exit 0 | **Met** | `device_status__enrolled_vault__outputs_table_and_next_replicate_status`. |
| **AC4** `device list` empty keeps T198, no `next:`; enrolled has `local`, no `next:` | **Met** | Two named tests (empty + enrolled). No `for` loop in a single `#[test]`. |
| **AC5** Existing fingerprint empty test stays green (T198, no `next:`) | **Met** | `empty_states_exit_hygiene.rs` `device_fingerprint__no_enroll__bootstrap_message_exit_0` still asserts the full T198 line. Companion hermetic asserts no `next:`. |
| **AC6** `device --help` lists `status`; combined help contains `ai-brains device status` | **Met** | `device_status__help__lists_status`. Parent `after_help` includes the example. |
| **AC7** `replicate status` empty still has `enrolled_count` / honesty / bootstrap hint | **Met** | `replicate_status__empty_vault__still_prints_enrolled_count_honesty_hint`. `replicate.rs` `run_status` not inlined or rewritten for T251. |
| **AC8** `device status --format json` → clap exit 2 | **Met** | `device_status__with_format_json_flag__fails_exit_2`. |
| **AC9** Docs set complete | **Met** | See F7. CHANGELOG uses **always** appends; CLI-EXIT-CODES has footnote + exit-0 row. |
| **AC10** Shared emitter + plural const; singular errors remain | **Met** | See F4. |
| **AC11** No contracts type; no `--format` on Status | **Met** | See F6. |
| **AC12** `cli_help_ia` group-label tests (additive Device after_help only) | **Met** (design) | `cli_help_ia.rs` still snapshots group labels / query after_help — **not** Device examples. Device after_help is additive only. Tests not re-executed in this review. |
| **AC13** Full CI gate; zero new crates; capture-independent | **Partial** | Code: no new crates; status is presentation-only (no models/graph/events). Workspace CI / `ledgerful verify` **not re-run** here. |
| **AC14** Manual live empty vault (do not bootstrap) | **Partial** | Operator AC; not executed in this read-only review. Hermetic empty path matches the specified live outcome. |
| **AC15** Live list / fingerprint / replicate unchanged except status now exists | **Partial** | Same as AC14 — not dogfooded here. Isolation + AC4/AC5/AC7 hermetics lock the frozen surfaces. |
| **AC16** No production change until go | **N/A** | Plan-time lock; flips to N/A after implement. |

---

## Named hermetics (plan Phase 1)

| Planned name | Present |
|--------------|---------|
| `device_status__empty_vault__outputs_hint_and_next_replicate_status` | Yes |
| `device_status__enrolled_vault__outputs_table_and_next_replicate_status` | Yes |
| `device_status__with_format_json_flag__fails_exit_2` | Yes |
| `device_list__regression__does_not_contain_next` | Split into empty + enrolled tests (stronger; no `for` in one `#[test]`) |
| Extra: help, replicate-status lock, fingerprint no-`next:` | Yes |
| Clap unit `device_status__parses` | Yes |

No placeholders / TODOs / `unimplemented!` / no-op paths in the T251 emit path.

---

## Isolation checklist

| Forbidden | Result |
|-----------|--------|
| `visible_alias` of List | Absent |
| `replicate.rs` rewrite | Singular error copy untouched; no T251 logic |
| T243–T250 product rewrite | No T251 markers outside device CLI + F7 docs |
| `OutputFormat::parse` change | Unchanged (governed default Json) |
| clap 5 / lock bump | clap workspace 4.5 / lock 4.6.1 |
| New contracts DTO | No `DeviceStatus*` in `ai-brains-contracts` |
| Doctor 16th check | Still 15 |
| Live bootstrap / daemon / `AI_BRAINS_KEY` print | Not in this change set |

---

## Findings

None.

Track is **not** marked completed by this review.
