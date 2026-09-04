use crate::cozo_proxy::{GraphBackend, GraphEdge, GraphNode};
use crate::errors::Result;
use ai_brains_events::{Envelope, Payload};

pub struct GraphProjector<'a> {
    backend: Box<dyn GraphBackend + Send + Sync + 'a>,
    node_buffer: Vec<GraphNode>,
    edge_buffer: Vec<GraphEdge>,
}

impl<'a> GraphProjector<'a> {
    pub fn new(backend: Box<dyn GraphBackend + Send + Sync + 'a>) -> Self {
        Self {
            backend,
            node_buffer: Vec::new(),
            edge_buffer: Vec::new(),
        }
    }

    pub fn flush(&mut self) -> Result<()> {
        if !self.node_buffer.is_empty() {
            self.backend.add_nodes(&self.node_buffer)?;
            self.node_buffer.clear();
        }
        if !self.edge_buffer.is_empty() {
            self.backend.add_edges(&self.edge_buffer)?;
            self.edge_buffer.clear();
        }
        Ok(())
    }

    #[allow(clippy::disallowed_methods)]
    pub fn apply(&mut self, envelope: &Envelope) -> Result<()> {
        match &envelope.payload {
            Payload::ProjectRegistered(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.project_id.to_string(),
                    label: p.name.clone(),
                    category: "project".to_string(),
                    metadata: serde_json::json!({}),
                });
            }
            Payload::SessionStarted(s) => {
                self.node_buffer.push(GraphNode {
                    id: s.session_id.to_string(),
                    label: "Session".to_string(),
                    category: "session".to_string(),
                    metadata: serde_json::json!({}),
                });
                self.edge_buffer.push(GraphEdge {
                    source: s.session_id.to_string(),
                    target: s.project_id.to_string(),
                    relation: "IN_PROJECT".to_string(),
                    confidence: 1.0,
                });
            }
            Payload::SessionReassigned(s) => {
                let session = s.session_id.to_string();
                let from = s.from_project_id.to_string();
                let to = s.to_project_id.to_string();
                self.edge_buffer.retain(|e| {
                    !(e.source == session && e.target == from && e.relation == "IN_PROJECT")
                });
                self.backend.remove_edge(&session, &from, "IN_PROJECT")?;
                self.node_buffer.push(GraphNode {
                    id: to.clone(),
                    label: "Project".to_string(),
                    category: "project".to_string(),
                    metadata: serde_json::json!({}),
                });
                self.edge_buffer.push(GraphEdge {
                    source: session,
                    target: to,
                    relation: "IN_PROJECT".to_string(),
                    confidence: 1.0,
                });
            }
            Payload::UserPromptRecorded(p) => {
                self.project_capture_turn(p.turn_id.as_ref(), &p.session_id, envelope);
            }
            Payload::AssistantFinalRecorded(p) => {
                self.project_capture_turn(p.turn_id.as_ref(), &p.session_id, envelope);
            }
            Payload::MemoryPinned(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.memory_id.to_string(),
                    label: "Memory".to_string(),
                    category: "memory".to_string(),
                    metadata: serde_json::json!({"status": "pinned"}),
                });
                // Add a direct RECALLS edge: session -> memory
                if let Some(session_id) = &p.session_id {
                    self.node_buffer.push(GraphNode {
                        id: session_id.to_string(),
                        label: "Session".to_string(),
                        category: "session".to_string(),
                        metadata: serde_json::json!({}),
                    });
                    self.edge_buffer.push(GraphEdge {
                        source: session_id.to_string(),
                        target: p.memory_id.to_string(),
                        relation: "RECALLS".to_string(),
                        confidence: 1.0,
                    });
                }
                if p.session_id.is_none()
                    && let Some(project_id) = &p.project_id
                {
                    self.node_buffer.push(GraphNode {
                        id: project_id.to_string(),
                        label: "Project".to_string(),
                        category: "project".to_string(),
                        metadata: serde_json::json!({}),
                    });
                    self.edge_buffer.push(GraphEdge {
                        source: p.memory_id.to_string(),
                        target: project_id.to_string(),
                        relation: "PINNED_IN_PROJECT".to_string(),
                        confidence: 1.0,
                    });
                }
            }
            Payload::SessionSummaryCreated(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.memory_id.to_string(),
                    label: "Summary".to_string(),
                    category: "memory".to_string(),
                    metadata: serde_json::json!({"type": "summary"}),
                });
            }
            Payload::MemorySynthesized(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.memory_id.to_string(),
                    label: "Synthesized".to_string(),
                    category: "memory".to_string(),
                    metadata: serde_json::json!({"level": p.level}),
                });

                for source_id in &p.source_memory_ids {
                    let source_kind = if p.level == 0 { "turn" } else { "memory" };
                    self.node_buffer.push(GraphNode {
                        id: source_id.to_string(),
                        label: "Source".to_string(),
                        category: source_kind.to_string(),
                        metadata: serde_json::json!({}),
                    });
                    self.edge_buffer.push(GraphEdge {
                        source: p.memory_id.to_string(),
                        target: source_id.to_string(),
                        relation: "SYNTHESIZED_FROM".to_string(),
                        confidence: 1.0,
                    });
                }
            }
            Payload::ConflictDetected(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.conflict_id.to_string(),
                    label: "Conflict".to_string(),
                    category: "conflict".to_string(),
                    metadata: serde_json::json!({}),
                });
                for memory_id in &p.memory_ids {
                    self.edge_buffer.push(GraphEdge {
                        source: p.conflict_id.to_string(),
                        target: memory_id.to_string(),
                        relation: "CONFLICTS_WITH".to_string(),
                        confidence: 1.0,
                    });
                }
            }
            Payload::RecipePromoted(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.recipe_id.to_string(),
                    label: "Recipe".to_string(),
                    category: "recipe".to_string(),
                    metadata: serde_json::json!({}),
                });
                for memory_id in &p.source_memory_ids {
                    self.edge_buffer.push(GraphEdge {
                        source: memory_id.to_string(),
                        target: p.recipe_id.to_string(),
                        relation: "PART_OF_RECIPE".to_string(),
                        confidence: 1.0,
                    });
                }
            }
            // --- Governed provenance (T149 Phase G) ---
            Payload::SourceRegistered(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.source_id.to_string(),
                    label: p.display_name.clone(),
                    category: "source".to_string(),
                    metadata: serde_json::json!({}),
                });
            }
            Payload::SourceVersionRecorded(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.source_id.to_string(),
                    label: "Source".to_string(),
                    category: "source".to_string(),
                    metadata: serde_json::json!({}),
                });
                self.node_buffer.push(GraphNode {
                    id: p.version_id.to_string(),
                    label: "SourceVersion".to_string(),
                    category: "source_version".to_string(),
                    metadata: serde_json::json!({}),
                });
                self.edge_buffer.push(GraphEdge {
                    source: p.source_id.to_string(),
                    target: p.version_id.to_string(),
                    relation: "CONTAINS".to_string(),
                    confidence: 1.0,
                });
            }
            Payload::EvidenceRecorded(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.evidence_id.to_string(),
                    label: "Evidence".to_string(),
                    category: "evidence".to_string(),
                    metadata: serde_json::json!({}),
                });
                if let Some(version_id) = &p.source_version_id {
                    self.node_buffer.push(GraphNode {
                        id: version_id.to_string(),
                        label: "SourceVersion".to_string(),
                        category: "source_version".to_string(),
                        metadata: serde_json::json!({}),
                    });
                    self.edge_buffer.push(GraphEdge {
                        source: p.evidence_id.to_string(),
                        target: version_id.to_string(),
                        relation: "OBSERVED_FROM".to_string(),
                        confidence: 1.0,
                    });
                }
            }
            Payload::EvidenceSuperseded(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.evidence_id.to_string(),
                    label: "Evidence".to_string(),
                    category: "evidence".to_string(),
                    metadata: serde_json::json!({}),
                });
                self.node_buffer.push(GraphNode {
                    id: p.superseded_by.to_string(),
                    label: "Evidence".to_string(),
                    category: "evidence".to_string(),
                    metadata: serde_json::json!({}),
                });
                // successor SUPERSEDES predecessor
                self.edge_buffer.push(GraphEdge {
                    source: p.superseded_by.to_string(),
                    target: p.evidence_id.to_string(),
                    relation: "SUPERSEDES".to_string(),
                    confidence: 1.0,
                });
            }
            Payload::ConclusionProposed(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.conclusion_id.to_string(),
                    label: "Conclusion".to_string(),
                    category: "conclusion".to_string(),
                    metadata: serde_json::json!({}),
                });
                for evidence_id in &p.evidence_ids {
                    self.node_buffer.push(GraphNode {
                        id: evidence_id.to_string(),
                        label: "Evidence".to_string(),
                        category: "evidence".to_string(),
                        metadata: serde_json::json!({}),
                    });
                    self.edge_buffer.push(GraphEdge {
                        source: p.conclusion_id.to_string(),
                        target: evidence_id.to_string(),
                        relation: "DERIVED_FROM".to_string(),
                        confidence: 1.0,
                    });
                }
            }
            Payload::DecisionProposed(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.decision_id.to_string(),
                    label: p.title.clone(),
                    category: "decision".to_string(),
                    metadata: serde_json::json!({}),
                });
                if let Some(conclusion_ids) = &p.conclusion_ids {
                    for conclusion_id in conclusion_ids {
                        self.node_buffer.push(GraphNode {
                            id: conclusion_id.to_string(),
                            label: "Conclusion".to_string(),
                            category: "conclusion".to_string(),
                            metadata: serde_json::json!({}),
                        });
                        self.edge_buffer.push(GraphEdge {
                            source: p.decision_id.to_string(),
                            target: conclusion_id.to_string(),
                            relation: "SUPPORTED_BY".to_string(),
                            confidence: 1.0,
                        });
                    }
                }
            }
            Payload::WorkspaceRegistered(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.workspace_id.to_string(),
                    label: p.name.clone(),
                    category: "workspace".to_string(),
                    metadata: serde_json::json!({}),
                });
            }
            Payload::RepositoryJoinedWorkspace(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.workspace_id.to_string(),
                    label: "Workspace".to_string(),
                    category: "workspace".to_string(),
                    metadata: serde_json::json!({}),
                });
                self.node_buffer.push(GraphNode {
                    id: p.project_id.to_string(),
                    label: "Project".to_string(),
                    category: "project".to_string(),
                    metadata: serde_json::json!({}),
                });
                self.edge_buffer.push(GraphEdge {
                    source: p.workspace_id.to_string(),
                    target: p.project_id.to_string(),
                    relation: "CONTAINS".to_string(),
                    confidence: 1.0,
                });
            }
            _ => {}
        }

        // Auto-flush if buffer gets too large
        if self.node_buffer.len() >= 100 || self.edge_buffer.len() >= 100 {
            self.flush()?;
        }

        Ok(())
    }

    /// T262: capture turns with a logged `turn_id` become memory nodes (F9).
    /// Legacy events without the field keep a rebuild-stable turn node at `event_id` (F10).
    fn project_capture_turn(
        &mut self,
        turn_id: Option<&ai_brains_core::ids::TurnId>,
        session_id: &ai_brains_core::ids::SessionId,
        envelope: &Envelope,
    ) {
        if let Some(tid) = turn_id {
            let memory_id = tid.to_string();
            self.node_buffer.push(GraphNode {
                id: memory_id.clone(),
                label: "Memory".to_string(),
                category: "memory".to_string(),
                metadata: serde_json::json!({}),
            });
            self.node_buffer.push(GraphNode {
                id: session_id.to_string(),
                label: "Session".to_string(),
                category: "session".to_string(),
                metadata: serde_json::json!({}),
            });
            self.edge_buffer.push(GraphEdge {
                source: session_id.to_string(),
                target: memory_id,
                relation: "RECALLS".to_string(),
                confidence: 1.0,
            });
        } else {
            let turn_node_id = envelope.event_id.to_string();
            self.node_buffer.push(GraphNode {
                id: turn_node_id.clone(),
                label: "Turn".to_string(),
                category: "turn".to_string(),
                metadata: serde_json::json!({}),
            });
            self.edge_buffer.push(GraphEdge {
                source: turn_node_id,
                target: session_id.to_string(),
                relation: "IN_SESSION".to_string(),
                confidence: 1.0,
            });
        }
    }
}
