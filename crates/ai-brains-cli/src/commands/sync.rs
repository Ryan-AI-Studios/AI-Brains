use crate::context::{AppContext, StoreSink};
use ai_brains_capture::{CaptureContext, CaptureService};
use ai_brains_contracts::bridge::{BridgePayload, BridgeRecord};
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::TurnId;
use ai_brains_store::EventStore;
use chrono;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;

struct TempFileCleanup {
    path: PathBuf,
}

impl Drop for TempFileCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub fn run_pull(
    ctx: &AppContext,
    from_file: Option<PathBuf>,
    hotspots: bool,
    ledger: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let _temp_cleanup;
    let actual_file = match from_file {
        Some(path) => {
            if !path.exists() {
                return Err(format!("File not found: {}", path.display()).into());
            }
            _temp_cleanup = None;
            path
        }
        None => {
            let temp_path = {
                let mut p = std::env::temp_dir();
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| format!("Clock error: {}", e))?
                    .as_nanos();
                p.push(format!("cg_export_{}.ndjson", now));
                p
            };
            _temp_cleanup = Some(TempFileCleanup {
                path: temp_path.clone(),
            });

            let mut cmd = std::process::Command::new("ledgerful");
            cmd.arg("bridge").arg("export");
            cmd.arg("--out").arg(&temp_path);

            let pull_hotspots = hotspots || !ledger;
            let pull_ledger = ledger || !hotspots;

            if pull_hotspots {
                cmd.arg("--hotspots");
            }
            if pull_ledger {
                cmd.arg("--ledger");
            }

            if quiet {
                cmd.stderr(std::process::Stdio::null());
            }

            let output = cmd.output()?;
            if !output.status.success() {
                if quiet {
                    return Ok(());
                }
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to export from ledgerful: {}", stderr).into());
            }
            temp_path
        }
    };

    let file = File::open(&actual_file)?;
    let reader = BufReader::new(file);

    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
    let mut sink = StoreSink {
        store: event_store,
        last_error: None,
        #[cfg(feature = "graph")]
        graph_hook: Some(crate::live_graph::LiveGraphHook::new(
            std::sync::Arc::clone(&ctx.conn),
        )),
    };

    let service = CaptureService::new();
    let capture_context = CaptureContext {
        git_working_dir: std::env::current_dir().ok(),
    };

    let mut count = 0;
    let mut last_hash: Option<String> = sink.store.get_sync_state("last_inbound_hash")?;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let record: BridgeRecord = match serde_json::from_str::<BridgeRecord>(&line) {
            Ok(r) => {
                // T92: Only enforce lineage when we have a prior hash (non-bootstrap).
                // On first-ever import last_hash is None; accept the chain as-is to
                // bootstrap state. On subsequent imports, verify continuity.
                if let (Some(actual_parent), Some(expected_last)) = (&r.parent_hash, &last_hash)
                    && actual_parent != expected_last
                {
                    tracing::warn!(
                        "Lineage verification failed: parent_hash mismatch. \
                             Expected {}, got {}. Skipping record.",
                        expected_last,
                        actual_parent
                    );
                    continue;
                }
                r
            }
            Err(err) => {
                tracing::warn!("Failed to parse BridgeRecord: {}. Skipping line.", err);
                continue;
            }
        };

        // Compute current record hash for the next iteration
        use sha2::{Digest, Sha256};
        let json_for_hash = serde_json::to_string(&record).unwrap_or_default();
        let mut hasher2 = Sha256::new();
        hasher2.update(json_for_hash.as_bytes());
        let hash_hex = hex::encode(hasher2.finalize());
        last_hash = Some(hash_hex.clone());
        sink.store.set_sync_state("last_inbound_hash", &hash_hex)?;

        // T92: Accept records regardless of direction tag — `bridge export` exports
        // Ledgerful-native data (hotspots, ledger) whose direction may not be
        // tagged as Inbound. Direction filtering belongs in Ledgerful's exporter,
        // not in our consumer.

        // Parse string IDs from the interchange format into typed IDs.
        let project_id = ai_brains_core::ids::ProjectId::from_str(&record.project_id)
            .unwrap_or_else(|_| ai_brains_core::ids::ProjectId::new());
        let session_id = match &record.session_id {
            Some(s) => ai_brains_core::ids::SessionId::from_str(s)
                .unwrap_or_else(|_| ai_brains_core::ids::SessionId::new()),
            None => ai_brains_core::ids::SessionId::new(),
        };
        let tx_id = record
            .tx_id
            .as_ref()
            .map(|s| ai_brains_core::ids::TransactionId::new(s.clone()));

        // Apply Privacy::combine() during sync ingestion — combine incoming record privacy with project session privacy.
        let session_privacy = sink
            .store
            .get_session_privacy(&session_id.to_string())?
            .unwrap_or(record.privacy);
        let combined_privacy = record.privacy.combine(session_privacy);

        // Ensure context exists
        ctx.ensure_project_and_session_exists(
            &mut sink,
            &service,
            &capture_context,
            project_id,
            session_id,
            ai_brains_core::ids::HarnessId::default(), // Unknown harness
            combined_privacy,
        )?;

        // Map record to IngestRequest
        let role = match record.record_kind.to_lowercase().as_str() {
            "user" | "prompt" => "user",
            "assistant" | "response" | "final" => "assistant",
            _ => "assistant", // Default to assistant for external signals
        };

        let content = record.formatted_payload();

        // Handle specific structured payloads
        if record.record_kind == "verify_outcome" {
            let payload_value =
                serde_json::to_value(&record.payload).unwrap_or(serde_json::Value::Null);
            if let Ok(outcome) = serde_json::from_value::<
                ai_brains_events::VerifyOutcomeRecordedPayload,
            >(payload_value)
            {
                let event = ai_brains_events::constructors::EventBuilder::new(
                    ai_brains_events::AggregateType::System,
                    uuid::Uuid::new_v4(),
                    ai_brains_events::Actor::System,
                    combined_privacy,
                )
                .build(ai_brains_events::Payload::VerifyOutcomeRecorded(outcome))?;
                sink.store.append_event(&event)?;
                count += 1;
                continue;
            }
        }

        let request = IngestRequest {
            session_id,
            project_id,
            harness_id: ai_brains_core::ids::HarnessId::default(),
            turn_id: TurnId::new(),
            role: role.to_string(),
            content,
            thinking: None,
            privacy: combined_privacy,
            tx_id,
        };

        service.ingest_request(request, capture_context.clone(), &mut sink)?;
        count += 1;
    }

    println!("Successfully synced {} records.", count);
    Ok(())
}

