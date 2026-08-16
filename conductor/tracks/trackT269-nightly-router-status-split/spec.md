# T269 — Nightly vs Router status split + probe honesty

- **Track ID:** T269-NightlyRouterStatusSplit
- **Status:** **Pending** (placeholder; plan-only until go)
- **Category:** OPS / UX
- **Owner:** —
- **Source:** Audit 2026-08-16 — friction: human mixes Nightly Last Result **0** with Router **267009**; full `--status` Completion `probe=timeout` while `daemon status` says LLM backend Open
- **Depends on:** T229/T247/T255 (JSON + read-only Router line)
- **Absorbs:** Operators (and this audit) misread 267009 as a failed nightly; 750 ms completion probe false-timeout
- **Not absorbed:** Mutating Router/Nightly tasks (T255 decline); doctor 16th check (T255 decline); persist probe (T255 decline)

---

## 1. Objective

1. Human `--status` must make **two tasks** unmistakable: Nightly Last Result vs Router Last Result (different names, different `/tr`).
2. Probe timeout must not contradict `daemon status` without saying the probe budget (750 ms vs daemon’s longer probe).

## 2. Problem (live 2026-08-16)

After a successful remediator (Last Result **0**, 15 sessions, errors []):

```
Last task result: 0
…
Router: Running  last result: 267009
task still running (SCHED_S_TASK_RUNNING)
```

JSON is already split (`last_task_result: "0"` vs `router.last_result: "267009"` / `task_to_run: C:\llm\router.bat`). Human is not.

Full `--status` (not `--quick`): Completion `probe=timeout`, Embedding `probe=ok`. `daemon status` same moment: both backends **Open**. T247 750 ms is too short for a busy :8081 (llama.cpp). Operators think completion is down.

## 3. Frozen intent (placeholder)

| ID | Intent |
|----|--------|
| **F0** | Plan-only until go. |
| **F1** | Human: `Nightly:` / `Router:` headings (or equivalent) so 267009 cannot be read as nightly. |
| **F2** | `--quick` still `probe=skipped`. Do not claim timeout. |
| **F3** | Full probe: either raise budget with evidence, or print `probe=timeout (750ms)` so it is not “backend down.” |
| **F4** | Do not register/repair Router. Read-only (T255). |
| **F5** | JSON keys frozen; human-only change preferred. |

## 4. Verification sketch

- Human fixture contains both task names.
- Timeout string includes budget if still 750 ms.
- JSON schema_version unchanged.
