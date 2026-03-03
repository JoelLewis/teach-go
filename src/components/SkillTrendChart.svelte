<script lang="ts">
  import type { SkillSnapshot } from "../lib/api/types";

  type WindowOption = 7 | 30 | null;

  type Props = {
    snapshots: SkillSnapshot[];
    onWindowChange: (days: WindowOption) => void;
    activeWindow: WindowOption;
  };

  let { snapshots, onWindowChange, activeWindow }: Props = $props();

  let containerEl: HTMLDivElement;
  let canvasEl: HTMLCanvasElement;
  let displayWidth = $state(0);

  const CHART_HEIGHT = 180;
  const PADDING = { top: 12, right: 16, bottom: 24, left: 44 };

  const windows: { label: string; value: WindowOption }[] = [
    { label: "7d", value: 7 },
    { label: "30d", value: 30 },
    { label: "All", value: null },
  ];

  $effect(() => {
    if (!containerEl) return;

    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        displayWidth = entry.contentRect.width;
      }
    });
    observer.observe(containerEl);

    return () => observer.disconnect();
  });

  $effect(() => {
    if (!canvasEl || displayWidth === 0) return;
    // Track snapshots and activeWindow reactivity
    void snapshots.length;
    void activeWindow;
    draw();
  });

  function getComputedColor(prop: string, fallback: string): string {
    const val = getComputedStyle(document.documentElement).getPropertyValue(prop).trim();
    return val || fallback;
  }

  function draw() {
    const dpr = window.devicePixelRatio || 1;
    canvasEl.width = displayWidth * dpr;
    canvasEl.height = CHART_HEIGHT * dpr;
    canvasEl.style.width = `${displayWidth}px`;
    canvasEl.style.height = `${CHART_HEIGHT}px`;

    const ctx = canvasEl.getContext("2d")!;
    ctx.scale(dpr, dpr);

    const w = displayWidth;
    const h = CHART_HEIGHT;
    const plotLeft = PADDING.left;
    const plotRight = w - PADDING.right;
    const plotTop = PADDING.top;
    const plotBottom = h - PADDING.bottom;
    const plotWidth = plotRight - plotLeft;
    const plotHeight = plotBottom - plotTop;

    // Background
    const bgColor = getComputedColor("--surface-primary", "#1c1917");
    ctx.fillStyle = bgColor;
    ctx.fillRect(0, 0, w, h);

    if (snapshots.length < 2) {
      ctx.fillStyle = getComputedColor("--text-muted", "#78716c");
      ctx.font = "13px sans-serif";
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText("Play more games to see your trend", w / 2, h / 2);
      return;
    }

    const ranks = snapshots.map((s) => s.overall_rank);
    const minRank = Math.max(Math.floor(Math.min(...ranks)) - 2, 1);
    const maxRank = Math.min(Math.ceil(Math.max(...ranks)) + 2, 30);
    const rankRange = maxRank - minRank || 1;

    const count = snapshots.length;
    const xScale = (i: number) => plotLeft + (i / (count - 1)) * plotWidth;
    // Inverted: lower rank (stronger) at top
    const yScale = (rank: number) =>
      plotTop + ((rank - minRank) / rankRange) * plotHeight;

    // Horizontal grid lines at 5k intervals
    const gridColor = getComputedColor("--border-secondary", "#292524");
    const labelColor = getComputedColor("--text-muted", "#78716c");
    ctx.strokeStyle = gridColor;
    ctx.lineWidth = 0.5;
    ctx.setLineDash([3, 3]);
    ctx.font = "10px monospace";
    ctx.fillStyle = labelColor;
    ctx.textAlign = "right";
    ctx.textBaseline = "middle";

    for (let rank = 5; rank <= 25; rank += 5) {
      if (rank >= minRank && rank <= maxRank) {
        const gy = yScale(rank);
        ctx.beginPath();
        ctx.moveTo(plotLeft, gy);
        ctx.lineTo(plotRight, gy);
        ctx.stroke();
        ctx.fillText(`${rank}k`, plotLeft - 4, gy);
      }
    }
    ctx.setLineDash([]);

    // Y-axis boundary labels
    ctx.fillText(`${minRank}k`, plotLeft - 4, yScale(minRank));
    ctx.fillText(`${maxRank}k`, plotLeft - 4, yScale(maxRank));

    // Area fill under the trend line
    const accentColor = getComputedColor("--accent-primary", "#f59e0b");
    ctx.beginPath();
    ctx.moveTo(xScale(0), plotBottom);
    for (let i = 0; i < count; i++) {
      ctx.lineTo(xScale(i), yScale(ranks[i]));
    }
    ctx.lineTo(xScale(count - 1), plotBottom);
    ctx.closePath();
    ctx.fillStyle = accentColor + "1a"; // ~10% opacity
    ctx.fill();

    // Trend line
    ctx.beginPath();
    for (let i = 0; i < count; i++) {
      const x = xScale(i);
      const y = yScale(ranks[i]);
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.strokeStyle = accentColor;
    ctx.lineWidth = 2;
    ctx.stroke();

    // Dots at each data point (if few enough to not overlap)
    if (count <= 50) {
      for (let i = 0; i < count; i++) {
        ctx.beginPath();
        ctx.arc(xScale(i), yScale(ranks[i]), 2.5, 0, Math.PI * 2);
        ctx.fillStyle = accentColor;
        ctx.fill();
      }
    }

    // Date labels at endpoints
    ctx.font = "9px sans-serif";
    ctx.fillStyle = labelColor;
    ctx.textBaseline = "top";
    ctx.textAlign = "left";
    ctx.fillText(formatDate(snapshots[0].recorded_at), plotLeft, plotBottom + 6);
    ctx.textAlign = "right";
    ctx.fillText(formatDate(snapshots[count - 1].recorded_at), plotRight, plotBottom + 6);
  }

  function formatDate(dateStr: string): string {
    const d = new Date(dateStr + "Z");
    return d.toLocaleDateString(undefined, { month: "short", day: "numeric" });
  }
</script>

<div class="w-full">
  <div class="mb-2 flex items-center justify-between">
    <h3 class="text-sm font-semibold text-[var(--text-secondary,#a8a29e)]">Rank Trend</h3>
    <div class="flex gap-1">
      {#each windows as w}
        <button
          onclick={() => onWindowChange(w.value)}
          class="rounded px-2 py-0.5 text-xs transition-colors {activeWindow === w.value
            ? 'bg-[var(--accent-primary,#f59e0b)] text-[var(--surface-primary,#1c1917)]'
            : 'text-[var(--text-muted,#78716c)] hover:text-[var(--text-secondary,#a8a29e)]'}"
        >
          {w.label}
        </button>
      {/each}
    </div>
  </div>
  <div bind:this={containerEl} class="w-full">
    <canvas
      bind:this={canvasEl}
      class="w-full rounded"
      style="height: {CHART_HEIGHT}px"
    ></canvas>
  </div>
</div>
