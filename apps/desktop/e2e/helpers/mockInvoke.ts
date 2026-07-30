import type { BrowserContext } from "@playwright/test";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const fixturesDir = join(dirname(fileURLToPath(import.meta.url)), "..", "fixtures");

export function loadFixture<T = unknown>(name: string): T {
  const raw = readFileSync(join(fixturesDir, name), "utf8");
  return JSON.parse(raw) as T;
}

export type MockResponse =
  | { ok: true; value: unknown }
  | { ok: false; error: unknown };

export type MockTable = Record<string, MockResponse | ((payload: unknown) => MockResponse)>;

/**
 * Default offline-first mock: briefings offline; connection info honest;
 * scope resolve optional; review empty; connectors not invoked.
 */
export function defaultMockTable(overrides: MockTable = {}): MockTable {
  const offline = loadFixture("offline-error.json");
  const reviewEmpty = loadFixture("review-empty.json");
  const wipeDry = loadFixture("wipe-dry-run.json");

  const base: MockTable = {
    ping: {
      ok: true,
      value: { ok: true, service: "ai-brains-desktop", version: "0.1.1" },
    },
    get_daemon_connection_info: {
      ok: true,
      value: {
        loopback_base_url: "http://127.0.0.1:7432",
        token_file_present: true,
      },
    },
    project_briefing: { ok: false, error: offline },
    personal_briefing: { ok: false, error: offline },
    list_review_items: { ok: true, value: reviewEmpty },
    resolve_review_item: {
      ok: true,
      value: loadFixture("resolve-success.json"),
    },
    resolve_scope: {
      ok: true,
      value: {
        api_version: "1",
        scope: "Repository:e2e-fixture",
        confidence: "High",
        authoritative: true,
        evidence: [],
        warnings: [],
        alternatives: [],
      },
    },
    inspect_source: {
      ok: true,
      value: loadFixture("source-missing.json"),
    },
    wipe_content_envelope: { ok: true, value: wipeDry },
    request_erasure: {
      ok: true,
      value: {
        api_version: "1",
        request_id: "erase-req-1",
        status: "accepted",
        warnings: ["ticket only — wipe was not performed"],
      },
    },
    probe_health: { ok: true, value: { status: "ok" } },
    open_url: { ok: true, value: null },
    reveal_path: { ok: true, value: null },
  };

  return { ...base, ...overrides };
}

/**
 * Source string installed via context.addInitScript BEFORE any page script.
 * Installs window.__TAURI_INTERNALS__ invoke mock matching Tauri v2 shape.
 */
export function buildInitScript(table: MockTable): string {
  // Serialize responses; functions cannot cross the boundary — only static tables.
  const serializable: Record<string, MockResponse> = {};
  for (const [cmd, entry] of Object.entries(table)) {
    if (typeof entry === "function") {
      throw new Error(
        `buildInitScript cannot serialize function handlers for cmd=${cmd}; use static overrides only (installMockInvoke + addInitScript)`,
      );
    }
    serializable[cmd] = entry;
  }

  return `(() => {
  const table = ${JSON.stringify(serializable)};
  window.__TAURI_INTERNALS__ = window.__TAURI_INTERNALS__ || {};
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = window.__TAURI_EVENT_PLUGIN_INTERNALS__ || {};
  const callbacks = new Map();
  window.__TAURI_INTERNALS__.transformCallback = (callback, once = false) => {
    const id = window.crypto.getRandomValues(new Uint32Array(1))[0];
    callbacks.set(id, (data) => {
      if (once) callbacks.delete(id);
      return callback && callback(data);
    });
    return id;
  };
  window.__TAURI_INTERNALS__.unregisterCallback = (id) => { callbacks.delete(id); };
  window.__TAURI_INTERNALS__.runCallback = (id, data) => {
    const cb = callbacks.get(id);
    if (cb) cb(data);
  };
  window.__TAURI_INTERNALS__.callbacks = callbacks;
  window.__TAURI_INTERNALS__.metadata = {
    currentWindow: { label: "main" },
    currentWebview: { windowLabel: "main", label: "main" },
  };
  window.__TAURI_INTERNALS__.invoke = async (cmd, args) => {
    const entry = table[cmd];
    if (!entry) {
      throw { kind: "error", message: "unmocked invoke: " + cmd };
    }
    if (entry.ok) {
      return entry.value;
    }
    throw entry.error;
  };
})();`;
}

declare global {
  interface Window {
    __TAURI_INTERNALS__?: Record<string, unknown>;
  }
}

/** Install mock via context.addInitScript (required before app load). */
export async function installMockInvoke(
  context: BrowserContext,
  table: MockTable = defaultMockTable(),
): Promise<void> {
  await context.addInitScript(buildInitScript(table));
}
