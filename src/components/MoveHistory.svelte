<script lang="ts">
  import type { GameState, MoveEntry } from "../lib/api/types";
  import { toGtp } from "../lib/utils/coordinates";

  type Props = {
    game: GameState;
    viewingMove?: number | null;
    onNavigate?: (moveNumber: number) => void;
  };

  let { game, viewingMove = null, onNavigate }: Props = $props();

  function moveLabel(entry: MoveEntry): string {
    if (entry.is_pass) return "Pass";
    if (entry.row != null && entry.col != null) {
      return toGtp(entry.row, entry.col, game.board_size);
    }
    return "?";
  }

  // Group moves into pairs: [black, white?]
  let pairs = $derived.by(() => {
    const result: { number: number; black: MoveEntry | null; white: MoveEntry | null }[] = [];
    for (let i = 0; i < game.moves.length; i += 2) {
      const black = game.moves[i] ?? null;
      const white = game.moves[i + 1] ?? null;
      result.push({ number: Math.floor(i / 2) + 1, black, white });
    }
    return result;
  });
</script>

<div class="flex flex-col gap-1">
  <h3 class="text-sm font-semibold" style="color: var(--text-secondary);">Move History</h3>
  <div class="max-h-48 overflow-y-auto rounded p-2 text-xs font-mono" style="background-color: var(--surface-card);">
    {#if game.moves.length === 0}
      <p class="italic font-sans" style="color: var(--text-muted);">No moves yet</p>
    {:else}
      <div class="flex flex-col gap-0.5">
        {#each pairs as pair}
          <div class="flex gap-1 items-baseline">
            <span class="w-6 text-right shrink-0" style="color: var(--border-subtle);">{pair.number}.</span>
            {#if pair.black}
              <button
                class="px-1 rounded hover:opacity-90"
                style="{viewingMove === pair.black.move_number ? `background-color: var(--accent-primary); color: var(--btn-text);` : `color: var(--text-on-card);`}"
                onclick={() => onNavigate?.(pair.black!.move_number)}
              >
                {moveLabel(pair.black)}
              </button>
            {/if}
            {#if pair.white}
              <button
                class="px-1 rounded hover:opacity-90"
                style="{viewingMove === pair.white.move_number ? `background-color: var(--accent-primary); color: var(--btn-text);` : `color: var(--text-on-card);`}"
                onclick={() => onNavigate?.(pair.white!.move_number)}
              >
                {moveLabel(pair.white)}
              </button>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
