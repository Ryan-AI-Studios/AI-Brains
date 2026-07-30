# Shadow Dogfood Gate + Live Enablement Stop (T170 / P9.4)

Progressive dogfood gate for AI-Brains governed memory: prove synthetic trust gates, rehearse on a fixture vault, optionally dogfood a redacted shadow of real data, and **stop** before live enablement without explicit human approval.

**Locks:** D1–D26 in `conductor/tracks/trackT170-shadow-dogfood-gate/spec.md`.  
**Related:** [GOVERNED-MEMORY-MVP.md](GOVERNED-MEMORY-MVP.md) (T169), [OPERATIONS.md](../OPERATIONS.md).

---

## 0. Non-negotiable rules (operator summary)

| ID | Rule |
|----|------|
| **D1** | **Live vault never mutated.** All shadow/migrate/report artifacts stay under a non-live work directory. |
| **D2** | **Stop-before live enablement.** No permanent vault re-point, no User-level env, no migrate dest=live without **explicit in-session user approval**. Scripts never run Stage D. |
| **D3** | **Progressive order:** A → B → C → D (D only after approval). |
| **D4** | Stage A (and Stage C re-check) require `evaluate governed` exit **0**. Exit **7** = product trust fail; exit **1** = tool/infra broken. |
| **D5** | **Shadow before live-source migrate.** Prefer redacted `shadow create`. Live migrate destination refused by T168. |
| **D6** | **Redaction default** (`redact-turn-content` unless documented exception). |
| **D7** | Human review: ≥**20** distinct claim ids (or all if fewer) **and** **100%** of risk-kind warnings as `(kind, subject_id)` — **not** warning ids (DTO has none). |
| **D11** | **Rollback primary = feature flag** (`AI_BRAINS_GOVERNED_BRIEFING=0` / unset). Secondary: discard work-dir artifacts. No reverse-migrate claim. |
| **D12** | **No auto-enable.** Orchestrator never sets User-level env; never edits shell profiles; never writes live vault. |
| **D15** | Sampling is **stage-specific** (see §5). |
| **D21** | Rollback verify via `preflight --format json` `(governed)` probe + `briefing project --format json` for authority — **NEVER** `preflight --summary` for governed. |
| **D23** | User-env emergency clear is **manual only** (see §8). |
| **D24** | Live vault SHA-256 **pre** and **post** must match when a live vault exists. |
| **D25** | Stage D observation minimum: **1 full working session** or **≥3** governed-path invocations. |
| **D26** | Use global **`--vault-path`** for shadow/migrated compare. **Never** set `AI_BRAINS_VAULT_PATH` to a shadow/migrated path. |

---

## 1. Stage A — Synthetic trust gates (automated)

```powershell
$WorkDir = "C:\temp\ai-brains-dogfood"   # example; use your work dir
New-Item -ItemType Directory -Force -Path $WorkDir | Out-Null

ai-brains evaluate governed `
  --fixtures fixtures/governed-memory/scenarios `
  --report "$WorkDir\evaluate-report.json"

# require $LASTEXITCODE -eq 0
```

| Exit | Meaning | Dogfood branching |
|------|---------|-------------------|
| **0** | Hard gates passed | Continue |
| **1** | Internal / path refuse (tool broke) | **Stop** — fix harness/infra |
| **6** | Invalid scenario payload | Fix fixtures |
| **7** | `HARD_GATE_FAILED` | **Stop** — product trust regression |

**Record:** `report_hash`, `hard_gates_passed`, `human_review_seed` from the report JSON.

**Pass:** exit **0**. Fail: stop (do not shadow live data).

Orchestrator: `scripts/dogfood-shadow.ps1` runs Stage A first and aborts on non-zero.

---

## 2. Stage B — Synthetic runbook rehearsal (no real vault)

Stage B proves the **pipeline** (shadow, optional migrate, compare sources, rollback drill, D24) on a **fixture vault** you create — not the live vault.

### 2.1 Create fixture vault

T169 scenario JSON files are **not** vault databases. Build a minimal vault:

