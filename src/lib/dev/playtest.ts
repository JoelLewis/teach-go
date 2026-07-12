// Dev-only playtest hooks, exposed as window.__playtest for MCP-driven
// testing (see docs/PLAYTESTING.md). This module is only ever loaded behind
// import.meta.env.DEV checks (main.ts, App.svelte), so Vite statically
// excludes it from production bundles.
import { gameStore } from "../stores/game.svelte";

// Matches BoardSvg's column labels: "I" is skipped, Go convention.
const COLUMN_LETTERS = "ABCDEFGHJKLMNOPQRST";

type ViewportPoint = { x: number; y: number };

type GameStateSummary = {
  boardSize: number;
  moveNumber: number;
  currentColor: string;
  phase: string;
  stoneCount: number;
  lastMove: [number, number] | null;
  capturesBlack: number;
  capturesWhite: number;
  result: unknown;
};

type PlaytestApi = {
  getView: () => string | null;
  getGameState: () => GameStateSummary | null;
  pointCenter: (point: string) => ViewportPoint;
  clickPoint: (point: string) => ViewportPoint;
};

let viewGetter: (() => string) | null = null;

/** Called from App.svelte (dev-gated) so getView can report the active view. */
export function registerViewGetter(fn: () => string): void {
  viewGetter = fn;
}

function getGameState(): GameStateSummary | null {
  const s = gameStore.state;
  if (!s) return null;
  return {
    boardSize: s.board_size,
    moveNumber: s.move_number,
    currentColor: s.current_color,
    phase: s.phase,
    stoneCount: s.stones.length,
    lastMove: s.last_move,
    capturesBlack: s.captures_black,
    capturesWhite: s.captures_white,
    result: s.result,
  };
}

/** Go coordinate ("E5", "Q16") to zero-based board row/col. */
function parsePoint(point: string, boardSize: number): { row: number; col: number } {
  const match = /^([A-HJ-T])(\d{1,2})$/.exec(point.trim().toUpperCase());
  if (!match) {
    throw new Error(`Invalid point "${point}" — expected a Go coordinate like "E5" or "Q16"`);
  }
  const col = COLUMN_LETTERS.indexOf(match[1]);
  const row = boardSize - Number(match[2]);
  if (col >= boardSize || row < 0 || row >= boardSize) {
    throw new Error(`Point "${point}" is outside the ${boardSize}x${boardSize} board`);
  }
  return { row, col };
}

function activeBoard(): Element {
  const boards = Array.from(document.querySelectorAll('[data-testid="go-board"]'));
  const visible = boards.find((b) => b.getBoundingClientRect().width > 0);
  if (!visible) throw new Error("No visible go board in the DOM");
  return visible;
}

function intersectionAt(point: string): Element {
  const board = activeBoard();
  const cells = board.querySelectorAll('[data-testid="intersection"]');
  const boardSize = Math.round(Math.sqrt(cells.length));
  const { row, col } = parsePoint(point, boardSize);
  const target = board.querySelector(
    `[data-testid="intersection"][data-row="${row}"][data-col="${col}"]`,
  );
  if (!target) throw new Error(`No intersection at ${point} (row ${row}, col ${col})`);
  return target;
}

/**
 * Viewport-pixel center of an intersection. Reads the rendered click target's
 * bounding rect, so BoardSvg's padding/showCoordinates geometry is handled for
 * free. Feed the result to the MCP plugin's simulate_mouse_movement for a
 * trusted click, or use clickPoint for a synthetic one.
 */
function pointCenter(point: string): ViewportPoint {
  const rect = intersectionAt(point).getBoundingClientRect();
  return { x: rect.x + rect.width / 2, y: rect.y + rect.height / 2 };
}

/** Synthetic click on an intersection (BoardSvg binds plain onclick). */
function clickPoint(point: string): ViewportPoint {
  const target = intersectionAt(point);
  const center = pointCenter(point);
  target.dispatchEvent(
    new MouseEvent("click", { bubbles: true, clientX: center.x, clientY: center.y }),
  );
  return center;
}

export function installPlaytestHooks(): void {
  const api: PlaytestApi = {
    getView: () => viewGetter?.() ?? null,
    getGameState,
    pointCenter,
    clickPoint,
  };
  (window as unknown as { __playtest?: PlaytestApi }).__playtest = api;
}
