<script lang="ts">
  import type { SkillProfile } from "../lib/api/types";

  type Props = {
    profile: SkillProfile;
  };

  let { profile }: Props = $props();

  let containerEl: HTMLDivElement;
  let canvasEl: HTMLCanvasElement;
  let displayWidth = $state(0);

  const CHART_HEIGHT = 260;
  const LABELS = ["Reading", "Shape", "Direction", "Endgame", "Life & Death", "Fighting"];

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
    draw();
  });

  function dimensionValue(index: number): number {
    const dims = [
      profile.reading,
      profile.shape,
      profile.direction,
      profile.endgame,
      profile.life_death,
      profile.fighting,
    ];
    // Normalize: 30 (worst) → 0, 1 (best) → 1
    return (30 - dims[index].mu) / 29;
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
    const cx = w / 2;
    const cy = h / 2;
    const maxR = Math.min(cx, cy) - 36;
    const numAxes = 6;
    const angleStep = (Math.PI * 2) / numAxes;
    const startAngle = -Math.PI / 2; // Start at top

    // Background
    ctx.fillStyle = "#1c1917";
    ctx.fillRect(0, 0, w, h);

    // Concentric hexagon grid lines (at 25%, 50%, 75%, 100%)
    ctx.strokeStyle = "#44403c"; // stone-700
    ctx.lineWidth = 0.5;
    for (const frac of [0.25, 0.5, 0.75, 1.0]) {
      ctx.beginPath();
      for (let i = 0; i <= numAxes; i++) {
        const angle = startAngle + i * angleStep;
        const x = cx + Math.cos(angle) * maxR * frac;
        const y = cy + Math.sin(angle) * maxR * frac;
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.stroke();
    }

    // Axis lines
    ctx.strokeStyle = "#57534e"; // stone-600
    ctx.lineWidth = 0.5;
    for (let i = 0; i < numAxes; i++) {
      const angle = startAngle + i * angleStep;
      ctx.beginPath();
      ctx.moveTo(cx, cy);
      ctx.lineTo(cx + Math.cos(angle) * maxR, cy + Math.sin(angle) * maxR);
      ctx.stroke();
    }

    // Skill polygon (filled)
    ctx.beginPath();
    for (let i = 0; i <= numAxes; i++) {
      const idx = i % numAxes;
      const angle = startAngle + idx * angleStep;
      const val = Math.max(dimensionValue(idx), 0.02); // Minimum visible radius
      const x = cx + Math.cos(angle) * maxR * val;
      const y = cy + Math.sin(angle) * maxR * val;
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.fillStyle = "rgba(245, 158, 11, 0.3)"; // amber-500/0.3
    ctx.fill();
    ctx.strokeStyle = "#f59e0b"; // amber-500
    ctx.lineWidth = 2;
    ctx.stroke();

    // Vertex dots
    ctx.fillStyle = "#f59e0b";
    for (let i = 0; i < numAxes; i++) {
      const angle = startAngle + i * angleStep;
      const val = Math.max(dimensionValue(i), 0.02);
      const x = cx + Math.cos(angle) * maxR * val;
      const y = cy + Math.sin(angle) * maxR * val;
      ctx.beginPath();
      ctx.arc(x, y, 3.5, 0, Math.PI * 2);
      ctx.fill();
    }

    // Axis labels
    ctx.fillStyle = "#a8a29e"; // stone-400
    ctx.font = "11px sans-serif";
    for (let i = 0; i < numAxes; i++) {
      const angle = startAngle + i * angleStep;
      const labelR = maxR + 20;
      const x = cx + Math.cos(angle) * labelR;
      const y = cy + Math.sin(angle) * labelR;

      ctx.textBaseline = "middle";
      if (Math.abs(Math.cos(angle)) < 0.1) {
        ctx.textAlign = "center";
      } else if (Math.cos(angle) > 0) {
        ctx.textAlign = "left";
      } else {
        ctx.textAlign = "right";
      }

      // Nudge top/bottom labels
      const yOffset = Math.sin(angle) < -0.5 ? -4 : Math.sin(angle) > 0.5 ? 4 : 0;
      ctx.fillText(LABELS[i], x, y + yOffset);
    }
  }
</script>

<div bind:this={containerEl} class="w-full">
  <canvas
    bind:this={canvasEl}
    class="w-full rounded"
    style="height: {CHART_HEIGHT}px"
    role="img"
    aria-label="Skill profile radar chart showing ratings for Reading, Shape, Direction, Endgame, Life and Death, and Fighting"
  ></canvas>
</div>
