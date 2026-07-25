use ai_brains_core::ids::{ConclusionId, MemoryId, PrincipalId, ProjectId};
use ai_brains_core::model_provenance::ModelProvenance;
use ai_brains_core::privacy::Privacy;
use ai_brains_events::payload::ConclusionProposedPayload;
use ai_brains_events::{EventKind, MemorySynthesizedPayload, Payload};
use ai_brains_models::{CompletionRequest, ModelProvider};
use ai_brains_store::{EventStore, QueryStore};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use time::OffsetDateTime;
use uuid::Uuid;

/// Env flag: when `1`/`true`/`yes`, hierarchical synthesis emits Candidate
/// [`Payload::ConclusionProposed`] instead of authoritative MemorySynthesized.
pub const GOVERNED_SYNTHESIS_ENV: &str = "AI_BRAINS_GOVERNED_SYNTHESIS";

/// Stable well-known system principal for hierarchical memory synthesis.
///
/// Not [`Uuid::nil`]; used as `proposer` on governed `ConclusionProposed` events so
/// attribution is non-sentinel and stable across runs.
pub const SYSTEM_SYNTHESIS_PRINCIPAL_UUID: Uuid =
    Uuid::from_u128(0xA1B2_A1B2_A1B2_A1B2_A1B2_A1B2_A1B2_0001);

/// Workflow version string attached to governed synthesis provenance.
pub const HIERARCHICAL_SYNTHESIS_WORKFLOW_VERSION: &str = "hierarchical-synthesis/v1";

/// Returns the well-known system principal for hierarchical synthesis.
pub fn system_synthesis_principal() -> PrincipalId {
    PrincipalId::from_uuid(SYSTEM_SYNTHESIS_PRINCIPAL_UUID)
}

/// Returns true when governed synthesis mode is enabled (default: off).
pub fn governed_synthesis_enabled() -> bool {
    match std::env::var(GOVERNED_SYNTHESIS_ENV) {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    }
}

fn sha256_hex(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hex::encode(hasher.finalize())
}

/// Strictest privacy among source memory ids (defaults to LocalOnly when unknown).
fn strictest_source_privacy(
    query_store: &dyn QueryStore,
    cluster: &[(MemoryId, String)],
) -> Result<Privacy, Box<dyn std::error::Error>> {
    let mut strictest = Privacy::LocalOnly;
    for (id, _) in cluster {
        if let Some(p) = query_store.get_memory_privacy(id)? {
            strictest = strictest.combine(p);
        }
    }
    Ok(strictest)
}

/// Sealed / NeverInject must never enter automatic model synthesis (content must not leave the vault).
fn is_excluded_from_automatic_synthesis(privacy: Privacy) -> bool {
    matches!(privacy, Privacy::Sealed | Privacy::NeverInject)
}

/// Drop memories whose privacy forbids automatic synthesis; log exclusions.
fn filter_synthesis_eligible(
    query_store: &dyn QueryStore,
    memories: Vec<(MemoryId, String)>,
) -> Result<Vec<(MemoryId, String)>, Box<dyn std::error::Error>> {
    let mut eligible = Vec::with_capacity(memories.len());
    for (id, content) in memories {
        let privacy = query_store
            .get_memory_privacy(&id)?
            .unwrap_or(Privacy::LocalOnly);
        if is_excluded_from_automatic_synthesis(privacy) {
            tracing::info!(
                memory_id = %id,
                ?privacy,
                "skipping memory for automatic synthesis (privacy excludes model routing)"
            );
            continue;
        }
        eligible.push((id, content));
    }
    Ok(eligible)
}

/// LocalOnly (and stricter eligible tiers if any) require a local model provider.
fn provider_allowed_for_privacy(
    privacy: Privacy,
    provider: &dyn ModelProvider,
) -> Result<(), Box<dyn std::error::Error>> {
    // NeverInject/Sealed are filtered earlier; LocalOnly must not hit cloud providers.
    if privacy >= Privacy::LocalOnly && !provider.is_local() {
        return Err(format!(
            "model provider '{}' is not local; refused for privacy {:?}",
            provider.name(),
            privacy
        )
        .into());
    }
    Ok(())
}

