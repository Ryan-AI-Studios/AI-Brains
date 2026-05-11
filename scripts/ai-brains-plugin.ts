import type { Plugin } from "@opencode-ai/plugin";

export const AiBrainsPlugin: Plugin = async ({ directory, worktree, client, $ }) => {
  console.log("[ai-brains-opencode] Plugin initialized");

  const runHook = async (eventType: string, payload: any) => {
    try {
      const fullPayload = {
        event_type: eventType,
        directory,
        worktree,
        ...payload
      };

      const result = await $`powershell -NoProfile -Command "[Console]::In.ReadToEnd() | powershell -NoProfile -Command \"& 'C:\\Users\\RyanB\\.ai-brains\\scripts\\target-opencode-hook.ps1'\""`
        .stdin(JSON.stringify(fullPayload))
        .quiet();

      if (result.exitCode === 0) {
        try {
          return JSON.parse(result.stdout.toString());
        } catch (e) {
          return { success: true };
        }
      }
    } catch (error) {
      console.error(`[ai-brains-opencode] Hook error (${eventType}):`, error);
    }
    return { success: true };
  };

  return {
    event: async ({ event }) => {
      // Handle session events
      if (event.type === "session.created" || event.type === "session.idle") {
        const response = await runHook(event.type, { sessionId: event.properties?.sessionId });
        if (response?.additionalContext) {
          // OpenCode context injection during session start
          // Note: OpenCode might need a specific way to inject context post-init
          // For now, we log it.
          console.log("[ai-brains-opencode] Context received from preflight");
        }
      }

      // Handle message events
      if (event.type === "message.updated" || event.type === "message.created") {
        await runHook("message.updated", { message: event.properties?.message, sessionId: event.properties?.sessionId });
      }
    },
    "experimental.session.compacting": async (input, output) => {
      const response = await runHook("experimental.session.compacting", { input });
      if (response?.additionalContext) {
        output.context.push(response.additionalContext);
      }
    }
  };
};
