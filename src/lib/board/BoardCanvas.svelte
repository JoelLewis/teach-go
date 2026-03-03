<script lang="ts">
  import { BoardRenderer, type Highlight } from "./renderer";
  import type { Severity, StoneColor, StonePosition } from "../api/types";
  import type { BoardTheme } from "./themes";

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
    onIntersectionClick: (row: number, col: number) => void;
  };

  let { boardSize, stones, currentColor, lastMove, showCoordinates = false, ownership = null, highlights = [], lastMoveSeverity = null, theme, animate = false, onIntersectionClick }: Props = $props();

  let canvasEl: HTMLCanvasElement;
  let renderer: BoardRenderer | null = null;

  const CANVAS_SIZE = 600;

  $effect(() => {
    if (!canvasEl) return;

    // Recreate renderer when boardSize or theme changes
    const _theme = theme;
    renderer?.destroy();
    renderer = new BoardRenderer({
      boardSize,
      canvasSize: CANVAS_SIZE,
      theme: _theme,
      showCoordinates,
      onIntersectionClick,
    });
    renderer.init(canvasEl);

    return () => {
      renderer?.destroy();
      renderer = null;
    };
  });

  $effect(() => {
    if (!renderer) return;
    renderer.setHoverColor(currentColor);
    renderer.drawStones(stones, lastMove, animate);
  });

  $effect(() => {
    if (!renderer) return;
    renderer.drawOwnership(ownership ?? null, boardSize);
  });

  $effect(() => {
    if (!renderer) return;
    renderer.drawHighlights(highlights);
  });

  $effect(() => {
    if (!renderer) return;
    if (lastMove && lastMoveSeverity) {
      renderer.drawMoveQuality(lastMoveSeverity, lastMove[0], lastMove[1]);
    } else {
      renderer.drawMoveQuality(null, 0, 0);
    }
  });
</script>

<canvas
  bind:this={canvasEl}
  width={CANVAS_SIZE}
  height={CANVAS_SIZE}
  class="max-w-full aspect-square cursor-pointer rounded-lg shadow-lg"
></canvas>
