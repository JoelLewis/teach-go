import { describe, it, expect } from "vitest";
import { createNavHistory, type NavEntry } from "../navHistory.svelte";

function entry(view: NavEntry["view"], reviewGameId?: number): NavEntry {
  return { view, reviewGameId };
}

describe("createNavHistory", () => {
  it("starts with empty back and forward stacks", () => {
    const nav = createNavHistory();
    expect(nav.canGoBack).toBe(false);
    expect(nav.canGoForward).toBe(false);
    expect(nav.goBack(entry("home"))).toBeNull();
    expect(nav.goForward(entry("home"))).toBeNull();
  });

  it("push records the view being left", () => {
    const nav = createNavHistory();
    nav.push(entry("home"));
    expect(nav.canGoBack).toBe(true);
    expect(nav.canGoForward).toBe(false);
  });

  it("goBack returns the last pushed entry and moves current to forward", () => {
    const nav = createNavHistory();
    nav.push(entry("home"));
    const back = nav.goBack(entry("dashboard"));
    expect(back).toEqual(entry("home"));
    expect(nav.canGoBack).toBe(false);
    expect(nav.canGoForward).toBe(true);
  });

  it("goForward reverses goBack", () => {
    const nav = createNavHistory();
    nav.push(entry("home"));
    nav.goBack(entry("review", 7));
    const fwd = nav.goForward(entry("home"));
    expect(fwd).toEqual(entry("review", 7));
    expect(nav.canGoBack).toBe(true);
    expect(nav.canGoForward).toBe(false);
  });

  it("walks back and forward through multiple entries in order", () => {
    const nav = createNavHistory();
    nav.push(entry("home"));
    nav.push(entry("dashboard"));
    nav.push(entry("review", 3));
    // current view is "problem"
    expect(nav.goBack(entry("problem"))).toEqual(entry("review", 3));
    expect(nav.goBack(entry("review", 3))).toEqual(entry("dashboard"));
    expect(nav.goForward(entry("dashboard"))).toEqual(entry("review", 3));
    expect(nav.goBack(entry("review", 3))).toEqual(entry("dashboard"));
    expect(nav.goBack(entry("dashboard"))).toEqual(entry("home"));
    expect(nav.goBack(entry("home"))).toBeNull();
  });

  it("push clears the forward stack", () => {
    const nav = createNavHistory();
    nav.push(entry("home"));
    nav.goBack(entry("dashboard"));
    expect(nav.canGoForward).toBe(true);
    nav.push(entry("home"));
    expect(nav.canGoForward).toBe(false);
  });

  it("caps the back stack, dropping the oldest entries", () => {
    const nav = createNavHistory(3);
    nav.push(entry("home"));
    nav.push(entry("dashboard"));
    nav.push(entry("problem"));
    nav.push(entry("review", 1));
    expect(nav.goBack(entry("play"))).toEqual(entry("review", 1));
    expect(nav.goBack(entry("review", 1))).toEqual(entry("problem"));
    expect(nav.goBack(entry("problem"))).toEqual(entry("dashboard"));
    // "home" was dropped by the cap
    expect(nav.goBack(entry("dashboard"))).toBeNull();
  });

  it("caps at 50 by default", () => {
    const nav = createNavHistory();
    for (let i = 0; i < 60; i++) {
      nav.push(entry("review", i));
    }
    let steps = 0;
    while (nav.goBack(entry("home")) !== null) {
      steps++;
    }
    expect(steps).toBe(50);
  });

  it("clear empties both stacks", () => {
    const nav = createNavHistory();
    nav.push(entry("home"));
    nav.push(entry("dashboard"));
    nav.goBack(entry("problem"));
    nav.clear();
    expect(nav.canGoBack).toBe(false);
    expect(nav.canGoForward).toBe(false);
  });
});
