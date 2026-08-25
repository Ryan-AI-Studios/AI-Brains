# T298 — Empty `device` / `replicate` status must still name this machine

- **Track ID:** T298-DeviceReplicate
- **Status:** **Planned** (Pending until **go**; not Placeholder)
- **Category:** UX / HONESTY
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `device status` **5/8**, `replicate status` **5/8**; T274–T284 declined optional empty (E=8) — **reopened** U&lt;8. Placeholder minted with T285–T300 (`76c4db9`). T297 F17 pointed here.
- **Depends on:** T251 ✅ first-class `device status` = roster + **always** `next: ai-brains replicate status`; T198 ✅ F7 plural empty-enroll copy; T176/T177 ✅ enroll + fake-relay dashboard; T178 ✅ honesty residuals
- **Blocks / feeds:** Agents learn local-only / not-PQ / this-machine identity **without** `device bootstrap`. Forget-list useful empty **T299**. Graph sparse **T300**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “device/replicate U=5”; T251 F14 “status does not reprint honesty” **as a partial lift** (one short line, not the replicate paragraph)
- **Not absorbed (DoD):** Live `device bootstrap` / enroll / revoke; PQ product; remote wipe; `--format` on `device`; replicate JSON new keys; combined list+replicate dashboard; doctor 16th; clap 5 / rusqlite 0.40; T299–T300; T240 F2
- **Research date:** 2026-08-25 (plan dogfood HEAD `01fb0db` T297 `#213`. Product `src/` = T297. PATH **0.1.2** 2026-08-22 19:41 **has T251 empty+next**, not this identity/honesty. Live vault **zero** enrolled — do not bootstrap.)
- **Ledger:** planning DOCS TX `839a62a1-2881-4fbb-b918-4ce5673d721c`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** rewrite `.env` (T240 F2). Do **not** `device bootstrap` / enroll / revoke on the live vault. Do **not** `daemon start` / `stop` / `install`. Do **not** add crate `hostname`. Do **not** grow hotspot `project.rs` / `sync.rs` / `governed_common.rs` / `context.rs` / `forget.rs`. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** live `retention apply --confirm`, `graph rebuild`, leftover `rebind-path --write --yes`, or `safety sync` without `--dry-run`.

---

## 1. Objective

1. **Empty `device status` is useful.** Today it is the T198 one-liner plus T251 `next:`. Honest, but agents learn nothing: not that this is local-only, not that it is not PQ, not which machine they are on, not that enrollment is optional. Print **this-machine identity** and a **short honesty line** without requiring bootstrap.
2. **Empty `replicate status` is useful.** `enrolled_count: 0` + honesty + bootstrap hint already exist. Add the same this-machine label (fingerprint when enrolled; `{hostname} (not enrolled)` when empty). Do **not** claim sync is running. JSON keys stay frozen.
3. **Keep T251 / T198 / T176 contracts.** `next: ai-brains replicate status` remains the **last** line on `device status` (empty and enrolled). `device list` / `fingerprint` do not grow `next:` / this-machine / the short honesty const. No `--format` on `device`. Capture independence: strings only.

This unblocks daily ops honesty for the Windows-first vault: a local-only machine with zero enrollment is a **complete** status, not a blank optional feature.

---