```powershell
$WorkDir = "C:\temp\ai-brains-dogfood-b"
New-Item -ItemType Directory -Force -Path $WorkDir | Out-Null
$Fixture = "$WorkDir\fixture.db"

ai-brains --vault-path $Fixture init

# Provide project/session context for pin (or set AI_BRAINS_PROJECT_ID / SESSION_ID)
# Prefer process-scoped env only; do not point AI_BRAINS_VAULT_PATH at shadow later.
$env:AI_BRAINS_VAULT_PATH = $Fixture   # allowed for fixture init/pin only
ai-brains context --new-project --new-session
ai-brains pin "DECISION: Dogfood fixture decision for T170 Stage B"
ai-brains pin "CONSTRAINT: Fixture vault must not be treated as live"
# Clear process vault env after pin so resolve_live_vault_path does not stick on fixture
Remove-Item Env:AI_BRAINS_VAULT_PATH -ErrorAction SilentlyContinue
```

> **Note:** Setting `AI_BRAINS_VAULT_PATH` to the **fixture** under WorkDir for init/pin is fine. **D26** forbids pointing it at **shadow/migrated** when comparing or when live refuse must still protect the real live vault. Prefer `--vault-path $Fixture` for pin/init when the CLI allows it so the process env never holds a dogfood path.

### 2.2 D24 pre-hash (live)

Resolve the **live** vault path **without** mutating env to shadow:

1. If process `AI_BRAINS_VAULT_PATH` is set and the file exists → that path (only if it is **not** under WorkDir).
2. Else `~/.ai-brains/.env` → `AI_BRAINS_VAULT_PATH` if present.
3. Else try `$env:USERPROFILE\.ai-brains\vault.db` if it exists.
4. If none → record **N/A** (no live vault).

```powershell
# Example hash helper
function Get-FileSha256([string]$Path) {
  if (-not (Test-Path -LiteralPath $Path)) { return $null }
  (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}
$LiveVault = ... # resolved live path or $null
$ShaPre = if ($LiveVault) { Get-FileSha256 $LiveVault } else { $null }
```

### 2.3 Shadow (+ optional migrate)

```powershell
# Dry-run first (optional)
ai-brains shadow create --source $Fixture --destination "$WorkDir\shadow.db" --dry-run

# Default redaction (D6)
ai-brains shadow create --source $Fixture --destination "$WorkDir\shadow.db"

# Optional migrate (destination under WorkDir only)
ai-brains migrate governed `
  --source "$WorkDir\shadow.db" `
  --destination "$WorkDir\migrated.db" `
  --report "$WorkDir\migrate-report.json" `
  --confirm
```

### 2.4 Capture compare inputs (D26: `--vault-path` only)

```powershell
$Db = "$WorkDir\shadow.db"   # or migrated.db
# NEVER: $env:AI_BRAINS_VAULT_PATH = $Db

# Governed (typed ProjectBriefingPacket)
ai-brains briefing project --vault-path $Db --format json > "$WorkDir\governed-packet.json"

# Legacy preflight (flag off)
$env:AI_BRAINS_GOVERNED_BRIEFING = "0"
ai-brains preflight --vault-path $Db --format json > "$WorkDir\legacy-preflight.json"
```

### 2.5 Emit compare packet

```powershell
ai-brains dogfood compare `
  --governed "$WorkDir\governed-packet.json" `
  --legacy "$WorkDir\legacy-preflight.json" `
  --out "$WorkDir\dogfood-compare.json" `
  --stage B `
  --evaluate-report "$WorkDir\evaluate-report.json" `
  --shadow "$WorkDir\shadow.db" `
  --live-vault $LiveVault `
  --sha256-pre $ShaPre `
  --sha256-post $ShaPost `
  --t169-exit 0 `
  --t169-report-hash <from evaluate-report>
