<script lang="ts">
  import type { CoachingMessage } from "../lib/api/types";
  import { severityColor, severityLabel } from "../lib/utils/colors";

  type Props = {
    messages: CoachingMessage[];
    streamingMoveNumber?: number | null;
    onNavigate?: (moveNumber: number) => void;
  };

  let { messages, streamingMoveNumber = null, onNavigate }: Props = $props();

  function hexColor(severity: CoachingMessage["severity"]): string {
    return `#${severityColor(severity).toString(16).padStart(6, "0")}`;
  }
</script>

<div class="flex flex-col gap-2">
  <h3 class="text-sm font-semibold" style="color: var(--text-secondary);">Coaching</h3>
  <div class="max-h-64 space-y-2 overflow-y-auto">
    {#if messages.length === 0}
      <p class="text-xs italic" style="color: var(--text-muted);">
        Coaching feedback will appear here during play.
      </p>
    {:else}
      {#each messages as msg (msg.move_number)}
        <button
          type="button"
          class="w-full rounded p-2 text-left text-xs {onNavigate ? 'cursor-pointer hover:opacity-90' : ''}"
          style="background-color: var(--surface-card);"
          onclick={() => onNavigate?.(msg.move_number)}
        >
          <span
            class="inline-block rounded px-1.5 py-0.5 text-[10px] font-bold"
            style="background-color: {hexColor(msg.severity)}; color: white;"
          >
            {severityLabel(msg.severity)}
          </span>
          <span class="ml-1" style="color: var(--text-secondary);">Move {msg.move_number}</span>
          <p class="mt-1" style="color: var(--text-on-card);">
            {msg.message}{#if streamingMoveNumber === msg.move_number}<span class="animate-pulse" style="color: var(--accent-primary);">|</span>{/if}
          </p>
          {#if msg.simplest_move}
            <p class="mt-0.5" style="color: var(--text-secondary);">
              Try: <span class="font-mono" style="color: var(--accent-primary);">{msg.simplest_move}</span>
              <span style="color: var(--text-muted);">(simpler alternative)</span>
            </p>
          {/if}
        </button>
      {/each}
    {/if}
  </div>
</div>