pub struct MemorySynthesizer {
    query_store: Arc<dyn QueryStore>,
    event_store: Arc<dyn EventStore>,
    model_provider: Arc<dyn ModelProvider>,
}

impl MemorySynthesizer {
    pub fn new(
        query_store: Arc<dyn QueryStore>,
        event_store: Arc<dyn EventStore>,
        model_provider: Arc<dyn ModelProvider>,
    ) -> Self {
        Self {
            query_store,
            event_store,
            model_provider,
        }
    }

    pub async fn run_synthesis(
        &self,
        target_level: u32,
        project_id: ProjectId,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        if target_level == 0 {
            return Ok(0);
        }

        // 1. Get all memories at source level that haven't been synthesized yet
        let source_level = target_level - 1;
        let batch_size = std::env::var("AI_BRAINS_SYNTHESIS_BATCH")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(50);
        let source_memories = self
            .query_store
            .get_memories_by_level(source_level, Some(batch_size))?;

        // Exclude Sealed/NeverInject before clustering or any model call (Codex P1-1).
        let source_memories =
            filter_synthesis_eligible(self.query_store.as_ref(), source_memories)?;

        if source_memories.len() < 2 {
            return Ok(0);
        }

        // 2. Cluster them
        let clusters = self.cluster_memories(&source_memories).await?;

        let governed = governed_synthesis_enabled();
        let mut count = 0;
        for cluster in clusters {
            if cluster.len() < 2 {
                continue;
            }

            // Privacy for routing + inheritance: computed before any model call.
            let privacy = strictest_source_privacy(self.query_store.as_ref(), &cluster)?;
            if let Err(e) = provider_allowed_for_privacy(privacy, self.model_provider.as_ref()) {
                tracing::warn!(
                    ?privacy,
                    error = %e,
                    "skipping synthesis cluster: provider privacy routing refused"
                );
                continue;
            }

            // 3. Summarize the cluster (capture model lineage; never store CoT).
            let started_at = OffsetDateTime::now_utc();
            let (synthesis, model_name) = self.synthesize_cluster(&cluster, target_level).await?;
            let completed_at = OffsetDateTime::now_utc();

            // 4. CRAG: Verify the synthesis
            if !self.verify_synthesis(&cluster, &synthesis).await? {
                tracing::warn!(
                    "Synthesized level {} memory was rejected by CRAG verification: {}",
                    target_level,
                    synthesis
                );
                continue;
            }

            // 5. Emit event — governed path proposes Candidate conclusion (no MemoryPinned).
            // Legacy path emits MemorySynthesized (unchanged when flag off).
            // No CoT; statement is the synthesis text only.
            let event = if governed {
                let conclusion_id = ConclusionId::new();
                let scope = format!("Repository:{project_id}");
                let input_ids: Vec<String> = cluster.iter().map(|(id, _)| id.to_string()).collect();
                let deployment = if self.model_provider.is_local() {
                    "local"
                } else {
                    "cloud"
                };
                let model = if model_name.is_empty() {
                    "unknown".to_string()
                } else {
                    model_name
                };
                let provenance = ModelProvenance {
                    provider: self.model_provider.name().to_string(),
                    model,
                    model_version: Some("unknown".to_string()),
                    workflow_version: Some(HIERARCHICAL_SYNTHESIS_WORKFLOW_VERSION.to_string()),
                    deployment: Some(deployment.to_string()),
                    input_ids: Some(input_ids),
                    output_hash: Some(sha256_hex(&synthesis)),
                    started_at: Some(started_at),
                    completed_at: Some(completed_at),
                };
                ai_brains_events::constructors::EventBuilder::new(
                    ai_brains_events::AggregateType::Conclusion,
                    conclusion_id.as_uuid(),
                    ai_brains_events::Actor::System,
                    privacy,
                )
                .build(Payload::ConclusionProposed(ConclusionProposedPayload {
                    conclusion_id,
                    statement: synthesis,
                    // Hierarchical memory sources are not EvidenceIds; flag unsupported.
                    evidence_ids: vec![],
                    proposer: system_synthesis_principal(),
                    valid_from: None,
                    valid_until: None,
                    scope,
                    protected_category: None,
                    unsupported: true,
                    model_provenance: Some(provenance),
                }))?
            } else {
                let memory_id = MemoryId::new();
                let source_memory_ids = cluster.iter().map(|(id, _)| *id).collect();
                ai_brains_events::constructors::EventBuilder::new(
                    ai_brains_events::AggregateType::Memory,
                    memory_id.as_uuid(),
                    ai_brains_events::Actor::System,
                    privacy,
                )
                .build(Payload::MemorySynthesized(MemorySynthesizedPayload {
                    memory_id,
                    content: synthesis,
                    source_memory_ids,
                    level: target_level,
                    project_id,
                }))?
            };

            debug_assert_eq!(
                event.event_type,
                EventKind::from(&event.payload),
                "event_type must derive from payload"
            );

            self.event_store.append_event(&event)?;
            count += 1;
        }

        Ok(count)
    }

