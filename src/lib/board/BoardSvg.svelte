<script lang="ts">
  import type { Severity, StoneColor, StonePosition } from "../api/types";
  import { type BoardTheme, defaultTheme, starPoints } from "./themes";

  const COLUMN_LETTERS = "ABCDEFGHJKLMNOPQRST";

  export type Highlight =
    | { type: "area"; minRow: number; maxRow: number; minCol: number; maxCol: number }
    | { type: "candidates"; points: [number, number][] }
    | { type: "answer"; point: [number, number] }
    | { type: "flash"; point: [number, number]; color: "correct" | "wrong" };

  type Props = {
    boardSize: number;
    stones: StonePosition[];
    currentColor: StoneColor;
    lastMove: [number, number] | null;
    showCoordinates?: boolean;
    ownership?: number[] | null;
    highlights?: Highlight[];
    lastMoveSeverity?: Severity | null;
    theme?: BoardTheme;
    animate?: boolean;
    interactive?: boolean;
    onIntersectionClick: (row: number, col: number) => void;
  };

  let {
    boardSize,
    stones,
    currentColor,
    lastMove,
    showCoordinates = false,
    ownership = null,
    highlights = [],
    lastMoveSeverity = null,
    theme,
    animate = false,
    interactive = true,
    onIntersectionClick,
  }: Props = $props();

  // --- Layout geometry ---
  const SVG_SIZE = 600;

  let padding = $derived(SVG_SIZE * (showCoordinates ? 0.1 : 0.06));
  let cellSize = $derived((SVG_SIZE - 2 * padding) / (boardSize - 1));
  let stoneRadius = $derived(cellSize * 0.45);
  let activeTheme = $derived(theme ?? defaultTheme);

  // Convert JS hex number (0xd4c5a9) to CSS hex string (#d4c5a9)
  function hexColor(n: number): string {
    return "#" + n.toString(16).padStart(6, "0");
  }

  function severityHexColor(severity: Severity): string {
    switch (severity) {
      case "Excellent": return "#10b981";
      case "Good": return "#4caf50";
      case "Inaccuracy": return "#ffc107";
      case "Mistake": return "#ff9800";
      case "Blunder": return "#f44336";
    }
  }

  // Intersection pixel position (same formula as coordinates.ts)
  function pos(row: number, col: number): { x: number; y: number } {
    return { x: padding + col * cellSize, y: padding + row * cellSize };
  }

  // --- Grid lines ---
  type GridLine = { x1: number; y1: number; x2: number; y2: number };

  let gridLines = $derived.by((): GridLine[] => {
    const lines: GridLine[] = [];
    const start = padding;
    const end = padding + (boardSize - 1) * cellSize;
    for (let i = 0; i < boardSize; i++) {
      const p = padding + i * cellSize;
      lines.push({ x1: start, y1: p, x2: end, y2: p }); // horizontal
      lines.push({ x1: p, y1: start, x2: p, y2: end }); // vertical
    }
    return lines;
  });

  // --- Star points ---
  let starPointPositions = $derived(
    starPoints(boardSize).map(([r, c]) => pos(r, c))
  );

  // --- Coordinate labels ---
  type CoordLabel = { x: number; y: number; text: string };

  let coordLabels = $derived.by((): CoordLabel[] => {
    if (!showCoordinates) return [];
    const labels: CoordLabel[] = [];
    const offset = padding * 0.55;
    const gridEnd = padding + (boardSize - 1) * cellSize;

    for (let col = 0; col < boardSize; col++) {
      const x = padding + col * cellSize;
      const letter = COLUMN_LETTERS[col];
      labels.push({ x, y: padding - offset, text: letter });
      labels.push({ x, y: gridEnd + offset, text: letter });
    }
    for (let row = 0; row < boardSize; row++) {
      const y = padding + row * cellSize;
      const number = String(boardSize - row);
      labels.push({ x: padding - offset, y, text: number });
      labels.push({ x: gridEnd + offset, y, text: number });
    }
    return labels;
  });

  let coordFontSize = $derived(Math.max(9, Math.min(14, cellSize * 0.35)));

  // --- Ownership overlay ---
  type OwnershipCell = { x: number; y: number; color: string; alpha: number; row: number; col: number };

  let ownershipCells = $derived.by((): OwnershipCell[] => {
    if (!ownership || ownership.length === 0) return [];
    const cells: OwnershipCell[] = [];
    const size = cellSize * 0.85;
    const half = size / 2;
    for (let row = 0; row < boardSize; row++) {
      for (let col = 0; col < boardSize; col++) {
        const value = ownership[row * boardSize + col];
        const absValue = Math.abs(value);
        if (absValue < 0.1) continue;
        const { x, y } = pos(row, col);
        const color = value > 0 ? "#1a1a3a" : "#f0f0e0";
        const alpha = ((absValue - 0.1) / 0.9) * 0.5;
        cells.push({ x: x - half, y: y - half, color, alpha, row, col });
      }
    }
    return cells;
  });

  let ownershipSquareSize = $derived(cellSize * 0.85);

  // --- Intersections for click targets ---
  type Intersection = { row: number; col: number; x: number; y: number };

  let intersections = $derived.by((): Intersection[] => {
    const pts: Intersection[] = [];
    for (let row = 0; row < boardSize; row++) {
      for (let col = 0; col < boardSize; col++) {
        const { x, y } = pos(row, col);
        pts.push({ row, col, x, y });
      }
    }
    return pts;
  });

  let occupiedPoints = $derived(new Set(stones.map((stone) => `${stone.row},${stone.col}`)));

  // --- Hover state ---
  let hoverPoint = $state<{ row: number; col: number } | null>(null);

  let hoverPos = $derived(
    hoverPoint ? pos(hoverPoint.row, hoverPoint.col) : null
  );

  let hoverFill = $derived(
    currentColor === "black"
      ? hexColor(activeTheme.stoneBlack)
      : hexColor(activeTheme.stoneWhite)
  );

  function isOccupied(row: number, col: number): boolean {
    return occupiedPoints.has(`${row},${col}`);
  }

  function handlePointerEnter(row: number, col: number) {
    if (!interactive || isOccupied(row, col)) {
      hoverPoint = null;
      return;
    }
    hoverPoint = { row, col };
  }

  function handleClick(row: number, col: number) {
    if (!interactive || isOccupied(row, col)) return;
    onIntersectionClick(row, col);
  }

  // --- Keyboard cursor ---
  let cursorPoint = $state<{ row: number; col: number } | null>(null);
  let boardFocused = $state(false);

  let cursorPos = $derived(
    cursorPoint ? pos(cursorPoint.row, cursorPoint.col) : null
  );

  function initCursor() {
    if (!cursorPoint) {
      const center = Math.floor(boardSize / 2);
      cursorPoint = { row: center, col: center };
    }
  }

  function moveCursor(dRow: number, dCol: number) {
    if (!cursorPoint) { initCursor(); return; }
    const newRow = Math.max(0, Math.min(boardSize - 1, cursorPoint.row + dRow));
    const newCol = Math.max(0, Math.min(boardSize - 1, cursorPoint.col + dCol));
    cursorPoint = { row: newRow, col: newCol };
    if (!isOccupied(newRow, newCol)) {
      hoverPoint = { row: newRow, col: newCol };
    } else {
      hoverPoint = null;
    }
  }

  function handleBoardKeydown(e: KeyboardEvent) {
    if (!interactive) return;

    switch (e.key) {
      case "ArrowUp":
        e.preventDefault();
        moveCursor(-1, 0);
        break;
      case "ArrowDown":
        e.preventDefault();
        moveCursor(1, 0);
        break;
      case "ArrowLeft":
        e.preventDefault();
        moveCursor(0, -1);
        break;
      case "ArrowRight":
        e.preventDefault();
        moveCursor(0, 1);
        break;
      case "Enter":
      case " ":
        e.preventDefault();
        if (cursorPoint && !isOccupied(cursorPoint.row, cursorPoint.col)) {
          onIntersectionClick(cursorPoint.row, cursorPoint.col);
          hoverPoint = null;
        }
        break;
      case "Home":
        e.preventDefault();
        cursorPoint = { row: 0, col: 0 };
        break;
      case "End":
        e.preventDefault();
        cursorPoint = { row: boardSize - 1, col: boardSize - 1 };
        break;
      default:
        return;
    }
  }

  function handleBoardFocus() {
    boardFocused = true;
    initCursor();
  }

  function handleBoardBlur() {
    boardFocused = false;
    hoverPoint = null;
  }

  // --- Animation tracking ---
  // Track previous stone keys for capture detection
  let previousStoneKeys = $state(new Set<string>());

  type AnimatingStone = {
    row: number;
    col: number;
    kind: "place" | "capture";
  };

  let animatingStones = $state<AnimatingStone[]>([]);

  // On stones change, detect captures and newly placed stones
  $effect(() => {
    const newKeys = new Set(stones.map((s) => `${s.row},${s.col}`));

    if (animate) {
      const newAnims: AnimatingStone[] = [];

      // Detect captures (were present, now gone)
      for (const key of previousStoneKeys) {
        if (!newKeys.has(key)) {
          const [r, c] = key.split(",").map(Number) as [number, number];
          newAnims.push({ row: r, col: c, kind: "capture" });
        }
      }

      // Detect placement (lastMove that's new)
      if (lastMove && !previousStoneKeys.has(`${lastMove[0]},${lastMove[1]}`)) {
        newAnims.push({ row: lastMove[0], col: lastMove[1], kind: "place" });
      }

      if (newAnims.length > 0) {
        animatingStones = newAnims;
        // Clear after animation duration
        setTimeout(() => { animatingStones = []; }, 150);
      }
    }

    previousStoneKeys = newKeys;
  });

  function isAnimating(row: number, col: number, kind: "place" | "capture"): boolean {
    return animatingStones.some((a) => a.row === row && a.col === col && a.kind === kind);
  }

  // Reset cursor when board size changes (new game)
  $effect(() => {
    const _size = boardSize;
    cursorPoint = null;
  });

  // Clear cursor when board becomes non-interactive
  $effect(() => {
    if (!interactive) {
      cursorPoint = null;
      boardFocused = false;
    }
  });

  // --- Stone gradient IDs (unique per theme to avoid collisions) ---
  // Gradient offset for 3D lighting (upper-left highlight)
  let gradientOffset = $derived(stoneRadius * 0.3);

  // Last move indicator color (opposite of stone color)
  function lastMoveIndicatorColor(): string {
    if (!lastMove) return "#000";
    const stone = stones.find((s) => s.row === lastMove[0] && s.col === lastMove[1]);
    return stone?.color === "black" ? "#ffffff" : "#000000";
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<svg
  viewBox="0 0 {SVG_SIZE} {SVG_SIZE}"
  class="max-w-full max-h-full aspect-square rounded-lg shadow-lg"
  class:cursor-pointer={interactive}
  aria-label="Go board{cursorPoint && boardFocused ? `, cursor at ${COLUMN_LETTERS[cursorPoint.col]}${boardSize - cursorPoint.row}` : ''}"
  role="grid"
  aria-roledescription="game board"
  data-testid="go-board"
  xmlns="http://www.w3.org/2000/svg"
  tabindex={interactive ? 0 : -1}
  onkeydown={handleBoardKeydown}
  onfocus={handleBoardFocus}
  onblur={handleBoardBlur}
>
  <!-- ══════════ Definitions ══════════ -->
  <defs>
    <!-- Wood grain texture pattern (Study theme) -->
    {#if activeTheme.useWoodTexture}
      <pattern id="wood-grain" patternUnits="userSpaceOnUse" width="512" height="512">
        <rect width="512" height="512" fill={hexColor(activeTheme.boardColor)} />
        <filter id="wood-filter" x="0%" y="0%" width="100%" height="100%">
          <feTurbulence type="fractalNoise" baseFrequency="0.02 0.15" numOctaves="5" seed="3" stitchTiles="stitch" result="noise" />
          <feColorMatrix type="matrix" in="noise"
            values="0 0 0 0 0.83
                    0 0 0 0 0.77
                    0 0 0 0 0.66
                    0 0 0 0.12 0" result="grain" />
        </filter>
        <rect width="512" height="512" filter="url(#wood-filter)" />
      </pattern>
    {/if}

    <!-- Gradient stones (Study theme) -->
    {#if activeTheme.useGradientStones}
      <radialGradient id="black-stone-gradient"
        cx={0.5 - 0.3 * (stoneRadius / (stoneRadius * 2))}
        cy={0.5 - 0.3 * (stoneRadius / (stoneRadius * 2))}
        r="0.5"
        fx={0.5 - 0.15}
        fy={0.5 - 0.15}
      >
        <stop offset="0%" stop-color="#4a4a4a" />
        <stop offset="40%" stop-color="#2a2a2a" />
        <stop offset="100%" stop-color="#0a0a0a" />
      </radialGradient>

      <radialGradient id="white-stone-gradient"
        cx={0.5 - 0.3 * (stoneRadius / (stoneRadius * 2))}
        cy={0.5 - 0.3 * (stoneRadius / (stoneRadius * 2))}
        r="0.5"
        fx={0.5 - 0.15}
        fy={0.5 - 0.15}
      >
        <stop offset="0%" stop-color="#ffffff" />
        <stop offset="40%" stop-color="#f0f0e8" />
        <stop offset="100%" stop-color="#c8c8c0" />
      </radialGradient>
    {/if}

    <!-- Contact shadow filter -->
    {#if activeTheme.contactShadowAlpha && activeTheme.contactShadowAlpha > 0}
      <filter id="contact-shadow" x="-20%" y="-20%" width="140%" height="140%">
        <feDropShadow
          dx={stoneRadius * 0.08}
          dy={stoneRadius * 0.08}
          stdDeviation="1.5"
          flood-color="#000000"
          flood-opacity={activeTheme.contactShadowAlpha}
        />
      </filter>
    {/if}
  </defs>

  <!-- ══════════ Layer 1: Board background ══════════ -->
  {#if activeTheme.useWoodTexture}
    <rect width={SVG_SIZE} height={SVG_SIZE} fill="url(#wood-grain)" />
  {:else}
    <rect width={SVG_SIZE} height={SVG_SIZE} fill={hexColor(activeTheme.boardColor)} />
  {/if}

  <!-- ══════════ Layer 2: Grid lines ══════════ -->
  {#each gridLines as line}
    <line
      x1={line.x1} y1={line.y1}
      x2={line.x2} y2={line.y2}
      stroke={hexColor(activeTheme.lineColor)}
      stroke-width={activeTheme.lineWidth}
    />
  {/each}

  <!-- ══════════ Layer 3: Star points ══════════ -->
  {#each starPointPositions as pt}
    <circle
      cx={pt.x} cy={pt.y}
      r={activeTheme.starPointRadius}
      fill={hexColor(activeTheme.lineColor)}
    />
  {/each}

  <!-- ══════════ Layer 4: Coordinate labels ══════════ -->
  {#each coordLabels as label}
    <text
      x={label.x} y={label.y}
      text-anchor="middle"
      dominant-baseline="central"
      fill={hexColor(activeTheme.coordinateColor)}
      font-size={coordFontSize}
      font-family="sans-serif"
      class="select-none pointer-events-none"
    >{label.text}</text>
  {/each}

  <!-- ══════════ Layer 5: Ownership overlay ══════════ -->
  {#each ownershipCells as cell}
    <rect
      x={cell.x} y={cell.y}
      width={ownershipSquareSize} height={ownershipSquareSize}
      fill={cell.color}
      opacity={cell.alpha}
      data-row={cell.row}
      data-col={cell.col}
      class="pointer-events-none"
    />
  {/each}

  <!-- ══════════ Layer 6: Highlights ══════════ -->
  {#each highlights as h}
    {#if h.type === "area"}
      {@const x1 = padding + h.minCol * cellSize - cellSize * 0.5}
      {@const y1 = padding + h.minRow * cellSize - cellSize * 0.5}
      {@const w = (h.maxCol - h.minCol + 1) * cellSize}
      {@const ht = (h.maxRow - h.minRow + 1) * cellSize}
      <rect
        x={x1} y={y1} width={w} height={ht}
        fill="#f59e0b" opacity="0.15"
        data-type="area"
        class="highlight pointer-events-none"
      />
    {:else if h.type === "candidates"}
      {#each h.points as [row, col]}
        {@const p = pos(row, col)}
        <circle
          cx={p.x} cy={p.y} r={cellSize * 0.25}
          fill="#f59e0b" opacity="0.5"
          data-type="candidates"
          class="highlight pointer-events-none"
        />
      {/each}
    {:else if h.type === "answer"}
      {@const p = pos(h.point[0], h.point[1])}
      <circle
        cx={p.x} cy={p.y} r={cellSize * 0.3}
        fill="#f59e0b" opacity="0.8"
        data-type="answer"
        class="highlight pointer-events-none"
      />
    {:else if h.type === "flash"}
      {@const p = pos(h.point[0], h.point[1])}
      {@const color = h.color === "correct" ? "#10b981" : "#ef4444"}
      <circle
        cx={p.x} cy={p.y} r={cellSize * 0.5}
        fill={color} opacity="0.4"
        data-type="flash"
        class="highlight pointer-events-none"
      />
    {/if}
  {/each}

  <!-- ══════════ Layer 7: Stones ══════════ -->
  {#each stones as stone (stone.row + "," + stone.col)}
    {@const p = pos(stone.row, stone.col)}
    {@const isPlacing = isAnimating(stone.row, stone.col, "place")}
    <g
      data-testid="stone"
      data-row={stone.row}
      data-col={stone.col}
      data-color={stone.color}
      class="pointer-events-none"
      class:stone-entering={isPlacing}
      style:transform-origin="{p.x}px {p.y}px"
    >
      <!-- Stone circle -->
      {#if activeTheme.useGradientStones}
        <circle
          cx={p.x} cy={p.y} r={stoneRadius}
          fill="url(#{stone.color === 'black' ? 'black' : 'white'}-stone-gradient)"
          stroke={hexColor(activeTheme.stoneStroke)}
          stroke-width="0.5"
          stroke-opacity="0.3"
          filter={activeTheme.contactShadowAlpha ? "url(#contact-shadow)" : undefined}
        />
      {:else}
        <circle
          cx={p.x} cy={p.y} r={stoneRadius}
          fill={hexColor(stone.color === "black" ? activeTheme.stoneBlack : activeTheme.stoneWhite)}
          stroke={hexColor(activeTheme.stoneStroke)}
          stroke-width="1"
          filter={activeTheme.contactShadowAlpha ? "url(#contact-shadow)" : undefined}
        />
      {/if}
    </g>
  {/each}

  <!-- Capture animation ghosts -->
  {#each animatingStones.filter(a => a.kind === "capture") as ghost (ghost.row + "," + ghost.col + ",capture")}
    {@const p = pos(ghost.row, ghost.col)}
    <circle
      cx={p.x} cy={p.y} r={stoneRadius}
      fill="#888888" opacity="0.6"
      class="stone-leaving pointer-events-none"
      style:transform-origin="{p.x}px {p.y}px"
    />
  {/each}

  <!-- ══════════ Layer 8: Last move indicator ══════════ -->
  {#if lastMove}
    {@const p = pos(lastMove[0], lastMove[1])}
    <circle
      cx={p.x} cy={p.y} r={stoneRadius * 0.3}
      fill={lastMoveIndicatorColor()}
      class="pointer-events-none"
      data-testid="last-move"
    />
  {/if}

  <!-- ══════════ Layer 8b: Move quality indicator ══════════ -->
  {#if lastMoveSeverity && lastMove}
    {@const p = pos(lastMove[0], lastMove[1])}
    <circle
      cx={p.x} cy={p.y} r={cellSize * 0.5}
      fill="none"
      stroke={severityHexColor(lastMoveSeverity)}
      stroke-width="2.5"
      opacity="0.85"
      class="pointer-events-none"
      data-testid="move-quality"
    />
  {/if}

  <!-- ══════════ Layer 9: Hover preview ══════════ -->
  {#if hoverPos}
    <circle
      cx={hoverPos.x} cy={hoverPos.y} r={stoneRadius}
      fill={hoverFill}
      opacity={activeTheme.hoverAlpha}
      class="pointer-events-none"
      data-testid="hover-preview"
    />
  {/if}

  <!-- ══════════ Layer 9b: Keyboard cursor ══════════ -->
  {#if cursorPos && boardFocused && interactive}
    <circle
      cx={cursorPos.x} cy={cursorPos.y} r={cellSize * 0.48}
      fill="none"
      stroke={hexColor(activeTheme.lastMoveIndicator)}
      stroke-width="2"
      stroke-dasharray="4 3"
      opacity="0.8"
      class="pointer-events-none"
      data-testid="keyboard-cursor"
    />
  {/if}

  <!-- ══════════ Layer 10: Click targets ══════════ -->
  {#each intersections as pt}
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <rect
      data-testid="intersection"
      data-row={pt.row}
      data-col={pt.col}
      x={pt.x - cellSize / 2}
      y={pt.y - cellSize / 2}
      width={cellSize}
      height={cellSize}
      fill="transparent"
      role="gridcell"
      tabindex="-1"
      aria-disabled={!interactive}
      onclick={() => handleClick(pt.row, pt.col)}
      onpointerenter={() => { handlePointerEnter(pt.row, pt.col); }}
      onpointerleave={() => { hoverPoint = null; }}
    />
  {/each}
</svg>

<style>
  @keyframes stone-place {
    from { transform: scale(0); }
    to { transform: scale(1); }
  }

  @keyframes stone-capture {
    from { transform: scale(1); opacity: 0.6; }
    to { transform: scale(0.7); opacity: 0; }
  }

  .stone-entering {
    animation: stone-place 120ms cubic-bezier(0.33, 1, 0.68, 1) both;
  }

  .stone-leaving {
    animation: stone-capture 120ms ease-in forwards;
  }

  .select-none {
    user-select: none;
  }

  svg:focus-visible {
    outline: 2px solid var(--accent-primary, #c9a84c);
    outline-offset: 2px;
  }
</style>
