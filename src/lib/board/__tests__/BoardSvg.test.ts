// Regression test: BoardSvg's stone-animation $effect used to read AND
// reassign a $state Set (previousStoneKeys), making it self-invalidating.
// Svelte then threw effect_update_depth_exceeded on mount whenever
// animate=true, killing all reactivity in the app (the "stuck ghost
// board" bug). The tracking set must stay non-reactive.
import { describe, it, expect } from "vitest";
import { render } from "@testing-library/svelte";
import { tick } from "svelte";
import BoardSvg from "../BoardSvg.svelte";
import type { StonePosition } from "../../api/bindings";

const baseProps = {
  boardSize: 9,
  stones: [] as StonePosition[],
  currentColor: "black" as const,
  lastMove: null as [number, number] | null,
  animate: true,
  interactive: true,
  onIntersectionClick: () => {},
};

async function settle(ms: number) {
  await new Promise((resolve) => setTimeout(resolve, ms));
  await tick();
}

describe("BoardSvg", () => {
  it("mounts with animate=true without killing reactivity", async () => {
    render(BoardSvg, { props: baseProps });
    await settle(50);
    expect(document.querySelector('[data-testid="go-board"]')).toBeTruthy();
  });

  it("still animates newly placed stones after stones change", async () => {
    const { rerender } = render(BoardSvg, { props: baseProps });
    await settle(20);

    await rerender({
      stones: [{ row: 4, col: 4, color: "black" }],
      lastMove: [4, 4] as [number, number],
    });
    await tick();

    const stone = document.querySelector('[data-testid="stone"]');
    expect(stone).toBeTruthy();
    expect(stone?.classList.contains("stone-entering")).toBe(true);

    // Reactivity must survive: a further update still renders.
    await rerender({
      stones: [
        { row: 4, col: 4, color: "black" },
        { row: 2, col: 2, color: "white" },
      ],
      lastMove: [2, 2] as [number, number],
    });
    await tick();
    expect(document.querySelectorAll('[data-testid="stone"]').length).toBe(2);
  });
});
