/**
 * Invoke wrappers only — no domain logic, policy, or authority.
 * Primary transport is Tauri invoke; never add webview fetch to T161.
 */
import { invoke } from "@tauri-apps/api/core";
import type {
  ContentEnvelopeWipedResponse,
  DaemonConnectionInfo,
  ErasureAcceptedResponse,
  HandlePreview,
  PersonalBriefingResponse,
  PingResponse,
  ProgressiveQueryResponse,
  ProjectBriefingResponse,
  ReviewQueueResponse,
  ReviewResolvedResponse,
  ScopeResolvedResponse,
  SourceDto,
} from "./types";

export type {
  DaemonConnectionInfo,
  PingResponse,
} from "./types";

export async function ping(): Promise<PingResponse> {
  return invoke<PingResponse>("ping");
}

export async function getDaemonConnectionInfo(): Promise<DaemonConnectionInfo> {
  return invoke<DaemonConnectionInfo>("get_daemon_connection_info");
}

export async function probeHealth(): Promise<{ status?: string }> {
  return invoke("probe_health");
}

export async function projectBriefing(args: {
  scope?: string;
  cwd?: string;
  principal_id?: string;
  max_words?: number;
  governed_briefing?: boolean;
}): Promise<ProjectBriefingResponse> {
  return invoke("project_briefing", {
    args: { api_version: "1", ...args },
  });
}

export async function personalBriefing(args: {
  scope?: string;
  principal_id?: string;
  max_words?: number;
  governed_briefing?: boolean;
}): Promise<PersonalBriefingResponse> {
  return invoke("personal_briefing", {
    args: { api_version: "1", ...args },
  });
}

export async function queryKnowledge(args: {
  query: string;
  scope?: string;
  principal_id?: string;
  limit?: number;
}): Promise<ProgressiveQueryResponse> {
  return invoke("query_knowledge", {
    args: { api_version: "1", ...args },
  });
}

export async function inspectEvidence(args: {
  id: string;
  scope?: string;
  principal_id?: string;
  max_chars?: number;
}): Promise<HandlePreview> {
  return invoke("inspect_evidence", {
    args: { api_version: "1", ...args },
  });
}

export async function inspectSource(args: {
  id: string;
  scope?: string;
  principal_id?: string;
}): Promise<SourceDto | Record<string, unknown>> {
  return invoke("inspect_source", {
    args: { api_version: "1", ...args },
  });
}

export async function listReviewItems(args?: {
  principal_id?: string;
  scope?: string;
  status?: string;
}): Promise<ReviewQueueResponse> {
  return invoke("list_review_items", {
    args: { api_version: "1", ...args },
  });
}

export async function resolveReviewItem(args: {
  id: string;
  resolution: string;
  principal_id?: string;
  note?: string;
  scope?: string;
  command_id?: string;
}): Promise<ReviewResolvedResponse> {
  return invoke("resolve_review_item", {
    args: { api_version: "1", ...args },
  });
}

export async function resolveScope(args?: {
  cwd?: string;
  signals?: Record<string, string>;
  explicit_project_id?: string;
  force_personal?: boolean;
  personal_user_id?: string;
}): Promise<ScopeResolvedResponse> {
  return invoke("resolve_scope", {
    args: {
      api_version: "1",
      force_personal: false,
      ...args,
    },
  });
}

export async function requestErasure(args: {
  ids: string[];
  reason?: string;
  scope?: string;
  principal_id?: string;
  command_id?: string;
}): Promise<ErasureAcceptedResponse> {
  return invoke("request_erasure", {
    args: { api_version: "1", ...args },
  });
}

export async function wipeContentEnvelope(args: {
  content_key_id: string;
  scope: string;
  reason?: string;
  principal_id?: string;
  command_id?: string;
  dry_run?: boolean;
  confirm?: boolean;
}): Promise<ContentEnvelopeWipedResponse> {
  return invoke("wipe_content_envelope", {
    args: {
      api_version: "1",
      dry_run: true,
      confirm: false,
      ...args,
    },
  });
}
