import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { createLlmStore } from "../llm.svelte";

const mockInvoke = vi.mocked(invoke);

function initCalls() {
  return mockInvoke.mock.calls.filter(([cmd]) => cmd === "init_llm_model").length;
}

describe("createLlmStore.ensureLoaded", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("loads the model via init_llm_model", async () => {
    mockInvoke.mockResolvedValue("ready");
    const store = createLlmStore();
    await store.ensureLoaded();
    expect(initCalls()).toBe(1);
    expect(store.status).toBe("ready");
  });

  it("attempts at most once per session, even when the load does not succeed", async () => {
    // Backend answers something other than "ready" (e.g. stubbed/null):
    // status falls back to not_installed with no error — the one case that
    // would loop forever if callers retried reactively.
    mockInvoke.mockResolvedValue(null);
    const store = createLlmStore();
    await store.ensureLoaded();
    expect(store.status).toBe("not_installed");
    await store.ensureLoaded();
    await store.ensureLoaded();
    expect(initCalls()).toBe(1);
  });

  it("does not attempt again after a failed load", async () => {
    mockInvoke.mockRejectedValue("boom");
    const store = createLlmStore();
    await store.ensureLoaded();
    expect(store.status).toBe("not_installed");
    expect(store.error).toContain("boom");
    await store.ensureLoaded();
    expect(initCalls()).toBe(1);
  });

  it("is a no-op when the model is already loaded", async () => {
    mockInvoke.mockResolvedValue("ready");
    const store = createLlmStore();
    await store.ensureLoaded();
    mockInvoke.mockClear();
    await store.ensureLoaded();
    expect(initCalls()).toBe(0);
  });
});
