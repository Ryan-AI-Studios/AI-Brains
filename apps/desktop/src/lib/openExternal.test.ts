import { afterEach, describe, expect, it } from "vitest";
import {
  classifyLocator,
  openUrl,
  revealPath,
} from "./openExternal";
import {
  cleanupTauriMocks,
  setupTauriMocks,
} from "../test/setupTauriMocks";

describe("classifyLocator", () => {
  it("classifyLocator__https_url__kind_https", () => {
    const r = classifyLocator("https://example.com/docs");
    expect(r).toEqual({ kind: "https", value: "https://example.com/docs" });
  });

  it("classifyLocator__https_case_insensitive__kind_https", () => {
    const r = classifyLocator("HTTPS://Example.COM/x");
    expect(r.kind).toBe("https");
  });

  it("classifyLocator__windows_path__kind_path", () => {
    const r = classifyLocator("C:\\Users\\me\\file.txt");
    expect(r).toEqual({ kind: "path", value: "C:\\Users\\me\\file.txt" });
  });

  it("classifyLocator__unix_path__kind_path", () => {
    const r = classifyLocator("/home/me/notes.md");
    expect(r.kind).toBe("path");
  });

  it("classifyLocator__null__kind_none_no_fabricated_open", () => {
    expect(classifyLocator(null)).toEqual({ kind: "none" });
  });

  it("classifyLocator__undefined__kind_none", () => {
    expect(classifyLocator(undefined)).toEqual({ kind: "none" });
  });

  it("classifyLocator__empty_or_whitespace__kind_none", () => {
    expect(classifyLocator("")).toEqual({ kind: "none" });
    expect(classifyLocator("   ")).toEqual({ kind: "none" });
  });

  it("classifyLocator__plain_text_label__kind_text_not_open", () => {
    const r = classifyLocator("local-vault-source-label");
    expect(r).toEqual({
      kind: "text",
      value: "local-vault-source-label",
    });
  });

  it("classifyLocator__http_url__kind_text_not_path_not_https", () => {
    // Only https is openable via open_url. Other URI schemes are display-only
    // so Source UI never shows "Open URL" or "Reveal path" for http.
    const r = classifyLocator("http://example.com");
    expect(r).toEqual({ kind: "text", value: "http://example.com" });
    expect(r.kind).not.toBe("https");
    expect(r.kind).not.toBe("path");
  });

  it("classifyLocator__file_and_javascript_schemes__kind_text_not_open", () => {
    expect(classifyLocator("file:///C:/secret.txt")).toEqual({
      kind: "text",
      value: "file:///C:/secret.txt",
    });
    expect(classifyLocator("javascript:alert(1)")).toEqual({
      kind: "text",
      value: "javascript:alert(1)",
    });
  });
});

describe("openUrl / revealPath invoke wrappers", () => {
  afterEach(() => {
    cleanupTauriMocks();
  });

  it("openUrl__https__invokes_open_url_command", async () => {
    const calls: Array<{ cmd: string; args: unknown }> = [];
    setupTauriMocks((cmd, args) => {
      calls.push({ cmd, args });
      return null;
    });
    await openUrl("https://example.com");
    expect(calls).toHaveLength(1);
    expect(calls[0]?.cmd).toBe("open_url");
    expect(calls[0]?.args).toEqual({ url: "https://example.com" });
  });

  it("revealPath__windows_path__invokes_reveal_path_command", async () => {
    const calls: Array<{ cmd: string; args: unknown }> = [];
    setupTauriMocks((cmd, args) => {
      calls.push({ cmd, args });
      return null;
    });
    await revealPath("C:\\Users\\me\\notes.md");
    expect(calls).toHaveLength(1);
    expect(calls[0]?.cmd).toBe("reveal_path");
    expect(calls[0]?.args).toEqual({ path: "C:\\Users\\me\\notes.md" });
  });
});
