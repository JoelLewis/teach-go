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
  <h3 class="text-sm font-semibold text-stone-400">Coaching</h3>
  <div class="max-h-64 space-y-2 overflow-y-auto">
    {#if messages.length === 0}
      <p class="text-xs text-stone-500 italic">
        Coaching feedback will appear here during play.
      </p>
    {:else}
      {#each messages as msg (msg.move_number)}
        <button
          type="button"
          class="w-full rounded bg-stone-800 p-2 text-left text-xs {onNavigate ? 'cursor-pointer hover:bg-stone-700' : ''}"
          onclick={() => onNavigate?.(msg.move_number)}
        >
          <span
            class="inline-block rounded px-1.5 py-0.5 text-[10px] font-bold"
            style="background-color: {hexColor(msg.severity)}; color: white;"
          >
            {severityLabel(msg.severity)}
          </span>
          <span class="ml-1 text-stone-400">Move {msg.move_number}</span>
          <p class="mt-1 text-stone-300">
            {msg.message}{#if streamingMoveNumber === msg.move_number}<span class="animate-pulse text-amber-400">|</span>{/if}
          </p>
          {#if msg.simplest_move}
            <p class="mt-0.5 text-stone-400">
              Try: <span class="font-mono text-amber-400">{msg.simplest_move}</span>
              <span class="text-stone-500">(simpler alternative)</span>
            </p>
          {/if}
        </button>
      {/each}
    {/if}
  </div>
</div>
