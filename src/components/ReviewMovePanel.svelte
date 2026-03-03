<script lang="ts">
  import type { MoveAnalysis } from "../lib/api/types";

  type Props = {
    analysis: MoveAnalysis | null;
  };

  let { analysis }: Props = $props();

  const severityStyles: Record<string, string> = {
    Excellent: "bg-emerald-900/50 text-emerald-200",
    Good: "bg-green-900/50 text-green-200",
    Inaccuracy: "bg-yellow-900/50 text-yellow-200",
    Mistake: "bg-orange-900/50 text-orange-200",
    Blunder: "bg-red-900/50 text-red-200",
  };
</script>

<div class="flex flex-col gap-2">
  {#if analysis}
    {#if analysis.move_number === 0}
      <div class="rounded bg-stone-800 p-3 text-sm text-stone-400">
        Initial position
      </div>
    {:else}
      <!-- Severity badge -->
      <div class="flex items-center gap-2">
        <span
          class="rounded px-2 py-0.5 text-xs font-semibold {severityStyles[analysis.severity] ?? 'bg-stone-700 text-stone-300'}"
        >
          {analysis.severity}
        </span>
        {#if analysis.color}
          <span class="text-xs text-stone-500">
            {analysis.color === "black" ? "Black" : "White"} &mdash; {analysis.player_move ?? "pass"}
          </span>
        {/if}
      </div>

      <!-- Score loss -->
      {#if analysis.score_loss > 0}
        <div class="text-xs text-stone-400">
          Score loss: <span class="font-mono text-stone-200">{analysis.score_loss.toFixed(1)}</span> pts
        </div>
      {/if}

      <!-- Best move suggestion -->
      {#if analysis.best_move && analysis.severity !== "Good" && analysis.severity !== "Excellent"}
        <div class="text-xs text-stone-400">
          Suggested: <span class="font-mono text-amber-400">{analysis.best_move}</span>
          {#if analysis.best_variation.length > 1}
            <span class="text-stone-600">
              ({analysis.best_variation.slice(0, 5).join(" → ")})
            </span>
          {/if}
        </div>
      {/if}

      <!-- Coaching message -->
      {#if analysis.coaching_message}
        <div class="rounded bg-stone-800 p-2 text-sm leading-relaxed text-stone-300">
          {analysis.coaching_message}
        </div>
      {:else if analysis.severity === "Excellent"}
        <div class="rounded bg-stone-800 p-2 text-sm text-emerald-400">
          Excellent move!
        </div>
      {:else if analysis.severity === "Good"}
        <div class="rounded bg-stone-800 p-2 text-sm text-stone-400">
          Good move.
        </div>
      {/if}

      <!-- Win rate -->
      <div class="text-xs text-stone-500">
        Black win: {(analysis.winrate_black * 100).toFixed(1)}% &middot;
        Score: {analysis.score_lead > 0 ? "B" : "W"}+{Math.abs(analysis.score_lead).toFixed(1)}
      </div>
    {/if}
  {:else}
    <div class="rounded bg-stone-800 p-3 text-sm text-stone-500">
      No analysis for this position.
    </div>
  {/if}
</div>
