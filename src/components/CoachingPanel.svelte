<script lang="ts">
  import type { CoachingMessage } from "../lib/api/types";
  import { severityColor, severityLabel } from "../lib/utils/colors";

  type Props = {
    messages: CoachingMessage[];
  };

  let { messages }: Props = $props();

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
      {#each messages as msg}
        <div class="rounded bg-stone-800 p-2 text-xs">
          <span
            class="inline-block rounded px-1.5 py-0.5 text-[10px] font-bold"
            style="background-color: {hexColor(msg.severity)}; color: white;"
          >
            {severityLabel(msg.severity)}
          </span>
          <span class="ml-1 text-stone-400">Move {msg.move_number}</span>
          <p class="mt-1 text-stone-300">{msg.message}</p>
        </div>
      {/each}
    {/if}
  </div>
</div>
