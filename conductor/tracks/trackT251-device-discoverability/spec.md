# T251 — Device discoverability

- **Track ID:** T251-DeviceDiscoverability
- **Status:** ✅ **Completed** (2026-08-14 PR #167 `038098e`)
- **Category:** UX / FEATURE
- **Owner:** Grok
- **Source:** CLI audit 2026-08-11 P3 — `device status` unrecognized (exit **2**); multi-device discoverability; `replicate status` already scored good
- **Depends on:** T176 `device` / `replicate` CLI + migration 0027; T177 fake-relay status; T178 honesty residuals; T198 F7 empty enroll SOOT (`device list` / `device fingerprint`); T249 `next:` style
- **Blocks / feeds:** Operators who type `device status` get the enrolled roster plus a pointer to `replicate status`. **T252** (ingest stdin), **T253** (Claude/Codex), **T255** (nightly/router) stay separate.
- **Absorbs:** deferred.md “device status missing”; placeholder F1–F3 / AC1–AC2; README `device status` missing
- **Not absorbed (DoD):** T176 device-list JSON; T177 CLI bootstrap→outbox; T178/ADR-0018 ceremony/crypto; T189 DataKey / #34.2; T198 empty-copy rewrite; T243–T250 product rewrite; doctor 16th check; live `device bootstrap` on the operator vault; clap 5 / pin bumps
- **Research date:** 2026-08-14 (live dogfood + T176/T198 SOOT + CLIG + clap 4.6.6 + crates.io pins)
- **AI fold-in:** 2026-08-14 `C:\dev\AI-review.md` **T251** AI1 + AI2. No Highs. **Agree:** AI1 M1–M4 / L1 / O1 already planned (keep AC1–AC16). **Agree hard:** AI2 L1 CLI-EXIT-CODES footnote; AI2 L2 singular error copies stay; AI2 L3 CHANGELOG “always appends”; AI2 L4 revoked-only = empty roster. **Decline:** AI1 remapped AC7/AC8 (keep our AC1–AC16). Disposition **§14**.
- **Ledger:** plan-only until go (`ledgerful ledger start T251-device-discoverability --category UX --message "device status = enrolled roster + next: replicate status; list/fingerprint frozen"`)
- **Isolation:** Do **not** rewrite T243–T250. Do **not** change T176/T177/T178 crypto, schema, enroll/revoke, or `replicate.rs`. Do **not** change T198 empty-enroll copy. Do **not** bootstrap/enroll/revoke the live vault. Do **not** start/stop/install the live daemon. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Make `device status` work.** Operators who type it today get clap `unrecognized subcommand` (exit **2**). Ship a first-class subcommand that prints the same enrolled roster as `device list` plus `next: ai-brains replicate status`.
2. **Keep list / fingerprint / replicate frozen.** Empty enroll stays the T198 line. `device list` does **not** grow a `next:` line. `replicate status` stays the replication dashboard.
3. **Stay capture-independent.** Presentation / discoverability only. No models, no graph, no new events, no new crates, no pin bumps, no multi-device product fill.

---

## 2. Live baseline (re-scan 2026-08-14)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| `device status` | clap `unrecognized subcommand 'status'` → **exit 2**. No did-you-mean (nothing named status under `device`). |
| `device list` | `No enrolled devices. Run \`ai-brains device bootstrap\` first.` Exit **0**. |
| `device fingerprint` | Same T198 empty line. Exit **0**. |
| `device --help` | Subcommands: bootstrap / fingerprint / list / package-export / enroll / revoke. **No** `status`. after_help examples omit status. Honesty line present. |
| `replicate status` | Human dashboard. `relay: not configured`. `enrolled_count: 0`. `cursors: 0`. honesty line. `hint: run \`ai-brains device bootstrap\` to enroll first device`. Exit **0**. |
| Live vault | **Zero** enrolled devices. Do **not** bootstrap it in this track. |

### 2.2 Why the audit scored P3

| Surface | Truth |
|---------|--------|
| `device status` | Missing. `status` is the verb operators type after `daemon` / `harness` / `replicate` / `nightly`. |
| `device list` | Already honest empty + table. Discoverability gap is the missing synonym + missing pointer to replication. |
| `replicate status` | Already good (audit: scored ≥8 / not tracked). Do not fork it into `device`. |
| Multi-device fill | Enroll ceremony, fake relay, wraps, CE ACK are **shipped** (T176–T178). T251 is the missing `status` noun, not a product rewrite. |

### 2.3 Code truth

| Site | Role |
|------|------|
| `main.rs` `DeviceCommands` | Bootstrap / Fingerprint / **List** / PackageExport / Enroll / Revoke. **No** Status. |
| `Device` after_help | Examples for bootstrap/list/fingerprint/package-export/enroll/revoke. No status. |
| `commands/device.rs` `run_list` | Empty → T198 line + `Ok`. Else table `DEVICE_ID STATUS ENROLLED_BY FINGERPRINT`. **No** `--format`. |
| `run_fingerprint` | T198 F7: same empty line + exit 0. |
| `commands/replicate.rs` `run_status` | Human + `--format json` + `--quiet`. Keys: `relay`, `enrolled_count`, `cursors`, `gap_or_blocked`, `devices`, `honesty`. |
| `empty_states_exit_hygiene.rs` | Locks fingerprint empty copy. |
| `device_replicate_cli.rs` | Hermetic bootstrap → `list` contains `local`. |
| `cli_help_ia.rs` | Group labels only — Device after_help is **not** snapshotted. |

### 2.4 Honesty (do not “fix” here)

- Live vault has no enrolled device. That is **true**. Do not bootstrap to make status look populated.
- `replicate status` already points empty operators at `device bootstrap`. Status points the other way (`list` → replicate).
- Multi-device remains optional / not PQ / not remote wipe / not metadata-private. Do not restate the full honesty dump on every `device status` line — `replicate status` already owns it.
- `AI_BRAINS_KEY` never printed.

---

## 3. Research (2026-08-14)

| Topic | Finding | Use in T251 |
|-------|---------|-------------|
| **[CLIG — Suggest next commands](https://clig.dev/)** | Workflows should name the next command (git status pattern) | Last line `next: ai-brains replicate status` (same prefix as T249 daemon Stopped) |
| **CLIG — Discoverable** | Comprehensive help + examples | First-class `status` in Commands + after_help example |
| **CLIG — Human-readable / future-proof** | Human output may evolve; scripts pin `--json` | Status is human-only. Machine path stays `replicate status --format json`. Do not invent a device-status DTO. |
| **CLIG — Say just enough** | Do not dump the replicate dashboard twice | Pointer, **not** a combined view |
| **clap 4.6.6** ([docs.rs](https://docs.rs/clap/4.6.6/clap/_derive/index.html), 2026-08-06) | `visible_alias` = same handler, shown in help. `alias` = hidden. First-class variant when about/output differ | **Status is first-class**, not `#[command(visible_alias = "status")]` on List (that would drop `next:`). T243 `search`→`recall` is the true-alias pattern; this is not that. Workspace clap **4.5** / lock **4.6.1**. clap **5 not released**. **No bump**. |
| **ADR-0018 / T176 R3** | CLI freeze is `device` + `replicate`. Forbidden to extend `sync` / `safety sync` | Status stays under `device`. Do not add `ai-brains status`. |
| **T176 spec §9.1** | `device list` promised “Table/JSON” | Live is table-only (T176 review residual). JSON list is **not** T251 DoD. |
| **T198 F7** | Fingerprint empty **same copy** as list, exit 0 | Status empty = that copy **plus** `next:`. List/fingerprint strings stay byte-identical. |
| **T177 residual** | Bootstrap does not auto-enqueue DeviceEnrolled to outbox | Product fill — **not** absorbed |
| **serde_json** | lock **1.0.150** / crates.io **1.0.151** | **No bump**; no DTO |
| **chrono** | lock **0.4.44** / crates.io **0.4.45** | Unused here |
| **is-terminal** | **0.4.17** | Soft only — status is not TTY-switched |
| **rustc** | **1.95.0** | Edition 2024 unchanged |

---

## 4. Frozen decisions (F1–F16)

| ID | Decision |
|----|----------|
| **F1 — First-class `device status` (hard)** | Add `DeviceCommands::Status` (unit variant, no flags). Dispatch to `commands::device::run_status`. **Not** a clap `visible_alias` of `List`. Place it immediately after `List` in the enum so help clusters the two read commands. |
| **F2 — Body = list + next (hard)** | `run_status` prints the **exact** `run_list` body, then one line `next: ai-brains replicate status`. **Always** append (empty **and** enrolled) — not daemon-style conditional (AI2 L3). Empty roster = `list_enrolled_devices` empty (active/local only; **revoked-only** vaults show T198 + `next:`, same as `list` — AI2 L4). Do **not** inline `replicate::run_status`. Do **not** add `--fake-relay` / `--quiet` / `--format`. Do **not** add a revoke-ceremony hermetic (F3). |
| **F3 — No multi-device product fill (hard)** | No enroll/revoke/bootstrap/package changes. No outbox enqueue. No wrap/relay/CE/ACK work. No doctor `device_enrolled` check (T192/T249 15-check matrix stays 15). No top-level `ai-brains status`. No default-subcommand when `device` is invoked bare. |
| **F4 — Shared roster emitter (hard)** | Extract `emit_device_roster(ctx)` (or equivalent) used by `run_list` and `run_status`. Empty copy is **one** `const` matching T198 F7 plural: `No enrolled devices. Run \`ai-brains device bootstrap\` first.` Fingerprint uses the **const only** (not the emitter). Do **not** fork a third empty-success helper. **Do not** unify the singular **error** copies `No enrolled device on this vault…` at `device.rs` `load_local_signing_key` (~139) and `replicate.rs` `load_local_device` (~206) (AI2 L2 — different sentence, mutating paths; leave untouched). AC10 grep must target the **exact plural** T198 sentence. |
| **F5 — List / fingerprint frozen** | `device list` stdout must **not** contain `next:`. Fingerprint empty must stay the T198 one-liner (no next). Enrolled list table columns unchanged. |
| **F6 — Human-only; no JSON DTO** | Status has no `--format`. Extra args → clap exit **2**. Scripts that want machine enrollment use `replicate status --format json` (keys frozen). Do not grow `ai-brains-contracts`. |
| **F7 — Docs** | CAPABILITIES OutputFormat table: `device list` / `device status` / note `replicate status` JSON unchanged. CHANGELOG Unreleased: **always** appends `next:` (empty and enrolled — AI2 L3). OPERATIONS multi-device residuals: one-liner that `device status` = roster + replicate pointer. INSTALL §7 optional tip. PROTOCOL-COMPAT §5 additive rows (human-only; not a compact↔pretty flip). **CLI-EXIT-CODES.md** one-line footnote (AI2 L1): `device status` exits **0** empty and enrolled (like `daemon status`); unexpected args stay generic clap **2**. `device --help` after_help adds `ai-brains device status` (additive; `cli_help_ia` does not snapshot Device examples). |
| **F8 — Capture independence / zero new crates** | String emit + existing `list_enrolled_devices`. No pager, no `comfy-table`, no clap 5. |
| **F9 — Exit codes** | Recognized status → **0** (empty and enrolled). Unrecognized extra args / unknown format flags → clap **2**. Missing vault key stays today’s `VAULT_KEY_MISSING` (not a T251 rewrite). |
| **F10 — Pins** | No workspace/lock bumps. clap 4.5 / lock 4.6.1 stay. |
| **F11 — Isolation** | No T243–T250 rewrite. No `OutputFormat::parse` change. No live `device bootstrap` / enroll / revoke. No live `daemon start` / `install`. No `AI_BRAINS_KEY` print. |
| **F12 — Soft residuals** | `device list --format json` (T176 promise); bootstrap→outbox; doctor enrollment check; combined list+replicate dashboard; `visible_alias = "stat"`; default `device` → status; is-terminal → std; clap 4.6 workspace pin; unify singular error copy (`No enrolled device on this vault…`) in `load_local_signing_key` / `load_local_device` |
| **F13 — T198 empty SOOT** | Empty success copy is shared. Tests must assert the **full** T198 sentence, not a looser “contains bootstrap”. |
| **F14 — Honesty owner** | Device after_help honesty stays. Status does **not** reprint the PQ / remote-wipe / metadata paragraph (replicate status already does). |
| **F15 — High findings** | Silent alias of list (drops `next:`); inlining replicate status; adding a contracts DTO; changing T198 copy; bootstrapping the live vault; adding a 16th doctor check; extending `sync` / `safety sync`; clap 5. |
| **F16 — Plan-only until go** | No production code until the user says **go**. |

---

## 5. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | Hermetic: `device status` is a recognized subcommand (not “unrecognized subcommand”). Exit **0** on an empty temp vault (`list_enrolled_devices` empty, including revoked-only — AI2 L4). No extra revoke-ceremony test required. |
| **AC2** | Hermetic empty: stdout contains the **exact** T198 **plural** line `No enrolled devices. Run \`ai-brains device bootstrap\` first.` **and** a last data line `next: ai-brains replicate status`. |
| **AC3** | Hermetic after `device bootstrap`: stdout contains `DEVICE_ID` (or `local`) **and** `next: ai-brains replicate status`. Exit **0**. |
| **AC4** | Hermetic regression: `device list` empty has the T198 line and **does not** contain `next:`. Enrolled `device list` contains `local` and **does not** contain `next:`. |
| **AC5** | Existing `device_fingerprint__no_enroll__bootstrap_message_exit_0` stays green (T198 one-liner, no `next:`). |
| **AC6** | Hermetic: `device --help` lists `status`. Combined help/after_help contains `ai-brains device status`. |
| **AC7** | Hermetic: `replicate status` on the same empty vault still prints `enrolled_count` / honesty / bootstrap **hint** (no T251 rewrite). |
| **AC8** | Hermetic: `device status --format json` is clap unexpected argument → exit **2** (no silent JSON). |
| **AC9** | Docs: CAPABILITIES OutputFormat + CHANGELOG Unreleased (**always** appends `next:`) + OPERATIONS one-liner + PROTOCOL-COMPAT §5 additive human-only rows + **CLI-EXIT-CODES.md** footnote (AI2 L1). |
| **AC10** | Shared roster emitter: empty-success copy is a single `const` used by list + status + fingerprint. Grep the **exact plural** T198 sentence (AI2 L2). Singular error copies at `load_local_signing_key` / `load_local_device` **must remain**. |
| **AC11** | No new `ai-brains-contracts` type. No `DeviceStatusResponse`. No `--format` field on `DeviceCommands::Status`. |
| **AC12** | `cli_help_ia` group-label tests still pass (additive Device after_help only). |
| **AC13** | Full CI gate; zero new crates; capture-independent. |
| **AC14** | Manual dogfood uses the **live** vault as-is (this machine: empty enroll). Do **not** bootstrap it. After implement: `device status` exit 0 + T198 line + `next:`. |
| **AC15** | `device list` / `fingerprint` / `replicate status` live output unchanged except status now exists. |
| **AC16** | No production file change until **go**. (Plan-time lock; flips to N/A after implement.) |

---

## 6. Non-goals

- Multi-device enroll/revoke/bootstrap/package/outbox product fill
- `device list --format json` / new contracts DTO
- Combined `device status` that dumps `replicate status`
- Top-level `ai-brains status`
- Default `device` (no subcommand) → status
- Doctor enrollment check / 16th matrix row
- T189 DataKey / peer re-wrap / live relay
- TTY-switching status vs list
- clap 5 / lockfile pin bumps / is-terminal migrate
- Live vault bootstrap / daemon start / install
- Rewriting T243–T250

---

## 7. Verification plan

| Phase | Proof |
|-------|-------|
| Red | AC1 fails today (`unrecognized subcommand`, exit 2) |
| Green F1/F2/F4 | `Status` variant + shared emitter + `next:` |
| Regression | AC4 / AC5 / AC7 |
| Clap honesty | AC8 unexpected `--format json` |
| Help | AC6 / AC12 |
| Docs | AC9 |
| Targeted | `cargo nextest run -p ai-brains-cli device` + `empty_states` + `cli_help_ia` + clippy `-p ai-brains-cli` |
| Manual | AC14 / AC15 on live empty vault — **no** bootstrap |
| Full gate | fmt, clippy workspace, nextest workspace, deny, audit, `ledgerful verify` |
| Review | `review.md`; UX/FEATURE; cross-model soft (alias vs first-class + T198 SOOT) |

---

## 8. Coordination

- **T176:** `device` / `replicate` split + list table — keep; do not implement the leftover list-JSON promise.
- **T177/T178:** relay / honesty / ACK — do not touch.
- **T198 F7:** empty enroll copy SOOT.
- **T249:** `next:` prefix style only.
- **T243:** `visible_alias` is for *same handler*. Status is **not** that.
- **T250:** Completed PR #165 — do not rewrite preflight.
- **T192/T249:** 15-check doctor matrix frozen.

---

## 9. Suggested implement snippet (guidance only)

```rust
const EMPTY_ENROLL_HINT: &str =
    "No enrolled devices. Run `ai-brains device bootstrap` first.";
const DEVICE_STATUS_NEXT: &str = "next: ai-brains replicate status";

fn emit_device_roster(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    // today's run_list body; empty prints EMPTY_ENROLL_HINT
}

pub fn run_list(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    emit_device_roster(ctx)
}

pub fn run_status(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    emit_device_roster(ctx)?;
    println!("{DEVICE_STATUS_NEXT}");
    Ok(())
}
```

```rust
enum DeviceCommands {
    Bootstrap,
    Fingerprint { raw: bool },
    List,
    /// Enrolled roster + pointer to `replicate status`
    Status,
    // …
}
```

Live on go (do **not** bootstrap; do **not** start daemon):

```powershell
ai-brains device status          # expect exit 0 + T198 line + next:
ai-brains device list            # T198 line; no next:
ai-brains device fingerprint     # T198 line; no next:
ai-brains replicate status       # unchanged dashboard
ai-brains device --help          # lists status + after_help example
ai-brains device status --format json   # expect clap exit 2

# Full gate
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace ; cargo deny check ; cargo audit
ledgerful verify --scope full
```

---

## 10. Risk / review

- **Category:** UX / FEATURE (not SECURITY). Cross-model soft: first-class vs alias; T198 copy; no DTO.
- **Highest regression:** changing list/fingerprint empty copy; adding `next:` to `list`; treating status as `visible_alias` of list; inlining replicate; bootstrapping the live vault “to have something to show.”
- **Cap deferred mediums:** ≤3; leftover presentation softs go to F12 / deferred.md.

---

## 11. Suggested implement order (locked)

1. Extract `EMPTY_ENROLL_HINT` + `emit_device_roster` (list + fingerprint still compile / AC5 green)
2. Red AC1/AC2 (`status` still unrecognized)
3. Green F1/F2 clap + `run_status` + AC1–AC4 / AC8
4. Help after_help AC6
5. Docs AC9

---

## 12. Placeholder disposition

| Draft | Disposition |
|-------|-------------|
| F1 `device status` → list + replicate pointer or combined view | **Absorbed** F1/F2 as first-class Status + pointer. Combined view **declined**. |
| F2 Help after_help examples updated | **Absorbed** F7 / AC6 |
| F3 No product multi-device fill | **Absorbed** F3 |
| AC1 exit 0 with honest empty/enrolled | **Absorbed** AC1–AC3 |
| AC2 Docs tip | **Absorbed** AC9 |

---

## 13. Deferred fold-in

| Item | Source | Disposition |
|------|--------|-------------|
| device status missing | deferred.md / README P3 | **DoD** F1–F2 / AC1–AC3 |
| Placeholder F1–F3 / AC1–AC2 | spec draft | **Absorbed** as above |
| T176 list Table/JSON leftover | T176 review / spec §9.1 | **Not absorbed** F12 |
| T177 bootstrap→outbox | deferred.md §53 | **Not absorbed** F3/F12 |
| T178 / ADR-0018 residuals | deferred.md §55 | **Not absorbed** |
| #34.2 DataKey | T189 shipped; multi-device = per-device | **Not absorbed** |
| T198 fingerprint empty | T198 F7 | **Keep** F13 / AC5 — do not rewrite |
| T249 F13 leftover “T251 device” | T249 | **This track** |
| T252–T255 placeholders | peers | **Not absorbed** |
| ISSUES.md | absent repo-wide | Residuals → **deferred.md** |

---

## 14. AI fold-in disposition (2026-08-14) — `C:\dev\AI-review.md` AI1 + AI2

No Highs. AI1 restates F1–F7 / AC1–AC6 as M1–M4 + remapped ACs (T248–T250 repeat). AI2 L1–L4 are the useful pins: exit-code footnote, singular error-copy isolation, unconditional `next:`, revoked-only empty.

### Disposition

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **AI1 M1** | AI1 | **Agree** | Already **F1** — first-class `Status`, not `visible_alias` of List |
| **AI1 M2** | AI1 | **Agree** | Already **F4** — `EMPTY_ENROLL_HINT` + `emit_device_roster`. Do not copy the review snippet blindly; keep `pub(crate)`/`fn` visibility as implementer chooses (emitter stays private in `device.rs`) |
| **AI1 M3** | AI1 | **Agree** | Already **F5** / AC4 / AC5 |
| **AI1 M4** | AI1 | **Agree** | Already **F6** / AC8 / AC11 |
| **AI1 L1** | AI1 | **Agree** | Already **F7** / AC6 / AC9 (docs + after_help) |
| **AI1 O1** | AI1 | **Agree** | Named hermetics in plan Phase 1 (`device_status__empty_vault__…`, enrolled, `--format json` exit 2, list no `next:`) — still map to **AC1–AC4 / AC8** |
| **AI1 remapped AC7/AC8** | AI1 | **Decline** | Their AC7 is our **AC8**; their AC8 is our **AC13**. Keep **AC1–AC16** (do not drop AC7 replicate-status lock) |
| **AI2 L1** | AI2 | **Agree hard** | **F7 / AC9**: CLI-EXIT-CODES.md one-line `device status` footnote (exit 0 empty/enrolled; clap 2 extra args) |
| **AI2 L2** | AI2 | **Agree hard** | **F4 / AC10 / F12**: plural T198 extract only; singular error copies at `device.rs:139` and `replicate.rs:206` stay |
| **AI2 L3** | AI2 | **Agree hard** | **F2 / F7**: `next:` **always**; CHANGELOG must say so |
| **AI2 L4** | AI2 | **Agree** | **F2 / AC1**: revoked-only ≡ empty roster; no revoke hermetic |

### Pins locked by fold-in

1. **F1:** first-class `Status`, never `visible_alias` of `List`.
2. **F2/F7:** `next:` always (empty and enrolled); CHANGELOG states that.
3. **F4/AC10:** extract **plural** T198 only; grep that exact sentence.
4. **F4/F12:** singular `No enrolled device on this vault…` in `load_local_signing_key` / `load_local_device` untouched.
5. **F7/AC9:** CLI-EXIT-CODES.md footnote.
6. **AC1–AC16** numbering stays (do not adopt AI1’s remapped matrix).

Do not implement until **go**.
