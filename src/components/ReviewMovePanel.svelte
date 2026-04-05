<script lang="ts">
  import type { MoveAnalysis } from "../lib/api/types";

  type Props = {
    analysis: MoveAnalysis | null;
  };

  let { analysis }: Props = $props();

  const severityStyles: Record<string, string> = {
    Excellent: "background-color: var(--severity-excellent); color: var(--severity-excellent-text);",
    Good: "background-color: var(--severity-good); color: var(--severity-good-text);",
    Inaccuracy: "background-color: var(--severity-inaccuracy); color: var(--severity-inaccuracy-text);",
    Mistake: "background-color: var(--severity-mistake); color: var(--severity-mistake-text);",
    Blunder: "background-color: var(--severity-blunder); color: var(--severity-blunder-text);",
  };
</script>

<div class="flex flex-col gap-2">
  {#if analysis}
    {#if analysis.move_number === 0}
      <div class="rounded p-3 text-sm" style="background-color: var(--surface-card); color: var(--text-secondary);">
        Initial position
      </div>
    {:else}
      <!-- Severity badge -->
      <div class="flex items-center gap-2">
        <span
          class="rounded px-2 py-0.5 text-xs font-semibold"
          style="{severityStyles[analysis.severity] ?? `background-color: var(--surface-input); color: var(--text-on-card);`}"
        >
          {analysis.severity}
        </span>
        {#if analysis.color}
          <span class="text-xs" style="color: var(--text-muted);">
            {analysis.color === "black" ? "Black" : "White"} &mdash; {analysis.player_move ?? "pass"}
          </span>
        {/if}
      </div>

      <!-- Score loss -->
      {#if analysis.score_loss > 0}
        <div class="text-xs" style="color: var(--text-secondary);">
          Score loss: <span class="font-mono" style="color: var(--text-on-card);">{analysis.score_loss.toFixed(1)}</span> pts
        </div>
      {/if}

      <!-- Best move suggestion -->
      {#if analysis.best_move && analysis.severity !== "Good" && analysis.severity !== "Excellent"}
        <div class="text-xs" style="color: var(--text-secondary);">
          Suggested: <span class="font-mono" style="color: var(--accent-primary);">{analysis.best_move}</span>
          {#if analysis.best_variation.length > 1}
            <span style="color: var(--border-subtle);">
              ({analysis.best_variation.slice(0, 5).join(" \u2192 ")})
            </span>
          {/if}
        </div>
      {/if}

      <!-- Coaching message -->
      {#if analysis.coaching_message}
        <div class="rounded p-2 text-sm leading-relaxed" style="background-color: var(--surface-card); color: var(--text-on-card);">
          {analysis.coaching_message}
        </div>
      {:else if analysis.severity === "Excellent"}
        <div class="rounded p-2 text-sm" style="background-color: var(--surface-card); color: var(--success);">
          Excellent move!
        </div>
      {:else if analysis.severity === "Good"}
        <div class="rounded p-2 text-sm" style="background-color: var(--surface-card); color: var(--text-secondary);">
          Good move.
        </div>
      {/if}

      <!-- Win rate -->
      <div class="text-xs" style="color: var(--text-muted);">
        Black win: {(analysis.winrate_black * 100).toFixed(1)}% &middot;
        Score: {analysis.score_lead > 0 ? "B" : "W"}+{Math.abs(analysis.score_lead).toFixed(1)}
      </div>
    {/if}
  {:else}
    <div class="rounded p-3 text-sm" style="background-color: var(--surface-card); color: var(--text-muted);">
      No analysis for this position.
    </div>
  {/if}
</div>
