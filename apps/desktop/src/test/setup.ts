/**
 * Vitest global setup: jest-dom matchers + jsdom polyfills for crypto + dialog.
 * Idempotent — safe if setupFiles runs more than once.
 */
import { randomFillSync } from "node:crypto";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";
import "@testing-library/jest-dom/vitest";

// Unmount React trees between tests (jsdom document is shared).
afterEach(() => {
  cleanup();
});

// --- crypto.getRandomValues (needed by @tauri-apps/api mocks callback ids) ---
const g = globalThis as typeof globalThis & {
  crypto?: Crypto;
  window?: Window & typeof globalThis;
};

function ensureCrypto(): void {
  const target = g.window ?? g;
  const existing = target.crypto;
  if (existing && typeof existing.getRandomValues === "function") {
    return;
  }
  const polyfill = {
    getRandomValues<T extends ArrayBufferView>(array: T): T {
      randomFillSync(array as unknown as NodeJS.ArrayBufferView);
      return array;
    },
  };
  Object.defineProperty(target, "crypto", {
    value: polyfill,
    configurable: true,
    writable: true,
  });
}

ensureCrypto();

// --- HTMLDialogElement showModal / close (jsdom incomplete) ---
function ensureDialogPolyfill(): void {
  if (typeof HTMLDialogElement === "undefined") {
    return;
  }
  const proto = HTMLDialogElement.prototype;
  if (typeof proto.showModal !== "function") {
    proto.showModal = function showModal(this: HTMLDialogElement) {
      this.setAttribute("open", "");
      // jsdom may not expose open as a live property; force it when possible.
      try {
        Object.defineProperty(this, "open", {
          configurable: true,
          enumerable: true,
          get: () => this.hasAttribute("open"),
          set: (v: boolean) => {
            if (v) {
              this.setAttribute("open", "");
            } else {
              this.removeAttribute("open");
            }
          },
        });
      } catch {
        // already defined
      }
    };
  }
  if (typeof proto.close !== "function") {
    proto.close = function close(this: HTMLDialogElement) {
      this.removeAttribute("open");
      this.dispatchEvent(new Event("close"));
    };
  }
  // Ensure `open` property tracks the attribute when missing.
  const desc = Object.getOwnPropertyDescriptor(proto, "open");
  if (!desc || typeof desc.get !== "function") {
    try {
      Object.defineProperty(proto, "open", {
        configurable: true,
        enumerable: true,
        get(this: HTMLDialogElement) {
          return this.hasAttribute("open");
        },
        set(this: HTMLDialogElement, v: boolean) {
          if (v) {
            this.setAttribute("open", "");
          } else {
            this.removeAttribute("open");
          }
        },
      });
    } catch {
      // ignore if non-configurable
    }
  }
}

ensureDialogPolyfill();