#[allow(clippy::disallowed_methods, clippy::type_complexity)]
pub fn run_push(
    ctx: &AppContext,
    _with_impact: bool,
    _with_verify: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !quiet {
        println!("AI-Brains: Exporting insights to Ledgerful...");
    }

    use ai_brains_contracts::bridge::{BridgeDirection, BridgeRecord};
    use std::io::Write;

    use std::str::FromStr;
    let project_id = if let Ok(val) = std::env::var("AI_BRAINS_PROJECT_ID") {
        ai_brains_core::ids::ProjectId::from_str(&val)?
    } else {
        ai_brains_core::ids::ProjectId::new()
    };
    let session_id = if let Ok(val) = std::env::var("AI_BRAINS_SESSION_ID") {
        ai_brains_core::ids::SessionId::from_str(&val)?
    } else {
        ai_brains_core::ids::SessionId::new()
    };

    let mut out_records = Vec::new();
    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
    let mut last_hash: Option<String> = event_store.get_sync_state("last_outbound_hash")?;

    let rows_data: Vec<(String, String, String, Option<String>, Option<String>)> = {
        let conn = ctx.conn.lock()?;
        let mut stmt = conn.prepare("SELECT memory_id, content, privacy, project_id, session_id FROM memory_projection WHERE level > 0")?;
        let mut rows = stmt.query([])?;
        let mut data = Vec::new();
        while let Some(row) = rows.next()? {
            data.push((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ));
        }
        data
    };

    for (memory_id, content, privacy_json, row_project_id_str, row_session_id_str) in rows_data {
        let privacy = serde_json::from_str::<ai_brains_core::privacy::Privacy>(&privacy_json)
            .unwrap_or_default();
        if privacy == ai_brains_core::privacy::Privacy::NeverInject
            || privacy == ai_brains_core::privacy::Privacy::Sealed
        {
            continue;
        }

        let record_project_id = if let Some(pid_str) = row_project_id_str {
            ai_brains_core::ids::ProjectId::from_str(&pid_str).unwrap_or(project_id)
        } else {
            project_id
        };

        let record_session_id = if let Some(sid_str) = row_session_id_str {
            ai_brains_core::ids::SessionId::from_str(&sid_str).unwrap_or(session_id)
        } else {
            session_id
        };

        let payload = BridgePayload::Insight {
            type_field: "Insight".to_string(),
            memory_id,
            relevance: 1.0,
            content,
        };
        let record = BridgeRecord {
            bridge_version: "0.3".to_string(),
            direction: BridgeDirection::Outbound,
            timestamp: chrono::Utc::now(),
            parent_hash: last_hash.clone(),
            project_id: record_project_id.to_string(),
            session_id: Some(record_session_id.to_string()),
            tx_id: None,
            record_kind: "insight".to_string(),
            payload,
            privacy,
        };

        // Compute current record hash for next record's parent_hash
        use sha2::{Digest, Sha256};
        let json_for_hash = serde_json::to_string(&record).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json_for_hash.as_bytes());
        let hash_hex = hex::encode(hasher.finalize());
        last_hash = Some(hash_hex.clone());

        out_records.push(record);
    }

    if let Some(hash) = last_hash {
        event_store.set_sync_state("last_outbound_hash", &hash)?;
    }

    if out_records.is_empty() {
        println!("No insights to push.");
        return Ok(());
    }

    let temp_dir = std::env::temp_dir();
    let export_path = temp_dir.join("aibrains_export.ndjson");
    let mut file = std::fs::File::create(&export_path)?;

    for record in out_records {
        let json = serde_json::to_string(&record)?;
        writeln!(file, "{}", json)?;
    }
    file.flush()?;

    println!("Triggering ledgerful bridge import...");
    let mut cmd = std::process::Command::new("ledgerful");
    cmd.args([
        "bridge",
        "import",
        "--input",
        export_path.to_string_lossy().as_ref(),
    ]);

    if quiet {
        cmd.stderr(std::process::Stdio::null());
    }

    let output = cmd.output();

    match output {
        Ok(out) if out.status.success() => {
            if !quiet {
                println!("{}", String::from_utf8_lossy(&out.stdout));
                println!("Successfully pushed insights to Ledgerful.");
            }
        }
        Ok(out) => {
            if !quiet {
                tracing::warn!(
                    "ledgerful import failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }
        Err(e) => {
            if !quiet {
                tracing::info!("ledgerful CLI not found or failed to execute. Error: {}", e);
            }
        }
    }

    Ok(())
}

#[allow(clippy::disallowed_methods)]
pub async fn run_query(
    ctx: &AppContext,
    query: String,
    format: Option<String>,
    quiet: bool,
    global: bool,
    no_bridge: bool,
    limit: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let fmt = format.unwrap_or_else(|| "pretty".to_string());

    let project_id = if global {
        None
    } else {
        let project_id_str =
            std::env::var("AI_BRAINS_PROJECT_ID").unwrap_or_else(|_| "default-project".to_string());
        use std::str::FromStr;
        Some(
            ai_brains_core::ids::ProjectId::from_str(&project_id_str)
                .unwrap_or_else(|_| ai_brains_core::ids::ProjectId::new()),
        )
    };

    #[cfg(feature = "graph")]
    let graph_vault = ai_brains_graph::GraphVault::new((*ctx.conn).clone());
    #[cfg(feature = "graph")]
    let graph_search = Some(ai_brains_graph::queries::GraphSearch::new(&graph_vault));
    #[cfg(not(feature = "graph"))]
    let graph_search: Option<ai_brains_retrieval::MockGraphSearch> = None;

    if fmt == "ndjson" {
        let project_id = project_id.unwrap_or_else(ai_brains_core::ids::ProjectId::new);

        let hits = ai_brains_retrieval::recall(
            &ctx.conn,
            graph_search.as_ref(),
            &query,
            limit,
            ai_brains_retrieval::RecallOptions {
                project_id: Some(project_id),
                session_id: None,
                semantic: false,
                graph_boost: 0.1,
                graph_hop_depth: 1,
                quiet,
                no_bridge,
                min_semantic_score: None,
            },
        )?;

        use ai_brains_contracts::bridge::{BridgeDirection, BridgePayload, BridgeRecord};
        let timestamp = chrono::Utc::now();

        for h in hits {
            let payload = BridgePayload::Insight {
                type_field: "Insight".to_string(),
                memory_id: h.memory_id,
                relevance: h.score.unwrap_or(1.0),
                content: h.content,
            };

            let record = BridgeRecord {
                bridge_version: "0.3".to_string(),
                direction: BridgeDirection::Outbound,
                timestamp,
                parent_hash: None,
                project_id: project_id.to_string(),
                session_id: h.session_id.as_ref().map(|s| s.to_string()),
                tx_id: None,
                record_kind: "insight".to_string(),
                payload,
                privacy: ai_brains_core::privacy::Privacy::LocalOnly,
            };

            let json = serde_json::to_string(&record)?;
            println!("{}", json);
        }
        return Ok(());
    }

    // F37: pretty path calls recall_full directly (not recall::run) so hits are
    // inspectable for F12 ledger-first. Vault arm always skips IPC bridge
    // (ledger is a separate section below).
    let outcome = ai_brains_retrieval::recall_full(
        &ctx.conn,
        graph_search.as_ref(),
        &query,
        limit,
        ai_brains_retrieval::RecallOptions {
            project_id,
            session_id: None,
            semantic: false,
            graph_boost: 0.1,
            graph_hop_depth: 1,
            quiet,
            no_bridge: true,
            min_semantic_score: None,
        },
    )?;
    let hits = outcome.hits;

    // F12: when not --no-bridge, probe ledger JSON for non-empty results.
    let ledger_section = if no_bridge {
        None
    } else {
        probe_ledger_search(&query, quiet)
    };
    let ledger_non_empty = ledger_section
        .as_ref()
        .map(|s| s.non_empty)
        .unwrap_or(false);
    // F12: ledger-first when top vault hit is Plan-class Decision, OR every
    // vault Decision hit is Plan (no Shipped/Unknown Decision above plan noise).
    let top_is_plan = hits.first().is_some_and(|h| h.is_plan_demoted);
    let every_decision_is_plan = {
        let decisions: Vec<_> = hits
            .iter()
            .filter(|h| {
                ai_brains_retrieval::classify_pin_kind(&h.content)
                    == ai_brains_retrieval::PinKind::Decision
            })
            .collect();
        !decisions.is_empty() && decisions.iter().all(|h| h.is_plan_demoted)
    };
    let ledger_first = ledger_non_empty && (top_is_plan || every_decision_is_plan);

    let print_vault = || -> Result<(), Box<dyn std::error::Error>> {
        println!("--- AI-Brains Recall ---");
        if hits.is_empty() {
            // T207 empty pretty: Scope + hint, no TTY gate (F37 preserve).
            crate::commands::recall::print_pretty_empty_sync(ctx, &query, global, project_id)?;
        } else {
            crate::commands::recall::print_pretty_hits(&hits);
        }
        Ok(())
    };

    let print_ledger = |section: &LedgerProbeResult| {
        println!("\n--- Ledgerful Ledger Search ---");
        if let Some(ref text) = section.display {
            println!("{}", text);
        }
    };

    if ledger_first {
        println!("Note: vault top hit is plan/stale; ledger results shown first.");
        if let Some(ref section) = ledger_section {
            print_ledger(section);
        }
        println!();
        print_vault()?;
    } else {
        print_vault()?;
        if let Some(ref section) = ledger_section
            && (section.non_empty || section.display.is_some())
        {
            print_ledger(section);
        }
    }

    Ok(())
}

/// Result of a `ledgerful ledger search --json` probe (T211 F12).
struct LedgerProbeResult {
    non_empty: bool,
    /// Human-readable display text (from re-run without --json, or pretty JSON).
    display: Option<String>,
}

/// Probe ledger for non-empty results; fail/empty/missing → vault-only (no panic).
#[allow(clippy::disallowed_methods)]
fn probe_ledger_search(query: &str, quiet: bool) -> Option<LedgerProbeResult> {
    use is_terminal::IsTerminal;
    let is_tty = std::io::stdout().is_terminal();

    // T91: strip ANSI; T90: sanitize FTS before forwarding.
    let clean_query = ai_brains_retrieval::strip_ansi(query);
    let sanitized_query = ai_brains_retrieval::sanitize_fts_query(&clean_query);

    let mut json_cmd = std::process::Command::new("ledgerful");
    json_cmd.args(["ledger", "search", "--json", &sanitized_query]);
    if !is_tty {
        json_cmd.env("NO_COLOR", "1");
    }
    if quiet {
        json_cmd.stderr(std::process::Stdio::null());
    }

    let json_output = match json_cmd.output() {
        Ok(out) => out,
        Err(_) => {
            if !quiet {
                tracing::info!("ledgerful CLI not found or failed to execute.");
            }
            return None;
        }
    };

    if !json_output.status.success() {
        if !quiet {
            tracing::warn!(
                "ledgerful search failed: {}",
                String::from_utf8_lossy(&json_output.stderr)
            );
        }
        return None;
    }

    let stdout = String::from_utf8_lossy(&json_output.stdout);
    let non_empty = ledger_json_non_empty(&stdout);

    // Prefer a human display: re-run without --json when non-empty (or always if free).
    let display = {
        let mut human_cmd = std::process::Command::new("ledgerful");
        human_cmd.args(["ledger", "search", &sanitized_query]);
        if !is_tty {
            human_cmd.env("NO_COLOR", "1");
        }
        if quiet {
            human_cmd.stderr(std::process::Stdio::null());
        }
        match human_cmd.output() {
            Ok(out) if out.status.success() => {
                let s = String::from_utf8_lossy(&out.stdout).into_owned();
                let s = if is_tty {
                    s
                } else {
                    ai_brains_retrieval::strip_ansi(&s)
                };
                if s.trim().is_empty() { None } else { Some(s) }
            }
            _ => {
                // Fall back to raw JSON probe stdout.
                let s = if is_tty {
                    stdout.into_owned()
                } else {
                    ai_brains_retrieval::strip_ansi(&stdout)
                };
                if s.trim().is_empty() { None } else { Some(s) }
            }
        }
    };

    Some(LedgerProbeResult { non_empty, display })
}

/// F12 non-empty detection: success already checked; JSON array/object with ≥1
/// entry OR ≥1 non-empty JSON line that parses.
fn ledger_json_non_empty(stdout: &str) -> bool {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Whole-stdout JSON array or object.
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
        return match v {
            serde_json::Value::Array(a) => !a.is_empty(),
            serde_json::Value::Object(o) => !o.is_empty(),
            _ => false,
        };
    }

    // NDJSON / multi-line: any non-empty line that parses as JSON value.
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            match v {
                serde_json::Value::Array(a) if !a.is_empty() => return true,
                serde_json::Value::Object(o) if !o.is_empty() => return true,
                serde_json::Value::Null
                | serde_json::Value::Bool(_)
                | serde_json::Value::Number(_) => {
                    continue;
                }
                serde_json::Value::String(s) if s.is_empty() => continue,
                serde_json::Value::String(_) => return true,
                serde_json::Value::Array(_) | serde_json::Value::Object(_) => continue,
            }
        }
    }
    false
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::ledger_json_non_empty;

    #[test]
    #[allow(non_snake_case)]
    fn ledger_json_non_empty__array_with_item() {
        assert!(ledger_json_non_empty(r#"[{"id":1}]"#));
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_json_non_empty__empty_array() {
        assert!(!ledger_json_non_empty("[]"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_json_non_empty__ndjson_object_line() {
        assert!(ledger_json_non_empty("{\"a\":1}\n"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn ledger_json_non_empty__blank() {
        assert!(!ledger_json_non_empty("  \n"));
    }
}
