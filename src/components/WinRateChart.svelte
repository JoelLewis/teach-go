<script lang="ts">
  import type { MoveAnalysis } from "../lib/api/types";

  type Props = {
    analyses: MoveAnalysis[];
    currentMove: number;
    topMistakes: number[];
    onMoveSelect: (move: number) => void;
  };

  let { analyses, currentMove, topMistakes, onMoveSelect }: Props = $props();

  let containerEl: HTMLDivElement;
  let canvasEl: HTMLCanvasElement;
  let displayWidth = $state(0);
  let displayHeight = $state(0);

  const CHART_HEIGHT = 160;
  const PADDING = { top: 8, right: 12, bottom: 20, left: 36 };

  $effect(() => {
    if (!containerEl) return;

    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        displayWidth = entry.contentRect.width;
        displayHeight = CHART_HEIGHT;
      }
    });
    observer.observe(containerEl);

    return () => observer.disconnect();
  });

  $effect(() => {
    if (!canvasEl || displayWidth === 0 || analyses.length === 0) return;
    draw();
  });

  function draw() {
    const dpr = window.devicePixelRatio || 1;
    canvasEl.width = displayWidth * dpr;
    canvasEl.height = displayHeight * dpr;
    canvasEl.style.width = `${displayWidth}px`;
    canvasEl.style.height = `${displayHeight}px`;

    const ctx = canvasEl.getContext("2d")!;
    ctx.scale(dpr, dpr);

    const w = displayWidth;
    const h = displayHeight;
    const plotLeft = PADDING.left;
    const plotRight = w - PADDING.right;
    const plotTop = PADDING.top;
    const plotBottom = h - PADDING.bottom;
    const plotWidth = plotRight - plotLeft;
    const plotHeight = plotBottom - plotTop;
    const totalMoves = analyses.length - 1;

    // Background
    const bgColor = getComputedStyle(document.documentElement).getPropertyValue("--surface-primary").trim() || "#1c1917";
    ctx.fillStyle = bgColor;
    ctx.fillRect(0, 0, w, h);

    if (totalMoves <= 0) return;

    const xScale = (move: number) => plotLeft + (move / totalMoves) * plotWidth;
    const yScale = (winrate: number) => plotTop + (1 - winrate) * plotHeight;

    // Fill areas: Black above 50%, White below 50%
    const midY = yScale(0.5);

    // Black fill (above midline)
    ctx.beginPath();
    ctx.moveTo(xScale(0), midY);
    for (const a of analyses) {
      const x = xScale(a.move_number);
      const y = yScale(Math.max(a.winrate_black, 0.5));
      ctx.lineTo(x, y);
    }
    ctx.lineTo(xScale(totalMoves), midY);
    ctx.closePath();
    ctx.fillStyle = "rgba(23, 23, 23, 0.6)"; // dark fill for Black's advantage
    ctx.fill();

    // White fill (below midline)
    ctx.beginPath();
    ctx.moveTo(xScale(0), midY);
    for (const a of analyses) {
      const x = xScale(a.move_number);
      const y = yScale(Math.min(a.winrate_black, 0.5));
      ctx.lineTo(x, y);
    }
    ctx.lineTo(xScale(totalMoves), midY);
    ctx.closePath();
    ctx.fillStyle = "rgba(245, 245, 244, 0.15)"; // light fill for White's advantage
    ctx.fill();

    // Win-rate line
    ctx.beginPath();
    for (let i = 0; i < analyses.length; i++) {
      const x = xScale(analyses[i].move_number);
      const y = yScale(analyses[i].winrate_black);
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.strokeStyle = "#a8a29e"; // stone-400
    ctx.lineWidth = 1.5;
    ctx.stroke();

    // 50% dashed line
    ctx.beginPath();
    ctx.setLineDash([4, 4]);
    ctx.moveTo(plotLeft, midY);
    ctx.lineTo(plotRight, midY);
    ctx.strokeStyle = "#57534e"; // stone-600
    ctx.lineWidth = 1;
    ctx.stroke();
    ctx.setLineDash([]);

    // Top mistake dots
    for (const moveNum of topMistakes) {
      const a = analyses.find((an) => an.move_number === moveNum);
      if (!a) continue;
      const x = xScale(a.move_number);
      const y = yScale(a.winrate_black);
      ctx.beginPath();
      ctx.arc(x, y, 4, 0, Math.PI * 2);
      ctx.fillStyle = "#ef4444"; // red-500
      ctx.fill();
    }

    // Current move vertical line
    const curX = xScale(currentMove);
    ctx.beginPath();
    ctx.moveTo(curX, plotTop);
    ctx.lineTo(curX, plotBottom);
    ctx.strokeStyle = "#f59e0b"; // amber-500
    ctx.lineWidth = 1.5;
    ctx.stroke();

    // Current position dot
    const curAnalysis = analyses.find((a) => a.move_number === currentMove);
    if (curAnalysis) {
      const cy = yScale(curAnalysis.winrate_black);
      ctx.beginPath();
      ctx.arc(curX, cy, 4, 0, Math.PI * 2);
      ctx.fillStyle = "#f59e0b";
      ctx.fill();
    }

    // Y-axis labels
    ctx.fillStyle = "#78716c"; // stone-500
    ctx.font = "10px monospace";
    ctx.textAlign = "right";
    ctx.textBaseline = "middle";
    ctx.fillText("100%", plotLeft - 4, plotTop);
    ctx.fillText("50%", plotLeft - 4, midY);
    ctx.fillText("0%", plotLeft - 4, plotBottom);

    // X-axis labels
    ctx.textAlign = "center";
    ctx.textBaseline = "top";
    ctx.fillText("0", plotLeft, plotBottom + 4);
    if (totalMoves > 0) {
      ctx.fillText(String(totalMoves), plotRight, plotBottom + 4);
    }
    const midMove = Math.round(totalMoves / 2);
    if (totalMoves > 10) {
      ctx.fillText(String(midMove), xScale(midMove), plotBottom + 4);
    }

    // Side labels
    ctx.font = "9px sans-serif";
    ctx.fillStyle = "#a8a29e";
    ctx.textAlign = "left";
    ctx.textBaseline = "top";
    ctx.fillText("B", plotLeft + 2, plotTop + 2);
    ctx.textBaseline = "bottom";
    ctx.fillText("W", plotLeft + 2, plotBottom - 2);
  }

  function handleClick(e: MouseEvent) {
    if (!canvasEl || analyses.length <= 1) return;
    const rect = canvasEl.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const plotLeft = PADDING.left;
    const plotRight = displayWidth - PADDING.right;
    const plotWidth = plotRight - plotLeft;
    const totalMoves = analyses.length - 1;

    const ratio = Math.max(0, Math.min(1, (x - plotLeft) / plotWidth));
    const moveNum = Math.round(ratio * totalMoves);
    onMoveSelect(moveNum);
  }
</script>

<div bind:this={containerEl} class="w-full">
  <canvas
    bind:this={canvasEl}
    class="w-full cursor-crosshair rounded"
    style="height: {CHART_HEIGHT}px"
    role="img"
    aria-label="Win rate chart showing black and white advantage throughout the game"
    onclick={handleClick}
  ></canvas>
</div>
