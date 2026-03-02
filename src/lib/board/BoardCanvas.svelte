<script lang="ts">
  import { BoardRenderer } from "./renderer";
  import type { StoneColor, StonePosition } from "../api/types";

  type Props = {
    boardSize: number;
    stones: StonePosition[];
    currentColor: StoneColor;
    lastMove: [number, number] | null;
    showCoordinates?: boolean;
    onIntersectionClick: (row: number, col: number) => void;
  };

  let { boardSize, stones, currentColor, lastMove, showCoordinates = false, onIntersectionClick }: Props = $props();

  let canvasEl: HTMLCanvasElement;
  let renderer: BoardRenderer | null = null;

  const CANVAS_SIZE = 600;

  $effect(() => {
    if (!canvasEl) return;

    renderer?.destroy();
    renderer = new BoardRenderer({
      boardSize,
      canvasSize: CANVAS_SIZE,
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
    renderer.drawStones(stones, lastMove);
  });
</script>

<canvas
  bind:this={canvasEl}
  width={CANVAS_SIZE}
  height={CANVAS_SIZE}
  class="max-w-full aspect-square cursor-pointer rounded-lg shadow-lg"
></canvas>
