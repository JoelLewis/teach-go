// Regression test for the "dead ghost board" stall: after Home → New Game →
// Start, the transition overlay must clear and the real play UI (board +
// side panel) must render and stay interactive.
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, cleanup } from "@testing-library/svelte";
import { tick } from "svelte";
import { invoke, type InvokeArgs } from "@tauri-apps/api/core";
import App from "../App.svelte";
import { gameStore } from "../lib/stores/game.svelte";
import { navHistory } from "../lib/stores/navHistory.svelte";
import type { GameState, Settings } from "../lib/api/bindings";

const mockInvoke = vi.mocked(invoke);

const settings: Settings = {
  board_size: 9,
  komi: 5.5,
  show_coordinates: false,
  show_move_numbers: false,
  ai_strength: "20k",
  sound_enabled: false,
  feedback_timing: "immediate",
  theme: "study",
  onboarding_completed: true,
  experience_level: "beginner",
};

function emptyGame(boardSize = 9): GameState {
  return {
    board_size: boardSize,
    stones: [],
    current_color: "black",
    move_number: 0,
    captures_black: 0,
    captures_white: 0,
    phase: "Playing",
    result: null,
    last_move: null,
    moves: [],
  };
}

function stubBackend() {
  mockInvoke.mockImplementation((cmd: string, args?: InvokeArgs): Promise<unknown> => {
    switch (cmd) {
      case "get_settings":
        return Promise.resolve(settings);
      case "update_settings":
        return Promise.resolve((args as { settings: Settings } | undefined)?.settings);
      case "get_download_status":
        return Promise.resolve({ katago: { state: "ready" }, llm: { state: "ready" } });
      case "list_games":
        return Promise.resolve([]);
      case "get_skill_profile":
        return Promise.reject("no games yet"); // HomeView catches
      case "new_game":
        return Promise.resolve(emptyGame());
      case "get_ai_engine":
        return Promise.resolve("katago");
      case "play_move":
        return Promise.resolve({
          ...emptyGame(),
          stones: [{ row: 4, col: 4, color: "black" }],
          current_color: "white",
          move_number: 1,
          last_move: [4, 4],
        });
      case "request_ai_move":
        return Promise.resolve({
          ...emptyGame(),
          stones: [
            { row: 4, col: 4, color: "black" },
            { row: 2, col: 2, color: "white" },
          ],
          current_color: "black",
          move_number: 2,
          last_move: [2, 2],
        });
      default:
        return Promise.resolve(null);
    }
  });
}

async function settle(ms: number) {
  await new Promise((resolve) => setTimeout(resolve, ms));
  await tick();
}

describe("App start-game flow", () => {
  beforeEach(() => {
    stubBackend();
    gameStore.clear();
    navHistory.clear();
  });

  afterEach(() => {
    cleanup();
    mockInvoke.mockReset();
  });

  it("renders the play UI after New Game → Start (overlay clears, board is interactive)", async () => {
    render(App);
    await settle(20);

    // Home is showing
    const newGameButton = screen.getByRole("button", { name: "New Game" });
    newGameButton.click();
    await tick();

    const startButton = screen.getByRole("button", { name: "Start" });
    startButton.click();

    // startGame waits 400ms behind the ghost-board overlay, then clears it
    // 50ms after navigating. Give it comfortably more than 450ms.
    await settle(600);

    // The real board must be mounted...
    const board = screen.getByTestId("go-board");
    expect(board).toBeTruthy();

    // ...the transition overlay must be gone (it has no test id; it is the
    // only fixed inset-0 element)...
    expect(document.querySelector(".fixed.inset-0")).toBeNull();

    // ...and the side panel must show the game controls.
    expect(screen.getByRole("button", { name: "Pass" })).toBeTruthy();

    // The board accepts a move: clicking an intersection reaches play_move.
    const intersection = document.querySelector(
      '[data-testid="intersection"][data-row="4"][data-col="4"]',
    ) as SVGRectElement;
    expect(intersection).toBeTruthy();
    intersection.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    await settle(20);

    expect(mockInvoke).toHaveBeenCalledWith("play_move", { row: 4, col: 4 });
    expect(document.querySelector('[data-testid="stone"]')).toBeTruthy();
  });
});