```

Or use `scripts/dogfood-shadow.ps1` which orchestrates capture + compare.

### 2.6 Human checklist (Stage B seed)

Use T169 `human_review_seed.claim_ids_sample` from the evaluate report (may be &lt;20 — document). Fill [templates/dogfood-human-checklist.md](templates/dogfood-human-checklist.md).

### 2.7 Rollback drill (D21 / D23)

See **§8**. Prove flag on → governed observable; flag off → legacy. **Do not** use `preflight --summary`.

### 2.8 D24 post-hash

Recompute live vault SHA-256. **Must equal** pre-hash when both exist.

**Pass:** commands succeed; live hash stable; checklist template validated; rollback observable.

---

## 3. Stage C — Redacted shadow dogfood

**Source preference:** operator-provided **test vault** first (lowest risk). Active user vault allowed when the operator accepts a redacted shadow of real data — document which was used.

### Steps

0. **Re-run** `evaluate governed`; require exit **0**. `report_hash` must match Stage A **or** document intentional fixture drift + new baseline hash.  
1. Backup awareness (see OPERATIONS) — dogfood does **not** create backups.  
2. **D24** pre-hash live vault.  
3. `shadow create --source <chosen-vault> --destination <work>\shadow.db` (default redact; dry-run first). **Do not** set `AI_BRAINS_VAULT_PATH` to shadow.  
4. Optional migrate from **shadow** → work-dir migrated dest + report.  
5. Capture compare inputs (§2.4 / §9).  
6. Emit `dogfood-compare.json` (§6).  
7. Human review: **D15 Stage C** stratified sample + all risk warning refs; sign-off.  
8. **D24** post-hash; must match pre.  
9. **Pass:** T169 re-check green; live hash stable; human sign-off; hard checks pass.

Stage C is **operator-dependent** and not required in CI. Defer with owner + reason when no test vault is available.

---

## 4. Stage D — Live enablement (explicit approval only)

**Out of automated scripts** (`scripts/dogfood-shadow.ps1` refuses Stage D).

On **explicit** approval only:

```powershell
# Session-scoped only — scripts never set User scope
$env:AI_BRAINS_GOVERNED_BRIEFING = "1"
# optional:
$env:AI_BRAINS_PREFLIGHT_PRINCIPAL_ID = "<uuid>"
```

### D25 observation minimum

Record **one full working session** **or** **≥3** governed-path invocations (`briefing project` and/or preflight with flag on). Watch for unexpected `denied` / empty authority (grant misconfiguration). Record duration + observations in evidence.

Track records approval quote **or** “Stage D deferred.” MVP may complete at Stage C without Stage D.

---

## 5. Deterministic sampling (D15)

| Stage | Source | Algorithm |
|-------|--------|-----------|
| **B** | T169 evaluate report `human_review_seed.claim_ids_sample` | Synthetic claim ids from harness (≤20). |
| **C** | Governed `ProjectBriefingPacket` on shadow/migrated | Stratify by claim `kind`: up to **5** `Decision` + up to **5** `Conclusion` (each sorted by `id`), then fill remaining slots to **20** by global sorted `id` across both kinds. Prefer ≥2 projects when multi-project packets exist (document if single-project only). Do **not** invent Evidence/Review strata from briefing packets. |

Risk warnings for D7: kinds ∈ `{stale, disputed, open_conflict, unavailable, denied, low_confidence}` as `{kind, subject_id}` sorted by `(kind, subject_id)`. Diff maps may include kind `other` for info only.

---

## 6. Comparison packet schema (`dogfood-compare.json` v1)

```json
{
  "schema_version": 1,
  "created_at": "…",
  "compare_hash": "…",
  "stage": "B|C",
  "paths": {
    "shadow": "…normalized…",
    "migrated": null,
    "evaluate_report": "…",
    "migrate_report": null,
    "live_vault": "…normalized…"
  },
  "live_vault_integrity": {
    "sha256_pre": "…",
    "sha256_post": "…",
    "unchanged": true
  },
  "t169": {
    "exit_code": 0,
    "report_hash": "…",
    "hard_gates_passed": true
  },
  "legacy_preflight": {
    "mode": "legacy",
    "source_command": "ai-brains preflight --vault-path … --format json",
    "decision_marker_count": 0,
    "constraint_marker_count": 0,
    "hotspot_marker_count": 0,
    "word_count": 0,
    "text_fingerprint": "…"
  },
  "governed_briefing": {
    "mode": "governed",
    "source_command": "ai-brains briefing project --vault-path … --format json",
    "decision_count": 0,
    "conclusion_count": 0,
    "warning_kinds": [],
    "uncited_current_count": 0,
    "denied": false,
    "content_fingerprint": "…"
  },
  "diff": {
    "warning_kinds_only_in_governed": [],
    "note": "diff may include kind=other; D7 human review covers six risk kinds only",
    "hard_checks": {
      "t169_passed": true,
      "live_vault_mutated": false,
      "live_checksum_unchanged": true
    }
  },
  "human_review_seed": {
    "claim_ids_sample": [],
    "warning_refs_all": [{ "kind": "stale", "subject_id": "…" }]
  },
  "limitations": []
}
```

### 6.1 Field sources

| Section | Source command | Notes |
|---------|----------------|-------|
| `governed_briefing.*` | **`ai-brains briefing project --vault-path <db> --format json`** | Typed packet only. **Do not** scrape governed markdown from preflight. |
| `legacy_preflight.*` | **`ai-brains preflight --vault-path <db> --format json`** with flag **off** | Count `DECISION:` / `CONSTRAINT:` / `HOTSPOT:` in `text`; `word_count` from response. Marker counts ≠ “claim_count.” |
| Flag observability | preflight `--format json` flag on/off | `text -match '\(governed\)'` for mode probe only — not for authority counts. |

### 6.2 Fingerprints

| Field | Algorithm |
|-------|-----------|
| `governed_briefing.content_fingerprint` | SHA-256 of canonical JSON of packet with **BTreeMap key order**, arrays sorted by `id` where present; **exclude** `briefing_id`, `generated_at` |
| `legacy_preflight.text_fingerprint` | SHA-256 of UTF-8 preflight `text` (soft integrity only; do not hard-fail dogfood on text drift alone) |
| Integrity anchors | T169 `report_hash`; shadow/migrate fingerprints; **D24** live file SHA-256 |

### 6.3 `compare_hash` canonicalization

1. Build sorted view of packet (serde map keys sorted / BTreeMap).  
2. Sort order-independent arrays (`warning_kinds`, `claim_ids_sample`, `warning_refs_all` by `(kind, subject_id)`).  
3. Normalize path strings best-effort (`ai-brains-path` / resolve) before hash.  
4. **Exclude:** `created_at`, any `latency_ms`, and `compare_hash` itself.  
5. `compare_hash = hex(SHA-256(canonical_bytes))`.

CLI: `ai-brains dogfood compare …` implements this pure-serde path (zero new crates).

---

## 7. Human review checklist

Template: [templates/dogfood-human-checklist.md](templates/dogfood-human-checklist.md).

Required fields (spec §7):

- Run id / date / operator  
- Stage (B/C); Stage C source = test vault vs active user vault  
- Evaluate report_hash + exit (Stage A + Stage C re-check)  
- Live vault sha256 pre/post (**D24**) when live exists  
- Shadow path + redaction_policy  
- Migrate report_hash if used  
- Per sample claim: **id**, **kind**, cited? (Y/N), stale-as-current? (Y/N), notes  
- Per **risk** warning: **kind + subject_id** (or message if no subject_id), acceptable? (Y/N)  
- Cross-scope leakage / cloud-Sealed violation?  
- Overall pass / fail / pass-with-followups  
- Reviewer name; Stage D requested?

---

## 8. Feature flag & rollback

### Enable (session only — after Stage D approval)

```powershell
$env:AI_BRAINS_GOVERNED_BRIEFING = "1"
$env:AI_BRAINS_PREFLIGHT_PRINCIPAL_ID = "<uuid>"   # optional
ai-brains briefing project --vault-path <live-or-work> --format json
```

### Rollback (primary — D11)

```powershell
$env:AI_BRAINS_GOVERNED_BRIEFING = "0"
# or:
Remove-Item Env:AI_BRAINS_GOVERNED_BRIEFING -ErrorAction SilentlyContinue
```

### Emergency circuit breaker (User-level — **manual only**, D23)

Scripts **never** set User scope. Only if an operator previously set persistent User env by hand:

```powershell
[Environment]::SetEnvironmentVariable("AI_BRAINS_GOVERNED_BRIEFING", $null, "User")
# Open a new shell after User clear so process does not inherit a stale value.
```

### Verification (D21 — **not** `--summary` for governed)

| Step | Command | Expect |
|------|---------|--------|
| Flag off → legacy | `ai-brains preflight --vault-path <db> --format json` with flag `0` | `text` does **not** match `\(governed\)` |
| Flag on → governed probe | same with flag `1` | `text` matches `\(governed\)` **or** use briefing project success |
| Governed authority | **`ai-brains briefing project --vault-path <db> --format json`** | Typed `decisions`/`conclusions`/`warnings` (or honest empty + `denied`) |
| After rollback | flag `0` + preflight json | no `(governed)` marker |

**Why not `--summary`:** summary counts legacy tokens `DECISION:`/`CONSTRAINT:`/`HOTSPOT:`. Governed renderer emits markdown headings — summary reports **zeros** even when authority is present (false negative).

**Config-file flag:** still unwired (T152-R1-07) — env-only.

---

## 9. Manual compare procedure

Required when not using `dogfood compare` / orchestrator:

```powershell
$Db = "$WorkDir\shadow.db"   # never set AI_BRAINS_VAULT_PATH to this

