# T281 review log — Nightly HTTP `/health` vs daemon TCP contrast line

**Track:** T281-NightlyProbeVsTcp
**Status:** Completed (full gate green; Phase 6 pending this commit)
**BUGFIX TX:** `435f6228-5052-406c-baf1-5bd2234cafaf`
**HEAD (implement):** `track/T281-nightly-probe-vs-tcp`

## Reviewers / rounds

| Round | Reviewer | Result |
|-------|----------|--------|
| R1 | Implementer (Grok) vs spec AC1–AC14 / DoD | **PASS** — red then green; F1 31-char U+2260; helper Some iff raw `"timeout"`; print uses `completion_label`; 750 ms frozen |
| R1b | Explore subagent (read-only DoD) | **PASS WITH DEFERRED P3** — P3-2 Completion qualifier **verified_fixed**; P3-1 README Planned → closeout |
| CX1 | Codex gpt-5.6-luna | **FAIL** — product complete; P1-1 process; P2-1 call-site coverage |
| CX2 | Codex gpt-5.6-luna | **Product PASS**; P2-1 `verified_fixed`; P1-1 process (gate + closeout) |
| Gate | `scripts/dev-check.ps1` + `ledgerful verify --scope full` | **PASS** nextest **3312** / 1 skipped |

## Finding fields

id, severity, description, source, files, required_fix, status, evidence.

## Findings

| id | severity | description | source | files | required_fix | status | evidence |
|----|----------|-------------|--------|-------|--------------|--------|----------|
| — | — | R1: no product findings | R1 | — | — | — | AC1 `assert_eq!` F1 + 31 chars + U+2260; AC2 `"timeout (750ms)"` None; cargo run timeout next-line = F1 |
| P3-2 | low-info | Docs said “On human timeout” without Completion qualifier (F26 embedding-only) | R1b | CAPABILITIES `:439`; OPERATIONS `:680/:689` | Say Completion | `verified_fixed` | “On Completion human timeout”; OPERATIONS notes embedding-only is not this line |
| P3-1 | low-info | Series README / deferred header still **Planned** | R1b | `README-T274-T284-CLI-QUALITY.md`; `deferred.md` | Closeout Completed | `verified_fixed` | README + conductor Completed this commit |
| P1-1 | high (process) | Full gate / ledger / conductor closure unfinished at CX1/CX2 | CX1/CX2 | conductor + review.md | Run gate + closeout | `fixed_pending_verification` | `dev-check` 3306 then `verify --scope full` 3312; closeout this commit; Phase 6 remaining |
| P2-1 | medium | Production timeout call site not locked by tests (human mis-wire silent no-op) | CX1 | `nightly.rs` print | Extract helper + units | `verified_fixed` | `completion_status_human_lines(raw)`; CX2 product PASS |

## DoD matrix (AC1–AC14)

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | Met | `http_vs_tcp_contrast__equals_frozen_line` — F1 literal, 31 chars, `/health` `750ms` `daemon TCP`, U+2260, `assert_ne!` ASCII `!=` |
| AC2 | Met | Some on `"timeout"`; rstest None including `"timeout (750ms)"` (F32) |
| AC3 | Met | `format_probe_label_human("timeout", 750) == "timeout (750ms)"`; passthrough unchanged |
| AC4 | Met | JSON timeout fixture raw `"timeout"`; no ` (750ms)`; no `HTTP /health` / U+2260. Live `--format json --quick` `schema_version` 1, probe skipped |
| AC5 | Met | `format_router_status_lines` first line `Router: Running  last result: 267009` |
| AC6 | Met | `nightly__help__names_nightly_heading_and_probe_budget` PASS; after_help freeze |
| AC7 | Met | Hermetic `--quick`: heading + `probe=skipped`; no `(750ms)`; no `HTTP /health`; no `daemon TCP`. T255 AC10 / T269 AC8 comments kept |
| AC8 | Met | `format_status_schedule_block` `lines[1] == "Last task result: 101"` |
| AC9 | Met | Hermetic JSON `--quick` `probe == "skipped"` |
| AC10 | Met | `cargo run -p ai-brains-cli -- nightly --status`: Completion `probe=timeout (750ms)` **next line** F1; Embedding timeout **no** second F1; Router 267009; exit 0. `--quick`: skipped, no F1. `daemon status`: Stopped + LLM/Embedding **Open**. Did not mutate schtasks. Did not `cargo install`. |
| AC11 | Met | CAPABILITIES + OPERATIONS name the timeout next-line (Completion-qualified). CHANGELOG T281. PROTOCOL-COMPAT no new keys. CLI-EXIT-CODES status exit 0. Skill skipped (F19) |
| AC12 | Met | Diff omits `project.rs` / `sync.rs` / `forget.rs` / `daemon.rs` / `doctor.rs` / `llama_cpp.rs`. `NIGHTLY_STATUS_PROBE_TIMEOUT` still 750 ms. No clap/rusqlite bump |
| AC13 | Met | `format_endpoint_line` still 4 args; `format_endpoint_line__quick__probe_skipped` PASS |
| AC14 | Met | `--quick` still `"skipped"` without `LlamaCppProvider` |

## Targeted gates (R1)

```text
cargo nextest run -p ai-brains-cli http_vs_tcp_contrast completion_timeout_contrast_line format_probe_label_human format_router_status_lines__running_267009 format_endpoint_line__quick nightly__help__names_nightly_heading_and_probe_budget format_status_schedule_block__order build_nightly_status_json__timeout
  23 passed
cargo nextest run -p ai-brains-cli --test nightly_status
  3 passed
cargo clippy -p ai-brains-cli --all-targets -- -D warnings
  exit 0
cargo fmt --check -p ai-brains-cli
  exit 0
```

## Manual (classify-only)

```text
cargo run -p ai-brains-cli -- nightly --status
  Completion: 127.0.0.1:8081  model=gemma-4-E4B-it-Q6_K.gguf  probe=timeout (750ms)
  HTTP /health 750ms ≠ daemon TCP
  Embedding: … probe=timeout (750ms)   # no second F1 (F26)
  Router: Running  last result: 267009
  exit 0
  Did not mutate schtasks. Did not force llama load. Did not cargo install.

cargo run -p ai-brains-cli --quiet -- nightly --status --quick
  probe=skipped; no F1; heading + 267009

cargo run -p ai-brains-cli --quiet -- nightly --status --format json --quick
  schema_version 1; completion.probe "skipped"; no F1

cargo run -p ai-brains-cli --quiet -- daemon status
  Stopped; LLM 127.0.0.1:8081 Open; Embedding Open
```

PATH `ai-brains` remains T270-era until `cargo install` (F13). Source/`cargo run` is DoD.
