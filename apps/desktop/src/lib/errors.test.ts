import { describe, expect, it } from "vitest";
import { isUiErrorKind, mapInvokeError } from "./errors";

describe("mapInvokeError", () => {
  it("mapInvokeError__json_offline_payload__kind_offline", () => {
    const raw = JSON.stringify({
      kind: "offline",
      message: "daemon unreachable (connection failed)",
    });
    const mapped = mapInvokeError(raw);
    expect(mapped.kind).toBe("offline");
    expect(mapped.message).toBe("daemon unreachable (connection failed)");
  });

  it("mapInvokeError__error_with_json_message__kind_denied", () => {
    const raw = new Error(
      JSON.stringify({
        kind: "denied",
        message: "session token missing",
        status: 401,
      }),
    );
    const mapped = mapInvokeError(raw);
    expect(mapped.kind).toBe("denied");
    expect(mapped.message).toBe("session token missing");
    expect(mapped.status).toBe(401);
  });

  it("mapInvokeError__object_transient__kind_transient_with_status", () => {
    const mapped = mapInvokeError({
      kind: "transient",
      message: "request timed out",
      status: 408,
    });
    expect(mapped.kind).toBe("transient");
    expect(mapped.message).toBe("request timed out");
    expect(mapped.status).toBe(408);
  });

  it("mapInvokeError__object_error__kind_error", () => {
    const mapped = mapInvokeError({
      kind: "error",
      message: "HTTP 400: bad request",
      status: 400,
    });
    expect(mapped.kind).toBe("error");
    expect(mapped.status).toBe(400);
  });

  it("mapInvokeError__object_unavailable__kind_unavailable", () => {
    const mapped = mapInvokeError({
      kind: "unavailable",
      message: "surface not wired",
    });
    expect(mapped.kind).toBe("unavailable");
    expect(mapped.message).toBe("surface not wired");
  });

  it("mapInvokeError__plain_string__kind_error_message_preserved", () => {
    const mapped = mapInvokeError("something went wrong");
    expect(mapped.kind).toBe("error");
    expect(mapped.message).toBe("something went wrong");
  });

  it("mapInvokeError__unknown_kind_in_json__normalized_to_error", () => {
    const mapped = mapInvokeError(
      JSON.stringify({ kind: "weird", message: "mystery" }),
    );
    expect(mapped.kind).toBe("error");
    expect(mapped.message).toBe("mystery");
  });

  it("mapInvokeError__incomplete_object__falls_back_to_string", () => {
    const mapped = mapInvokeError({ foo: 1 });
    expect(mapped.kind).toBe("error");
    // String(object) without kind/message → "[object Object]" (honest fallback).
    expect(mapped.message).toBe("[object Object]");
  });

  it("mapInvokeError__status_string_coerced_to_number", () => {
    const mapped = mapInvokeError({
      kind: "denied",
      message: "nope",
      status: "403",
    });
    expect(mapped.status).toBe(403);
  });
});

describe("isUiErrorKind", () => {
  it("isUiErrorKind__known_kinds__true", () => {
    for (const k of [
      "offline",
      "denied",
      "transient",
      "error",
      "unavailable",
    ] as const) {
      expect(isUiErrorKind(k)).toBe(true);
    }
  });

  it("isUiErrorKind__unknown__false", () => {
    expect(isUiErrorKind("nope")).toBe(false);
  });
});