# 1) Governed typed packet
ai-brains briefing project --vault-path $Db --format json > $WorkDir\governed-packet.json

# 2) Legacy preflight text (flag off)
$env:AI_BRAINS_GOVERNED_BRIEFING = "0"
ai-brains preflight --vault-path $Db --format json > $WorkDir\legacy-preflight.json

# 3) Fill dogfood-compare.json via CLI:
ai-brains dogfood compare `
  --governed $WorkDir\governed-packet.json `
  --legacy $WorkDir\legacy-preflight.json `
  --out $WorkDir\dogfood-compare.json `
  --stage C
```

### Orchestrator

```powershell
.\scripts\dogfood-shadow.ps1 `
  -WorkDir C:\temp\ai-brains-dogfood `
  -SourceVault C:\path\to\operator-test-vault.db `
  -EvaluateFixtures fixtures\governed-memory\scenarios
```

Parameters: `-WorkDir`, `-SourceVault` (optional Stage C), `-SkipMigrate`, `-EvaluateFixtures`, `-DryRun`, `-SkipEvaluate`, `-SkipShadow`.

Behavior:

- Stage A evaluate → abort on non-0; record `report_hash`.  
- **D24** pre/post live hashes.  
- Shadow/migrate under WorkDir only; **never** assign `AI_BRAINS_VAULT_PATH` to shadow (**D26**).  
- Compare via **briefing project** + **preflight** JSON.  
- Never User-level env; never Stage D.  
- Style: `#Requires -Version 5.1`, `$ErrorActionPreference = 'Stop'`, `[CmdletBinding()]`.

---

## 10. Limitations / T185 non-claims (D22)

1. Dogfood **pass** ≠ certification, product readiness certification, or perfect deletion.  
2. T169 synthetic fixtures are not LoCoMo / LongMemEval / BEAM scores.  
3. No LLM-as-judge; no AGPL/SaaS evaluators required for vault content.  
4. CE honesty: legacy migrate ≠ cryptographic erasure (ADR-0016).  
5. Stage C/D may be deferred; MVP may close at Stage C.  
6. Soft metrics and latency are not product quality claims for T185.  
7. Live enablement remains an explicit human decision after this gate.

T185 indexes report paths + hashes; this runbook is the human-readable procedure for dogfood evidence.

---

## 11. Evidence location

`conductor/tracks/trackT170-shadow-dogfood-gate/evidence/` (gitignore `*.db`, vault blobs). Sanitized notes only — no full claim bodies, no PII, no real vault files in git.
