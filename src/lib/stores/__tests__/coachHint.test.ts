import { describe, it, expect } from "vitest";
import { createCoachHint } from "../coachHint.svelte";

describe("createCoachHint", () => {
  it("is hidden initially", () => {
    const hint = createCoachHint();
    expect(hint.visible).toBe(false);
  });

  it("shows when a coaching message lands while the model is downloading", () => {
    const hint = createCoachHint();
    hint.noteCoachingMessage(true);
    expect(hint.visible).toBe(true);
  });

  it("does not show when the model is not downloading", () => {
    const hint = createCoachHint();
    hint.noteCoachingMessage(false);
    expect(hint.visible).toBe(false);
  });

  it("stays visible across further messages while downloading", () => {
    const hint = createCoachHint();
    hint.noteCoachingMessage(true);
    hint.noteCoachingMessage(true);
    expect(hint.visible).toBe(true);
  });

  it("clears when the model becomes ready", () => {
    const hint = createCoachHint();
    hint.noteCoachingMessage(true);
    hint.noteModelState("ready");
    expect(hint.visible).toBe(false);
  });

  it("clears when the download errors", () => {
    const hint = createCoachHint();
    hint.noteCoachingMessage(true);
    hint.noteModelState("error");
    expect(hint.visible).toBe(false);
  });

  it("stays visible while the download keeps running", () => {
    const hint = createCoachHint();
    hint.noteCoachingMessage(true);
    hint.noteModelState("downloading");
    expect(hint.visible).toBe(true);
  });

  it("shows at most once per session — never re-shows after being cleared", () => {
    const hint = createCoachHint();
    hint.noteCoachingMessage(true);
    hint.noteModelState("ready");
    // A later download (e.g. retry) with more template messages must not nag again
    hint.noteCoachingMessage(true);
    expect(hint.visible).toBe(false);
  });

  it("model state changes alone never show the hint", () => {
    const hint = createCoachHint();
    hint.noteModelState("downloading");
    expect(hint.visible).toBe(false);
  });
});