    async fn cluster_memories(
        &self,
        memories: &[(MemoryId, String)],
    ) -> Result<Vec<Vec<(MemoryId, String)>>, Box<dyn std::error::Error>> {
        // Heuristic: Group by 5 for now.
        let mut clusters = Vec::new();
        for chunk in memories.chunks(5) {
            clusters.push(chunk.to_vec());
        }
        Ok(clusters)
    }

    async fn synthesize_cluster(
        &self,
        cluster: &[(MemoryId, String)],
        level: u32,
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        let mut contents = String::new();
        for (_, content) in cluster {
            contents.push_str("--- SOURCE MEMORY ---\n");
            contents.push_str(content);
            contents.push('\n');
        }

        let prompt = format!(
            "Synthesize the following Level {} memories into a single Level {} Knowledge Node in JSON format.\n\n\
             Rules:\n\
             1. Aggregate recurring patterns and permanent constraints.\n\
             2. Maintain technical density.\n\
             3. Output ONLY valid JSON.\n\n\
             JSON Schema:\n\
             {{\n\
               \"title\": \"Synthesis Title\",\n\
               \"aggregated_context\": \"Combined summary of work\",\n\
               \"invariants\": [\"Shared technical invariants identified across sessions\"],\n\
               \"cumulative_progress\": [\"Overall progress made across these nodes\"]\n\
             }}\n\n\
             Source Memories:\n{}",
            level - 1,
            level,
            contents
        );

        let request = CompletionRequest {
            prompt,
            system_prompt: Some(
                "You are a factual synthesis engine for a hierarchical knowledge vault. You output ONLY valid JSON.".to_string(),
            ),
            max_tokens: Some(1000),
            temperature: Some(0.0),
        };

        let response = self.model_provider.complete(request).await?;
        Ok((response.text, response.model))
    }

    async fn verify_synthesis(
        &self,
        cluster: &[(MemoryId, String)],
        synthesis: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let mut sources = String::new();
        for (_, content) in cluster {
            sources.push_str("--- SOURCE ---\n");
            sources.push_str(content);
            sources.push('\n');
        }

        let prompt = format!(
            "Perform a rigorous factual audit of the following JSON synthesis against its sources.\n\n\
             Check for:\n\
             1. Factual contradictions.\n\
             2. Hallucinations (e.g. paths, features, or events NOT in the sources).\n\
             3. Over-reaching claims.\n\n\
             If the JSON is factually grounded in the sources, respond with 'SUPPORTED'.\n\
             If it contains any unsupported claims, respond with 'UNSUPPORTED' and list the errors.\n\n\
             Source Data:\n{}\n\nSynthesis JSON:\n{}",
            sources, synthesis
        );

        let request = CompletionRequest {
            prompt,
            system_prompt: Some(
                "You are a strict technical auditor. You verify facts and reject hallucinations."
                    .to_string(),
            ),
            max_tokens: Some(200),
            temperature: Some(0.0),
        };

        let response = self.model_provider.complete(request).await?;
        let text = response.text.to_uppercase();

        if text.contains("UNSUPPORTED") {
            tracing::warn!("CRAG REJECTED: {}", response.text);
            return Ok(false);
        }

        Ok(text.contains("SUPPORTED"))
    }
}