## 2. Live baseline (re-scan 2026-08-25)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `01fb0db` T297 squash `#213`. Tree **CLEAN**. `origin/main` = HEAD (`left-right` `0 0`). Branch `main`. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-22 19:41**, 25 139 712 bytes, **0.1.2**. **Has T251 `device status` + always `next:`.** **Does not have T298 this-machine / short honesty.** **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **4137** (volatile). In-context **0/0/0**. Word **970**. Capture independence holds. |
| PATH `device status` | `No enrolled devices. Run \`ai-brains device bootstrap\` first.` then `next: ai-brains replicate status`. Exit **0**. **No** this-machine line. **No** `local-only`. |
| PATH `device list` / `fingerprint` | Same T198 one-liner. Exit **0**. No `next:`. |
| PATH `replicate status` | `relay: not configured`. `enrolled_count: 0`. `cursors: 0`. honesty `optional multi-device; not PQ; not remote wipe; not metadata-private`. `hint: run \`ai-brains device bootstrap\` to enroll first device`. Exit **0**. **No** `this machine:`. Does **not** claim sync is running. |
| PATH `replicate status --format json` | Pretty object keys **exactly** `cursors`, `devices`, `enrolled_count`, `gap_or_blocked`, `honesty`, `relay`. `enrolled_count: 0`. `devices: []`. `cursors: []`. **No** `this_machine`. |
| Live OS identity | `COMPUTERNAME=DESKTOP`. `HOSTNAME` unset. Do **not** treat live `DESKTOP` as a golden — hermetic injects `T298-HOST`. |
| Last GitHub PR | [#213](https://github.com/Ryan-AI-Studios/AI-Brains/pull/213) T297 (merged 2026-08-25T01:55:03Z). `gh pr view --comments`, `/reviews`, `/comments`, `issues/213/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, `#59` tokio, `#60` thiserror, `#58` tower-http, actions `#68–#72`). **No leftover to mint. No T301.** |
| Identity / hotspots | Hotspot **#1** `project.rs` (**3.888**) — **do not touch.** `sync.rs` #2. `governed_common.rs` #3. `context.rs` #4. `forget.rs` #5. `device.rs` / `replicate.rs` **not** top-10. |
| Ledger | **0 pending / 0 drift** at scan (before this DOCS TX). |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why this still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| Empty `device status` U=5 | T251 made the verb work and pointed at replicate. Agents still cannot tell local-only vs PQ vs “this machine has no enrollment.” Populate **without** bootstrap. **DoD.** |
| Empty `replicate status` U=5 | Dashboard is honest. `enrolled_count: 0` does not name the machine. **DoD as one human line.** |
| T251 F14 “status does not reprint honesty” | Replicate already owns the **paragraph**. A **short** device-status line is the U=5 fix. Partial lift, not a dump. |
| Combined list+replicate dashboard (T251 F12) | Would fork `replicate.rs` into `device`. Pointer + this-machine is enough. **Decline.** |
| `device list --format json` / `--format` on status | T176 leftover / T251 F6 freeze. Scripts use `replicate status --format json`. **Decline.** |
| Live `device bootstrap` | Would invent enrollment on the operator vault. Hermetic bootstrap is enough for the enrolled arm. **Decline as DoD.** |
| Crate `hostname` 0.4.2 | Current on crates.io (docs.rs 2026-06-11). `hostname::set` is a footgun. Workspace has **zero** hostname dep. Windows `COMPUTERNAME` is Microsoft-documented. **Decline crate.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Empty enroll copy | `device.rs` `EMPTY_ENROLL_HINT` **`:324`** | Exact T198 F7 plural: `No enrolled devices. Run \`ai-brains device bootstrap\` first.` Used by list + status + fingerprint. **Do not change the string.** |
| Status next | `DEVICE_STATUS_NEXT` **`:326`** | `next: ai-brains replicate status`. **Keep last.** |
| Shared roster | `emit_device_roster` **`:354–380`** | Empty → T198 + `Ok`. Else table `DEVICE_ID STATUS ENROLLED_BY FINGERPRINT`. **Do not put T298 lines inside the emitter** (list would grow them). |
| `run_list` | **`:383–385`** | Emitter only. Frozen. |
| `run_status` (device) | **`:387–392`** | Emitter then `println!(DEVICE_STATUS_NEXT)`. **Insert this-machine + honesty between emitter and next.** |
| `run_fingerprint` | **`:329–351`** | Empty uses **const only** (not emitter). Frozen. |
| `DeviceCommands::Status` | `main.rs` **`:2109–2110`** | Unit variant. No flags. `device_status__parses` **`:1276–1291`**. |
| Device after_help | `main.rs` **`:2013`** | Examples include `device status`. Honesty paragraph already on the parent. |
| Replicate human | `replicate.rs` `run_status` **`:131–158`** | Prints relay / enrolled_count / cursors / honesty / hint-or-devices. **Add `this machine:` after `enrolled_count`.** |
| Replicate JSON | **`:78–123`** | Keys: `relay`, `enrolled_count`, `cursors`, `gap_or_blocked`, `devices`, `honesty`. PROTOCOL-COMPAT `:109` **keys unchanged**. |
| Replicate `--quiet` | **`:126–128`** | Relay line only. **Unchanged.** |
| Fingerprint format | `ai-brains-sync` `format_fingerprint_hyphen` | 16 lowercase 4-char groups. Reuse. |
| Enrolled set | `list_enrolled_devices` | `status IN ('active','local')`. Local preferred, else first active (same as `load_local_signing_key` **`:135–138`**). |
| Hermetic tests | `tests/device_status_discoverability.rs` | AC1 last-line `next:` still SoT. AC7 replicate honesty/hint still SoT. **Extend** this file. |
| T176 smoke | `tests/device_replicate_cli.rs` | Bootstrap / list / push / pull. Stay green. |
| T198 fingerprint | `tests/empty_states_exit_hygiene.rs` | Empty fingerprint one-liner. Stay green. |
| `hermetic_bin` denylist | `tests/common/mod.rs` **`:42–69`** | Does **not** strip `COMPUTERNAME` / `HOSTNAME`. New tests **inject** `COMPUTERNAME=T298-HOST`. |
| Hotspot | `project.rs` #1 | Do not touch. `device.rs` **711** lines / `replicate.rs` **317** — helpers stay in those files. |

### 2.4 Dependency / standards research (2026-08-25) — snapshot, re-verify at execute

| Pin | Workspace / lock | crates.io / docs (today) | Action |
|-----|------------------|--------------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | **4.6.6** (GitHub latest 2026-08-06). **clap 5 not released.** | **No bump.** No `--format` on `device`. |
| `rusqlite` | workspace **0.39.0** | **0.40.2** (Dependabot `#61` open) | **No bump.** |
| `serde_json` | lock **1.0.150** | current 1.0.x | **No bump.** JSON keys frozen. |
| `rstest` | cli dev-dep **0.25** | already in crate | Reuse for helper `#[case]`. |
| `hostname` crate | **not in lock** | **0.4.2** (docs.rs 2026-06-11, MIT; `get` + **`set`**) | **Do not add.** |
| `tokio` | workspace **1.52** / lock **1.52.3** | crates.io **1.53.1** (`#59`) | Unused here. **No bump.** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged. |
| workspace version | **0.1.2** | — | **No bump.** |
| New crates | — | — | **Zero.** |

### 2.5 Online best-practice / implementation research

| Topic | Finding | Use in T298 |
|-------|---------|-------------|
| **[CLIG — Saying (just) enough](https://clig.dev/)** (current) | Too little = user wonders what is going on; too much = dump | One this-machine line + one short honesty line. Do **not** inline the replicate dashboard. |
| **[CLIG — Ease of discovery](https://clig.dev/)** | Suggest what to run next | Keep T251 last line `next: ai-brains replicate status`. |
| **[CLIG — Human-first / future-proof](https://clig.dev/)** | Human output may evolve; scripts pin JSON | Human-only additive. `replicate status --format json` keys frozen. No device DTO. |
| **[Microsoft Learn — `hostname`](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/hostname)** (current) | `%COMPUTERNAME%` usually prints the same string as `hostname`, **uppercase** | Windows-first identity = `COMPUTERNAME` then `HOSTNAME` then `unknown`. Do **not** spawn `hostname.exe`. |
| **[Microsoft Learn — Computer Names](https://learn.microsoft.com/en-us/windows/win32/sysinfo/computer-names)** (updated 2026-08-05) | `GetComputerNameEx` is the Win32 API | Decline Win32 call / `windows` crate growth; env is testable with `TempEnv` / `assert_cmd` `.env`. |
| **docs.rs `hostname` 0.4.2** | `hostname::get()` / `hostname::set()` | Decline: new crate + `set` footgun. Env SoT. |
| **clap 4.6.6** ([docs.rs derive](https://docs.rs/clap/4.6.6/clap/_derive/index.html)) | `after_help` already on Device / Replicate parents | Additive Status **about** text only. Unknown `--format` still clap **2**. |

**N/A:** SQLCipher / schtasks / contracts DTO / HTTP probes — this track does not touch them.

**ledgerful / ai-brains:** `preflight --summary` pinned **4137** / 0/0/0; lexical `recall` returned T251 ship notes (roster + always `next:`); semantic recall returned T251 fold-in sessions; `sync query` ledger hits T251 FEATURE `627392d8` + T176 ARCHITECTURE `5dae83e3` — **no contradicting “do not add honesty to status” pin beyond T251 F14**, which this track **partially lifts**; `ledgerful ledger status --compact` 0 pending / 0 drift at scan; `scan --impact` CLEAN at `01fb0db`; `hotspots` `project.rs` #1 — do not grow; `search emit_device_roster` = `device.rs:354` callers `run_list` / `run_status` only.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — Device empty body (hard)** | Empty `device status` stdout is **exactly four** non-empty data lines, in order: (1) T198 `EMPTY_ENROLL_HINT`, (2) `this machine: {os_hostname} (not enrolled)`, (3) `local-only; not PQ; not remote wipe`, (4) `next: ai-brains replicate status`. Exit **0**. |
| **F2 — Shared this-machine label (hard)** | `pub(crate) fn this_machine_label(devices: &[DeviceIdentityRow]) -> String`. Local (`status=="local"`) else first `active`. If that row has `fingerprint_sha256.len()==32` → `format_fingerprint_hyphen`. Else `{os_hostname()} (not enrolled)`. Fail-open on bad length (do **not** panic). Used by **both** `device status` and human `replicate status`. |
| **F3 — OS hostname (hard)** | `os_hostname()` reads `COMPUTERNAME` then `HOSTNAME` (first non-empty trimmed first line). Else `unknown`. **No** `hostname` crate. **No** `hostname.exe`. **No** Win32 `GetComputerNameW`. Hermetic tests inject `COMPUTERNAME=T298-HOST` and remove `HOSTNAME`. |
| **F4 — Short honesty (hard)** | Frozen const `DEVICE_STATUS_HONESTY: &str = "local-only; not PQ; not remote wipe"`. Printed on `device status` only (empty **and** enrolled). Do **not** reprint replicate’s full `optional multi-device; not PQ; not remote wipe; not metadata-private` paragraph. This **partially lifts T251 F14**. |
| **F5 — `next:` last (hard)** | T251 F2 stands: always append `DEVICE_STATUS_NEXT` as the **last** non-empty line (empty and enrolled). |
| **F6 — Emitter frozen (hard)** | `emit_device_roster` unchanged. T298 lines live **only** in `device::run_status` after the emitter returns. `run_list` / `run_fingerprint` must not contain `this machine:` / `DEVICE_STATUS_HONESTY` / `next:`. |
| **F7 — Enrolled device status (hard)** | After the roster table: `this machine: {hyphen fingerprint}` then honesty then `next:`. Do **not** also print `(not enrolled)`. |
| **F8 — Replicate human (hard)** | After `enrolled_count` print `  this machine:    {this_machine_label}` (same label as device, padded to match existing columns). Keep relay / cursors / honesty / hint. Do **not** print `sync: running` / `replication: active` or any claim that sync is happening. |
| **F9 — Replicate JSON freeze (hard)** | `--format json` keys stay exactly today’s six: `relay`, `enrolled_count`, `cursors`, `gap_or_blocked`, `devices`, `honesty`. **No** `this_machine`. PROTOCOL-COMPAT `:109` stays “keys unchanged”. |
| **F10 — `--quiet` freeze** | `replicate status --quiet` still prints only `format_relay_status`. No this-machine. |
| **F11 — No `--format` on device** | T251 F6 stands. `device status --format json` clap unexpected argument exit **2**. No DTO. |
| **F12 — T198 copy freeze** | `EMPTY_ENROLL_HINT` byte-identical. Singular error copies at `load_local_signing_key` / `load_local_device` **untouched** (T251 F12 / AI2 L2). |
| **F13 — No live bootstrap (hard)** | Do **not** `device bootstrap` / enroll / revoke the operator vault. Hermetic temp vault bootstrap is the enrolled proof. |
| **F14 — Pins / crates** | No workspace/lock bumps. No `hostname` crate. clap 4.5 / lock 4.6.1 stay. rusqlite 0.39.0 stays. |
| **F15 — Capture independence** | String emit + existing `list_enrolled_devices`. No events. No models. No graph. No contracts crate. |
| **F16 — Isolation** | No T240 F2 `.env` rewrite. No daemon start/stop/install. No doctor 16th. No combined dashboard. No default `device` → status. No `visible_alias = "stat"`. No T299/T300 steal. |
| **F17 — PATH** | Do not `cargo install`. Source/hermetic SoT. PATH 0.1.2 until owner asks. |
| **F18 — last-PR Cursor** | **#213** comments/reviews/issue **empty**. **No T301.** Dependabot `#61` rusqlite / `#58–#62` / `#68–#72` **not stolen**. |
| **F19 — Docs** | CAPABILITIES `:112` additive (this-machine + short honesty **before** last-line `next:`). `:113` additive human this-machine; JSON keys frozen. OPERATIONS `:1082` additive. INSTALL `:197` tip additive. PROTOCOL-COMPAT `:107` additive human-only; `:109` keys unchanged. CHANGELOG T298 Unreleased. CLI-EXIT-CODES footnote still exit **0** (no change required unless a sentence is missing “useful empty is still 0”). `cli_help_ia` does not snapshot Device examples — stay green. |
| **F20 — Placeholder `none` rewrite** | Placeholder said replicate `this machine: &lt;fingerprint-or-none&gt;`. Live hole is **identity**, not the token `none` (`enrolled_count: 0` already says none). Empty label is `{hostname} (not enrolled)` on **both** surfaces (same helper). Literal `none` is **not** the empty string. |
| **F21 — High findings** | Inventing a fingerprint without enroll; putting T298 lines inside `emit_device_roster`; adding JSON `this_machine`; `--format` on device; bootstrapping the live vault; adding `hostname` crate; claiming sync is running; clap 5. |
| **F22 — Help** | `DeviceCommands::Status` about may add “this-machine + local-only”. Parent Device after_help honesty paragraph **stays**. No required new after_help dump. In-process `Cli::try_parse_from(["ai-brains", "device", "status"])` still `Ok`. |
| **F23 — Exit** | Recognized status → **0** (empty and enrolled). Unexpected `--format` → clap **2**. Missing vault key stays today’s `VAULT_KEY_MISSING`. |
| **F24 — Decline peers** | T299 forget-list; T300 graph sparse; leftover `--write`; T240 F2; T263 H2; T255 750 raise; T251 F12 bag; doctor 16th. |
| **F25 — Soft residuals** | PATH until install; live vault stays 0 enrolled (honest); `device list --format json`; bootstrap→outbox; unify singular error copy; clap 4.6 workspace pin; is-terminal migrate. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Hermetic empty vault: `device status` exit **0**. Stdout contains exact T198 plural. Contains `this machine: T298-HOST (not enrolled)` (child env `COMPUTERNAME=T298-HOST`, `HOSTNAME` removed). Contains exact `local-only; not PQ; not remote wipe`. Last non-empty line is `next: ai-brains replicate status`. Four non-empty lines. |
| **AC2** | Hermetic after `device bootstrap`: stdout contains `DEVICE_ID` or `local`. Contains `this machine:` **without** `(not enrolled)`. Contains a hyphenated fingerprint (at least one `-` of 4-char groups). Contains honesty const. Last non-empty line is `next:`. Exit **0**. |
| **AC3** | Hermetic `device list` empty: T198 present; **does not** contain `this machine:` / honesty const / `next:`. Enrolled list contains `local` and **does not** contain those three. |
| **AC4** | Hermetic `device fingerprint` empty: T198 one-liner; **does not** contain `this machine:` / `next:`. Existing T198 empty-fingerprint test stays green. |
| **AC5** | `device status --format json` clap unexpected argument exit **2** (T251 AC8 stay-green). |
| **AC6** | Hermetic empty `replicate status`: exit **0**. Contains `enrolled_count` and `0`. Contains `this machine:` and `(not enrolled)` and `T298-HOST`. Contains honesty (`not PQ`). Contains bootstrap `hint`. Does **not** contain `running` as a sync-state claim (no `sync: running` / `replication: running`). |
| **AC7** | Hermetic empty `replicate status --format json`: parse object; key set **equals** `{relay, enrolled_count, cursors, gap_or_blocked, devices, honesty}`; **no** `this_machine` / `this machine`; `enrolled_count == 0`. |
| **AC8** | Hermetic `replicate status --quiet`: stdout is the relay line (`not configured`); **does not** contain `this machine:`. |
| **AC9** | Hermetic enrolled: `device fingerprint` stdout (hyphen form) is a substring of `replicate status` `this machine:` line **and** of `device status` `this machine:` line. |
| **AC10** | Unit rstest `os_hostname`: `COMPUTERNAME` wins; `HOSTNAME` used when COMPUTERNAME missing/blank; both missing → `unknown`. Use `TempEnv` RAII. `#[serial(env)]` if the same test binary mutates those keys in-process. |
| **AC11** | Unit rstest `this_machine_label`: empty slice → `{host} (not enrolled)`; local 32-byte fp → hyphen form (no `(not enrolled)`); 31-byte fp fail-open enrolled-empty form. |
| **AC12** | Docs: CAPABILITIES `:112`/`:113` additive; OPERATIONS `:1082` additive; INSTALL `:197` additive; PROTOCOL-COMPAT `:107` additive / `:109` keys unchanged; CHANGELOG T298 Unreleased. |
| **AC13** | `device --help` still lists `status`. Combined help still contains `ai-brains device status`. `cli_help_ia` group labels stay green. |
| **AC14** | Manual on **live** vault (do **not** bootstrap): `device status` + `replicate status`. Pass: empty-enrolled; device contains **local-only** (or the frozen const) **and** a this-machine identifier (`DESKTOP` or `unknown` or `(not enrolled)`); `next:` last on device; replicate `enrolled_count: 0` + this-machine + honesty; does not claim sync is running; exit **0**. Record PATH vs `cargo run` if they differ (PATH-behind is F17). |
| **AC15** | No `ai-brains-contracts` type. No pin bumps. No new crate. `emit_device_roster` body unchanged (grep: T298 consts not referenced from `run_list`). |
| **AC16** | T251 AC7 stay-green: empty replicate still prints `enrolled_count` / honesty / bootstrap hint (additive this-machine allowed). |

---

## 5. Design notes

### 5.1 Human shape (empty)

```
No enrolled devices. Run `ai-brains device bootstrap` first.
this machine: T298-HOST (not enrolled)
local-only; not PQ; not remote wipe
next: ai-brains replicate status
```

```
Multi-device replication status
  relay:           not configured
  enrolled_count:  0
  this machine:    T298-HOST (not enrolled)
  cursors:         0
  honesty:         optional multi-device; not PQ; not remote wipe; not metadata-private
  hint:            run `ai-brains device bootstrap` to enroll first device
```

### 5.2 Helper sketch (in `device.rs`, `pub(crate)`)

```rust
pub(crate) const DEVICE_STATUS_HONESTY: &str = "local-only; not PQ; not remote wipe";

pub(crate) fn os_hostname() -> String { /* F3 */ }

pub(crate) fn this_machine_label(devices: &[DeviceIdentityRow]) -> String { /* F2 */ }

pub fn run_status(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    emit_device_roster(ctx)?;
    let devices = /* list_enrolled_devices under lock, or return the vec from emit — do not duplicate emit's print */;
    println!("this machine: {}", this_machine_label(&devices));
    println!("{DEVICE_STATUS_HONESTY}");
    println!("{DEVICE_STATUS_NEXT}");
    Ok(())
}
```

**Do not** list devices twice as two SQL round-trips if a small refactor is cleaner: `emit_device_roster` may return `Vec<DeviceIdentityRow>` (print side-effect kept) so `run_list` ignores the vec and `run_status` uses it. Returning the vec is **allowed**; changing empty **copy** is **not**.

`replicate.rs` already imports `data_key_from_sqlcipher` from `device`. Import `this_machine_label` the same way. Print after `enrolled_count`.

### 5.3 Why not literal `none`

Placeholder allowed `this machine: none`. `enrolled_count: 0` already says none. U=5 is “agents learn nothing about **this machine**.” Hostname + `(not enrolled)` is the useful empty. Same helper on both commands avoids two different “this machine” strings.

### 5.4 Why not fingerprint-without-enroll

`run_fingerprint` empty already prints T198 (no keygen). Generating keys on status would look enrolled and write nothing — a lie. Fingerprint only from `device_identity` rows.

### 5.5 Why honesty on enrolled too

U=5 is “agents learn nothing.” An enrolled roster still does not say **not PQ**. One short line on every `device status` is cheaper than a second dual-truth. Replicate keeps its existing paragraph (do not duplicate it on device).

---

## 6. Non-goals

- Live `device bootstrap` / enroll / revoke / package-export
- PQ / remote wipe / metadata-privacy **product** (honesty only)
- `device list --format json` / device status DTO / `--format` on status
- Combined list+replicate dashboard / top-level `ai-brains status`
- Default `device` (no subcommand) → status
- Doctor enrollment check / 16th matrix row
- Crate `hostname` / Win32 `GetComputerNameEx` / spawn `hostname.exe`
- Replicate JSON new keys / `--quiet` growth
- clap 5 / rusqlite 0.40 / workspace 0.1.3
- T299–T300 / leftover `--write` / T240 F2 / T263 H2
- `cargo install`

---

## 7. Verification plan (TDD)

**Red first (must fail on current tree):**

1. `device_status__empty_vault__this_machine_honesty_next` (AC1 — four lines; today two)
2. `os_hostname__computername_then_hostname_then_unknown` (AC10)
3. `this_machine_label__empty_and_local_fp` (AC11)
4. `replicate_status__empty_vault__this_machine_not_enrolled` (AC6)
5. `replicate_status__format_json__keys_frozen_no_this_machine` (AC7 — may already pass keys; must assert **absence** of `this_machine` **and** human AC6 still red)

**Then green:** helpers + `run_status` insert + replicate human line + docs.

**Stay-green:** AC3–AC5 / AC8 / AC13 / AC16 / T176 smokes / T198 fingerprint empty.

**Manual:** AC14 classify-only. Pass-with-observed-data on live empty vault. **Do not bootstrap.**

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Two “this machine” meanings (hostname vs fingerprint) | F2 **one** helper; empty = hostname+(not enrolled); enrolled = fingerprint only. |
| List grows T298 lines | F6 emitter frozen; AC3. |
| JSON scripts break | F9 / AC7 exact key set. |
| Live bootstrap “to have something to show” | F13 / AC14 empty vault. |
| `hostname` crate / Win32 | F3 / F14. |
| PATH 0.1.2 hides T298 | F17; hermetic + `cargo run` SoT. |
| Hotspot `project.rs` | Do not touch. |
| COMPUTERNAME missing on Linux CI | F3 HOSTNAME fallback; hermetic **injects** `T298-HOST`. |

---

## 9. Deferred absorb / decline

**Entire `conductor/deferred.md` scanned** (T142 archive through T297 closeout + T285–T300 mint). Overlapping open rows:

| Item | Disposition |
|------|-------------|
| Audit / mint “device/replicate U=5 empty” | **Absorb** F1–F8 / AC1–AC9 / AC14 |
| Placeholder Manual `device status` + `replicate status` | **Absorb** AC14 / F13 |
| Placeholder hostname **or** fingerprint + `local-only; not PQ; not remote wipe` + existing `next:` | **Absorb** F1 / F4 / F5 |
| Placeholder replicate `this machine: fingerprint-or-none` | **Rewrite** F20 — empty is `{hostname} (not enrolled)`, not token `none` |
| T251 F2 always `next:` last / no `--format` | **Affirm** F5 / F11 |
| T251 F14 status does not reprint honesty paragraph | **Partial lift** F4 — one short line; paragraph stays on replicate |
| T251 F6 / PROTOCOL-COMPAT JSON keys | **Affirm** F9 / AC7 |
| T251 F12 list JSON / combined dashboard / doctor 16th / default device→status / `stat` alias / singular error unify | **Decline** F16 / F25 |
| T198 F7 plural empty copy | **Affirm** F12 |
| T297 closeout “T298–T300 not stolen” | **Absorb** this track; **T299–T300** still not stolen |
| T299 forget-list empty | **Decline** F24 |
| T300 graph sparse live rebuild | **Decline** F24 |
| T294 leftover `--write` | **Decline** — Completed `#210` |
| T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F14 / F24 |
| last-PR Cursor **#213** | **N/A empty** — **no T301** F18 |
| Identity leftover `7d97a456` vs `fcb8a40f` | **Not this track** — T258 / leftover data |
| Closed T176/T177/T178/T198/T251/T297 DoDs | **Stay closed** |

---

## 10. Implement order (on go)

1. Phase 0 re-verify (plan.md) + FEATURE TX.
2. Red AC1 / AC6 / AC10 / AC11 (and AC7 absence lock).
3. Green helpers + `run_status` insert + replicate human line. Optional `emit_device_roster` → return `Vec` if it removes a second SQL list.
4. Stay-green AC3–AC5 / AC8 / AC9 / AC16.
5. Docs AC12.
6. Manual AC14 (read-only; **no** bootstrap).
7. `scripts/dev-check.ps1`; Phase-1 review; `codex-review`.
8. conductor Completed + deferred closeout + pin.
9. Phase 6 publish (`track/T298-*` → PR → watch GHA `CI` green → squash-merge). Never `git push origin main`.

---

## 11. Soft residuals

| Residual | Notes |
|----------|--------|
| PATH until `cargo install` | F17 — source/hermetic SoT |
| Live vault 0 enrolled | Honest; AC14 empty is the Manual SoT |
| `device list --format json` | T251 F12 / T176 leftover |
| Bootstrap → outbox | T177 residual |
| Singular error copy unify | T251 F12 |
| clap 4.6 workspace pin | lock 4.6.1 stays |
| T299–T300 | Next placeholders |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/device.rs` | F2/F3 helpers + honesty const; `run_status` prints label + honesty + next; units AC10/AC11 |
| `crates/ai-brains-cli/src/commands/replicate.rs` | Human `this machine:` after `enrolled_count`; JSON untouched |
| `crates/ai-brains-cli/src/main.rs` | Optional Status about text (F22). No `--format` field. |
| `crates/ai-brains-cli/tests/device_status_discoverability.rs` | AC1/AC2/AC6–AC9 hermetics (inject `T298-HOST`) |
| `Docs/CAPABILITIES.md` | `:112` / `:113` additive |
| `Docs/OPERATIONS.md` | `:1082` additive |
| `Docs/INSTALL.md` | `:197` tip additive |
| `Docs/PROTOCOL-COMPAT.md` | `:107` additive; `:109` keys unchanged |
| `CHANGELOG.md` | T298 Unreleased |
| `conductor/conductor.md` / `deferred.md` / this spec+plan / README-T285-T300 | Planning now; Completed on go |

**Do not touch:** `doctor.rs`; `project.rs`; `forget.rs`; `ai-brains-contracts`; `Cargo.lock`; `ai-brains-sync` fingerprint formatter (reuse); T176 enroll/revoke; live vault; PROTOCOL-COMPAT JSON key **set**.
