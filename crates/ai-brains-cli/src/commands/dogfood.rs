//! `ai-brains dogfood compare` — pure-serde dogfood compare packet (T170).
//!
//! Reads governed ProjectBriefingPacket JSON + legacy preflight JSON and emits
//! `dogfood-compare.json` (schema v1). Zero new crates; path-free sync path.

use crate::commands::governed_common::{EXIT_INVALID_PAYLOAD, GovernedCliError, emit_json};
use ai_brains_path::resolve_best_effort;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Risk warning kinds covered by human review D7 (not `other`).
pub const RISK_WARNING_KINDS: &[&str] = &[
    "stale",
    "disputed",
    "open_conflict",
    "unavailable",
    "denied",
    "low_confidence",
];

/// CLI options for `dogfood compare`.
#[derive(Debug, Clone)]
pub struct DogfoodCompareOptions {
    pub governed: PathBuf,
    pub legacy: PathBuf,
    pub out: PathBuf,
    pub stage: Option<String>,
    pub evaluate_report: Option<PathBuf>,
    pub shadow: Option<PathBuf>,
    pub migrated: Option<PathBuf>,
    pub live_vault: Option<PathBuf>,
    pub sha256_pre: Option<String>,
    pub sha256_post: Option<String>,
    pub t169_exit: Option<i32>,
    pub t169_report_hash: Option<String>,
    pub t169_hard_gates_passed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WarningRef {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DogfoodComparePacket {
    pub schema_version: u32,
    pub created_at: String,
    pub compare_hash: String,
    pub stage: String,
    pub paths: PathsSection,
    pub live_vault_integrity: LiveVaultIntegrity,
    pub t169: T169Section,
    pub legacy_preflight: LegacyPreflightSection,
    pub governed_briefing: GovernedBriefingSection,
    pub diff: DiffSection,
    pub human_review_seed: HumanReviewSeedSection,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathsSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub migrated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluate_report: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub migrate_report: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live_vault: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveVaultIntegrity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256_pre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256_post: Option<String>,
    pub unchanged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct T169Section {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hard_gates_passed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyPreflightSection {
    pub mode: String,
    pub source_command: String,
    pub decision_marker_count: u64,
    pub constraint_marker_count: u64,
    pub hotspot_marker_count: u64,
    pub word_count: u64,
    pub text_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernedBriefingSection {
    pub mode: String,
    pub source_command: String,
    pub decision_count: u64,
    pub conclusion_count: u64,
    pub warning_kinds: Vec<String>,
    pub uncited_current_count: u64,
    pub denied: bool,
    pub content_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSection {
    pub warning_kinds_only_in_governed: Vec<String>,
    pub note: String,
    pub hard_checks: HardChecks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardChecks {
    pub t169_passed: bool,
    pub live_vault_mutated: bool,
    pub live_checksum_unchanged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HumanReviewSeedSection {
    pub claim_ids_sample: Vec<String>,
    pub warning_refs_all: Vec<WarningRef>,
}

/// Run `dogfood compare`: load inputs, build packet, write `--out`, emit JSON stdout.
pub fn run_compare(opts: DogfoodCompareOptions) -> Result<(), Box<dyn std::error::Error>> {
    let governed_raw = read_json_file(&opts.governed)?;
    let legacy_raw = read_json_file(&opts.legacy)?;
    let evaluate_raw = match &opts.evaluate_report {
        Some(p) => Some(read_json_file(p)?),
        None => None,
    };

    let packet = build_compare_packet(&opts, &governed_raw, &legacy_raw, evaluate_raw.as_ref())?;

    write_compare_out(&opts.out, &packet)?;
    emit_json(&packet)?;
    Ok(())
}

fn read_json_file(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path).map_err(|e| {
        Box::new(GovernedCliError::emitted(
            EXIT_INVALID_PAYLOAD,
            format!("failed to read {}: {e}", path.display()),
        )) as Box<dyn std::error::Error>
    })?;
    let value: Value = serde_json::from_str(&text).map_err(|e| {
        Box::new(GovernedCliError::emitted(
            EXIT_INVALID_PAYLOAD,
            format!("invalid JSON {}: {e}", path.display()),
        )) as Box<dyn std::error::Error>
    })?;
    Ok(value)
}

fn write_compare_out(
    path: &Path,
    packet: &DogfoodComparePacket,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(packet)?;
    let mut file = fs::File::create(path)?;
    file.write_all(json.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

/// Build the full compare packet from loose JSON inputs (public for tests).
pub fn build_compare_packet(
    opts: &DogfoodCompareOptions,
    governed_raw: &Value,
    legacy_raw: &Value,
    evaluate_raw: Option<&Value>,
) -> Result<DogfoodComparePacket, Box<dyn std::error::Error>> {
    let packet_value = extract_briefing_packet(governed_raw)?;
    let legacy = parse_legacy_preflight(legacy_raw)?;

    let decisions = packet_value
        .get("decisions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let conclusions = packet_value
        .get("conclusions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let warnings = packet_value
        .get("warnings")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let decision_count = decisions.len() as u64;
    let conclusion_count = conclusions.len() as u64;
    let denied = packet_value
        .get("denied")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let uncited_current_count = count_uncited(&decisions) + count_uncited(&conclusions);

    let mut warning_kinds: Vec<String> = warnings
        .iter()
        .filter_map(|w| w.get("kind").and_then(|k| k.as_str()).map(str::to_string))
        .collect();
    warning_kinds.sort();
    warning_kinds.dedup();

    let warning_refs_all = collect_risk_warning_refs(&warnings);
    let content_fingerprint = content_fingerprint_from_packet(&packet_value)?;

    let stage = opts
        .stage
        .clone()
        .unwrap_or_else(|| "B".to_string())
        .to_uppercase();

    let claim_ids_sample = match stage.as_str() {
        "C" => stratified_claim_sample(&decisions, &conclusions, 20),
        _ => {
            // Stage B: prefer T169 human_review_seed when evaluate report present.
            if let Some(eval) = evaluate_raw {
                extract_t169_claim_sample(eval)
            } else {
                stratified_claim_sample(&decisions, &conclusions, 20)
            }
        }
    };

    let (t169_exit, t169_hash, t169_hard) = resolve_t169_fields(opts, evaluate_raw);

    let sha_pre = opts.sha256_pre.clone().map(|s| s.to_lowercase());
    let sha_post = opts.sha256_post.clone().map(|s| s.to_lowercase());
    let checksum_unchanged = match (&sha_pre, &sha_post) {
        (Some(a), Some(b)) => a == b,
        (None, None) => true, // no live vault → N/A treated as unchanged
        _ => false,
    };
    let live_vault_mutated = !checksum_unchanged && sha_pre.is_some() && sha_post.is_some();

    let t169_passed = t169_exit == Some(0) && t169_hard.unwrap_or(true);

    let paths = PathsSection {
        shadow: opts.shadow.as_ref().map(|p| normalize_path_str(p)),
        migrated: opts.migrated.as_ref().map(|p| normalize_path_str(p)),
        evaluate_report: opts.evaluate_report.as_ref().map(|p| normalize_path_str(p)),
        migrate_report: None,
        live_vault: opts.live_vault.as_ref().map(|p| normalize_path_str(p)),
    };

    let mut packet = DogfoodComparePacket {
        schema_version: 1,
        created_at: now_rfc3339(),
        compare_hash: String::new(),
        stage,
        paths,
        live_vault_integrity: LiveVaultIntegrity {
            sha256_pre: sha_pre,
            sha256_post: sha_post,
            unchanged: checksum_unchanged,
        },
        t169: T169Section {
            exit_code: t169_exit,
            report_hash: t169_hash,
            hard_gates_passed: t169_hard,
        },
        legacy_preflight: LegacyPreflightSection {
            mode: "legacy".to_string(),
            source_command: "ai-brains preflight --vault-path … --format json".to_string(),
            decision_marker_count: legacy.decision_marker_count,
            constraint_marker_count: legacy.constraint_marker_count,
            hotspot_marker_count: legacy.hotspot_marker_count,
            word_count: legacy.word_count,
            text_fingerprint: legacy.text_fingerprint,
        },
        governed_briefing: GovernedBriefingSection {
            mode: "governed".to_string(),
            source_command: "ai-brains briefing project --vault-path … --format json".to_string(),
            decision_count,
            conclusion_count,
            warning_kinds: warning_kinds.clone(),
            uncited_current_count,
            denied,
            content_fingerprint,
        },
        diff: DiffSection {
            warning_kinds_only_in_governed: warning_kinds,
            note: "diff may include kind=other; D7 human review covers six risk kinds only"
                .to_string(),
            hard_checks: HardChecks {
                t169_passed,
                live_vault_mutated,
                live_checksum_unchanged: checksum_unchanged,
            },
        },
        human_review_seed: HumanReviewSeedSection {
            claim_ids_sample,
            warning_refs_all,
        },
        limitations: default_limitations(),
    };

    packet.compare_hash = compute_compare_hash(&packet).map_err(|e| {
        Box::new(GovernedCliError::emitted(
            EXIT_INVALID_PAYLOAD,
            format!("compare_hash failed: {e}"),
        )) as Box<dyn std::error::Error>
    })?;

    Ok(packet)
}

fn resolve_t169_fields(
    opts: &DogfoodCompareOptions,
    evaluate_raw: Option<&Value>,
) -> (Option<i32>, Option<String>, Option<bool>) {
    let mut exit = opts.t169_exit;
    let mut hash = opts.t169_report_hash.clone();
    let mut hard = opts.t169_hard_gates_passed;

    if let Some(eval) = evaluate_raw {
        if hash.is_none() {
            hash = eval
                .get("report_hash")
                .and_then(|v| v.as_str())
                .map(str::to_string);
        }
        if hard.is_none() {
            hard = eval.get("hard_gates_passed").and_then(|v| v.as_bool());
        }
        if exit.is_none() {
            // Infer: hard pass → 0, hard fail → 7 when report present without explicit exit.
            exit = match hard {
                Some(true) => Some(0),
                Some(false) => Some(7),
                None => None,
            };
        }
    }
    (exit, hash, hard)
}

fn extract_t169_claim_sample(eval: &Value) -> Vec<String> {
    eval.get("human_review_seed")
        .and_then(|s| s.get("claim_ids_sample"))
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Accept either a bare ProjectBriefingPacket or `{ "packet": {…} }` / ApiResult wrapper.
fn extract_briefing_packet(raw: &Value) -> Result<Value, Box<dyn std::error::Error>> {
    if raw.get("decisions").is_some() || raw.get("conclusions").is_some() {
        return Ok(raw.clone());
    }
    if let Some(packet) = raw.get("packet") {
        return Ok(packet.clone());
    }
    if let Some(data) = raw.get("data")
        && (data.get("decisions").is_some() || data.get("packet").is_some())
    {
        return extract_briefing_packet(data);
    }
    Err(Box::new(GovernedCliError::emitted(
        EXIT_INVALID_PAYLOAD,
        "governed JSON missing decisions/conclusions (expected ProjectBriefingPacket)",
    )))
}

#[derive(Debug)]
struct LegacyParsed {
    decision_marker_count: u64,
    constraint_marker_count: u64,
    hotspot_marker_count: u64,
    word_count: u64,
    text_fingerprint: String,
}

fn parse_legacy_preflight(raw: &Value) -> Result<LegacyParsed, Box<dyn std::error::Error>> {
    // Accept bare {text, word_count} or nested under data / context.
    let obj = if raw.get("text").is_some() {
        raw
    } else if let Some(data) = raw.get("data") {
        data
    } else {
        raw
    };

    let text = obj
        .get("text")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let word_count = obj
        .get("word_count")
        .and_then(|v| v.as_u64())
        .unwrap_or_else(|| count_words(&text));

    let (d, c, h) = count_legacy_markers(&text);
    Ok(LegacyParsed {
        decision_marker_count: d,
        constraint_marker_count: c,
        hotspot_marker_count: h,
        word_count,
        text_fingerprint: sha256_hex(text.as_bytes()),
    })
}

/// Count `DECISION:` / `CONSTRAINT:` / `HOTSPOT:` markers in legacy preflight text.
pub fn count_legacy_markers(text: &str) -> (u64, u64, u64) {
    let decision = count_marker(text, "DECISION:");
    let constraint = count_marker(text, "CONSTRAINT:");
    let hotspot = count_marker(text, "HOTSPOT:");
    (decision, constraint, hotspot)
}

fn count_marker(text: &str, marker: &str) -> u64 {
    text.match_indices(marker).count() as u64
}

fn count_words(text: &str) -> u64 {
    text.split_whitespace().count() as u64
}

fn count_uncited(claims: &[Value]) -> u64 {
    claims
        .iter()
        .filter(|c| {
            c.get("evidence_handles")
                .and_then(|h| h.as_array())
                .map(|a| a.is_empty())
                .unwrap_or(true)
        })
        .count() as u64
}

/// Collect risk-kind warning refs sorted by (kind, subject_id).
pub fn collect_risk_warning_refs(warnings: &[Value]) -> Vec<WarningRef> {
    let mut refs: Vec<WarningRef> = warnings
        .iter()
        .filter_map(|w| {
            let kind = w.get("kind").and_then(|k| k.as_str())?.to_string();
            if !RISK_WARNING_KINDS.contains(&kind.as_str()) {
                return None;
            }
            let subject_id = w
                .get("subject_id")
                .and_then(|s| s.as_str())
                .map(str::to_string);
            Some(WarningRef { kind, subject_id })
        })
        .collect();
    refs.sort_by(|a, b| {
        a.kind
            .cmp(&b.kind)
            .then_with(|| a.subject_id.cmp(&b.subject_id))
    });
    refs.dedup();
    refs
}

/// D15 Stage C stratification: up to 5 Decision + 5 Conclusion by sorted id, fill to `limit`.
pub fn stratified_claim_sample(
    decisions: &[Value],
    conclusions: &[Value],
    limit: usize,
) -> Vec<String> {
    let mut decision_ids: Vec<String> = decisions
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_str()).map(str::to_string))
        .collect();
    decision_ids.sort();
    decision_ids.dedup();

    let mut conclusion_ids: Vec<String> = conclusions
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_str()).map(str::to_string))
        .collect();
    conclusion_ids.sort();
    conclusion_ids.dedup();

    let mut sample: Vec<String> = Vec::new();
    for id in decision_ids.iter().take(5) {
        sample.push(id.clone());
    }
    for id in conclusion_ids.iter().take(5) {
        if !sample.contains(id) {
            sample.push(id.clone());
        }
    }

    let mut all: Vec<String> = decision_ids.into_iter().chain(conclusion_ids).collect();
    all.sort();
    all.dedup();

    for id in all {
        if sample.len() >= limit {
            break;
        }
        if !sample.contains(&id) {
            sample.push(id);
        }
    }
    sample.truncate(limit);
    sample
}

/// SHA-256 of canonical JSON of packet excluding briefing_id / generated_at.
pub fn content_fingerprint_from_packet(packet: &Value) -> Result<String, String> {
    let mut stripped = packet.clone();
    if let Some(obj) = stripped.as_object_mut() {
        obj.remove("briefing_id");
        obj.remove("generated_at");
    }
    let canon = canonicalize_value(&stripped);
    let bytes =
        serde_json::to_vec(&canon).map_err(|e| format!("content_fingerprint serialize: {e}"))?;
    Ok(sha256_hex(&bytes))
}

/// Hex SHA-256 of compare packet with created_at / latency_ms / compare_hash excluded.
pub fn compute_compare_hash(packet: &DogfoodComparePacket) -> Result<String, String> {
    let value = serde_json::to_value(packet).map_err(|e| format!("compare serialize: {e}"))?;
    let mut map = match value {
        Value::Object(m) => m,
        _ => return Err("compare packet root must be object".into()),
    };
    map.remove("created_at");
    map.remove("compare_hash");
    strip_latency_ms(&mut Value::Object(map.clone()));
    // Re-build after strip on owned value:
    let mut root = Value::Object(map);
    strip_latency_ms(&mut root);
    // Sort arrays that are order-independent.
    sort_compare_arrays(&mut root);
    let canon = canonicalize_value(&root);
    let bytes = serde_json::to_vec(&canon).map_err(|e| format!("compare_hash serialize: {e}"))?;
    Ok(sha256_hex(&bytes))
}

fn strip_latency_ms(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("latency_ms");
            for v in map.values_mut() {
                strip_latency_ms(v);
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                strip_latency_ms(v);
            }
        }
        _ => {}
    }
}

fn sort_compare_arrays(root: &mut Value) {
    let Some(obj) = root.as_object_mut() else {
        return;
    };
    if let Some(gb) = obj
        .get_mut("governed_briefing")
        .and_then(|v| v.as_object_mut())
        && let Some(Value::Array(kinds)) = gb.get_mut("warning_kinds")
    {
        kinds.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
    }
    if let Some(diff) = obj.get_mut("diff").and_then(|v| v.as_object_mut())
        && let Some(Value::Array(kinds)) = diff.get_mut("warning_kinds_only_in_governed")
    {
        kinds.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
    }
    if let Some(seed) = obj
        .get_mut("human_review_seed")
        .and_then(|v| v.as_object_mut())
    {
        if let Some(Value::Array(ids)) = seed.get_mut("claim_ids_sample") {
            // Sample order is significant for Stage C stratification display; keep as-is for hash
            // stability of the emitted packet (already deterministic from builder).
            let _ = ids;
        }
        if let Some(Value::Array(refs)) = seed.get_mut("warning_refs_all") {
            refs.sort_by(|a, b| {
                let ka = a.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                let kb = b.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                let sa = a.get("subject_id").and_then(|v| v.as_str()).unwrap_or("");
                let sb = b.get("subject_id").and_then(|v| v.as_str()).unwrap_or("");
                ka.cmp(kb).then_with(|| sa.cmp(sb))
            });
        }
    }
}

/// Recursively convert maps to BTreeMap key order; sort arrays of objects by `id` when present.
pub fn canonicalize_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted = Map::new();
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            for k in keys {
                if let Some(v) = map.get(k) {
                    sorted.insert(k.clone(), canonicalize_value(v));
                }
            }
            Value::Object(sorted)
        }
        Value::Array(arr) => {
            let mut items: Vec<Value> = arr.iter().map(canonicalize_value).collect();
            // Sort by id when every element has an id string (claim-like arrays).
            if items
                .iter()
                .all(|v| v.get("id").and_then(|i| i.as_str()).is_some())
            {
                items.sort_by(|a, b| {
                    let ia = a.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let ib = b.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    ia.cmp(ib)
                });
            }
            Value::Array(items)
        }
        other => other.clone(),
    }
}

fn normalize_path_str(path: &Path) -> String {
    let raw = path.to_string_lossy();
    resolve_best_effort(&raw)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let dig = hasher.finalize();
    dig.iter().map(|b| format!("{b:02x}")).collect()
}

fn now_rfc3339() -> String {
    // Prefer chrono when available via workspace; fall back to UNIX secs string.
    use std::time::UNIX_EPOCH;
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Stable enough for created_at (excluded from compare_hash).
    format!("{secs}")
}

fn default_limitations() -> Vec<String> {
    vec![
        "Dogfood pass is not product certification or perfect deletion (T185 non-claim).".into(),
        "Legacy marker counts are not typed claim counts.".into(),
        "content_fingerprint excludes briefing_id/generated_at; text_fingerprint is soft only."
            .into(),
        "Never set AI_BRAINS_VAULT_PATH to shadow/migrated (D26); use --vault-path.".into(),
    ]
}

// ---------------------------------------------------------------------------
// Unit tests (TDD)
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_packet() -> Value {
        json!({
            "api_version": "1",
            "briefing_id": "brief-1",
            "kind": "Project",
            "generated_at": "2026-01-01T00:00:00Z",
            "denied": false,
            "decisions": [
                {"id": "d-b", "kind": "Decision", "statement": "s", "state": "Approved", "evidence_handles": [{"id": "e1"}]},
                {"id": "d-a", "kind": "Decision", "statement": "s", "state": "Approved", "evidence_handles": []},
                {"id": "d-c", "kind": "Decision", "statement": "s", "state": "Approved", "evidence_handles": [{"id": "e2"}]},
                {"id": "d-d", "kind": "Decision", "statement": "s", "state": "Approved", "evidence_handles": [{"id": "e3"}]},
                {"id": "d-e", "kind": "Decision", "statement": "s", "state": "Approved", "evidence_handles": [{"id": "e4"}]},
                {"id": "d-f", "kind": "Decision", "statement": "s", "state": "Approved", "evidence_handles": [{"id": "e5"}]},
            ],
            "conclusions": [
                {"id": "c-b", "kind": "Conclusion", "statement": "s", "state": "Active", "evidence_handles": [{"id": "e6"}]},
                {"id": "c-a", "kind": "Conclusion", "statement": "s", "state": "Active", "evidence_handles": [{"id": "e7"}]},
                {"id": "c-c", "kind": "Conclusion", "statement": "s", "state": "Active", "evidence_handles": [{"id": "e8"}]},
                {"id": "c-d", "kind": "Conclusion", "statement": "s", "state": "Active", "evidence_handles": [{"id": "e9"}]},
                {"id": "c-e", "kind": "Conclusion", "statement": "s", "state": "Active", "evidence_handles": [{"id": "e10"}]},
                {"id": "c-f", "kind": "Conclusion", "statement": "s", "state": "Active", "evidence_handles": [{"id": "e11"}]},
            ],
            "warnings": [
                {"kind": "stale", "message": "m", "subject_id": "d-a"},
                {"kind": "other", "message": "info"},
                {"kind": "denied", "message": "m", "subject_id": "x"},
                {"kind": "stale", "message": "m", "subject_id": "c-a"},
            ],
            "constraints": [],
            "freshness": {
                "total_sources": 0,
                "fresh_count": 0,
                "stale_count": 0,
                "unavailable_count": 0,
                "worst_state": "Unknown"
            },
            "evidence_handles": [],
            "budget": {
                "max_words": 100,
                "used_words": 10,
                "truncated_sections": [],
                "more_available": false
            },
            "scope": {
                "scope_key": "Repository:00000000-0000-0000-0000-000000000001",
                "confidence": "High",
                "warnings": [],
                "alternatives": [],
                "authoritative": true
            }
        })
    }

    fn sample_legacy() -> Value {
        json!({
            "text": "DECISION: one\nCONSTRAINT: two\nHOTSPOT: three\nDECISION: four",
            "word_count": 8
        })
    }

    fn base_opts() -> DogfoodCompareOptions {
        DogfoodCompareOptions {
            governed: PathBuf::from("g.json"),
            legacy: PathBuf::from("l.json"),
            out: PathBuf::from("out.json"),
            stage: Some("C".into()),
            evaluate_report: None,
            shadow: Some(PathBuf::from("shadow.db")),
            migrated: None,
            live_vault: None,
            sha256_pre: Some("abc".into()),
            sha256_post: Some("abc".into()),
            t169_exit: Some(0),
            t169_report_hash: Some("rh1".into()),
            t169_hard_gates_passed: Some(true),
        }
    }

    #[test]
    fn compare_hash__excludes_created_at__same_hash() {
        let opts = base_opts();
        let mut p1 = build_compare_packet(&opts, &sample_packet(), &sample_legacy(), None).unwrap();
        let mut p2 = p1.clone();
        p1.created_at = "111".into();
        p2.created_at = "999".into();
        // Recompute hashes after mutating created_at
        p1.compare_hash = compute_compare_hash(&p1).unwrap();
        p2.compare_hash = compute_compare_hash(&p2).unwrap();
        assert_eq!(p1.compare_hash, p2.compare_hash);
    }

    #[test]
    fn warning_refs__sort_stable_and_filter_risk_kinds() {
        let warnings = sample_packet()
            .get("warnings")
            .unwrap()
            .as_array()
            .unwrap()
            .clone();
        let refs = collect_risk_warning_refs(&warnings);
        // `other` excluded; risk kinds only
        assert!(
            refs.iter()
                .all(|r| RISK_WARNING_KINDS.contains(&r.kind.as_str()))
        );
        // sorted by (kind, subject_id)
        let kinds: Vec<_> = refs
            .iter()
            .map(|r| (r.kind.as_str(), r.subject_id.as_deref()))
            .collect();
        let mut sorted = kinds.clone();
        sorted.sort();
        assert_eq!(kinds, sorted);
        // denied before stale (lexicographic)
        assert_eq!(refs[0].kind, "denied");
        assert_eq!(refs[1].kind, "stale");
        assert_eq!(refs[1].subject_id.as_deref(), Some("c-a"));
        assert_eq!(refs[2].subject_id.as_deref(), Some("d-a"));
    }

    #[test]
    fn d15_stratification__five_plus_five_then_fill() {
        let packet = sample_packet();
        let decisions = packet.get("decisions").unwrap().as_array().unwrap();
        let conclusions = packet.get("conclusions").unwrap().as_array().unwrap();
        let sample = stratified_claim_sample(decisions, conclusions, 20);
        // 6 decisions + 6 conclusions = 12 total; all should appear
        assert_eq!(sample.len(), 12);
        // First up to 5 decisions sorted: d-a, d-b, d-c, d-d, d-e
        assert_eq!(&sample[0..5], &["d-a", "d-b", "d-c", "d-d", "d-e"]);
        // Then up to 5 conclusions: c-a .. c-e
        assert_eq!(&sample[5..10], &["c-a", "c-b", "c-c", "c-d", "c-e"]);
        // Fill remaining by global sorted id: c-f, d-f
        assert!(sample.contains(&"c-f".to_string()));
        assert!(sample.contains(&"d-f".to_string()));
    }

    #[test]
    fn d15_stratification__empty_inputs__empty_sample() {
        let sample = stratified_claim_sample(&[], &[], 20);
        assert!(sample.is_empty());
    }

    #[test]
    fn legacy_markers__count_decision_constraint_hotspot() {
        let text = "DECISION: a\nCONSTRAINT: b\nHOTSPOT: c\nHOTSPOT: d\nnope DECISION:";
        let (d, c, h) = count_legacy_markers(text);
        assert_eq!(d, 2);
        assert_eq!(c, 1);
        assert_eq!(h, 2);
    }

    #[test]
    fn content_fingerprint__excludes_briefing_id_and_generated_at() {
        let mut a = sample_packet();
        let mut b = sample_packet();
        a.as_object_mut()
            .unwrap()
            .insert("briefing_id".into(), json!("other-id"));
        b.as_object_mut()
            .unwrap()
            .insert("generated_at".into(), json!("2099-01-01T00:00:00Z"));
        let fa = content_fingerprint_from_packet(&a).unwrap();
        let fb = content_fingerprint_from_packet(&b).unwrap();
        let f0 = content_fingerprint_from_packet(&sample_packet()).unwrap();
        assert_eq!(fa, f0);
        assert_eq!(fb, f0);
    }

    #[test]
    fn hard_checks__live_checksum_mismatch__mutated_true() {
        let mut opts = base_opts();
        opts.sha256_pre = Some("aaa".into());
        opts.sha256_post = Some("bbb".into());
        let p = build_compare_packet(&opts, &sample_packet(), &sample_legacy(), None).unwrap();
        assert!(!p.live_vault_integrity.unchanged);
        assert!(p.diff.hard_checks.live_vault_mutated);
        assert!(!p.diff.hard_checks.live_checksum_unchanged);
    }

    #[test]
    fn stage_b__uses_t169_human_review_seed_when_present() {
        let mut opts = base_opts();
        opts.stage = Some("B".into());
        opts.t169_report_hash = None;
        opts.t169_exit = None;
        opts.t169_hard_gates_passed = None;
        let eval = json!({
            "report_hash": "rh-eval",
            "hard_gates_passed": true,
            "human_review_seed": {
                "claim_ids_sample": ["seed-1", "seed-2"],
                "warning_ids_all": []
            }
        });
        let p =
            build_compare_packet(&opts, &sample_packet(), &sample_legacy(), Some(&eval)).unwrap();
        assert_eq!(
            p.human_review_seed.claim_ids_sample,
            vec!["seed-1".to_string(), "seed-2".to_string()]
        );
        assert_eq!(p.t169.report_hash.as_deref(), Some("rh-eval"));
        assert_eq!(p.t169.exit_code, Some(0));
    }

    #[test]
    fn canonicalize_value__sorts_object_keys() {
        let v = json!({"z": 1, "a": 2});
        let c = canonicalize_value(&v);
        let keys: Vec<_> = c.as_object().unwrap().keys().cloned().collect();
        assert_eq!(keys, vec!["a".to_string(), "z".to_string()]);
    }
}
