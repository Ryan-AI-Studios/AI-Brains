/**
 * Invoke wrappers only — no domain logic, policy, or authority.
 * Primary transport is Tauri invoke; do not add webview fetch to T161 by default.
 */
import { invoke } from "@tauri-apps/api/core";

/** Static smoke response from the host `ping` command. */
export interface PingResponse {
  ok: boolean;
  service: string;
  version: string;
}

/**
 * Daemon connection metadata for the UI (E1 empty-state honest).
 * Never includes bearer token material.
 */
export interface DaemonConnectionInfo {
  /** Loopback base URL when a port can be resolved; otherwise null. */
  loopback_base_url: string | null;
  /** Whether `%USERPROFILE%\.ai-brains\http.token` exists (presence only). */
  token_file_present: boolean;
}

export async function ping(): Promise<PingResponse> {
  return invoke<PingResponse>("ping");
}

export async function getDaemonConnectionInfo(): Promise<DaemonConnectionInfo> {
  return invoke<DaemonConnectionInfo>("get_daemon_connection_info");
}
