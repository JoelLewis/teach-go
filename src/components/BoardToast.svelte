<script lang="ts">
  import type { CoachingMessage } from "../lib/api/types";
  import { severityLabel } from "../lib/utils/colors";
  import { onDestroy } from "svelte";

  type Props = {
    message: CoachingMessage | null;
    onClickMessage?: (moveNumber: number) => void;
  };

  let { message, onClickMessage }: Props = $props();

  let visible = $state(false);
  let fading = $state(false);
  let hideTimer: ReturnType<typeof setTimeout> | undefined;
  let fadeTimer: ReturnType<typeof setTimeout> | undefined;

  const SHOW_SEVERITIES = new Set(["Inaccuracy", "Mistake", "Blunder"]);

  $effect(() => {
    if (message && SHOW_SEVERITIES.has(message.severity)) {
      clearTimeout(hideTimer);
      clearTimeout(fadeTimer);
      fading = false;
      visible = true;
      fadeTimer = setTimeout(() => { fading = true; }, 4500);
      hideTimer = setTimeout(() => { visible = false; fading = false; }, 5000);
    } else {
      visible = false;
    }
  });

  onDestroy(() => {
    clearTimeout(hideTimer);
    clearTimeout(fadeTimer);
  });
</script>

{#if visible && message}
  <button
    type="button"
    class="absolute bottom-2 left-2 right-2 z-10 rounded-md p-3 text-left backdrop-blur-sm transition-opacity duration-300"
    style="background-color: color-mix(in srgb, var(--surface-primary) 90%, transparent);"
    class:opacity-0={fading}
    onclick={() => onClickMessage?.(message.move_number)}
  >
    <div class="text-xs font-bold" style="color: var(--severity-{message.severity.toLowerCase()}-text, var(--danger));">
      {severityLabel(message.severity)} · −{message.score_loss.toFixed(1)} pts
    </div>
    <div class="mt-1 text-sm line-clamp-2" style="color: var(--text-on-card);">
      {message.message}
    </div>
  </button>
{/if}
